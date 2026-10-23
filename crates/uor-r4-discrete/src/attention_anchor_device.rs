//! Prepared TRAIN identity windows and authenticated checkpoint metadata only.
//! No Device construction or numerical execution is available here.

use crate::attention_anchor::{AnchorPolicy, AnchorRow, TrainAnchorManifest};
use crate::attention_audit::{SnapshotPin, StateStamp, MAX_BATCHES};
use crate::corpus::{sha256, validate_sha256, CorpusBinding, FilePin, PreparedCorpus, Split};
use crate::training::{DriverCheckpoint, PreparedDriver};
use crate::{Error, Identity, Result};
use serde::{Deserialize, Serialize};

pub const FIT_SHEET_SHA256: &str =
    "fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450";
pub const REGISTERED_CSV_SHA256: &str =
    "8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1";

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("TRAIN anchor boundary: {message}"))
}

/// Values must come from the independently validated new registration, never
/// from accepting a manifest's self-declared policy as its own authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredAnchorBinding {
    pub corpus: CorpusBinding,
    pub policy: AnchorPolicy,
    pub manifest_sha256: String,
    pub csv_sha256: String,
    pub sheet_sha256: String,
    pub plan_sha256: String,
}

#[derive(Debug)]
pub struct PreparedAnchorBatch {
    rows: Vec<AnchorRow>,
    windows: Vec<Vec<Identity>>,
    targets: Vec<u32>,
}
impl PreparedAnchorBatch {
    pub fn rows(&self) -> &[AnchorRow] {
        &self.rows
    }
    pub fn windows(&self) -> &[Vec<Identity>] {
        &self.windows
    }
    pub fn targets(&self) -> &[u32] {
        &self.targets
    }
}

/// Only immutable identity metadata is cached. Model-bound carried vectors
/// must be recreated for each audit/generation by the future CUDA adapter.
#[derive(Debug)]
pub struct PreparedTrainAnchor {
    registration: RegisteredAnchorBinding,
    batches: Vec<PreparedAnchorBatch>,
}
impl PreparedTrainAnchor {
    pub fn prepare(
        corpus: &PreparedCorpus,
        manifest: &TrainAnchorManifest,
        expected: &RegisteredAnchorBinding,
    ) -> Result<Self> {
        Self::prepare_inner(corpus, manifest, expected, true)
    }
    fn prepare_inner(
        corpus: &PreparedCorpus,
        manifest: &TrainAnchorManifest,
        expected: &RegisteredAnchorBinding,
        registered: bool,
    ) -> Result<Self> {
        for pin in [
            &expected.manifest_sha256,
            &expected.csv_sha256,
            &expected.sheet_sha256,
            &expected.plan_sha256,
        ] {
            validate_sha256(pin)?;
        }
        if expected.corpus.source.split != Split::Train
            || corpus.binding() != &expected.corpus
            || manifest.manifest_sha256 != expected.manifest_sha256
            || manifest.row_csv_sha256 != expected.csv_sha256
            || (registered
                && (expected.policy
                    != (AnchorPolicy {
                        start_eligible_index: 1_049_856,
                        count: 3_847,
                        context: 192,
                    })
                    || expected.sheet_sha256 != FIT_SHEET_SHA256
                    || expected.csv_sha256 != REGISTERED_CSV_SHA256))
        {
            return Err(invalid(
                "anchor differs from independent TRAIN registration",
            ));
        }
        manifest.validate(corpus, &expected.policy)?;
        let mut batches = Vec::new();
        for chunk in manifest.rows.chunks(64) {
            let mut batch = PreparedAnchorBatch {
                rows: chunk.to_vec(),
                windows: Vec::new(),
                targets: Vec::new(),
            };
            for row in chunk {
                let original = corpus.batch(row.eligible_index, expected.policy.context, 1)?;
                batch.windows.push(
                    original
                        .windows
                        .into_iter()
                        .next()
                        .ok_or_else(|| invalid("missing original window"))?,
                );
                batch.targets.push(row.target_token);
            }
            batches.push(batch);
        }
        Ok(Self {
            registration: expected.clone(),
            batches,
        })
    }
    pub fn batches(&self) -> &[PreparedAnchorBatch] {
        &self.batches
    }
    pub fn registration(&self) -> &RegisteredAnchorBinding {
        &self.registration
    }
}

