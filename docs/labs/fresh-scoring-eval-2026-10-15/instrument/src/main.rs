#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uor_r4_discrete::{
    controller::{
        completion::{CompletionAdmission, CHECKPOINTS},
        validate_redirect_config, KindCounters,
    },
    corpus::{self, CorpusBatch, CorpusBinding, FilePin, PreparedCorpus, Split},
    training::{self, Cursor, DriverCheckpoint, PreparedDriver},
};

#[cfg(feature = "cuda")]
mod evaluator;
#[cfg(any(feature = "cuda", test))]
mod receipts;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REQUEST_SCHEMA: &str = "uor-r4.fresh-scoring-request/1";
const REPORT_SCHEMA: &str = "uor-r4.fresh-scoring-report/1";
const PACKET: &str =
    include_str!("../../../admission-redirect-packet-2026-10-15/data-binding.json");
const ROWS: usize = 2_000;
const BATCH_ROWS: u32 = 64;
const CONTEXT: u32 = 192;

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(value)?)
}
fn digest(value: &impl Serialize) -> Result<String> {
    Ok(corpus::sha256(&encode(value)?))
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    checkpoint: FilePin,
    admission: FilePin,
    expected_admission_sha256: String,
    expected_lineage_sha256: String,
    expected_model_storage_sha256: String,
    expected_cursor: Cursor,
    fresh_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompletionInput {
    schema: String,
    admission_sha256: String,
    pilot_parent: Option<FilePin>,
    checkpoint: DriverCheckpoint,
    baseline_evaluation: Value,
    baseline_snapshot_metadata: Value,
    vector: KindCounters,
    route: KindCounters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenRow {
    source_target_position: u64,
    output_target_position: u64,
    source_document_start: u64,
    target_token: u32,
    causal_row_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Correspondence {
    row_index: usize,
    target_position: u64,
    target_token: u32,
    context_length: usize,
    record: u64,
    causal_row_sha256: String,
}

struct FreshData {
    corpus: PreparedCorpus,
    rows: Vec<FrozenRow>,
    correspondence: Vec<Correspondence>,
    documents_sha256: String,
    rows_sha256: String,
}

fn source_sha256() -> String {
    let mut hash = Sha256::new();
    for bytes in [
        include_bytes!("main.rs").as_slice(),
        include_bytes!("evaluator.rs").as_slice(),
        include_bytes!("receipts.rs").as_slice(),
        include_bytes!("../Cargo.toml").as_slice(),
        include_bytes!("../Cargo.lock").as_slice(),
        PACKET.as_bytes(),
    ] {
        hash.update(bytes);
    }
    format!("{:x}", hash.finalize())
}

fn row_correspondence(
    batch: &CorpusBatch,
    first: usize,
    rows: &[FrozenRow],
) -> Result<Vec<Correspondence>> {
    let count = batch.targets.len();
    require(
        count > 0
            && count <= BATCH_ROWS as usize
            && batch.windows.len() == count
            && batch.target_positions.len() == count
            && first
                .checked_add(count)
                .is_some_and(|end| end <= rows.len())
            && batch.next_target == (first + count) as u64,
        "batch inventory/cursor mismatch",
    )?;
    let mut result = Vec::with_capacity(count);
    for (offset, window) in batch.windows.iter().enumerate() {
        let expected = &rows[first + offset];
        let position = batch.target_positions[offset];
        let target = batch.targets[offset];
        require(
            !window.is_empty()
                && window.len() <= CONTEXT as usize
                && target < 4096
                && expected.output_target_position == position
                && expected.target_token == target,
            "ordered target correspondence mismatch",
        )?;
        let record = window[0].record;
        require(
            record >> 62 == 2
                && window.iter().all(|identity| {
                    identity.record == record && identity.version == 0 && identity.token < 4096
                }),
            "fresh identity namespace mismatch",
        )?;
        let start = position
            .checked_sub(window.len() as u64)
            .ok_or("context underflow")?;
        require(
            window
                .iter()
                .enumerate()
                .all(|(index, identity)| identity.occurrence == start + index as u64),
            "causal window occurrence/order mismatch",
        )?;
        let mut hash = Sha256::new();
        hash.update(b"uor-r4.c192-context-plus-target/u16le/1\0");
        hash.update((window.len() as u64).to_le_bytes());
        for identity in window {
            hash.update((identity.token as u16).to_le_bytes());
        }
        hash.update((target as u16).to_le_bytes());
        let key = format!("{:x}", hash.finalize());
        require(
            key == expected.causal_row_sha256,
            "causal context+target digest mismatch",
        )?;
        result.push(Correspondence {
            row_index: first + offset,
            target_position: position,
            target_token: target,
            context_length: window.len(),
            record,
            causal_row_sha256: key,
        });
    }
    Ok(result)
}

fn load_fresh(directory: &Path) -> Result<FreshData> {
    let packet: Value = serde_json::from_str(PACKET)?;
    let mut expected: CorpusBinding = serde_json::from_value(packet["fresh"].clone())?;
    require(
        expected.source.split == Split::Fresh
            && expected.eligible_targets == ROWS as u64
            && expected.documents == 200
            && expected.payload_tokens == 505_085,
        "compiled packet inventory mismatch",
    )?;
    expected.source.tokens.path = directory.join(&expected.source.tokens.path);
    expected.source.mask.path = directory.join(&expected.source.mask.path);
    let prepared = PreparedCorpus::load(&expected.source, expected.vocabulary, &expected.bos)?;
    require(
        prepared.binding() == &expected,
        "fresh corpus metadata differs from frozen packet",
    )?;
    let rows_pin = FilePin {
        path: directory.join("fresh/rows.json"),
        sha256: packet["rows_sha256"]
            .as_str()
            .ok_or("packet rows hash missing")?
            .into(),
    };
    let documents_pin = FilePin {
        path: directory.join("fresh/documents.json"),
        sha256: packet["documents_sha256"]
            .as_str()
            .ok_or("packet document hash missing")?
            .into(),
    };
    let rows: Vec<FrozenRow> = serde_json::from_slice(&rows_pin.read()?)?;
    let documents: Vec<Value> = serde_json::from_slice(&documents_pin.read()?)?;
    require(
        rows.len() == ROWS && documents.len() == 200,
        "row/document inventory mismatch",
    )?;
    require(
        rows.windows(2)
            .all(|pair| pair[0].output_target_position < pair[1].output_target_position),
        "fresh rows must be strictly ascending and unique",
    )?;
    let mut correspondence = Vec::with_capacity(ROWS);
    for first in (0..ROWS).step_by(BATCH_ROWS as usize) {
        let batch = prepared.batch(first as u64, CONTEXT, BATCH_ROWS)?;
        correspondence.extend(row_correspondence(&batch, first, &rows)?);
    }
    require(
        correspondence.len() == ROWS,
        "fresh row coverage incomplete",
    )?;
    Ok(FreshData {
        corpus: prepared,
        rows,
        correspondence,
        documents_sha256: documents_pin.sha256,
        rows_sha256: rows_pin.sha256,
    })
}

fn authenticate(request: &Request, input: &CompletionInput) -> Result<()> {
    require(
        request.schema == REQUEST_SCHEMA,
        "unsupported request schema",
    )?;
    for hash in [
        &request.expected_admission_sha256,
        &request.expected_lineage_sha256,
        &request.expected_model_storage_sha256,
        &request.checkpoint.sha256,
    ] {
        corpus::validate_sha256(hash)?;
    }
    let checkpoint = &input.checkpoint;
    validate_redirect_config(&checkpoint.lineage.config)?;
    require(
        input.schema == "uor-r4.completion-checkpoint/1"
            && input.pilot_parent.is_none()
            && input.admission_sha256 == request.expected_admission_sha256
            && checkpoint.schema == training::CHECKPOINT_SCHEMA
            && checkpoint.cursor == request.expected_cursor
            && digest(&checkpoint.lineage)? == request.expected_lineage_sha256
            && checkpoint.lineage.driver_source_sha256 == training::driver_source_sha256()
            && checkpoint.model_sha256 == request.expected_model_storage_sha256
            && digest(&checkpoint.model)? == checkpoint.model_sha256,
        "checkpoint envelope/lineage/model/cursor authentication failed",
    )?;
    let completed = checkpoint.cursor.completed_batches;
    require(
        completed == 0 || CHECKPOINTS.contains(&completed),
        "only checkpoint zero and registered quarter cursors are supported",
    )?;
    require(
        input.vector.attempted == completed.min(16_384)
            && input.route.attempted == completed.saturating_sub(16_384)
            && input
                .vector
                .accepted_updates
                .checked_add(input.route.accepted_updates)
                == Some(checkpoint.cursor.accepted_updates),
        "checkpoint split-family counters disagree with cursor",
    )?;
    for counter in [&input.vector, &input.route] {
        require(
            counter.accepted_updates <= counter.any_candidate_cost_differs_from_identity
                && counter.any_candidate_cost_differs_from_identity <= counter.attempted,
            "checkpoint sensitivity counter ordering invalid",
        )?;
    }
    Ok(())
}

struct Prepared {
    request: Request,
    input: CompletionInput,
    driver: PreparedDriver,
    fresh: FreshData,
}

fn authenticate_admission(
    request: &Request,
    input: &CompletionInput,
    admission: &CompletionAdmission,
) -> Result<()> {
    let config = &input.checkpoint.lineage.config;
    let arm = if config.objective_policy == uor_r4_discrete::ObjectivePolicy::NearestAddressMarginV1
    {
        "a"
    } else {
        "r"
    };
    require(
        admission.schema == "uor-r4.redirect-owner-admission/1"
            && admission.plan.schema == "uor-r4.redirect-owner-release-plan/1"
            && admission.plan.scope
                == format!(
                    "m1-admission-redirect-k16-{arm}-seed{}-checkpoint0..16404",
                    config.seed
                )
            && admission.plan.pilot_parent.is_none()
            && admission.plan.redirect.is_some()
            && &admission.plan.config == config
            && digest(admission)? == request.expected_admission_sha256
            && admission.plan.execution.lineage_sha256 == request.expected_lineage_sha256
            && admission.plan.execution.driver_source_sha256 == training::driver_source_sha256()
            && admission.plan.execution.config_sha256 == digest(config)?,
        "training admission/configuration/lineage binding mismatch",
    )?;
    for (field, expected) in [
        ("source_sha256", &admission.plan.execution.source_sha256),
        ("kernel_sha256", &admission.plan.execution.kernel_sha256),
        ("geometry_sha256", &admission.plan.execution.geometry_sha256),
    ] {
        require(
            input.checkpoint.model[field].as_str() == Some(expected.as_str()),
            "checkpoint numerical identity differs from training admission",
        )?;
    }
    Ok(())
}

impl Prepared {
    fn load(request: Request) -> Result<Self> {
        let input: CompletionInput = serde_json::from_slice(&request.checkpoint.read()?)?;
        authenticate(&request, &input)?;
        let admission: CompletionAdmission = serde_json::from_slice(&request.admission.read()?)?;
        authenticate_admission(&request, &input, &admission)?;
        let driver = PreparedDriver::load(input.checkpoint.lineage.config.clone())?;
        driver.validate_checkpoint(&input.checkpoint)?;
        let fresh = load_fresh(&request.fresh_directory)?;
        Ok(Self {
            request,
            input,
            driver,
            fresh,
        })
    }

    fn plan(&self) -> Result<Value> {
        Ok(
            json!({"schema":REPORT_SCHEMA,"status":"METADATA_VALID_NOT_EVALUATED",
            "instrument_source_sha256":source_sha256(),"driver_source_sha256":training::driver_source_sha256(),
            "request":self.request,"request_canonical_sha256":digest(&self.request)?,
            "checkpoint_model_storage_sha256":self.input.checkpoint.model_sha256,
            "training_lineage":self.driver.lineage(),"training_lineage_sha256":self.driver.lineage_sha256()?,
            "cursor":self.input.checkpoint.cursor,"sensitivity":{"vector_block":self.input.vector,"route":self.input.route},
            "fresh_binding":self.fresh.corpus.binding(),"packet_binding_sha256":corpus::sha256(PACKET.as_bytes()),
            "rows_sha256":self.fresh.rows_sha256,"documents_sha256":self.fresh.documents_sha256,
            "ordered_correspondence_sha256":digest(&self.fresh.correspondence)?,
            "ordered_correspondence":self.fresh.correspondence,"scored_rows_expected":ROWS,"batch_rows":BATCH_ROWS,"context":CONTEXT,
            "frozen_row_inventory":self.fresh.rows.len(),
            "novelty_scope":"held out from this D32 contrast, historically used pre-D32 chat source, not globally unseen data",
            "authentication_scope":"artifact/configuration/data/cursor integrity, not proof or authorization of training",
            "model_execution":"NOT_RUN","updates_applied_by_evaluation":0,"training_authorization":"NONE"}),
        )
    }
}

#[cfg(any(feature = "cuda", test))]
fn verify_unchanged(
    before_sha: &str,
    after_sha: &str,
    before_epoch: u64,
    after_epoch: u64,
    expected_epoch: u64,
) -> Result<()> {
    require(
        before_sha == after_sha && before_epoch == expected_epoch && after_epoch == expected_epoch,
        "INSTRUMENT_FAULT: evaluation changed snapshot or epoch",
    )
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("inspect-data") if args.len() == 2 => {
            let fresh = load_fresh(Path::new(&args[1]))?;
            println!("{}", serde_json::to_string_pretty(&json!({"status":"DATA_VALID_NOT_MODEL_EVALUATED",
                "fresh_binding":fresh.corpus.binding(),"ordered_correspondence_sha256":digest(&fresh.correspondence)?,
                "rows":fresh.correspondence.len(),"documents_sha256":fresh.documents_sha256,
                "rows_sha256":fresh.rows_sha256,"model_operations":0}))?);
            Ok(())
        }
        Some("plan") if args.len() == 2 => {
            let prepared = Prepared::load(read_json(Path::new(&args[1]))?)?;
            println!("{}", serde_json::to_string_pretty(&prepared.plan()?)?);
            Ok(())
        }
        #[cfg(feature = "cuda")]
        Some("evaluate") if args.len() == 3 => evaluator::run(Path::new(&args[1]), Path::new(&args[2])),
        _ => Err("usage: inspect-data PACKET_ROOT | plan REQUEST.json | evaluate REQUEST.json NEW_OUTPUT_DIR (CUDA feature only; separate execution authorization required)".into()),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_discrete::{
        controller::completion::{CompletionPlan, RedirectEvidence},
        Identity,
    };

    fn checkpoint_fixture() -> Result<(Request, CompletionInput, CompletionAdmission)> {
        let summary: Value = serde_json::from_str(include_str!(
            "../../../a2-pilot-2026-10-14/report-summary.json"
        ))?;
        let mut lineage: training::Lineage =
            serde_json::from_value(summary["baseline_evaluation"]["evaluation_lineage"].clone())?;
        lineage.config = serde_json::from_str(include_str!(
            "../../../admission-redirect-packet-2026-10-15/configs/k16-R-seed1.json"
        ))?;
        lineage.driver_source_sha256 = training::driver_source_sha256();
        let model = json!({"fixture":"metadata-only, not a numerical model",
            "source_sha256":"a".repeat(64),"kernel_sha256":"b".repeat(64),"geometry_sha256":"c".repeat(64)});
        let unused = FilePin {
            path: "/no-file-created".into(),
            sha256: "d".repeat(64),
        };
        let mut plan: CompletionPlan = serde_json::from_str(include_str!(
            "../../../rung0-completion-release-2026-10-14/SEED2_PLAN.json"
        ))?;
        plan.schema = "uor-r4.redirect-owner-release-plan/1".into();
        plan.scope = "m1-admission-redirect-k16-r-seed1-checkpoint0..16404".into();
        plan.config = lineage.config.clone();
        plan.pilot_parent = None;
        plan.execution.lineage_sha256 = digest(&lineage)?;
        plan.execution.config_sha256 = digest(&plan.config)?;
        plan.execution.driver_source_sha256 = training::driver_source_sha256();
        plan.execution.source_sha256 = "a".repeat(64);
        plan.execution.kernel_sha256 = "b".repeat(64);
        plan.execution.geometry_sha256 = "c".repeat(64);
        plan.redirect = Some(RedirectEvidence {
            packet: unused.clone(),
            qualification: unused.clone(),
            pm_review: unused.clone(),
        });
        let admission = CompletionAdmission {
            schema: "uor-r4.redirect-owner-admission/1".into(),
            plan,
            owner_release: unused.clone(),
        };
        let checkpoint = DriverCheckpoint {
            schema: training::CHECKPOINT_SCHEMA.into(),
            lineage,
            cursor: Cursor::initial(),
            model_sha256: digest(&model)?,
            model,
        };
        let input = CompletionInput {
            schema: "uor-r4.completion-checkpoint/1".into(),
            admission_sha256: digest(&admission)?,
            pilot_parent: None,
            checkpoint,
            baseline_evaluation: json!({}),
            baseline_snapshot_metadata: json!({}),
            vector: KindCounters::default(),
            route: KindCounters::default(),
        };
        let request = Request {
            schema: REQUEST_SCHEMA.into(),
            checkpoint: FilePin {
                path: unused.path.clone(),
                sha256: digest(&input)?,
            },
            admission: FilePin {
                path: unused.path.clone(),
                sha256: digest(&admission)?,
            },
            expected_admission_sha256: input.admission_sha256.clone(),
            expected_lineage_sha256: digest(&input.checkpoint.lineage)?,
            expected_model_storage_sha256: input.checkpoint.model_sha256.clone(),
            expected_cursor: Cursor::initial(),
            fresh_directory: "/not-read".into(),
        };
        Ok((request, input, admission))
    }

    #[test]
    fn metadata_checkpoint_and_admission_are_bound_without_model_execution() -> Result<()> {
        let (request, input, admission) = checkpoint_fixture()?;
        authenticate(&request, &input)?;
        authenticate_admission(&request, &input, &admission)?;
        for mutation in 0..10 {
            let mut wrong = input.clone();
            match mutation {
                0 => wrong.admission_sha256 = "e".repeat(64),
                1 => wrong.checkpoint.model["fixture"] = json!("changed"),
                2 => wrong.checkpoint.lineage.config.seed = 2,
                3 => wrong.checkpoint.lineage.config.spec.kept_cells = 8,
                4 => wrong.checkpoint.cursor.accepted_updates = 1,
                5 => wrong.checkpoint.lineage.driver_source_sha256 = "f".repeat(64),
                6 => wrong.route.attempted = 1,
                7 => wrong.vector.any_candidate_cost_differs_from_identity = 1,
                8 => wrong.pilot_parent = Some(request.checkpoint.clone()),
                _ => wrong.schema = "uor-r4.pilot-checkpoint/1".into(),
            }
            assert!(
                authenticate(&request, &wrong).is_err(),
                "mutation {mutation}"
            );
        }
        for mutation in 0..5 {
            let mut wrong = admission.clone();
            match mutation {
                0 => wrong.plan.config.seed = 2,
                1 => wrong.plan.scope = "m1-a2-completion-seed1-checkpoint1640..16404".into(),
                2 => wrong.plan.execution.source_sha256 = "e".repeat(64),
                3 => wrong.plan.redirect = None,
                _ => wrong.schema = "uor-r4.completion-owner-admission/1".into(),
            }
            let mut rebound = request.clone();
            rebound.expected_admission_sha256 = digest(&wrong)?;
            assert!(
                authenticate_admission(&rebound, &input, &wrong).is_err(),
                "admission mutation {mutation}"
            );
        }
        Ok(())
    }

    #[test]
    fn quarter_cursor_and_full_route_tail_cannot_be_truncated() -> Result<()> {
        let (mut request, mut input, _) = checkpoint_fixture()?;
        for completed in [0, 4_101, 8_202, 12_303, 16_404] {
            input.checkpoint.cursor = Cursor {
                completed_batches: completed,
                accepted_updates: 0,
                corpus_pass: 0,
                next_target: completed * 64,
                next_coordinate: (completed % 16_404) as u32,
                block: None,
            };
            request.expected_cursor = input.checkpoint.cursor;
            input.vector.attempted = completed.min(16_384);
            input.route.attempted = completed.saturating_sub(16_384);
            authenticate(&request, &input)?;
        }
        input.route.attempted = 19;
        assert!(authenticate(&request, &input).is_err());
        input.route.attempted = 20;
        input.checkpoint.cursor.completed_batches = 16_403;
        request.expected_cursor = input.checkpoint.cursor;
        assert!(authenticate(&request, &input).is_err());
        Ok(())
    }

    fn batch_fixture() -> (CorpusBatch, Vec<FrozenRow>) {
        let window = vec![
            Identity {
                token: 0,
                occurrence: 0,
                record: 2 << 62,
                version: 0,
            },
            Identity {
                token: 3,
                occurrence: 1,
                record: 2 << 62,
                version: 0,
            },
        ];
        let mut hash = Sha256::new();
        hash.update(b"uor-r4.c192-context-plus-target/u16le/1\0");
        hash.update(2u64.to_le_bytes());
        for token in [0u16, 3, 5] {
            hash.update(token.to_le_bytes());
        }
        (
            CorpusBatch {
                windows: vec![window],
                targets: vec![5],
                target_positions: vec![2],
                next_target: 1,
            },
            vec![FrozenRow {
                source_target_position: 10,
                output_target_position: 2,
                source_document_start: 8,
                target_token: 5,
                causal_row_sha256: format!("{:x}", hash.finalize()),
            }],
        )
    }

    #[test]
    fn ordered_rows_bind_tokens_context_occurrences_and_fresh_namespace() -> Result<()> {
        let (batch, rows) = batch_fixture();
        assert_eq!(row_correspondence(&batch, 0, &rows)?.len(), 1);
        for mutation in 0..7 {
            let mut wrong = batch.clone();
            match mutation {
                0 => wrong.targets[0] = 6,
                1 => wrong.target_positions[0] = 3,
                2 => wrong.windows[0][0].record = 1 << 62,
                3 => wrong.windows[0][0].occurrence = 1,
                4 => wrong.windows[0][1].token = 4,
                5 => wrong.windows[0].reverse(),
                _ => wrong.next_target = 2,
            }
            assert!(
                row_correspondence(&wrong, 0, &rows).is_err(),
                "mutation {mutation}"
            );
        }
        let mut duplicate = batch.clone();
        duplicate.targets.push(5);
        assert!(row_correspondence(&duplicate, 0, &rows).is_err());
        Ok(())
    }

    #[test]
    fn frozen_packet_pins_are_not_user_overridable() -> Result<()> {
        let packet: Value = serde_json::from_str(PACKET)?;
        assert_eq!(packet["fresh"]["eligible_targets"], 2000);
        assert_eq!(packet["fresh"]["documents"], 200);
        assert_eq!(
            packet["fresh"]["source"]["tokens"]["sha256"],
            "82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b"
        );
        let (request, _, _) = checkpoint_fixture()?;
        let mut value = serde_json::to_value(request)?;
        value["fresh_rows_override"] = json!(1999);
        assert!(serde_json::from_value::<Request>(value).is_err());
        let directory = tempfile::tempdir()?;
        assert!(load_fresh(directory.path()).is_err());
        Ok(())
    }

    #[test]
    fn changed_state_is_an_instrument_fault() -> Result<()> {
        verify_unchanged("same", "same", 17, 17, 17)?;
        assert!(verify_unchanged("same", "changed", 17, 17, 17).is_err());
        assert!(verify_unchanged("same", "same", 17, 18, 17).is_err());
        assert!(verify_unchanged("same", "same", 17, 17, 0).is_err());
        Ok(())
    }

    #[test]
    fn evaluator_source_has_no_update_proposal_or_initializer_calls() {
        let source = include_str!("evaluator.rs");
        for forbidden in [
            concat!(".", "apply("),
            concat!(".", "propose("),
            concat!(".", "transition("),
            "issue_corpus_update_admission",
            "new_model(",
            "Device::new",
            "owner_release(",
        ] {
            assert!(!source.contains(forbidden), "forbidden {forbidden}");
        }
        assert!(source.contains("verify_unchanged("));
        assert!(source.contains("aggregate_evaluations("));
        assert!(source.contains("evaluation.rows.len() == batch.targets.len()"));
    }
}