/// Independently sealed controller boundary, not fields adopted from the file
/// being inspected. Raw file bytes contain exactly one DriverCheckpoint JSON;
/// enclosing attention checkpoint formats are deliberately not accepted here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryBinding {
    pub checkpoint_file: FilePin,
    pub snapshot: SnapshotPin,
    pub stamp: StateStamp,
    pub plan_sha256: String,
    pub sheet_sha256: String,
    pub anchor_manifest_sha256: String,
    pub lineage_sha256: String,
    pub config_sha256: String,
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
}

/// Authenticated storage descriptor only. This grants no model execution,
/// GPU admission, owner authorization, or claim of a physically live Device.
#[derive(Debug)]
pub struct ValidatedBoundary {
    checkpoint: DriverCheckpoint,
    binding: BoundaryBinding,
}
impl ValidatedBoundary {
    pub fn checkpoint(&self) -> &DriverCheckpoint {
        &self.checkpoint
    }
    pub fn stamp(&self) -> &StateStamp {
        &self.binding.stamp
    }
    pub fn binding(&self) -> &BoundaryBinding {
        &self.binding
    }
}

pub fn validate_boundary(
    anchor: &PreparedTrainAnchor,
    expected: &BoundaryBinding,
    observed: &StateStamp,
    checkpoint_file_bytes: &[u8],
    driver: &PreparedDriver,
) -> Result<ValidatedBoundary> {
    for hash in [
        &expected.checkpoint_file.sha256,
        &expected.snapshot.checkpoint_sha256,
        &expected.snapshot.storage_sha256,
        &expected.snapshot.snapshot_sha256,
        &expected.stamp.run_sha256,
        &expected.stamp.counters_sha256,
        &expected.plan_sha256,
        &expected.sheet_sha256,
        &expected.anchor_manifest_sha256,
        &expected.lineage_sha256,
        &expected.config_sha256,
        &expected.source_sha256,
        &expected.kernel_sha256,
        &expected.geometry_sha256,
    ] {
        validate_sha256(hash)?;
    }
    let actual_file = sha256(checkpoint_file_bytes);
    if expected.stamp != *observed
        || expected.stamp.runtime_model == 0
        || expected.stamp.epoch > expected.stamp.completed
        || expected.stamp.completed > MAX_BATCHES
        || (expected.stamp.completed != MAX_BATCHES && !expected.stamp.completed.is_multiple_of(64))
        || expected.stamp.generation > expected.stamp.completed.div_ceil(64)
        || expected.checkpoint_file.sha256 != actual_file
        || expected.snapshot.checkpoint_sha256 != actual_file
        || expected.plan_sha256 != anchor.registration.plan_sha256
        || expected.sheet_sha256 != anchor.registration.sheet_sha256
        || expected.anchor_manifest_sha256 != anchor.registration.manifest_sha256
        || driver.train().binding() != &anchor.registration.corpus
        || driver.config().spec.context != anchor.registration.policy.context
        || driver.lineage_sha256()? != expected.lineage_sha256
        || sha256(&serde_json::to_vec(driver.config()).map_err(|e| invalid(&e.to_string()))?)
            != expected.config_sha256
    {
        return Err(invalid(
            "boundary differs from sealed run/anchor/file/driver identity",
        ));
    }
    // Parse the actual authenticated file, never an independently supplied
    // checkpoint whose encoding may differ from the pinned raw-file domain.
    let checkpoint: DriverCheckpoint = serde_json::from_slice(checkpoint_file_bytes)
        .map_err(|e| invalid(&format!("DriverCheckpoint JSON: {e}")))?;
    driver.validate_checkpoint(&checkpoint)?;
    let hashes = crate::snapshot::stored_snapshot_hash_domains(&checkpoint.model)?;
    if checkpoint.cursor.completed_batches != expected.stamp.completed
        || checkpoint.cursor.accepted_updates != expected.stamp.epoch
        || checkpoint.model_sha256 != expected.snapshot.storage_sha256
        || hashes.model_storage_sha256 != expected.snapshot.storage_sha256
        || hashes.model_snapshot_sha256 != expected.snapshot.snapshot_sha256
        || checkpoint
            .model
            .get("source_sha256")
            .and_then(serde_json::Value::as_str)
            != Some(expected.source_sha256.as_str())
        || checkpoint
            .model
            .get("kernel_sha256")
            .and_then(serde_json::Value::as_str)
            != Some(expected.kernel_sha256.as_str())
        || checkpoint
            .model
            .get("geometry_sha256")
            .and_then(serde_json::Value::as_str)
            != Some(expected.geometry_sha256.as_str())
    {
        return Err(invalid(
            "checkpoint cursor/epoch/storage/snapshot/source differs",
        ));
    }
    Ok(ValidatedBoundary {
        checkpoint,
        binding: expected.clone(),
    })
}

/// The descriptor alone is insufficient: only the separately validated audited
/// controller can construct this permit. This is not an arbitrary evaluation CLI.
#[cfg(feature = "cuda")]
pub(crate) fn evaluate_boundary(
    permit: &crate::controller::attention_audited::ControllerAuditPermit,
    anchor: &PreparedTrainAnchor,
    driver: &PreparedDriver,
    model: &crate::device::Device,
    boundary: &ValidatedBoundary,
) -> Result<serde_json::Value> {
    permit.validate(anchor, boundary)?;
    driver.validate_checkpoint(boundary.checkpoint())?;
    model.synchronize()?;
    let live_binding = model.binding().clone();
    let epoch = model.epoch()?;
    let before = crate::snapshot::snapshot_hash_domains(&model.snapshot()?)?;
    let expected = boundary.binding();
    let config = driver.config();
    if live_binding.model != expected.stamp.runtime_model
        || epoch != expected.stamp.epoch
        || before.model_snapshot_sha256 != expected.snapshot.snapshot_sha256
        || model.spec() != config.spec
        || model.read_spec() != config.read_spec
        || model.objective_policy() != config.objective_policy
        || model.read_initialization() != config.read_initialization
        || live_binding.seed != config.seed
    {
        return Err(invalid("actual Device differs from sealed audit boundary"));
    }
    let mut evaluations = Vec::with_capacity(anchor.batches.len());
    for batch in &anchor.batches {
        let carried = model.carry_windows(&batch.windows)?;
        let mut evaluation = model.evaluate(&carried, &batch.targets)?;
        if evaluation.rows.len() != batch.rows.len() {
            return Err(invalid("anchor GPU row inventory differs"));
        }
        for (row, identity) in evaluation.rows.iter_mut().zip(&batch.rows) {
            row.target_position = Some(identity.target_position);
            row.target_token = Some(identity.target_token);
        }
        evaluations.push(evaluation);
    }
    let aggregate = model.aggregate_evaluations(&evaluations)?;
    model.synchronize()?;
    let after = crate::snapshot::snapshot_hash_domains(&model.snapshot()?)?;
    if model.binding() != &live_binding
        || model.epoch()? != epoch
        || after != before
        || aggregate.scored_rows != anchor.registration.policy.count
        || aggregate.target_admitted_rows > aggregate.scored_rows
    {
        return Err(invalid(
            "anchor evaluation changed state or returned wrong inventory",
        ));
    }
    Ok(
        serde_json::json!({"schema":"uor-r4.attention-audit-evaluation/1",
        "boundary":expected,"anchor_registration":anchor.registration,
        "evaluated_state":boundary.stamp(),"actual_snapshot_before":before,
        "actual_snapshot_after":after,"aggregate":aggregate,"batches":evaluations,
        "updates_applied":0,"evaluation_mutates_model":false,
        "dev_read":false,"fresh_read":false,"count_reference_executed":false,
        "host_admission_recomputed":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{sha256, BosIdentity, CorpusSource, FilePin, Split};

    fn corpus(split: Split) -> PreparedCorpus {
        let ids: &[u16] = &[0, 1, 2, 1, 2, 3, 0, 4, 5, 4, 5, 4, 5];
        let mask: &[u8] = &[0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1];
        let mut bytes = vec![0; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(ids.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
        for id in ids {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        PreparedCorpus::from_bytes(
            CorpusSource {
                split,
                tokens: FilePin {
                    path: "unreadable.uort".into(),
                    sha256: sha256(&bytes),
                },
                mask: FilePin {
                    path: "unreadable.mask".into(),
                    sha256: sha256(mask),
                },
                expected_payload_tokens: ids.len() as u64,
                expected_eligible_targets: 10,
            },
            &bytes,
            mask,
            8,
            BosIdentity {
                token: "BOS".into(),
                id: 0,
            },
        )
        .unwrap()
    }
    fn fixture() -> (PreparedCorpus, TrainAnchorManifest, RegisteredAnchorBinding) {
        let corpus = corpus(Split::Train);
        let policy = AnchorPolicy {
            start_eligible_index: 0,
            count: 3,
            context: 4,
        };
        let manifest = TrainAnchorManifest::build(&corpus, policy.clone()).unwrap();
        let binding = RegisteredAnchorBinding {
            corpus: corpus.binding().clone(),
            policy,
            manifest_sha256: manifest.manifest_sha256.clone(),
            csv_sha256: manifest.row_csv_sha256.clone(),
            sheet_sha256: FIT_SHEET_SHA256.into(),
            plan_sha256: sha256(b"plan"),
        };
        (corpus, manifest, binding)
    }
    #[test]
    fn prepared_sparse_original_windows_and_domains() {
        let (corpus, manifest, binding) = fixture();
        let prepared =
            PreparedTrainAnchor::prepare_inner(&corpus, &manifest, &binding, false).unwrap();
        assert_eq!(prepared.batches().len(), 1);
        assert_eq!(prepared.batches()[0].rows, manifest.rows);
        for (i, row) in manifest.rows.iter().enumerate() {
            assert_eq!(
                prepared.batches()[0].windows[i],
                corpus.batch(row.eligible_index, 4, 1).unwrap().windows[0]
            );
        }
        assert!(PreparedTrainAnchor::prepare(&corpus, &manifest, &binding).is_err());
    }
    #[test]
    fn independently_expected_binding_refuses_tampering_and_non_train() {
        let (corpus, manifest, mut binding) = fixture();
        binding.csv_sha256 = sha256(b"wrong");
        assert!(PreparedTrainAnchor::prepare_inner(&corpus, &manifest, &binding, false).is_err());
        let (corpus, mut manifest, binding) = fixture();
        manifest.rows[0].original_window.sha256 = sha256(b"wrong");
        assert!(PreparedTrainAnchor::prepare_inner(&corpus, &manifest, &binding, false).is_err());
        assert!(PreparedTrainAnchor::prepare_inner(
            &super::tests::corpus(Split::Dev),
            &manifest,
            &binding,
            false
        )
        .is_err());
    }

    fn boundary_fixture() -> (
        PreparedTrainAnchor,
        crate::training::PreparedDriver,
        BoundaryBinding,
        StateStamp,
        Vec<u8>,
    ) {
        use crate::training::*;
        use serde_json::json;
        use std::io::Write;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let (train, manifest, registration) = fixture();
        let prepared =
            PreparedTrainAnchor::prepare_inner(&train, &manifest, &registration, false).unwrap();
        let mut dev_bytes = vec![0; 64];
        dev_bytes[..4].copy_from_slice(b"UORT");
        dev_bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        dev_bytes[8..16].copy_from_slice(&3u64.to_le_bytes());
        dev_bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
        for token in [0u16, 2, 3] {
            dev_bytes.extend_from_slice(&token.to_le_bytes());
        }
        let dev_source = CorpusSource {
            split: Split::Dev,
            tokens: FilePin {
                path: "unreadable-dev.uort".into(),
                sha256: sha256(&dev_bytes),
            },
            mask: FilePin {
                path: "unreadable-dev.mask".into(),
                sha256: sha256(&[0, 1, 1]),
            },
            expected_payload_tokens: 3,
            expected_eligible_targets: 2,
        };
        let dev = PreparedCorpus::from_bytes(
            dev_source,
            &dev_bytes,
            &[0, 1, 1],
            8,
            train.binding().bos.clone(),
        )
        .unwrap();
        let tokenizer = br#"{"model":{"vocab":{"BOS":0}}}"#;
        let path = std::env::temp_dir().join(format!(
            "uor-anchor-tokenizer-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(tokenizer).unwrap();
        drop(file);
        let config = DriverConfig {
            schema: DRIVER_SCHEMA.into(),
            spec: crate::Spec {
                vocabulary: 8,
                blocks: 2,
                context: 4,
                kept_cells: 2,
            },
            read_spec: crate::ReadSpec::Route,
            objective_policy: crate::ObjectivePolicy::default(),
            read_initialization: crate::ReadInitialization::default(),
            normalization_mode: crate::normalization_mechanism::NormalizationMode::default(),
            relation_learning_policy: RelationLearningPolicy::LearnV1,
            counterfactual_path: CounterfactualPath::LanesV1,
            codebook_initialization: CodebookInitialization::SeededDeviceV1,
            coordinate_block: CoordinateBlock::ONE,
            seed: 1,
            batch_rows: 3,
            train: train.binding().source.clone(),
            dev: dev.binding().source.clone(),
            tokenizer: FilePin {
                path: path.clone(),
                sha256: sha256(tokenizer),
            },
            bos: train.binding().bos.clone(),
            schedule: SchedulePolicy::InterleavedReadEvery8V1,
        };
        let driver = PreparedDriver::from_prepared(config, train, dev);
        std::fs::remove_file(path).unwrap();
        let driver = driver.unwrap();
        let mut cursor = Cursor::initial();
        for i in 0..64 {
            cursor = driver.advance(&cursor, i < 3).unwrap();
        }
        let model = json!({"schema":crate::SCHEMA,"spec":driver.config().spec,"read_spec":driver.config().read_spec,
            "seed":1,"epoch":3,"table":[1,-1],"routes":[0],"gates":[0],"operator_kappa":"literal","composition_kappa":"literal",
            "source_sha256":sha256(b"source"),"kernel_sha256":sha256(b"kernel"),"geometry_sha256":sha256(b"geometry")});
        let hashes = crate::snapshot::stored_snapshot_hash_domains(&model).unwrap();
        let checkpoint = DriverCheckpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            lineage: driver.lineage().clone(),
            cursor,
            model_sha256: hashes.model_storage_sha256.clone(),
            model,
        };
        let bytes = serde_json::to_vec_pretty(&checkpoint).unwrap();
        let stamp = StateStamp {
            run_sha256: sha256(b"run"),
            counters_sha256: sha256(b"counters"),
            completed: 64,
            epoch: 3,
            generation: 1,
            runtime_model: 100,
        };
        let expected = BoundaryBinding {
            checkpoint_file: FilePin {
                path: "sealed-driver-checkpoint.json".into(),
                sha256: sha256(&bytes),
            },
            snapshot: SnapshotPin {
                checkpoint_sha256: sha256(&bytes),
                storage_sha256: hashes.model_storage_sha256,
                snapshot_sha256: hashes.model_snapshot_sha256,
            },
            stamp: stamp.clone(),
            plan_sha256: registration.plan_sha256,
            sheet_sha256: registration.sheet_sha256,
            anchor_manifest_sha256: registration.manifest_sha256,
            lineage_sha256: driver.lineage_sha256().unwrap(),
            config_sha256: sha256(&serde_json::to_vec(driver.config()).unwrap()),
            source_sha256: sha256(b"source"),
            kernel_sha256: sha256(b"kernel"),
            geometry_sha256: sha256(b"geometry"),
        };
        (prepared, driver, expected, stamp, bytes)
    }

    #[test]
    fn exact_checkpoint_file_and_live_speculative_epoch_are_bound() {
        let (anchor, driver, expected, stamp, bytes) = boundary_fixture();
        let valid = validate_boundary(&anchor, &expected, &stamp, &bytes, &driver).unwrap();
        assert_eq!(valid.checkpoint().cursor.accepted_updates, 3);
        assert_eq!(valid.stamp(), &stamp);
        let mut wrong = expected.clone();
        wrong.snapshot.checkpoint_sha256 = sha256(b"wrong");
        assert!(validate_boundary(&anchor, &wrong, &stamp, &bytes, &driver).is_err());
        let mut wrong = expected.clone();
        wrong.snapshot.snapshot_sha256 = wrong.snapshot.storage_sha256.clone();
        assert!(validate_boundary(&anchor, &wrong, &stamp, &bytes, &driver).is_err());
        let mut different = bytes.clone();
        different.push(b' ');
        assert!(validate_boundary(&anchor, &expected, &stamp, &different, &driver).is_err());
    }

    #[test]
    fn boundary_refuses_forged_generation_source_registration_and_driver_cursor() {
        let (anchor, driver, expected, stamp, bytes) = boundary_fixture();
        let mut wrong_stamp = stamp.clone();
        wrong_stamp.generation += 1;
        assert!(validate_boundary(&anchor, &expected, &wrong_stamp, &bytes, &driver).is_err());
        let mut wrong = expected.clone();
        wrong.source_sha256 = sha256(b"wrong");
        assert!(validate_boundary(&anchor, &wrong, &stamp, &bytes, &driver).is_err());
        let mut wrong = expected.clone();
        wrong.anchor_manifest_sha256 = sha256(b"wrong");
        assert!(validate_boundary(&anchor, &wrong, &stamp, &bytes, &driver).is_err());
        for field in [
            "accepted_updates",
            "corpus_pass",
            "next_target",
            "next_coordinate",
        ] {
            let mut checkpoint: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            checkpoint["cursor"][field] = 999.into();
            let altered = serde_json::to_vec(&checkpoint).unwrap();
            let mut rebound = expected.clone();
            rebound.checkpoint_file.sha256 = sha256(&altered);
            rebound.snapshot.checkpoint_sha256 = sha256(&altered);
            assert!(
                validate_boundary(&anchor, &rebound, &stamp, &altered, &driver).is_err(),
                "{field}"
            );
        }
    }

    #[test]
    fn boundary_refuses_rehashed_model_config_and_unknown_file_formats() {
        let (anchor, driver, expected, stamp, bytes) = boundary_fixture();
        for (field, replacement) in [
            ("seed", serde_json::json!(99)),
            ("schema", serde_json::json!("unknown")),
            ("epoch", serde_json::json!(2)),
        ] {
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            value["model"][field] = replacement;
            value["model_sha256"] = sha256(&serde_json::to_vec(&value["model"]).unwrap()).into();
            let altered = serde_json::to_vec(&value).unwrap();
            let mut rebound = expected.clone();
            rebound.checkpoint_file.sha256 = sha256(&altered);
            rebound.snapshot.checkpoint_sha256 = sha256(&altered);
            rebound.snapshot.storage_sha256 = value["model_sha256"].as_str().unwrap().into();
            assert!(
                validate_boundary(&anchor, &rebound, &stamp, &altered, &driver).is_err(),
                "{field}"
            );
        }
        let outer = serde_json::to_vec(&serde_json::json!({"checkpoint":serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()})).unwrap();
        let mut rebound = expected.clone();
        rebound.checkpoint_file.sha256 = sha256(&outer);
        rebound.snapshot.checkpoint_sha256 = sha256(&outer);
        assert!(validate_boundary(&anchor, &rebound, &stamp, &outer, &driver).is_err());
        let mut wrong = expected.clone();
        wrong.config_sha256 = sha256(b"another config");
        assert!(validate_boundary(&anchor, &wrong, &stamp, &bytes, &driver).is_err());
        let mut wrong = expected.clone();
        wrong.lineage_sha256 = sha256(b"another lineage");
        assert!(validate_boundary(&anchor, &wrong, &stamp, &bytes, &driver).is_err());
        let mut wrong = expected;
        wrong.stamp.counters_sha256 = "UPPERCASE".into();
        assert!(validate_boundary(&anchor, &wrong, &wrong.stamp, &bytes, &driver).is_err());
    }

    #[test]
    fn sparse_packing_has_exact_sixty_full_and_seven_row_tail() {
        let mut bytes = vec![0; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&3850u64.to_le_bytes());
        bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        for _ in 1..3850 {
            bytes.extend_from_slice(&1u16.to_le_bytes());
        }
        let mut mask = vec![1; 3850];
        mask[0] = 0;
        let source = CorpusSource {
            split: Split::Train,
            tokens: FilePin {
                path: "unreadable-many.uort".into(),
                sha256: sha256(&bytes),
            },
            mask: FilePin {
                path: "unreadable-many.mask".into(),
                sha256: sha256(&mask),
            },
            expected_payload_tokens: 3850,
            expected_eligible_targets: 3849,
        };
        let corpus = PreparedCorpus::from_bytes(
            source,
            &bytes,
            &mask,
            8,
            BosIdentity {
                token: "BOS".into(),
                id: 0,
            },
        )
        .unwrap();
        let policy = AnchorPolicy {
            start_eligible_index: 0,
            count: 3847,
            context: 192,
        };
        let manifest = TrainAnchorManifest::build(&corpus, policy.clone()).unwrap();
        let expected = RegisteredAnchorBinding {
            corpus: corpus.binding().clone(),
            policy,
            manifest_sha256: manifest.manifest_sha256.clone(),
            csv_sha256: manifest.row_csv_sha256.clone(),
            sheet_sha256: FIT_SHEET_SHA256.into(),
            plan_sha256: sha256(b"plan"),
        };
        let prepared =
            PreparedTrainAnchor::prepare_inner(&corpus, &manifest, &expected, false).unwrap();
        assert_eq!(prepared.batches().len(), 61);
        assert!(prepared.batches()[..60]
            .iter()
            .all(|batch| batch.rows().len() == 64));
        assert_eq!(prepared.batches()[60].rows().len(), 7);
        assert_eq!(
            prepared
                .batches()
                .iter()
                .flat_map(|batch| batch.rows())
                .cloned()
                .collect::<Vec<_>>(),
            manifest.rows
        );
    }
}
