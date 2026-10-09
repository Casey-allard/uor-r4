//! Frozen integer-only deployment and source-value substitution evaluation.
//! Labels are evaluator-only; native generation sees an owned complete bank.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::{
    answer_oracle::FrozenAnswers,
    native_geometric::learner::{
        geometric_continuation_field::NativeContinuationField,
        native_bank_generate::{
            BankPin, BoundNativeBytes, GenerationLimits, GenerationStop, NativeBankArtifacts,
            NativeBankGenerateStep, NativeBankGenerator, OwnedBankSegment, OwnedBankSource,
            PinnedBankSnapshot, SnapshotSourceStatus,
        },
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata,
    geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_realizer::NativeArtifactBinding,
    geometric_vocabulary_actions::{VocabularyAction, VocabularyTokenMass, SCORE_CLIP_Q24},
};
#[path = "native_bank_generalization/early_query.rs"]
mod early_query;
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use uor_r4_tokenizer::ByteBpeTokenizer;
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ModelKind {
    #[default]
    LegacyRecomposition,
    JointContinuation,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
enum EvaluationKind {
    #[default]
    #[serde(rename = "own_prefix")]
    OwnPrefix,
    #[serde(rename = "entry_zero_u_v2")]
    EntryZeroUV2,
    #[serde(rename = "component_cross_entry")]
    ComponentCrossEntry,
}
const ENTRY_REPORT_SHA: &str = "f46342484238b1f806be312bb2b0e712873c6e8493d92dab6cd9c526308ee342";
const ENTRY_MANIFEST_SHA: &str = "10c8d4598e361eda30d2fe8367f8ad105798a2841ab41f303a276153b4d80057";
const ENTRY_U_SHA: &str = "21b66d25271ca97551d094afed2e492d5a8981db47a527e845c96e41c518c141";
const ENTRY_INPUT_SHA: &str = "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";
const ENTRY_LABEL_SHA: &str = "84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155";
const ENTRY_ORIGINAL_EIGHT: [usize; 8] = [0, 1, 4, 5, 8, 9, 12, 13];
const COMPONENT_REPORT_SHA: &str =
    "9582f56c8d285920cd67977fd23d36e8a96beabe7c5f27ea45ad4b1113d3503c";
const COMPONENT_MANIFEST_SHA: &str =
    "45bbcbf2550b6d6a726df09b3c8ad6307ad20b4a8073306be458ca93f3376da5";
const COMPONENT_RECEIPT_SHA: &str =
    "ea8258461ed0409139490c9cab8cfacbc6f575aa6398f616a13d3620b5a507ca";
const COMPONENT_U_SHA: &str = "be6200897585dd26159d6e78cbfeb16cfce0912abf36fcfa4aab6f7b097314d0";
const COMPONENT_PARENT_REPORT_SHA: &str =
    "a1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278";
const COMPONENT_PARENT_MANIFEST_SHA: &str =
    "b37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca";
const COMPONENT_PARENT_RECEIPT_SHA: &str =
    "0b992aa45141fb1a60cef2dfc62c47af891e3d715c2f271c0662b918fa78f626";
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ComponentParent {
    model_root: PathBuf,
    expected_model_report_sha256: String,
    expected_model_manifest_sha256: String,
    expected_checkpoint_receipt_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    component_parent: Option<ComponentParent>,
    #[serde(default)]
    evaluation_kind: EvaluationKind,
    #[serde(default)]
    model_kind: ModelKind,
    #[serde(default)]
    checkpoint_step: Option<usize>,
    #[serde(default)]
    expected_checkpoint_receipt_sha256: Option<String>,
    model_root: PathBuf,
    expected_model_report_sha256: String,
    expected_model_manifest_sha256: String,
    inputs: PathBuf,
    expected_inputs_sha256: String,
    labels: PathBuf,
    expected_labels_sha256: String,
    output: PathBuf,
    #[serde(default = "default_report_bytes")]
    maximum_report_bytes: u64,
    #[serde(default)]
    baseline_parity: bool,
    /// Diagnostic evidence only; retain the already computed first-step bank replay.
    #[serde(default)]
    retain_entry_bank_trace: bool,
    #[serde(default)]
    continuation_field: Option<PathBuf>,
    #[serde(default)]
    expected_continuation_field_sha256: Option<String>,
}

/// Artifact construction has no panel, labels, target or generation settings.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ZeroContinuationConfig {
    model_root: PathBuf,
    expected_model_report_sha256: String,
    expected_model_manifest_sha256: String,
    output: PathBuf,
}

struct FrozenModel {
    generator: NativeBankGenerator,
    report: Value,
    admission: Value,
    checkpoint: PathBuf,
    native_directory: PathBuf,
    exp_sha256: String,
}
fn default_report_bytes() -> u64 {
    // Full traces for 32 cases at the 32-token limit can exceed 2 GiB.
    3 << 30
}
fn write_row(c: &Config, name: &str, row: &Value) -> Result<()> {
    let used = fs::read_dir(&c.output)?.try_fold(0u64, |sum, e| -> Result<u64> {
        Ok(sum
            .checked_add(e?.metadata()?.len())
            .ok_or_else(|| bad("output size overflow"))?)
    })?;
    let bytes = serde_json::to_vec_pretty(row)?;
    if used.saturating_add(bytes.len() as u64) > c.maximum_report_bytes - (1 << 20) {
        return Err(bad(
            "declared report storage boundary reached; no rows truncated",
        ));
    }
    Ok(fs::write(c.output.join(name), bytes)?)
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Panel {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    schema: String,
    protocol: String,
    membership_only: bool,
    cases: Vec<Label>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Label {
    id: String,
    answers: FrozenAnswers,
    #[serde(default)]
    pair_id: Option<String>,
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn bytes(p: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(p)?)
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn file_hash(p: &Path) -> Result<String> {
    Ok(hash(&bytes(p)?))
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&bytes(p)?)?)
}
fn write(p: &Path, v: &Value) -> Result<()> {
    Ok(fs::write(p, serde_json::to_vec_pretty(v)?)?)
}
fn expected_string(v: &Value) -> Result<&str> {
    v.as_str()
        .ok_or_else(|| bad("expected digest/string absent"))
}
fn seal_for(path: &Path) -> Result<PathBuf> {
    path.ancestors()
        .find(|p| p.join("manifest.json").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| bad("input has no enclosing report seal"))
}
fn snapshot(p: Packet) -> Result<PinnedBankSnapshot> {
    if !p.actual_prefix_ids.is_empty() {
        return Err(bad("generation must start with empty actual prefix"));
    }
    let mut scope = None;
    let mut commit = 0;
    for s in &p.segments {
        if let Segment::Source {
            scope: q,
            commit: c,
            ..
        } = s
        {
            if q.is_empty() || scope.as_ref().is_some_and(|old| old != q) {
                return Err(bad("panel bank has empty/mixed Source scope"));
            }
            scope = Some(q.clone());
            commit = commit.max(*c);
        }
    }
    let scope = scope.ok_or_else(|| bad("Source substitution panel requires Source records"))?;
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: 0,
            commit,
            scope: scope.into_bytes(),
        },
        query_ids: p.query_ids,
        segments: p
            .segments
            .into_iter()
            .map(|s| match s {
                Segment::Source {
                    event,
                    record,
                    commit,
                    scope,
                    entity,
                    relation,
                    view,
                    original_source_ids,
                } => OwnedBankSegment::Source(OwnedBankSource {
                    event,
                    record,
                    commit,
                    scope: scope.into_bytes(),
                    entity,
                    relation,
                    view,
                    status: SnapshotSourceStatus::Found,
                    original_token_ids: original_source_ids,
                }),
                Segment::Context {
                    event,
                    role,
                    token_ids,
                } => OwnedBankSegment::Context {
                    event,
                    role,
                    token_ids,
                },
            })
            .collect(),
    })
}
fn serve(
    g: &mut NativeBankGenerator,
    p: Packet,
) -> Result<uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGeneration> {
    let bank = g.admit_bank(snapshot(p)?)?;
    Ok(g.generate(
        &bank,
        GenerationLimits {
            maximum_tokens: 32,
            maximum_retained_steps: 32,
        },
    )?)
}
fn compare_step(actual: &NativeBankGenerateStep, expected: &Value) -> Result<()> {
    if expected["actual_prefix_ids"] != json!(actual.actual_prefix_ids)
        || expected["copy_token_ids"] != json!(actual.copy_token_ids)
        || expected["copy_raw_scores_q24"] != json!(actual.copy_raw_scores_q24)
        || expected["retained_state_codes"]
            != json!(actual
                .post_state
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>())
        || expected["generate_raw_scores_sha256"]
            != hash(&serde_json::to_vec(&actual.generate_raw_scores_q24)?)
        || expected["pool"]["summary"] != serde_json::to_value(&actual.actions.summary)?
        || expected["generate_costs"] != serde_json::to_value(&actual.generate_counts)?
    {
        return Err(bad(
            "native prefix/Copy/poststate/Generate/reducer summary parity differs",
        ));
    }
    if let Some(trace) = &actual.bank_trace {
        let bank = &trace.cue_bank.bank;
        if expected["source_provenance"]["candidates"] != serde_json::to_value(&bank.candidates)?
            || expected["source_provenance"]["causal_tokens"] != json!(bank.context.tokens)
            || expected["source_provenance"]["legacy_terminal_actions_discarded"] != true
        {
            return Err(bad("full occurrence chronology/provenance parity differs"));
        }
        let count = actual.copy_token_ids.len();
        let sum = |heads: &[Vec<i64>]| -> Result<Vec<i64>> {
            let mut result = vec![0i64; count];
            for head in heads {
                if head.len() != count {
                    return Err(bad("component candidate count differs"));
                }
                for (i, &score) in head.iter().enumerate() {
                    result[i] = result[i]
                        .checked_add(score)
                        .ok_or_else(|| bad("component overflow"))?;
                }
            }
            Ok(result)
        };
        let cue = sum(&trace.cue_bank.carrier.copy_q24)?;
        let prefix = sum(&trace.prefix.copy_q24)?;
        let context = actual
            .copy_raw_scores_q24
            .iter()
            .enumerate()
            .map(|(i, &score)| {
                score
                    .checked_sub(cue[i])
                    .and_then(|v| v.checked_sub(prefix[i]))
                    .ok_or_else(|| bad("attribution overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        if expected["source_provenance"]["copy_components_q24"]
            != json!({"cue":cue,"prefix":prefix,"contextual":context})
        {
            return Err(bad("Copy component attribution parity differs"));
        }
    } else if expected["source_provenance"]["no_source"] != true {
        return Err(bad("source availability parity differs"));
    }
    match &actual.bridge {
        Some(b) => {
            let saved = &expected["source_provenance"]["read_state_bridge"];
            if saved["selected_ordinal"] != json!(b.selected_ordinal)
                || saved["selected_candidate"] != serde_json::to_value(&b.selected_candidate)?
                || saved["query_state_codes"]
                    != json!(b.query_state.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["selected_source_state_codes"]
                    != json!(b.source_state.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["action_codes"]
                    != json!(b.action_codes.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["action_scores_q24"] != json!(b.action_scores_q24)
                || saved["native_costs"] != serde_json::to_value(&b.counts)?
            {
                return Err(bad("raw-selected categorical bridge parity differs"));
            }
        }
        None if !expected["source_provenance"]["read_state_bridge"].is_null() => {
            return Err(bad("unexpected saved bridge"))
        }
        None => {}
    }
    Ok(())
}

fn step_row(
    step: &NativeBankGenerateStep,
    query_ids: Option<&[u32]>,
    field_sha256: Option<&str>,
) -> Result<Value> {
    let mut row = json!({"actual_prefix_ids":step.actual_prefix_ids,"post_state_codes":step.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),
        "copy_token_ids":step.copy_token_ids,"copy_raw_scores_q24":step.copy_raw_scores_q24,"generate_raw_scores_q24":step.generate_raw_scores_q24,
        "actions":step.actions,"bridge":step.bridge.as_ref().map(|b|json!({"selected_ordinal":b.selected_ordinal,"selected_candidate":b.selected_candidate,
            "query_state_codes":b.query_state.iter().map(|v|v.index()).collect::<Vec<_>>(),"source_state_codes":b.source_state.iter().map(|v|v.index()).collect::<Vec<_>>(),
            "action_codes":b.action_codes.iter().map(|v|v.index()).collect::<Vec<_>>()}))});
    match (&step.continuation, query_ids, field_sha256) {
        (Some(witness), Some(query), Some(digest)) => {
            if witness.query_tokens != query.len()
                || witness.actual_prefix_tokens != step.actual_prefix_ids.len()
                || witness.delta_scores_q24.len() != step.generate_raw_scores_q24.len()
            {
                return Err(bad("continuation witness input/score dimensions differ"));
            }
            let local_ids = query
                .iter()
                .chain(&step.actual_prefix_ids)
                .copied()
                .collect::<Vec<_>>();
            row["continuation"] = json!({
                "continuation_sha256":digest,
                "local_input_ids":local_ids,
                "query_ids":query,
                "actual_prefix_ids":step.actual_prefix_ids,
                "query_tokens":witness.query_tokens,
                "actual_prefix_tokens":witness.actual_prefix_tokens,
                "state_codes":witness.state_codes.iter().map(|v|v.index()).collect::<Vec<_>>(),
                "delta_scores_q24":witness.delta_scores_q24,
                "encoding_coefficient_reads":witness.encoding_coefficient_reads,
                "field_counts":witness.counts,
                "carrier_policy":"from-identity;query-then-actual-emitted-prefix;no-fact-tokens",
            });
        }
        (None, None, None) => {}
        _ => return Err(bad("continuation field/witness presence differs")),
    }
    Ok(row)
}

fn load_frozen_model(
    model_root: &Path,
    expected_report_sha256: &str,
    expected_manifest_sha256: &str,
) -> Result<FrozenModel> {
    report_output::verify(model_root)?;
    if file_hash(&model_root.join("report.json"))? != expected_report_sha256
        || file_hash(&model_root.join("manifest.json"))? != expected_manifest_sha256
    {
        return Err(bad("frozen model report/seal identity differs"));
    }
    let r = read(&model_root.join("report.json"))?;
    let admission = read(&model_root.join("admission.json"))?;
    if r["schema"] != "uor-r4.geometric-prediction-control/1"
        || r["status"] != "COMPLETED"
        || r["updates"] != 0
        || r["native_prediction_control_win"] != true
        || r["resume"]["component_recomposition"] != true
        || r["resume"]["source_step"] != 48
        || r["resume"]["generate_step"] != 64
        || r["resume"]["producer_source_commit"] != "b451571aa638d1f93e3ccf25f718f27ed2ea731f"
    {
        return Err(bad(
            "model is not the completed successful zero-update48/64 recomposition",
        ));
    }
    let cp = model_root.join("checkpoint-0000");
    let receipt = read(&cp.join("receipt.json"))?;
    if receipt != r["final_receipt"]
        || receipt != r["initial_receipt"]
        || receipt["step"] != 0
        || receipt["native_independently_reloaded"] != true
        || receipt["masters_independently_reloaded"] != true
    {
        return Err(bad("coherent frozen native checkpoint receipt differs"));
    }
    load_checkpoint(cp, r, admission, receipt)
}

/// Both admission branches share only the authenticated native byte loader.
fn load_checkpoint(cp: PathBuf, r: Value, admission: Value, receipt: Value) -> Result<FrozenModel> {
    let gen = bytes(&cp.join("generate.bin"))?;
    let (generator, exp_sha256) = load_native_components(
        &cp,
        &receipt,
        BoundNativeBytes {
            bytes: &gen,
            sha256: expected_string(&receipt["generate_sha256"])?,
        },
    )?;
    Ok(FrozenModel {
        generator,
        report: r,
        admission,
        native_directory: cp.join("native"),
        checkpoint: cp,
        exp_sha256,
    })
}
/// The Source donor keeps its own source-bound sidecars. Generate bytes and hash
/// come directly from an authenticated donor; no receipt is rewritten.
fn load_native_components(
    cp: &Path,
    receipt: &Value,
    generate: BoundNativeBytes<'_>,
) -> Result<(NativeBankGenerator, String)> {
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let bridge = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let exp_hash = hash(&exp);
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let joint = if cp.join("cue/cue-joint-q4.bin").is_file() {
        Some(bytes(&cp.join("cue/cue-joint-q4.bin"))?)
    } else {
        None
    };
    let native_dir = cp.join("native");
    let g = NativeBankGenerator::load(NativeBankArtifacts {
        native_directory: &native_dir,
        source_binding: &binding,
        generate,
        bridge: Some(BoundNativeBytes {
            bytes: &bridge,
            sha256: expected_string(&receipt["categorical_sha256"])?,
        }),
        cue_packed: &cue,
        cue_joint_packed: joint.as_deref(),
        cue_metadata: &cue_metadata,
        prefix_packed: &prefix,
        prefix_metadata: &prefix_metadata,
        exp: BoundNativeBytes {
            bytes: &exp,
            sha256: &exp_hash,
        },
    })?;
    Ok((g, exp_hash))
}
fn hex_identity(value: &str, digits: usize) -> bool {
    value.len() == digits && value.bytes().all(|v| v.is_ascii_hexdigit())
}
fn admit_model_options(c: &Config) -> Result<()> {
    if c.retain_entry_bank_trace && c.evaluation_kind != EvaluationKind::OwnPrefix {
        return Err(bad(
            "entry bank trace is only supported by own_prefix evaluation",
        ));
    }
    if c.evaluation_kind != EvaluationKind::ComponentCrossEntry && c.component_parent.is_some() {
        return Err(bad(
            "component parent is only valid for component_cross_entry",
        ));
    }
    if c.evaluation_kind == EvaluationKind::ComponentCrossEntry {
        let p = c
            .component_parent
            .as_ref()
            .ok_or_else(|| bad("component parent absent"))?;
        entry_require(
            c.model_kind == ModelKind::JointContinuation
                && c.checkpoint_step == Some(1)
                && c.expected_model_report_sha256 == COMPONENT_REPORT_SHA
                && c.expected_model_manifest_sha256 == COMPONENT_MANIFEST_SHA
                && c.expected_checkpoint_receipt_sha256.as_deref() == Some(COMPONENT_RECEIPT_SHA)
                && c.expected_continuation_field_sha256.as_deref() == Some(COMPONENT_U_SHA)
                && p.expected_model_report_sha256 == COMPONENT_PARENT_REPORT_SHA
                && p.expected_model_manifest_sha256 == COMPONENT_PARENT_MANIFEST_SHA
                && p.expected_checkpoint_receipt_sha256 == COMPONENT_PARENT_RECEIPT_SHA
                && c.expected_inputs_sha256 == ENTRY_INPUT_SHA
                && c.expected_labels_sha256 == ENTRY_LABEL_SHA
                && c.maximum_report_bytes == 128 << 20,
            "component diagnostic requires exact parent/joint1, original512 and128MiB cap",
        )?;
    }
    match c.model_kind {
        ModelKind::LegacyRecomposition => {
            if c.checkpoint_step.is_some() || c.expected_checkpoint_receipt_sha256.is_some() {
                return Err(bad(
                    "joint checkpoint selectors require joint_continuation model_kind",
                ));
            }
        }
        ModelKind::JointContinuation => {
            if c.baseline_parity {
                return Err(bad(
                    "joint model cannot request unchanged legacy baseline parity",
                ));
            }
            if c.checkpoint_step.is_none()
                || !c
                    .expected_checkpoint_receipt_sha256
                    .as_deref()
                    .is_some_and(|v| hex_identity(v, 64))
                || c.continuation_field.is_none()
                || !c
                    .expected_continuation_field_sha256
                    .as_deref()
                    .is_some_and(|v| hex_identity(v, 64))
            {
                return Err(bad("joint model requires explicit final checkpoint step, receipt SHA and continuation path/SHA"));
            }
        }
    }
    if c.evaluation_kind == EvaluationKind::EntryZeroUV2
        && (c.model_kind != ModelKind::JointContinuation
            || c.checkpoint_step != Some(64)
            || c.expected_model_report_sha256 != ENTRY_REPORT_SHA
            || c.expected_model_manifest_sha256 != ENTRY_MANIFEST_SHA
            || c.expected_continuation_field_sha256.as_deref() != Some(ENTRY_U_SHA)
            || c.expected_inputs_sha256 != ENTRY_INPUT_SHA
            || c.expected_labels_sha256 != ENTRY_LABEL_SHA
            || c.maximum_report_bytes != 64 << 20)
    {
        return Err(bad("entry zero-U diagnostic requires exact final64, genuine learned U, original512 and64MiB cap"));
    }
    Ok(())
}
fn validate_joint_receipt(c: &Config, r: &Value, admission: &Value, receipt: &Value) -> Result<()> {
    admit_model_options(c)?;
    let step = c
        .checkpoint_step
        .ok_or_else(|| bad("joint checkpoint step absent"))?;
    if c.model_kind != ModelKind::JointContinuation
        || r["schema"] != "uor-r4.geometric-frozen-map-fit/1"
        || r["status"] != "COMPLETED"
        || r["mode"] != "joint_continuation"
        || r["updates"].as_u64() != Some(step as u64)
        || receipt != &r["final_receipt"]
        || receipt["mode"] != "joint_continuation"
        || receipt["step"].as_u64() != Some(step as u64)
        || receipt["native_independently_reloaded"] != true
        || receipt["masters_independently_reloaded"] != true
        || !r["source_commit"]
            .as_str()
            .is_some_and(|v| hex_identity(v, 40))
        || receipt["source_commit"] != r["source_commit"]
        || admission["source_commit"] != r["source_commit"]
        || admission["mode"] != "joint_continuation"
        || receipt["continuation_sha256"].as_str()
            != c.expected_continuation_field_sha256.as_deref()
    {
        return Err(bad(
            "coherent completed joint final checkpoint identity differs",
        ));
    }
    for key in [
        "parent_report_sha256",
        "parent_manifest_sha256",
        "training_input_sha256",
        "training_labels_sha256",
    ] {
        if !receipt[key].as_str().is_some_and(|v| hex_identity(v, 64))
            || receipt[key] != admission[key]
        {
            return Err(bad(
                "joint checkpoint/admission parent or training provenance differs",
            ));
        }
    }
    Ok(())
}
fn validate_joint_field_path(path: &Path, checkpoint: &Path) -> Result<()> {
    if fs::canonicalize(path)? != fs::canonicalize(checkpoint.join("continuation-field.bin"))? {
        return Err(bad(
            "joint continuation field must come from the selected checkpoint",
        ));
    }
    Ok(())
}
fn load_joint_model(c: &Config) -> Result<FrozenModel> {
    admit_model_options(c)?;
    report_output::verify(&c.model_root)?;
    if file_hash(&c.model_root.join("report.json"))? != c.expected_model_report_sha256
        || file_hash(&c.model_root.join("manifest.json"))? != c.expected_model_manifest_sha256
    {
        return Err(bad("joint model report/seal identity differs"));
    }
    let r = read(&c.model_root.join("report.json"))?;
    let admission = read(&c.model_root.join("admission.json"))?;
    let step = c
        .checkpoint_step
        .ok_or_else(|| bad("joint checkpoint step absent"))?;
    let cp = c.model_root.join(format!("checkpoint-{step:04}"));
    if Some(file_hash(&cp.join("receipt.json"))?.as_str())
        != c.expected_checkpoint_receipt_sha256.as_deref()
    {
        return Err(bad("joint checkpoint receipt SHA differs"));
    }
    let receipt = read(&cp.join("receipt.json"))?;
    validate_joint_receipt(c, &r, &admission, &receipt)?;
    let path = c
        .continuation_field
        .as_ref()
        .ok_or_else(|| bad("joint continuation path absent"))?;
    validate_joint_field_path(path, &cp)?;
    if Some(file_hash(path)?.as_str()) != c.expected_continuation_field_sha256.as_deref() {
        return Err(bad("joint continuation field SHA differs"));
    }
    load_checkpoint(cp, r, admission, receipt)
}
fn load_model(c: &Config) -> Result<FrozenModel> {
    admit_model_options(c)?;
    match c.model_kind {
        ModelKind::LegacyRecomposition => load_frozen_model(
            &c.model_root,
            &c.expected_model_report_sha256,
            &c.expected_model_manifest_sha256,
        ),
        ModelKind::JointContinuation => load_joint_model(c),
    }
}

fn run(c: &Config) -> Result<Value> {
    let FrozenModel {
        generator: mut g,
        report: r,
        admission,
        checkpoint: cp,
        native_directory: native_dir,
        exp_sha256: exp_hash,
    } = load_model(c)?;
    // This is an external authored panel, not a store enumeration or lineage proof.
    let input_root = seal_for(&c.inputs)?;
    let label_root = seal_for(&c.labels)?;
    report_output::verify(&input_root)?;
    report_output::verify(&label_root)?;
    if file_hash(&c.inputs)? != c.expected_inputs_sha256
        || file_hash(&c.labels)? != c.expected_labels_sha256
    {
        return Err(bad("panel identity differs"));
    }
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    let labels: Labels = serde_json::from_slice(&bytes(&c.labels)?)?;
    if panel.schema != "uor-r4.native-source-bank-probe-input/1"
        || labels.schema != "uor-r4.native-source-bank-labels/1"
        || labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !labels.membership_only
        || panel.cases.is_empty()
        || panel.cases.len() > 512
        || panel.cases.len() != labels.cases.len()
    {
        return Err(bad("bounded complete input/label schema coverage differs"));
    }
    if let (Some(path), Some(digest)) =
        (&c.continuation_field, &c.expected_continuation_field_sha256)
    {
        report_output::verify(&seal_for(path)?)?;
        let field = bytes(path)?;
        g = g.with_continuation_field(BoundNativeBytes {
            bytes: &field,
            sha256: digest,
        })?;
    }
    let tok =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes(&native_dir.join("tokenizer.json"))?)
            .ok_or_else(|| bad("ByteBPE unavailable"))?;
    // Keep the legacy saved-row contract; joint reports have development rows
    // instead and are never compared for unchanged legacy trace parity.
    let saved = match c.model_kind {
        ModelKind::LegacyRecomposition => read(&c.model_root.join("prediction-0000.json"))?,
        ModelKind::JointContinuation => json!({"rows": []}),
    };
    let saved_refs = saved["rows"]
        .as_array()
        .ok_or_else(|| bad("saved model generation rows absent"))?;
    if c.baseline_parity && (panel.cases.len() != 8 || saved_refs.len() != 8) {
        return Err(bad("baseline parity requires exactly original eight cases"));
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut complete = 0;
    let mut entry = 0;
    let mut pair_groups: BTreeMap<String, Vec<(String, bool, Vec<u32>)>> = BTreeMap::new();
    for (index, (p, l)) in panel.cases.into_iter().zip(labels.cases).enumerate() {
        if p.id.is_empty() || p.id != l.id || !seen.insert(p.id.clone()) {
            return Err(bad("input/label IDs mismatched or duplicate"));
        }
        l.answers.validate()?;
        let id = p.id.clone();
        let local_query_ids = g.continuation_sha256().map(|_| p.query_ids.clone());
        let generated = serve(&mut g, p)?; // No label, target or answer reaches this call.
        if generated
            .steps
            .iter()
            .any(|step| step.actions.summary.legal_generate_actions != 4096)
        {
            return Err(bad(
                "frozen model full 4096 legal Generate vocabulary differs",
            ));
        }
        let eos = generated.stop == GenerationStop::Eos;
        let output_ids = if eos {
            &generated.generated_ids[..generated.generated_ids.len() - 1]
        } else {
            &generated.generated_ids[..]
        };
        let decoded_bytes = tok.decode_bytes(output_ids);
        let valid_text = std::str::from_utf8(&decoded_bytes).ok();
        let accepted = eos && valid_text.is_some_and(|text| l.answers.accepts(text));
        let decoded = String::from_utf8_lossy(&decoded_bytes).into_owned();
        complete += usize::from(accepted);
        let target = tok.encode(&l.answers.accepted[0]);
        if tok.decode_bytes(&target) != l.answers.accepted[0].as_bytes() {
            return Err(bad("evaluator accepted form does not roundtrip tokenizer"));
        }
        let entry_correct = generated.generated_ids.first() == target.first();
        entry += usize::from(entry_correct);
        let mut parity = Value::Null;
        if c.baseline_parity {
            let reference = &saved_refs[index];
            if reference["id"] != id {
                return Err(bad("original baseline row order differs"));
            }
            let name = expected_string(&reference["row_file"])?;
            if Path::new(name)
                .components()
                .any(|p| !matches!(p, std::path::Component::Normal(_)))
            {
                return Err(bad("unsafe saved row path"));
            }
            let path = c.model_root.join(name);
            if file_hash(&path)? != reference["row_sha256"] {
                return Err(bad("saved row hash differs"));
            }
            let row = read(&path)?;
            let steps = row["generation"]
                .as_array()
                .ok_or_else(|| bad("saved actual-prefix steps absent"))?;
            if row["id"] != id
                || row["generated_ids"] != json!(generated.generated_ids)
                || row["eos"] != eos
                || steps.len() != generated.steps.len()
            {
                return Err(bad("original actual-prefix generation differs"));
            }
            for (step, old) in generated.steps.iter().zip(steps) {
                compare_step(step, old)?;
            }
            parity =
                json!({"status":"PASS","saved_row_file":name,"saved_row_sha256":file_hash(&path)?});
        }
        if let Some(pair) = l.pair_id {
            if pair.is_empty() {
                return Err(bad("empty pair metadata"));
            }
            pair_groups.entry(pair).or_default().push((
                id.clone(),
                accepted,
                generated.generated_ids.clone(),
            ));
        }
        let mut traces = generated
            .steps
            .iter()
            .map(|step| step_row(step, local_query_ids.as_deref(), g.continuation_sha256()))
            .collect::<Result<Vec<Value>>>()?;
        if c.retain_entry_bank_trace {
            let first = generated
                .steps
                .first()
                .ok_or_else(|| bad("entry step absent"))?;
            traces[0]["bank_trace"] = serde_json::to_value(&first.bank_trace)?;
        }
        let mut row = json!({"id":id,"generated_ids":generated.generated_ids,"decoded":decoded,"decoded_utf8_valid":valid_text.is_some(),"decoded_bytes_sha256":hash(&decoded_bytes),"eos":eos,"complete":accepted,"entry_correct":entry_correct,
            "stop":format!("{:?}",generated.stop),"executed_steps":generated.executed_steps,"pin": {"lineage":generated.pin.lineage,"commit":generated.pin.commit,"scope":String::from_utf8(generated.pin.scope)?},"steps":traces,"baseline_parity":parity});
        if let Some(digest) = g.continuation_sha256() {
            row["continuation_sha256"] = json!(digest);
        }
        let filename = format!("row-{index:04}.json");
        write_row(c, &filename, &row)?;
        rows.push(json!({"id":l.id,"row_file":filename,"row_bytes":fs::metadata(c.output.join(&filename))?.len(),"row_sha256":file_hash(&c.output.join(&filename))?,"complete":accepted,"entry_correct":entry_correct}));
    }
    let mut pair_complete = 0;
    let mut pairs = Vec::new();
    for (id, arms) in pair_groups {
        if arms.len() != 2 {
            return Err(bad("each evaluator pair must have two cases"));
        }
        let both = arms.iter().all(|v| v.1);
        let distinct = arms[0].2 != arms[1].2;
        pair_complete += usize::from(both && distinct);
        pairs.push(json!({"pair_id":id,"case_ids":arms.iter().map(|v|&v.0).collect::<Vec<_>>(),"both_complete":both,"outputs_distinct":distinct}));
    }
    let mut report = json!({"schema":"uor-r4.native-bank-generalization/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,
        "model_report_sha256":c.expected_model_report_sha256,"model_manifest_sha256":c.expected_model_manifest_sha256,"model_source_commit":r["source_commit"],"component_provenance":r["resume"],
        "checkpoint_receipt_sha256":file_hash(&cp.join("receipt.json"))?,"source_binding":g.source_binding(),"generate_sha256":g.generate_sha256(),"bridge_sha256":g.bridge_sha256(),"exp_sha256":exp_hash,
        "inputs_sha256":c.expected_inputs_sha256,"labels_sha256":c.expected_labels_sha256,"input_seal_sha256":file_hash(&input_root.join("manifest.json"))?,"label_seal_sha256":file_hash(&label_root.join("manifest.json"))?,
        "config_sha256":file_hash(&c.output.join("config.json"))?,"cases":rows.len(),"complete":complete,"entry_correct":entry,"pairs":pairs,"pairs_both_complete_distinct":pair_complete,"rows":rows,
        "baseline_parity":c.baseline_parity,"runtime":"CPU bounded integer/table native generator; no training/CUDA/optimizer","pin_scope":"Source-metadata-derived authored panel pin, lineage0; no store enumeration/lineage authenticity claim",
        "scope":"frozen source-value substitution evaluation in declared grammar; labels after actual-feedback generation; no general chat/held-out whole-program claim","model_admission":admission["resume"]});
    if c.model_kind == ModelKind::JointContinuation {
        report["model_kind"] = json!("joint_continuation");
        report["checkpoint_step"] = json!(c.checkpoint_step);
        // source_commit/executable_sha256 above identify this consumer;
        // model_source_commit and this receipt identify the separate trainer.
        report["component_provenance"] = r["final_receipt"].clone();
        report["model_admission"] = admission;
        report["scope"] = json!("saved joint checkpoint independent-packet evaluation; labels after actual-feedback generation; no construction qualification, multi-turn chat or held-out whole-program claim");
    }
    if let Some(digest) = g.continuation_sha256() {
        report["continuation_sha256"] = json!(digest);
        report["continuation_seal_sha256"] = json!(file_hash(
            &seal_for(
                c.continuation_field
                    .as_ref()
                    .ok_or_else(|| bad("continuation path absent"))?
            )?
            .join("manifest.json")
        )?);
    }
    Ok(report)
}
fn entry_require(ok: bool, message: &str) -> Result<()> {
    if !ok {
        return Err(bad(message));
    }
    Ok(())
}
fn json_hash<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    Ok(hash(&serde_json::to_vec(value)?))
}
fn saved_entry_row(root: &Path, reference: &Value, id: &str) -> Result<Value> {
    let name = expected_string(&reference["row_file"])?;
    entry_require(
        !Path::new(name)
            .components()
            .any(|p| !matches!(p, std::path::Component::Normal(_))),
        "unsafe saved entry row path",
    )?;
    let path = root.join(name);
    entry_require(
        file_hash(&path)? == expected_string(&reference["row_sha256"])?,
        "saved entry row digest differs",
    )?;
    let row = read(&path)?;
    entry_require(
        row["id"] == id && reference["id"] == id,
        "saved entry row identity/order differs",
    )?;
    Ok(row)
}
/// Compare dimensionless fractions; raw Q31 masses can have different references.
fn normalized_change(
    zero: i128,
    zero_total: u64,
    learned: i128,
    learned_total: u64,
) -> Result<(i128, u128)> {
    entry_require(
        zero_total > 0 && learned_total > 0,
        "zero probability denominator",
    )?;
    let numerator = zero
        .checked_mul(i128::from(learned_total))
        .and_then(|a| {
            learned
                .checked_mul(i128::from(zero_total))
                .and_then(|b| a.checked_sub(b))
        })
        .ok_or_else(|| bad("normalized difference overflow"))?;
    let denominator = u128::from(zero_total)
        .checked_mul(u128::from(learned_total))
        .ok_or_else(|| bad("normalized denominator overflow"))?;
    Ok((numerator, denominator))
}
fn rational(value: (i128, u128)) -> Value {
    json!({"numerator":value.0.to_string(),"denominator":value.1.to_string(),"direction":if value.0>0 {"improved"} else if value.0<0 {"worsened"} else {"equal"}})
}
struct EntryMasses {
    target: VocabularyTokenMass,
    rival: VocabularyTokenMass,
    winner: VocabularyTokenMass,
    total: u64,
    tied_maximum_tokens: usize,
    target_strict_mass_rank: usize,
    target_tiebroken_rank: usize,
}
impl EntryMasses {
    fn margin(&self) -> i128 {
        i128::from(self.target.weight_q31) - i128::from(self.rival.weight_q31)
    }
    fn value(&self) -> Value {
        json!({"target":self.target,"strongest_rival":self.rival,"winner":self.winner,
            "target_minus_rival_mass":self.margin().to_string(),"denominator":self.total,
            "target_probability":{"numerator":self.target.weight_q31,"denominator":self.total},
            "normalized_target_minus_rival":{"numerator":self.margin().to_string(),"denominator":self.total.to_string()},
            "maximum_mass_tie_count":self.tied_maximum_tokens,"smallest_maximum_id":self.winner.token_id,
            "target_strict_mass_rank":self.target_strict_mass_rank,"target_tiebroken_rank":self.target_tiebroken_rank,
            "target_tied_for_maximum":self.target.weight_q31==self.winner.weight_q31 && self.tied_maximum_tokens>1,
            "target_is_selected":self.target.token_id==self.winner.token_id})
    }
}
fn entry_masses(
    masses: &[VocabularyTokenMass],
    target: u32,
    total: u64,
    chosen: u32,
) -> Result<EntryMasses> {
    entry_require(
        masses.len() > 1 && masses.windows(2).all(|v| v[0].token_id < v[1].token_id),
        "token mass support/order differs",
    )?;
    let mut sum = 0u64;
    for m in masses {
        entry_require(
            m.weight_q31 > 0
                && m.generate_weight_q31.checked_add(m.copy_weight_q31) == Some(m.weight_q31),
            "token mass decomposition differs",
        )?;
        sum = sum
            .checked_add(m.weight_q31)
            .ok_or_else(|| bad("token mass sum overflow"))?;
    }
    entry_require(sum == total, "token denominator differs")?;
    let strongest = |exclude: Option<u32>| -> Result<VocabularyTokenMass> {
        masses
            .iter()
            .filter(|m| Some(m.token_id) != exclude)
            .max_by(|a, b| {
                a.weight_q31
                    .cmp(&b.weight_q31)
                    .then_with(|| b.token_id.cmp(&a.token_id))
            })
            .cloned()
            .ok_or_else(|| bad("entry rival/winner absent"))
    };
    let winner = strongest(None)?;
    entry_require(
        winner.token_id == chosen,
        "served winner differs from mass/smallest-ID tie rule",
    )?;
    let target_mass = masses
        .iter()
        .find(|m| m.token_id == target)
        .cloned()
        .ok_or_else(|| bad("entry target outside legal support"))?;
    let target_strict_mass_rank = 1 + masses
        .iter()
        .filter(|m| m.weight_q31 > target_mass.weight_q31)
        .count();
    let target_tiebroken_rank = target_strict_mass_rank
        + masses
            .iter()
            .filter(|m| m.weight_q31 == target_mass.weight_q31 && m.token_id < target)
            .count();
    Ok(EntryMasses {
        target: target_mass,
        target_strict_mass_rank,
        target_tiebroken_rank,
        rival: strongest(Some(target))?,
        tied_maximum_tokens: masses
            .iter()
            .filter(|m| m.weight_q31 == winner.weight_q31)
            .count(),
        winner,
        total,
    })
}
/// Every physical Copy occurrence, including duplicates, gets its token's U.
fn verify_entry_score_deltas(
    ids: &[u32],
    learned_gen: &[i64],
    zero_gen: &[i64],
    learned_copy: &[i64],
    zero_copy: &[i64],
    delta: &[i64],
) -> Result<()> {
    entry_require(
        learned_gen.len() == zero_gen.len()
            && delta.len() == zero_gen.len()
            && ids.len() == learned_copy.len()
            && ids.len() == zero_copy.len(),
        "matched score dimensions differ",
    )?;
    for ((&a, &b), &u) in learned_gen.iter().zip(zero_gen).zip(delta) {
        entry_require(
            i128::from(a) - i128::from(b) == i128::from(u),
            "Generate difference is not learned U",
        )?;
    }
    for ((&id, &a), &b) in ids.iter().zip(learned_copy).zip(zero_copy) {
        let u = *delta
            .get(id as usize)
            .ok_or_else(|| bad("Copy token outside U vocabulary"))?;
        entry_require(
            i128::from(a) - i128::from(b) == i128::from(u),
            "physical Copy difference is not token U",
        )?;
    }
    Ok(())
}
fn bridge_identity(step: &NativeBankGenerateStep) -> Result<Value> {
    let b = step
        .bridge
        .as_ref()
        .ok_or_else(|| bad("entry bridge witness missing"))?;
    Ok(
        json!({"selected_ordinal":b.selected_ordinal,"selected_candidate":b.selected_candidate,
        "query_state_codes":b.query_state.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "source_state_codes":b.source_state.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "action_codes":b.action_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "action_scores_q24":b.action_scores_q24,"counts":b.counts}),
    )
}
fn entry_continuation(step: &NativeBankGenerateStep) -> Result<Value> {
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("entry U witness absent"))?;
    Ok(
        json!({"query_tokens":u.query_tokens,"actual_prefix_tokens":u.actual_prefix_tokens,
        "state_codes":u.state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "delta_scores_q24_sha256":json_hash(&u.delta_scores_q24)?,"delta_scores":u.delta_scores_q24.len(),
        "minimum_delta_q24":u.delta_scores_q24.iter().min(),"maximum_delta_q24":u.delta_scores_q24.iter().max(),
        "encoding_coefficient_reads":u.encoding_coefficient_reads,"field_counts":u.counts}),
    )
}
fn matched_entry_geometry(
    a: &NativeBankGenerateStep,
    b: &NativeBankGenerateStep,
    query_tokens: usize,
) -> Result<Value> {
    entry_require(
        a.actual_prefix_ids.is_empty() && b.actual_prefix_ids.is_empty(),
        "entry prefix is not empty",
    )?;
    entry_require(
        a.bank_trace.is_some() && a.bank_trace == b.bank_trace,
        "factual bank replay differs across U arms",
    )?;
    entry_require(
        a.post_state == b.post_state && a.copy_token_ids == b.copy_token_ids,
        "post-state or physical Copy IDs differ",
    )?;
    let bridge = bridge_identity(a)?;
    entry_require(
        bridge == bridge_identity(b)?,
        "factual selected occurrence/bridge differs",
    )?;
    entry_require(
        serde_json::to_value(a.generate_counts)? == serde_json::to_value(b.generate_counts)?,
        "Generate work differs",
    )?;
    entry_require(
        a.actions.policy == b.actions.policy
            && a.actions.tokenizer_sha256 == b.actions.tokenizer_sha256
            && a.actions.eos_token_id == b.actions.eos_token_id
            && a.actions.period_token_id == b.actions.period_token_id,
        "reducer identities differ",
    )?;
    let u = a
        .continuation
        .as_ref()
        .ok_or_else(|| bad("learned U witness absent"))?;
    let z = b
        .continuation
        .as_ref()
        .ok_or_else(|| bad("zero U witness absent"))?;
    entry_require(
        u.state_codes == z.state_codes
            && u.query_tokens == query_tokens
            && z.query_tokens == query_tokens
            && u.actual_prefix_tokens == 0
            && z.actual_prefix_tokens == 0
            && u.encoding_coefficient_reads == z.encoding_coefficient_reads
            && u.counts == z.counts
            && u.delta_scores_q24.len() == 4096
            && z.delta_scores_q24.len() == 4096
            && z.delta_scores_q24.iter().all(|&v| v == 0),
        "local carrier differs or control U is nonzero",
    )?;
    for s in [a, b] {
        entry_require(
            s.actions.summary.legal_generate_actions == 4096
                && s.generate_raw_scores_q24.len() == 4096
                && s.actions.token_masses.len() == 4096
                && s.actions
                    .token_masses
                    .iter()
                    .enumerate()
                    .all(|(id, mass)| mass.token_id == id as u32)
                && s.copy_raw_scores_q24.len() == s.copy_token_ids.len()
                && s.actions.actions.len() == 4096 + s.copy_token_ids.len()
                && s.actions.summary.copy_actions == s.copy_token_ids.len(),
            "entry full support differs",
        )?;
        for (offset, action) in s.actions.actions.iter().enumerate() {
            let (kind, id, raw) = if offset < 4096 {
                (
                    VocabularyAction::Generate {
                        token_id: offset as u32,
                    },
                    offset as u32,
                    s.generate_raw_scores_q24[offset],
                )
            } else {
                let i = offset - 4096;
                (
                    VocabularyAction::Copy { source_offset: i },
                    s.copy_token_ids[i],
                    s.copy_raw_scores_q24[i],
                )
            };
            entry_require(
                action.action == kind
                    && action.action_offset == offset
                    && action.token_id == id
                    && action.raw_score_q24 == raw
                    && action.score_q24 == raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24),
                "ordered physical action identity/raw score differs",
            )?;
        }
    }
    entry_require(
        a.actions
            .actions
            .iter()
            .zip(&b.actions.actions)
            .all(|(x, y)| {
                x.action == y.action
                    && x.action_offset == y.action_offset
                    && x.token_id == y.token_id
            }),
        "action identity differs across arms",
    )?;
    let mut compact_bridge = bridge;
    let scores = compact_bridge
        .as_object_mut()
        .ok_or_else(|| bad("bridge object absent"))?
        .remove("action_scores_q24")
        .ok_or_else(|| bad("bridge score witness absent"))?;
    compact_bridge["action_scores_q24_sha256"] = json!(json_hash(&scores)?);
    Ok(
        json!({"factual_replay_equal":true,"factual_replay_sha256":json_hash(&a.bank_trace)?,
        "bridge_equal":true,"bridge":compact_bridge,"post_state_codes":a.post_state.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "local_carrier_equal":true,"local_state_codes":u.state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),
        "copy_ids_sha256":json_hash(&a.copy_token_ids)?,"physical_copy_occurrences":a.copy_token_ids.len(),
        "ordered_alias_identity_sha256":json_hash(&a.actions.actions.iter().map(|x|(&x.action,x.action_offset,x.token_id)).collect::<Vec<_>>())?,
        "zero_delta_exact":true,"generate_reads":a.generate_counts}),
    )
}
fn matched_entry_invariants(
    a: &NativeBankGenerateStep,
    b: &NativeBankGenerateStep,
    query_tokens: usize,
) -> Result<Value> {
    let mut result = matched_entry_geometry(a, b, query_tokens)?;
    let u = a
        .continuation
        .as_ref()
        .ok_or_else(|| bad("learned U witness absent"))?;
    verify_entry_score_deltas(
        &a.copy_token_ids,
        &a.generate_raw_scores_q24,
        &b.generate_raw_scores_q24,
        &a.copy_raw_scores_q24,
        &b.copy_raw_scores_q24,
        &u.delta_scores_q24,
    )?;
    result["all_score_differences_equal_token_u"] = json!(true);
    Ok(result)
}
fn entry_arm(step: &NativeBankGenerateStep, target: u32) -> Result<(EntryMasses, Value)> {
    let masses = entry_masses(
        &step.actions.token_masses,
        target,
        step.actions.summary.total_weight_q31,
        step.actions.summary.chosen_token_id,
    )?;
    entry_require(
        masses.winner.weight_q31 == step.actions.summary.chosen_weight_q31,
        "winner mass differs from reducer summary",
    )?;
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("entry U witness absent"))?;
    let get_score = |scores: &[i64], id: u32| -> Result<i64> {
        scores
            .get(id as usize)
            .copied()
            .ok_or_else(|| bad("entry score token outside vocabulary"))
    };
    let result = json!({"pool_summary":step.actions.summary,"masses":masses.value(),
        "target_generate_preclip_q24":get_score(&step.generate_raw_scores_q24,target)?,
        "strongest_rival_generate_preclip_q24":get_score(&step.generate_raw_scores_q24,masses.rival.token_id)?,
        "target_u_q24":get_score(&u.delta_scores_q24,target)?,
        "strongest_rival_u_q24":get_score(&u.delta_scores_q24,masses.rival.token_id)?,
        "target_physical_copy_aliases":step.copy_token_ids.iter().filter(|&&id|id==target).count(),
        "generate_preclip_sha256":json_hash(&step.generate_raw_scores_q24)?,"copy_preclip_sha256":json_hash(&step.copy_raw_scores_q24)?,
        "clipped_action_scores_sha256":json_hash(&step.actions.actions.iter().map(|a|a.score_q24).collect::<Vec<_>>())?,
        "token_masses_sha256":json_hash(&step.actions.token_masses)?,"continuation":entry_continuation(step)?});
    Ok((masses, result))
}
fn verify_saved_learned_entry(
    step: &NativeBankGenerateStep,
    saved: &Value,
    target: u32,
    target_mass: u64,
) -> Result<()> {
    let entry = &saved["canonical"][0];
    let native = &entry["native"];
    entry_require(
        saved["canonical_target_ids_labels_only"][0] == target
            && entry["target_label_only"] == target,
        "saved canonical entry target differs",
    )?;
    entry_require(
        native["pool"]["summary"] == serde_json::to_value(&step.actions.summary)?
            && native["post_state_codes"]
                == json!(step
                    .post_state
                    .iter()
                    .map(|c| c.index())
                    .collect::<Vec<_>>())
            && native["copy_token_ids"] == json!(step.copy_token_ids)
            && native["generate_raw_scores_sha256"] == json_hash(&step.generate_raw_scores_q24)?
            && entry["native_target_mass"] == target_mass
            && entry["native_denominator"] == step.actions.summary.total_weight_q31,
        "learned entry differs from saved final native result",
    )?;
    let witness = entry_continuation(step)?;
    for (key, value) in witness
        .as_object()
        .ok_or_else(|| bad("U witness object absent"))?
    {
        entry_require(
            native["continuation"][key] == *value,
            "learned local U witness differs from saved canonical entry",
        )?;
    }
    Ok(())
}
fn run_entry_zero_u(c: &Config) -> Result<Value> {
    let started = std::time::Instant::now();
    admit_model_options(c)?;
    entry_require(
        c.evaluation_kind == EvaluationKind::EntryZeroUV2,
        "explicit entry diagnostic mode required",
    )?;
    // Ordinary joint admission authenticates the genuine learned U first.
    let model = load_joint_model(c)?;
    let r = &model.report;
    let cp = &model.checkpoint;
    let receipt = read(&cp.join("receipt.json"))?;
    let learned_bytes = bytes(
        c.continuation_field
            .as_ref()
            .ok_or_else(|| bad("learned U path absent"))?,
    )?;
    let learned = NativeContinuationField::from_bytes(
        &learned_bytes,
        model.generator.source_binding(),
        model.generator.generate_model(),
    )?;
    entry_require(
        learned.applies_to_copy()
            && learned.score_shift() == 22
            && learned.packed_unary().iter().any(|&v| v != 0),
        "final64 learned U is not nonzero shared-action v2",
    )?;
    let zero = NativeContinuationField::compile_shared_action(
        model.generator.source_binding(),
        model.generator.generate_model(),
        &vec![0; model.generator.generate_model().lanes() * 60],
    )?;
    let zero_bytes = zero.to_bytes()?;
    // Derived bytes live only in this new diagnostic root, never the model seal.
    fs::write(
        c.output.join("learned-continuation-field.bin"),
        &learned_bytes,
    )?;
    fs::write(c.output.join("zero-continuation-field.bin"), &zero_bytes)?;
    let learned = NativeContinuationField::from_bytes(
        &bytes(&c.output.join("learned-continuation-field.bin"))?,
        model.generator.source_binding(),
        model.generator.generate_model(),
    )?;
    let zero = NativeContinuationField::from_bytes(
        &bytes(&c.output.join("zero-continuation-field.bin"))?,
        model.generator.source_binding(),
        model.generator.generate_model(),
    )?;
    entry_require(
        learned.to_bytes()? == learned_bytes
            && zero.to_bytes()? == zero_bytes
            && zero.packed_unary().iter().all(|&v| v == 0)
            && zero.applies_to_copy()
            && zero.score_shift() == 22,
        "same-v2 field independent roundtrip/zero check differs",
    )?;
    let mut matched_metadata = zero.metadata().clone();
    matched_metadata.payload_sha256 = learned.metadata().payload_sha256.clone();
    entry_require(
        &matched_metadata == learned.metadata(),
        "zero intervention changed nonpayload field metadata",
    )?;
    let zero_sha = hash(&zero_bytes);
    let fields = json!({"learned_sha256":hash(&learned_bytes),"zero_sha256":zero_sha,"learned_metadata":learned.metadata(),"zero_metadata":zero.metadata(),
        "same_metadata_except_payload_sha256":true,"independently_reloaded":true,"zero_payload_exact":true,"source":"derived v2 control after authentic joint admission; original learned artifact unchanged"});
    write_row(c, "field-receipt.json", &fields)?;
    let second = load_checkpoint(
        cp.clone(),
        r.clone(),
        model.admission.clone(),
        receipt.clone(),
    )?;
    entry_require(
        model.generator.source_binding() == second.generator.source_binding()
            && model.generator.generate_sha256() == second.generator.generate_sha256()
            && model.generator.bridge_sha256() == second.generator.bridge_sha256()
            && model.generator.generate_model().prototypes()
                == second.generator.generate_model().prototypes(),
        "two native generators have different upstream artifacts",
    )?;
    let mut learned_generator = model.generator.with_continuation_field(BoundNativeBytes {
        bytes: &learned_bytes,
        sha256: ENTRY_U_SHA,
    })?;
    let mut zero_generator = second.generator.with_continuation_field(BoundNativeBytes {
        bytes: &zero_bytes,
        sha256: &zero_sha,
    })?;
    let input_seal = seal_for(&c.inputs)?;
    let label_seal = seal_for(&c.labels)?;
    report_output::verify(&input_seal)?;
    report_output::verify(&label_seal)?;
    entry_require(
        file_hash(&c.inputs)? == ENTRY_INPUT_SHA && file_hash(&c.labels)? == ENTRY_LABEL_SHA,
        "exact512 entry panel identity differs",
    )?;
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    let labels: Labels = serde_json::from_slice(&bytes(&c.labels)?)?;
    entry_require(
        panel.schema == "uor-r4.native-source-bank-probe-input/1"
            && labels.schema == "uor-r4.native-source-bank-labels/1"
            && labels.protocol == "uor-r4.literal-role-dialogue/2"
            && labels.membership_only
            && panel.cases.len() == 512
            && labels.cases.len() == 512,
        "entry diagnostic requires complete original512 schema",
    )?;
    let final_summary = read(&c.model_root.join("development-0064.json"))?;
    let initial_summary = read(&c.model_root.join("development-0000.json"))?;
    entry_require(
        final_summary == r["final_evaluation"] && initial_summary == r["initial_evaluation"],
        "saved entry summaries differ from authenticated report",
    )?;
    let final_refs = final_summary["rows"]
        .as_array()
        .ok_or_else(|| bad("final row references absent"))?;
    let initial_refs = initial_summary["rows"]
        .as_array()
        .ok_or_else(|| bad("initial row references absent"))?;
    entry_require(
        final_refs.len() == 512 && initial_refs.len() == 512,
        "saved entry population differs",
    )?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes(
        &model.native_directory.join("tokenizer.json"),
    )?)
    .ok_or_else(|| bad("entry ByteBPE unavailable"))?;
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut transitions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut probability_changes = BTreeMap::<String, usize>::new();
    let mut margin_changes = BTreeMap::<String, usize>::new();
    let mut parent_rows = Vec::new();
    let mut learned_correct = 0usize;
    let mut zero_correct = 0usize;
    let mut winners_changed = Vec::new();
    let mut clipped_winner_changes = [0usize; 2];
    let mut forward_seconds = 0.;
    for (index, (packet, label)) in panel.cases.into_iter().zip(labels.cases).enumerate() {
        let id = packet.id.clone();
        let query_tokens = packet.query_ids.len();
        let packet_sha = json_hash(&packet)?;
        entry_require(
            !id.is_empty()
                && seen.insert(id.clone())
                && label.id == id
                && final_refs[index]["id"] == id
                && initial_refs[index]["id"] == id,
            "entry packet/label/saved identities differ",
        )?;
        let bank = learned_generator.admit_bank(snapshot(packet)?)?;
        let clock = std::time::Instant::now();
        let learned_step = learned_generator.step(&bank, &[])?;
        let zero_step = zero_generator.step(&bank, &[])?;
        forward_seconds += clock.elapsed().as_secs_f64();
        let invariants = matched_entry_invariants(&learned_step, &zero_step, query_tokens)?;
        // Only after BOTH target-free forwards may labels define the metric.
        label.answers.validate()?;
        let accepted = label
            .answers
            .accepted
            .first()
            .ok_or_else(|| bad("entry accepted form absent"))?;
        let target_ids = tok.encode(accepted);
        entry_require(
            tok.decode_bytes(&target_ids) == accepted.as_bytes(),
            "entry accepted form tokenizer roundtrip differs",
        )?;
        let target = *target_ids
            .first()
            .ok_or_else(|| bad("entry target empty"))?;
        let (learned_mass, learned_value) = entry_arm(&learned_step, target)?;
        let (zero_mass, zero_value) = entry_arm(&zero_step, target)?;
        let saved = saved_entry_row(&c.model_root, &final_refs[index], &id)?;
        verify_saved_learned_entry(
            &learned_step,
            &saved,
            target,
            learned_mass.target.weight_q31,
        )?;
        let lp = learned_mass.winner.token_id == target;
        let zp = zero_mass.winner.token_id == target;
        learned_correct += usize::from(lp);
        zero_correct += usize::from(zp);
        let transition = match (lp, zp) {
            (true, true) => "both_correct",
            (true, false) => "correct_to_wrong",
            (false, true) => "wrong_to_correct",
            (false, false) => "both_wrong",
        };
        transitions
            .entry(transition.into())
            .or_default()
            .push(id.clone());
        if learned_mass.winner.token_id != zero_mass.winner.token_id {
            winners_changed.push(id.clone());
        }
        let probability = normalized_change(
            i128::from(zero_mass.target.weight_q31),
            zero_mass.total,
            i128::from(learned_mass.target.weight_q31),
            learned_mass.total,
        )?;
        let margin = normalized_change(
            zero_mass.margin(),
            zero_mass.total,
            learned_mass.margin(),
            learned_mass.total,
        )?;
        let direction = |n: i128| {
            if n > 0 {
                "improved"
            } else if n < 0 {
                "worsened"
            } else {
                "equal"
            }
        };
        *probability_changes
            .entry(direction(probability.0).into())
            .or_default() += 1;
        *margin_changes
            .entry(direction(margin.0).into())
            .or_default() += 1;
        clipped_winner_changes[0] +=
            usize::from(learned_step.actions.summary.token_winner_changed_by_clip);
        clipped_winner_changes[1] +=
            usize::from(zero_step.actions.summary.token_winner_changed_by_clip);
        let mut parent = Value::Null;
        if ENTRY_ORIGINAL_EIGHT.contains(&index) {
            let old = saved_entry_row(&c.model_root, &initial_refs[index], &id)?;
            entry_require(
                old["canonical_target_ids_labels_only"][0] == target
                    && old["canonical"][0]["target_label_only"] == target
                    && old["canonical"][0]["native"]["pool"]["summary"]["chosen_token_id"]
                        == target,
                "original8 saved parent entry is not correct",
            )?;
            parent = json!({"index":index,"id":id,"saved_parent_row_sha256":initial_refs[index]["row_sha256"],"parent_entry_correct":true,"learned_entry_retained":lp,"zero_entry_retained":zp});
            parent_rows.push(parent.clone());
        }
        let filename = format!("entry-row-{index:04}.json");
        let row = json!({"index":index,"id":id,"packet_serde_sha256":packet_sha,"target_definition":"first token of first accepted answer, matching existing entry metric; labels after both forwards",
            "target_token_id":target,"saved_final_row_sha256":final_refs[index]["row_sha256"],"learned_saved_entry_parity":true,
            "invariants":invariants,"learned":learned_value,"zero":zero_value,"zero_minus_learned_target_probability":rational(probability),
            "zero_minus_learned_normalized_margin":rational(margin),"transition":transition,"original8_parent_entry":parent});
        write_row(c, &filename, &row)?;
        rows.push(json!({"index":index,"id":id,"row_file":filename,"row_sha256":file_hash(&c.output.join(&filename))?,"learned_correct":lp,"zero_correct":zp}));
    }
    entry_require(
        rows.len() == 512 && parent_rows.len() == 8,
        "diagnostic coverage incomplete",
    )?;
    let report = json!({"schema":"uor-r4.native-bank-entry-zero-u-v2/1","status":"COMPLETED","evaluation_kind":"entry_zero_u_v2",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,"model_source_commit":r["source_commit"],
        "model_report_sha256":ENTRY_REPORT_SHA,"model_manifest_sha256":ENTRY_MANIFEST_SHA,"checkpoint_step":64,"checkpoint_receipt_sha256":file_hash(&cp.join("receipt.json"))?,
        "source_binding":learned_generator.source_binding(),"generate_sha256":learned_generator.generate_sha256(),"bridge_sha256":learned_generator.bridge_sha256(),"exp_sha256":model.exp_sha256,
        "fields":fields,"inputs_sha256":ENTRY_INPUT_SHA,"labels_sha256":ENTRY_LABEL_SHA,"input_seal_sha256":file_hash(&input_seal.join("manifest.json"))?,"label_seal_sha256":file_hash(&label_seal.join("manifest.json"))?,
        "config_sha256":file_hash(&c.output.join("config.json"))?,"cases":512,"native_entry_calls":1024,"learned_saved_entry_parity_cases":512,
        "learned_entry_correct":learned_correct,"zero_entry_correct":zero_correct,"transitions":transitions,"winner_changed_ids":winners_changed,
        "target_probability_changes":probability_changes,"normalized_margin_changes":margin_changes,"clip_changed_winners":{"learned":clipped_winner_changes[0],"zero":clipped_winner_changes[1]},
        "original8_parent_entry":parent_rows,
        "original8_entry_retention":{"learned_retained_ids":parent_rows.iter().filter(|p|p["learned_entry_retained"]==true).map(|p|&p["id"]).collect::<Vec<_>>(),
            "learned_lost_ids":parent_rows.iter().filter(|p|p["learned_entry_retained"]==false).map(|p|&p["id"]).collect::<Vec<_>>(),
            "zero_retained_ids":parent_rows.iter().filter(|p|p["zero_entry_retained"]==true).map(|p|&p["id"]).collect::<Vec<_>>(),
            "zero_lost_ids":parent_rows.iter().filter(|p|p["zero_entry_retained"]==false).map(|p|&p["id"]).collect::<Vec<_>>()},
        "rows":rows,"native_forward_seconds":forward_seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "hash_encoding":"SHA256 of compact serde_json::to_vec; original data and artifact hashes bind exact file bytes",
        "raw_diagnostic_scope":"existing tail-limited raw-score reducer diagnostics, not the served common-clipped token selection",
        "interpretation":"conditional U effect on final coadapted Source/Generate; upstream family attribution unresolved; no interaction term identified",
        "scope":"all512 exposed empty-prefix native decisions only; exact same-v2 zero intervention; no fit, autoregressive rerun, EOS/complete-reply or transfer/chat qualification"});
    entry_require(
        serde_json::to_vec_pretty(&report)?.len() <= 512 << 10,
        "compact report exceeds reserved final-summary allowance",
    )?;
    Ok(report)
}

/// Exact file inventories constrain the substitution to native Context/Potential.
/// Source-bound metadata remains with its donor and is validated by the loader.
fn component_files(root: &Path) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, at: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
        for item in fs::read_dir(at)? {
            let item = item?;
            let path = item.path();
            let kind = item.file_type()?;
            entry_require(!kind.is_symlink(), "component tree contains symlink")?;
            if kind.is_dir() {
                visit(root, &path, out)?;
            } else {
                entry_require(kind.is_file(), "component tree contains non-file")?;
                let relative = path
                    .strip_prefix(root)?
                    .to_str()
                    .ok_or_else(|| bad("non-UTF8 component path"))?
                    .to_owned();
                out.insert(relative, file_hash(&path)?);
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out)?;
    Ok(out)
}
fn component_cue_equal(a: &CueCarrierMetadata, b: &CueCarrierMetadata) -> bool {
    let mut normalized = b.clone();
    normalized.parent_artifact = a.parent_artifact.clone();
    normalized.context_packed_sha256 = a.context_packed_sha256.clone();
    &normalized == a
}
fn component_frozen_inventory(
    a: &BTreeMap<String, String>,
    b: &BTreeMap<String, String>,
) -> Result<()> {
    entry_require(a.keys().eq(b.keys()), "native donor file set differs")?;
    for (name, sha) in a {
        if !matches!(
            name.as_str(),
            "metadata.json"
                | "consumer/metadata.json"
                | "consumer/context-q4.bin"
                | "consumer/potential-q4.bin"
        ) {
            entry_require(b.get(name) == Some(sha), "frozen native component differs")?;
        }
    }
    Ok(())
}
fn component_donor_invariants(a: &FrozenModel, b: &FrozenModel) -> Result<Value> {
    let left = component_files(&a.native_directory)?;
    let right = component_files(&b.native_directory)?;
    component_frozen_inventory(&left, &right)?;
    let mut gm = b.generator.generate_model().metadata().clone();
    gm.payload_sha256 = a
        .generator
        .generate_model()
        .metadata()
        .payload_sha256
        .clone();
    entry_require(
        &gm == a.generator.generate_model().metadata()
            && a.generator.generate_model().prototypes()
                == b.generator.generate_model().prototypes(),
        "Generate metadata/prototypes changed beyond coefficients",
    )?;
    entry_require(
        a.generator.generate_model().energy().edges()
            == b.generator.generate_model().energy().edges(),
        "Generate ordered factor topology differs",
    )?;
    let mut sidecars = BTreeMap::new();
    for name in [
        "read-state-bridge.bin",
        "read-state-bridge-categorical.bin",
        "cue/cue-q4.bin",
        "prefix/prefix-q4.bin",
        "cue/cue-joint-q4.bin",
    ] {
        let x = a.checkpoint.join(name);
        let y = b.checkpoint.join(name);
        entry_require(
            x.is_file() == y.is_file(),
            "numeric sidecar presence differs",
        )?;
        if x.is_file() {
            let sha = file_hash(&x)?;
            entry_require(sha == file_hash(&y)?, "frozen numeric sidecar differs")?;
            sidecars.insert(name, sha);
        } else {
            entry_require(
                name == "cue/cue-joint-q4.bin",
                "required frozen sidecar absent",
            )?;
        }
    }
    let ac: CueCarrierMetadata =
        serde_json::from_value(read(&a.checkpoint.join("cue/native-metadata.json"))?)?;
    let bc: CueCarrierMetadata =
        serde_json::from_value(read(&b.checkpoint.join("cue/native-metadata.json"))?)?;
    entry_require(
        component_cue_equal(&ac, &bc),
        "cue config or frozen payload binding differs",
    )?;
    let ap: PrefixTransportMetadata =
        serde_json::from_value(read(&a.checkpoint.join("prefix/native-metadata.json"))?)?;
    let mut bp: PrefixTransportMetadata =
        serde_json::from_value(read(&b.checkpoint.join("prefix/native-metadata.json"))?)?;
    entry_require(
        component_cue_equal(&ap.frozen_cue, &bp.frozen_cue),
        "prefix frozen cue differs",
    )?;
    bp.parent_artifact = ap.parent_artifact.clone();
    bp.context_packed_sha256 = ap.context_packed_sha256.clone();
    bp.frozen_cue = ap.frozen_cue.clone();
    entry_require(ap == bp, "prefix config or frozen payload binding differs")?;
    Ok(
        json!({"source0_native_files":left,"source1_native_files":right,"frozen_numeric_sidecars":sidecars,
        "generate_prototypes_sha256":json_hash(&a.generator.generate_model().prototypes())?,
        "generate_metadata_equal_except_payload":true,"source_sidecar_configs_equal_except_source_bindings":true}),
    )
}
fn component_loss_pattern(source_only_correct: bool, generate_only_correct: bool) -> &'static str {
    match (source_only_correct, generate_only_correct) {
        (false, true) => "source_alone_sufficient",
        (true, false) => "generate_alone_sufficient",
        (false, false) => "either_alone_sufficient",
        (true, true) => "combination_specific",
    }
}
fn component_fixed_source(
    a: &NativeBankGenerateStep,
    b: &NativeBankGenerateStep,
    query_tokens: usize,
) -> Result<Value> {
    let mut invariant = matched_entry_geometry(a, b, query_tokens)?;
    entry_require(
        a.continuation
            .as_ref()
            .is_some_and(|u| u.delta_scores_q24.iter().all(|&v| v == 0))
            && a.copy_raw_scores_q24 == b.copy_raw_scores_q24,
        "fixed Source Copy scores changed or zero U was nonzero",
    )?;
    invariant["copy_preclip_equal"] = json!(true);
    invariant["copy_preclip_sha256"] = json!(json_hash(&a.copy_raw_scores_q24)?);
    Ok(invariant)
}
fn run_component_cross_entry(c: &Config) -> Result<Value> {
    let started = std::time::Instant::now();
    admit_model_options(c)?;
    entry_require(
        c.evaluation_kind == EvaluationKind::ComponentCrossEntry,
        "explicit component mode required",
    )?;
    let donor = c
        .component_parent
        .as_ref()
        .ok_or_else(|| bad("component parent absent"))?;
    let joint = load_joint_model(c)?;
    let parent = load_frozen_model(
        &donor.model_root,
        &donor.expected_model_report_sha256,
        &donor.expected_model_manifest_sha256,
    )?;
    entry_require(
        file_hash(&parent.checkpoint.join("receipt.json"))?
            == donor.expected_checkpoint_receipt_sha256,
        "component parent checkpoint receipt differs",
    )?;
    let parent_receipt = read(&parent.checkpoint.join("receipt.json"))?;
    let joint_receipt = read(&joint.checkpoint.join("receipt.json"))?;
    entry_require(
        joint_receipt["parent_report_sha256"] == COMPONENT_PARENT_REPORT_SHA
            && joint_receipt["parent_manifest_sha256"] == COMPONENT_PARENT_MANIFEST_SHA,
        "joint1 parent provenance differs",
    )?;
    let initial_cp = c.model_root.join("checkpoint-0000");
    let initial_receipt = read(&initial_cp.join("receipt.json"))?;
    entry_require(
        initial_receipt == joint.report["initial_receipt"]
            && initial_receipt["step"] == 0
            && initial_receipt["mode"] == "joint_continuation"
            && initial_receipt["native_independently_reloaded"] == true
            && initial_receipt["masters_independently_reloaded"] == true
            && initial_receipt["parent"] == parent_receipt["parent"]
            && initial_receipt["generate_sha256"] == parent_receipt["generate_sha256"],
        "joint initial is not the authenticated parent",
    )?;
    entry_require(
        component_files(&initial_cp.join("native"))? == component_files(&parent.native_directory)?,
        "joint initial native files differ from parent",
    )?;
    for name in [
        "generate.bin",
        "read-state-bridge.bin",
        "read-state-bridge-categorical.bin",
        "cue/native-metadata.json",
        "cue/cue-q4.bin",
        "prefix/native-metadata.json",
        "prefix/prefix-q4.bin",
        "cue/cue-joint-q4.bin",
    ] {
        let a = initial_cp.join(name);
        let b = parent.checkpoint.join(name);
        entry_require(
            a.is_file() == b.is_file(),
            "initial-parent sidecar presence differs",
        )?;
        if a.is_file() {
            entry_require(
                bytes(&a)? == bytes(&b)?,
                "initial-parent component bytes differ",
            )?;
        }
    }
    let frozen = component_donor_invariants(&parent, &joint)?;
    let cps = [&parent.checkpoint, &joint.checkpoint];
    let receipts = [&parent_receipt, &joint_receipt];
    let generate_bytes = [
        bytes(&cps[0].join("generate.bin"))?,
        bytes(&cps[1].join("generate.bin"))?,
    ];
    let mut generators = Vec::new();
    let mut arm_receipts = Vec::new();
    // Arm order is S0G0,S0G1,S1G0,S1G1. Each field binds its actual pair.
    for source in 0..2 {
        for generate in 0..2 {
            let gen_sha = expected_string(&receipts[generate]["generate_sha256"])?;
            entry_require(
                hash(&generate_bytes[generate]) == gen_sha,
                "Generate donor bytes differ",
            )?;
            let (g, _) = load_native_components(
                cps[source],
                receipts[source],
                BoundNativeBytes {
                    bytes: &generate_bytes[generate],
                    sha256: gen_sha,
                },
            )?;
            let field = NativeContinuationField::compile_shared_action(
                g.source_binding(),
                g.generate_model(),
                &vec![0; g.generate_model().lanes() * 60],
            )?;
            let encoded = field.to_bytes()?;
            let name = format!("s{source}g{generate}-zero-v2.bin");
            fs::write(c.output.join(&name), &encoded)?;
            let reloaded_bytes = bytes(&c.output.join(&name))?;
            let reloaded = NativeContinuationField::from_bytes(
                &reloaded_bytes,
                g.source_binding(),
                g.generate_model(),
            )?;
            entry_require(
                reloaded.to_bytes()? == encoded
                    && reloaded.packed_unary().iter().all(|&v| v == 0)
                    && reloaded.applies_to_copy()
                    && reloaded.score_shift() == 22,
                "component zero-v2 roundtrip differs",
            )?;
            if source == generate {
                let original_cp = if source == 0 {
                    &initial_cp
                } else {
                    &joint.checkpoint
                };
                let original = bytes(&original_cp.join("continuation-field.bin"))?;
                let expected = if source == 0 {
                    &initial_receipt
                } else {
                    &joint_receipt
                };
                entry_require(
                    hash(&original) == expected_string(&expected["continuation_sha256"])?
                        && original == encoded,
                    "diagonal zero-v2 differs from genuine saved field",
                )?;
            }
            let field_sha = hash(&encoded);
            arm_receipts.push(json!({"arm":format!("s{source}g{generate}"),"source_donor":source,"generate_donor":generate,
                "source_binding":g.source_binding(),"source_receipt_sha256":file_hash(&cps[source].join("receipt.json"))?,
                "generate_sha256":gen_sha,"bridge_sha256":g.bridge_sha256(),"zero_field_file":name,"zero_field_sha256":field_sha,"zero_field_metadata":reloaded.metadata(),
                "generate_exact_donor_bytes":true,"zero_independently_reloaded":true,"derived_unaccepted_diagnostic":source!=generate}));
            generators.push(g.with_continuation_field(BoundNativeBytes {
                bytes: &reloaded_bytes,
                sha256: &field_sha,
            })?);
        }
    }
    write_row(
        c,
        "component-receipt.json",
        &json!({"parent":donor,"joint_report_sha256":COMPONENT_REPORT_SHA,"joint_manifest_sha256":COMPONENT_MANIFEST_SHA,
        "joint_initial_receipt_sha256":file_hash(&initial_cp.join("receipt.json"))?,"joint_initial_exact_parent":true,"frozen":frozen,"arms":arm_receipts}),
    )?;
    let input_seal = seal_for(&c.inputs)?;
    let label_seal = seal_for(&c.labels)?;
    report_output::verify(&input_seal)?;
    report_output::verify(&label_seal)?;
    entry_require(
        file_hash(&c.inputs)? == ENTRY_INPUT_SHA && file_hash(&c.labels)? == ENTRY_LABEL_SHA,
        "component512 data differs",
    )?;
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    let labels: Labels = serde_json::from_slice(&bytes(&c.labels)?)?;
    entry_require(
        panel.schema == "uor-r4.native-source-bank-probe-input/1"
            && labels.schema == "uor-r4.native-source-bank-labels/1"
            && labels.protocol == "uor-r4.literal-role-dialogue/2"
            && labels.membership_only
            && panel.cases.len() == 512
            && labels.cases.len() == 512,
        "component diagnostic requires complete original512",
    )?;
    let summaries = [
        read(&c.model_root.join("development-0000.json"))?,
        read(&c.model_root.join("development-0001.json"))?,
    ];
    entry_require(
        summaries[0] == joint.report["initial_evaluation"]
            && summaries[1] == joint.report["final_evaluation"],
        "diagonal summaries differ",
    )?;
    let refs = [
        summaries[0]["rows"]
            .as_array()
            .ok_or_else(|| bad("initial rows absent"))?,
        summaries[1]["rows"]
            .as_array()
            .ok_or_else(|| bad("final rows absent"))?,
    ];
    entry_require(
        refs.iter().all(|r| r.len() == 512),
        "diagonal population differs",
    )?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes(
        &parent.native_directory.join("tokenizer.json"),
    )?)
    .ok_or_else(|| bad("entry tokenizer absent"))?;
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut original = Vec::new();
    let mut patterns = BTreeMap::<String, Vec<String>>::new();
    let mut correct: [Vec<String>; 4] = std::array::from_fn(|_| Vec::new());
    let mut remaining_correct: [Vec<String>; 4] = std::array::from_fn(|_| Vec::new());
    let mut forward_seconds = 0.;
    for (index, (packet, label)) in panel.cases.into_iter().zip(labels.cases).enumerate() {
        let id = packet.id.clone();
        let query_tokens = packet.query_ids.len();
        let packet_sha = json_hash(&packet)?;
        entry_require(
            !id.is_empty()
                && seen.insert(id.clone())
                && label.id == id
                && refs.iter().all(|r| r[index]["id"] == id),
            "component packet/label/diagonal IDs differ",
        )?;
        let bank0 = generators[0].admit_bank(snapshot(packet.clone())?)?;
        let bank1 = generators[2].admit_bank(snapshot(packet)?)?;
        let clock = std::time::Instant::now();
        let steps = [
            generators[0].step(&bank0, &[])?,
            generators[1].step(&bank0, &[])?,
            generators[2].step(&bank1, &[])?,
            generators[3].step(&bank1, &[])?,
        ];
        forward_seconds += clock.elapsed().as_secs_f64();
        let invariants = [
            component_fixed_source(&steps[0], &steps[1], query_tokens)?,
            component_fixed_source(&steps[2], &steps[3], query_tokens)?,
        ];
        // ALL FOUR target-free native calls precede label-derived targets.
        label.answers.validate()?;
        let accepted = label
            .answers
            .accepted
            .first()
            .ok_or_else(|| bad("entry accepted form absent"))?;
        let target_ids = tok.encode(accepted);
        entry_require(
            tok.decode_bytes(&target_ids) == accepted.as_bytes(),
            "entry tokenizer roundtrip differs",
        )?;
        let target = *target_ids
            .first()
            .ok_or_else(|| bad("entry target absent"))?;
        let arms = steps
            .iter()
            .map(|s| entry_arm(s, target))
            .collect::<Result<Vec<_>>>()?;
        for (diagonal, arm) in [(0, 0), (1, 3)] {
            let saved = saved_entry_row(&c.model_root, &refs[diagonal][index], &id)?;
            verify_saved_learned_entry(&steps[arm], &saved, target, arms[arm].0.target.weight_q31)?;
        }
        let wins: Vec<_> = arms
            .iter()
            .map(|(m, _)| m.winner.token_id == target)
            .collect();
        for arm in 0..4 {
            if wins[arm] {
                correct[arm].push(id.clone());
                if !ENTRY_ORIGINAL_EIGHT.contains(&index) {
                    remaining_correct[arm].push(id.clone());
                }
            }
        }
        let mut contrasts = BTreeMap::new();
        for (name, from, to) in [
            ("source_at_g0", 0, 2),
            ("source_at_g1", 1, 3),
            ("generate_at_s0", 0, 1),
            ("generate_at_s1", 2, 3),
        ] {
            let a = &arms[from].0;
            let b = &arms[to].0;
            contrasts.insert(name,json!({"target_probability":rational(normalized_change(i128::from(b.target.weight_q31),b.total,i128::from(a.target.weight_q31),a.total)?),
                "normalized_margin":rational(normalized_change(b.margin(),b.total,a.margin(),a.total)?)}));
        }
        let pattern = if ENTRY_ORIGINAL_EIGHT.contains(&index) {
            entry_require(
                wins[0] && !wins[3],
                "original8 diagonal collapse not reproduced",
            )?;
            let pattern = component_loss_pattern(wins[2], wins[1]);
            patterns.entry(pattern.into()).or_default().push(id.clone());
            original.push(json!({"index":index,"id":id,"correct_s0g0_s0g1_s1g0_s1g1":wins,"loss_pattern":pattern}));
            Some(pattern)
        } else {
            None
        };
        let filename = format!("component-row-{index:04}.json");
        write_row(
            c,
            &filename,
            &json!({"index":index,"id":id,"packet_serde_sha256":packet_sha,"target_token_id":target,
            "target_definition":"first token of first accepted answer; labels after all four forwards","arm_order":["s0g0","s0g1","s1g0","s1g1"],
            "arms":arms.iter().map(|(_,v)|v).collect::<Vec<_>>(),"fixed_source_invariants":invariants,
            "initial_saved_row_sha256":refs[0][index]["row_sha256"],"final_saved_row_sha256":refs[1][index]["row_sha256"],"both_diagonals_exact_saved_entry":true,
            "component_contrasts":contrasts,"original8_loss_pattern":pattern}),
        )?;
        rows.push(json!({"index":index,"id":id,"row_file":filename,"row_sha256":file_hash(&c.output.join(&filename))?,"correct_s0g0_s0g1_s1g0_s1g1":wins}));
    }
    entry_require(
        rows.len() == 512 && original.len() == 8,
        "component coverage incomplete",
    )?;
    let report = json!({"schema":"uor-r4.native-bank-component-cross-entry/1","status":"COMPLETED","evaluation_kind":"component_cross_entry",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,"model_source_commit":joint.report["source_commit"],
        "parent_report_sha256":COMPONENT_PARENT_REPORT_SHA,"parent_manifest_sha256":COMPONENT_PARENT_MANIFEST_SHA,
        "joint_report_sha256":COMPONENT_REPORT_SHA,"joint_manifest_sha256":COMPONENT_MANIFEST_SHA,"joint_checkpoint_receipt_sha256":COMPONENT_RECEIPT_SHA,
        "component_receipt_sha256":file_hash(&c.output.join("component-receipt.json"))?,"config_sha256":file_hash(&c.output.join("config.json"))?,
        "inputs_sha256":ENTRY_INPUT_SHA,"labels_sha256":ENTRY_LABEL_SHA,"input_seal_sha256":file_hash(&input_seal.join("manifest.json"))?,"label_seal_sha256":file_hash(&label_seal.join("manifest.json"))?,
        "cases":512,"native_entry_calls":2048,"diagonal_parity_calls":1024,"arm_order":["s0g0","s0g1","s1g0","s1g1"],
        "entry_correct_counts":correct.iter().map(Vec::len).collect::<Vec<_>>(),"entry_correct_ids":correct,"remaining504_correct_ids":remaining_correct,
        "original8":original,"original8_loss_pattern_ids":patterns,"rows":rows,"native_forward_seconds":forward_seconds,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "hash_encoding":"SHA256 compact serde_json::to_vec for trace vectors; file pins bind exact bytes",
        "interpretation":"conditional Source (Context+Potential) and Generate coefficient substitutions at joint1 onset; Source effect is not proof of source identity loss; quantization and learning attribution unresolved",
        "scope":"all512 exposed empty-prefix decisions; four zero-v2 native arms; derived off-diagonals unaccepted; no fit, full-reply, fresh-transfer, chat, geometry advantage or final64 attribution"});
    entry_require(
        serde_json::to_vec_pretty(&report)?.len() <= 512 << 10,
        "component summary exceeds reserved allowance",
    )?;
    Ok(report)
}

fn admit_paths(c: &mut Config) -> Result<()> {
    admit_model_options(c)?;
    if c.maximum_report_bytes < 8 << 20 || c.maximum_report_bytes > 4 << 30 {
        return Err(bad("report storage limit must be8MiB..4GiB"));
    }
    for p in [&c.model_root, &c.inputs, &c.labels, &c.output] {
        if !p.is_absolute()
            || p.components()
                .any(|v| matches!(v, std::path::Component::ParentDir))
        {
            return Err(bad("absolute nontraversing paths required"));
        }
    }
    c.model_root = fs::canonicalize(&c.model_root)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    c.labels = fs::canonicalize(&c.labels)?;
    c.output = output_support::prospective_output(&c.output)?;
    for p in [&c.model_root, &c.inputs, &c.labels] {
        let seal = seal_for(p)?;
        if c.output.starts_with(p) || p.starts_with(&c.output) || c.output.starts_with(seal) {
            return Err(bad("output/input or sealed ancestry overlap"));
        }
    }
    if let Some(p) = &mut c.component_parent {
        if !p.model_root.is_absolute()
            || p.model_root
                .components()
                .any(|v| matches!(v, std::path::Component::ParentDir))
        {
            return Err(bad("absolute nontraversing component parent required"));
        }
        p.model_root = fs::canonicalize(&p.model_root)?;
        let seal = seal_for(&p.model_root)?;
        if c.output.starts_with(&p.model_root)
            || p.model_root.starts_with(&c.output)
            || c.output.starts_with(seal)
        {
            return Err(bad("output/component parent sealed ancestry overlap"));
        }
    }
    match (
        &mut c.continuation_field,
        &c.expected_continuation_field_sha256,
    ) {
        (Some(path), Some(_)) => {
            if !path.is_absolute()
                || path
                    .components()
                    .any(|v| matches!(v, std::path::Component::ParentDir))
            {
                return Err(bad("absolute nontraversing continuation path required"));
            }
            *path = fs::canonicalize(&*path)?;
            let seal = seal_for(path)?;
            if c.output.starts_with(&*path)
                || path.starts_with(&c.output)
                || c.output.starts_with(seal)
            {
                return Err(bad("output/continuation or sealed ancestry overlap"));
            }
        }
        (None, None) => {}
        _ => {
            return Err(bad(
                "continuation artifact path and expected SHA must be supplied together",
            ))
        }
    }
    Ok(())
}

fn prepare_zero_continuation(raw: &[u8]) -> Result<()> {
    let mut c: ZeroContinuationConfig = serde_json::from_slice(raw)?;
    for path in [&c.model_root, &c.output] {
        if !path.is_absolute()
            || path
                .components()
                .any(|v| matches!(v, std::path::Component::ParentDir))
        {
            return Err(bad(
                "absolute nontraversing zero-preparation paths required",
            ));
        }
    }
    c.model_root = fs::canonicalize(&c.model_root)?;
    c.output = output_support::prospective_output(&c.output)?;
    let model_seal = seal_for(&c.model_root)?;
    if c.output.starts_with(&c.model_root)
        || c.model_root.starts_with(&c.output)
        || c.output.starts_with(model_seal)
    {
        return Err(bad(
            "zero-preparation output/model or sealed ancestry overlap",
        ));
    }
    report_output::claim(&c.output)?;
    write(&c.output.join("config.json"), &serde_json::from_slice(raw)?)?;
    let result = (|| -> Result<Value> {
        let model = load_frozen_model(
            &c.model_root,
            &c.expected_model_report_sha256,
            &c.expected_model_manifest_sha256,
        )?;
        let generator = model.generator;
        let field = NativeContinuationField::zeroed(
            generator.source_binding(),
            generator.generate_model(),
        )?;
        let field_bytes = field.to_bytes()?;
        let reloaded = NativeContinuationField::from_bytes(
            &field_bytes,
            generator.source_binding(),
            generator.generate_model(),
        )?;
        if reloaded.to_bytes()? != field_bytes || reloaded.packed_unary().iter().any(|&v| v != 0) {
            return Err(bad(
                "zero-continuation independent reload or zero payload differs",
            ));
        }
        fs::write(c.output.join("continuation-field.bin"), &field_bytes)?;
        Ok(
            json!({"schema":"uor-r4.native-continuation-zero/1","status":"COMPLETED",
            "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,
            "model_report_sha256":c.expected_model_report_sha256,"model_manifest_sha256":c.expected_model_manifest_sha256,
            "checkpoint_receipt_sha256":file_hash(&model.checkpoint.join("receipt.json"))?,
            "source_binding":generator.source_binding(),"generate_sha256":generator.generate_sha256(),
            "continuation_sha256":hash(&field_bytes),"continuation_file":"continuation-field.bin",
            "continuation_metadata":reloaded.metadata(),"packed_unary_bytes":reloaded.packed_unary().len(),
            "shared_coefficients":reloaded.lanes()*120,"all_coefficients_zero":true,"native_independently_reloaded":true,
            "config_sha256":file_hash(&c.output.join("config.json"))?,
            "scope":"zero-field artifact construction only; no panel, labels, training or model-quality measurement"}),
        )
    })();
    let report = match &result {
        Ok(value) => value.clone(),
        Err(error) => json!({"schema":"uor-r4.native-continuation-zero/1","status":"FAILED",
            "error":error.to_string(),"scope":"artifact preparation failure; no model-quality verdict"}),
    };
    write(&c.output.join("report.json"), &report)?;
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 3 && args[1] == "verify-report" {
        report_output::verify(Path::new(&args[2]))?;
        return Ok(());
    }
    if args.len() == 3 && args[1] == "prepare-zero-continuation" {
        return prepare_zero_continuation(&bytes(Path::new(&args[2]))?);
    }
    if args.len() == 3 && args[1] == "prepare-early-query" {
        return early_query::prepare(&bytes(Path::new(&args[2]))?);
    }
    if args.len() != 2 {
        return Err(bad("usage: native-bank-generalization CONFIG.json | prepare-zero-continuation ZERO_CONFIG.json | prepare-early-query PREP_CONFIG.json | verify-report OUTPUT"));
    }
    let raw = bytes(Path::new(&args[1]))?;
    let mut c: Config = serde_json::from_slice(&raw)?;
    admit_paths(&mut c)?;
    report_output::claim(&c.output)?;
    write(
        &c.output.join("config.json"),
        &serde_json::from_slice(&raw)?,
    )?;
    let result = match c.evaluation_kind {
        EvaluationKind::OwnPrefix => run(&c),
        EvaluationKind::EntryZeroUV2 => run_entry_zero_u(&c),
        EvaluationKind::ComponentCrossEntry => run_component_cross_entry(&c),
    };
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            let mut failure = json!({"schema":"uor-r4.native-bank-generalization/1","status":"FAILED","error":e.to_string(),"scope":"execution/instrument failure; no model-quality verdict"});
            if c.evaluation_kind == EvaluationKind::EntryZeroUV2 {
                failure["schema"] = json!("uor-r4.native-bank-entry-zero-u-v2/1");
                failure["evaluation_kind"] = json!("entry_zero_u_v2");
            }
            if c.evaluation_kind == EvaluationKind::ComponentCrossEntry {
                failure["schema"] = json!("uor-r4.native-bank-component-cross-entry/1");
                failure["evaluation_kind"] = json!("component_cross_entry");
            }
            failure
        }
    };
    write(&c.output.join("report.json"), &report)?;
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn legacy_config() -> Value {
        json!({"model_root":"/fixture/model","expected_model_report_sha256":"a".repeat(64),
            "expected_model_manifest_sha256":"b".repeat(64),"inputs":"/fixture/inputs.json",
            "expected_inputs_sha256":"c".repeat(64),"labels":"/fixture/labels.json",
            "expected_labels_sha256":"d".repeat(64),"output":"/fixture/attempt"})
    }
    fn joint_config() -> Value {
        let mut c = legacy_config();
        c["model_kind"] = json!("joint_continuation");
        c["checkpoint_step"] = json!(1);
        c["expected_checkpoint_receipt_sha256"] = json!("e".repeat(64));
        c["continuation_field"] = json!("/fixture/model/checkpoint-0001/continuation-field.bin");
        c["expected_continuation_field_sha256"] = json!("f".repeat(64));
        c
    }
    fn component_config() -> Value {
        let mut c = joint_config();
        c["evaluation_kind"] = json!("component_cross_entry");
        c["expected_model_report_sha256"] = json!(COMPONENT_REPORT_SHA);
        c["expected_model_manifest_sha256"] = json!(COMPONENT_MANIFEST_SHA);
        c["expected_checkpoint_receipt_sha256"] = json!(COMPONENT_RECEIPT_SHA);
        c["expected_continuation_field_sha256"] = json!(COMPONENT_U_SHA);
        c["expected_inputs_sha256"] = json!(ENTRY_INPUT_SHA);
        c["expected_labels_sha256"] = json!(ENTRY_LABEL_SHA);
        c["maximum_report_bytes"] = json!(128 << 20);
        c["component_parent"] = json!({"model_root":"/fixture/parent",
            "expected_model_report_sha256":COMPONENT_PARENT_REPORT_SHA,
            "expected_model_manifest_sha256":COMPONENT_PARENT_MANIFEST_SHA,
            "expected_checkpoint_receipt_sha256":COMPONENT_PARENT_RECEIPT_SHA});
        c
    }
    #[test]
    fn component_admission_pins_both_donors_and_rejects_legacy_crossover() -> Result<()> {
        admit_model_options(&serde_json::from_value(component_config())?)?;
        for (key, value) in [
            ("component_parent", Value::Null),
            ("checkpoint_step", json!(64)),
            ("expected_checkpoint_receipt_sha256", json!("0".repeat(64))),
            ("expected_model_report_sha256", json!(ENTRY_REPORT_SHA)),
            ("expected_model_manifest_sha256", json!(ENTRY_MANIFEST_SHA)),
            ("expected_continuation_field_sha256", json!(ENTRY_U_SHA)),
            ("expected_inputs_sha256", json!("0".repeat(64))),
            ("expected_labels_sha256", json!("0".repeat(64))),
            ("maximum_report_bytes", json!(64 << 20)),
            ("continuation_field", Value::Null),
            ("baseline_parity", json!(true)),
            ("evaluation_kind", json!("own_prefix")),
        ] {
            let mut c = component_config();
            c[key] = value;
            assert!(
                admit_model_options(&serde_json::from_value(c)?).is_err(),
                "{key}"
            );
        }
        for key in [
            "expected_model_report_sha256",
            "expected_model_manifest_sha256",
            "expected_checkpoint_receipt_sha256",
        ] {
            let mut c = component_config();
            c["component_parent"][key] = json!("0".repeat(64));
            assert!(
                admit_model_options(&serde_json::from_value(c)?).is_err(),
                "parent {key}"
            );
        }
        let mut legacy = legacy_config();
        legacy["component_parent"] = component_config()["component_parent"].clone();
        assert!(admit_model_options(&serde_json::from_value(legacy)?).is_err());
        Ok(())
    }
    #[test]
    fn component_frozen_inventory_allows_only_context_potential_and_rebinding() -> Result<()> {
        let names = [
            "metadata.json",
            "consumer/metadata.json",
            "consumer/context-q4.bin",
            "consumer/potential-q4.bin",
            "consumer/exp-q31.bin",
            "tokenizer.json",
            "encoder.bin",
        ];
        let a: BTreeMap<String, String> = names
            .iter()
            .map(|s| (s.to_string(), "parent".into()))
            .collect();
        let mut b = a.clone();
        for name in &names[..4] {
            b.insert(name.to_string(), "joint".into());
        }
        component_frozen_inventory(&a, &b)?;
        for name in &names[4..] {
            let mut bad = b.clone();
            bad.insert(name.to_string(), "different".into());
            assert!(component_frozen_inventory(&a, &bad).is_err(), "{name}");
        }
        b.remove("encoder.bin");
        assert!(component_frozen_inventory(&a, &b).is_err());
        let mut extra = a.clone();
        extra.insert("new.bin".into(), "unknown".into());
        assert!(component_frozen_inventory(&a, &extra).is_err());
        Ok(())
    }
    #[test]
    fn component_patterns_preserve_both_sufficiency_and_combination_case() {
        assert_eq!(
            component_loss_pattern(false, true),
            "source_alone_sufficient"
        );
        assert_eq!(
            component_loss_pattern(true, false),
            "generate_alone_sufficient"
        );
        assert_eq!(
            component_loss_pattern(false, false),
            "either_alone_sufficient"
        );
        assert_eq!(component_loss_pattern(true, true), "combination_specific");
    }
    fn entry_config() -> Value {
        let mut c = joint_config();
        c["evaluation_kind"] = json!("entry_zero_u_v2");
        c["checkpoint_step"] = json!(64);
        c["continuation_field"] = json!("/fixture/model/checkpoint-0064/continuation-field.bin");
        c["expected_model_report_sha256"] = json!(ENTRY_REPORT_SHA);
        c["expected_model_manifest_sha256"] = json!(ENTRY_MANIFEST_SHA);
        c["expected_continuation_field_sha256"] = json!(ENTRY_U_SHA);
        c["expected_inputs_sha256"] = json!(ENTRY_INPUT_SHA);
        c["expected_labels_sha256"] = json!(ENTRY_LABEL_SHA);
        c["maximum_report_bytes"] = json!(64 << 20);
        c
    }
    #[test]
    fn entry_bank_trace_is_opt_in_and_own_prefix_only() -> Result<()> {
        let legacy: Config = serde_json::from_value(legacy_config())?;
        assert!(!legacy.retain_entry_bank_trace);
        let mut enabled = legacy_config();
        enabled["retain_entry_bank_trace"] = json!(true);
        admit_model_options(&serde_json::from_value(enabled)?)?;
        let mut wrong_mode = entry_config();
        wrong_mode["retain_entry_bank_trace"] = json!(true);
        assert!(admit_model_options(&serde_json::from_value(wrong_mode)?).is_err());
        Ok(())
    }
    #[test]
    fn entry_diagnostic_requires_exact_final64_full_panel_and_genuine_u() -> Result<()> {
        let old: Config = serde_json::from_value(legacy_config())?;
        assert_eq!(old.evaluation_kind, EvaluationKind::OwnPrefix);
        admit_model_options(&old)?;
        admit_model_options(&serde_json::from_value(entry_config())?)?;
        for (key, value) in [
            ("model_kind", json!("legacy_recomposition")),
            ("checkpoint_step", json!(32)),
            ("expected_model_report_sha256", json!("0".repeat(64))),
            ("expected_model_manifest_sha256", json!("0".repeat(64))),
            ("expected_continuation_field_sha256", json!("0".repeat(64))),
            ("expected_inputs_sha256", json!("0".repeat(64))),
            ("expected_labels_sha256", json!("0".repeat(64))),
            ("continuation_field", Value::Null),
            ("baseline_parity", json!(true)),
            ("maximum_report_bytes", json!(32 << 20)),
        ] {
            let mut c = entry_config();
            c[key] = value;
            assert!(
                admit_model_options(&serde_json::from_value(c)?).is_err(),
                "{key}"
            );
        }
        let mut subset = entry_config();
        subset["maximum_cases"] = json!(256);
        assert!(serde_json::from_value::<Config>(subset).is_err());
        Ok(())
    }
    #[test]
    fn entry_u_difference_checks_each_duplicate_copy_and_factual_base() -> Result<()> {
        let ids = [0, 0, 1];
        let base_generate = [10, 2, 2];
        let learned_generate = [12, -1, 2];
        let base_copy = [4, 7, 10];
        let learned_copy = [6, 9, 7];
        let delta = [2, -3, 0];
        verify_entry_score_deltas(
            &ids,
            &learned_generate,
            &base_generate,
            &learned_copy,
            &base_copy,
            &delta,
        )?;
        let mut wrong_duplicate = learned_copy;
        wrong_duplicate[1] -= 1;
        assert!(verify_entry_score_deltas(
            &ids,
            &learned_generate,
            &base_generate,
            &wrong_duplicate,
            &base_copy,
            &delta
        )
        .is_err());
        let mut different_base = base_generate;
        different_base[2] += 1;
        assert!(verify_entry_score_deltas(
            &ids,
            &learned_generate,
            &different_base,
            &learned_copy,
            &base_copy,
            &delta
        )
        .is_err());
        assert!(verify_entry_score_deltas(
            &ids,
            &learned_generate,
            &base_generate,
            &learned_copy,
            &base_copy[..2],
            &delta
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn entry_mass_ties_use_smallest_id_not_positive_margin_heuristics() -> Result<()> {
        let masses = [
            VocabularyTokenMass {
                token_id: 0,
                weight_q31: 4,
                generate_weight_q31: 1,
                copy_weight_q31: 3,
            },
            VocabularyTokenMass {
                token_id: 1,
                weight_q31: 4,
                generate_weight_q31: 4,
                copy_weight_q31: 0,
            },
            VocabularyTokenMass {
                token_id: 2,
                weight_q31: 2,
                generate_weight_q31: 2,
                copy_weight_q31: 0,
            },
        ];
        let lost = entry_masses(&masses, 1, 10, 0)?;
        assert_eq!(lost.margin(), 0);
        assert_eq!(lost.tied_maximum_tokens, 2);
        assert_eq!(lost.target_strict_mass_rank, 1);
        assert_eq!(lost.target_tiebroken_rank, 2);
        assert_eq!(lost.rival.token_id, 0);
        assert_ne!(lost.winner.token_id, lost.target.token_id);
        let won = entry_masses(&masses, 0, 10, 0)?;
        assert_eq!(won.margin(), 0);
        assert_eq!(won.winner.token_id, won.target.token_id);
        assert_eq!(won.target_tiebroken_rank, 1);
        assert!(entry_masses(&masses, 1, 10, 1).is_err());
        assert!(entry_masses(&masses, 1, 11, 0).is_err());
        Ok(())
    }
    #[test]
    fn entry_normalized_margin_handles_changed_references_and_wide_products() -> Result<()> {
        // The larger raw margin2 is worse after normalizing by100 rather than10.
        assert_eq!(normalized_change(2, 100, 1, 10)?, (-80, 1000));
        assert_eq!(normalized_change(4, 20, 2, 10)?, (0, 200));
        let wide = normalized_change(1 << 44, 1 << 45, 1 << 43, 1 << 45)?;
        assert!(wide.0 > i128::from(u64::MAX));
        let encoded = rational(wide);
        assert_eq!(encoded["numerator"], wide.0.to_string());
        assert_eq!(encoded["denominator"], wide.1.to_string());
        assert!(normalized_change(i128::MAX, 1, 0, 2).is_err());
        assert!(normalized_change(1, 0, 0, 1).is_err());
        Ok(())
    }
    #[test]
    fn joint_model_options_require_explicit_checkpoint_and_u_preserving_legacy_default(
    ) -> Result<()> {
        let legacy: Config = serde_json::from_value(legacy_config())?;
        assert_eq!(legacy.model_kind, ModelKind::LegacyRecomposition);
        admit_model_options(&legacy)?;
        let mut legacy_u = legacy_config();
        legacy_u["continuation_field"] = json!("/fixture/zero/continuation-field.bin");
        legacy_u["expected_continuation_field_sha256"] = json!("f".repeat(64));
        admit_model_options(&serde_json::from_value(legacy_u)?)?;
        for key in ["checkpoint_step", "expected_checkpoint_receipt_sha256"] {
            let mut mixed = legacy_config();
            mixed[key] = joint_config()[key].clone();
            assert!(
                admit_model_options(&serde_json::from_value(mixed)?).is_err(),
                "{key}"
            );
        }
        admit_model_options(&serde_json::from_value(joint_config())?)?;
        for key in [
            "checkpoint_step",
            "expected_checkpoint_receipt_sha256",
            "continuation_field",
            "expected_continuation_field_sha256",
        ] {
            let mut missing = joint_config();
            missing
                .as_object_mut()
                .ok_or_else(|| bad("config fixture"))?
                .remove(key);
            assert!(
                admit_model_options(&serde_json::from_value(missing)?).is_err(),
                "{key}"
            );
        }
        for (key, value) in [
            ("baseline_parity", json!(true)),
            ("expected_checkpoint_receipt_sha256", json!("not-a-digest")),
            ("expected_continuation_field_sha256", json!("")),
        ] {
            let mut invalid = joint_config();
            invalid[key] = value;
            assert!(
                admit_model_options(&serde_json::from_value(invalid)?).is_err(),
                "{key}"
            );
        }
        Ok(())
    }
    #[test]
    fn joint_receipt_rejects_wrong_endpoint_mixed_provenance_and_omitted_u() -> Result<()> {
        let c: Config = serde_json::from_value(joint_config())?;
        let receipt = json!({"mode":"joint_continuation","step":1,
            "source_commit":"1".repeat(40),"native_independently_reloaded":true,
            "masters_independently_reloaded":true,"continuation_sha256":"f".repeat(64),
            "parent_report_sha256":"2".repeat(64),"parent_manifest_sha256":"3".repeat(64),
            "training_input_sha256":"4".repeat(64),"training_labels_sha256":"5".repeat(64)});
        let admission = receipt.clone();
        let report = json!({"schema":"uor-r4.geometric-frozen-map-fit/1","mode":"joint_continuation",
            "status":"COMPLETED","updates":1,"source_commit":"1".repeat(40),"final_receipt":receipt});
        validate_joint_receipt(&c, &report, &admission, &receipt)?;
        for (key, value) in [
            ("schema", json!("uor-r4.geometric-prediction-control/1")),
            ("status", json!("FAILED")),
            ("updates", json!(64)),
            ("source_commit", json!("other-consumer-head")),
            ("final_receipt", Value::Null),
        ] {
            let mut changed = report.clone();
            changed[key] = value;
            assert!(
                validate_joint_receipt(&c, &changed, &admission, &receipt).is_err(),
                "{key}"
            );
        }
        // Even if a changed receipt is also copied into final_receipt, reject
        // incomplete exports, U substitution and source/provenance mixing.
        for (key, value) in [
            ("mode", json!("continuation")),
            ("step", json!(0)),
            ("native_independently_reloaded", json!(false)),
            ("masters_independently_reloaded", json!(false)),
            ("source_commit", json!("6".repeat(40))),
            ("continuation_sha256", Value::Null),
            ("continuation_sha256", json!("6".repeat(64))),
            ("parent_report_sha256", json!("6".repeat(64))),
            ("parent_manifest_sha256", json!("6".repeat(64))),
            ("training_input_sha256", json!("6".repeat(64))),
            ("training_labels_sha256", Value::Null),
        ] {
            let mut changed = receipt.clone();
            changed[key] = value;
            let mut changed_report = report.clone();
            changed_report["final_receipt"] = changed.clone();
            assert!(
                validate_joint_receipt(&c, &changed_report, &admission, &changed).is_err(),
                "{key}"
            );
        }
        Ok(())
    }
    #[test]
    fn joint_field_requires_selected_checkpoint_even_when_bytes_match() -> Result<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uor-joint-field-{}-{nonce}", std::process::id()));
        fs::create_dir(&root)?;
        struct OwnedTemp(PathBuf);
        impl Drop for OwnedTemp {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _owned = OwnedTemp(root.clone());
        let selected = root.join("checkpoint-0001");
        let old = root.join("checkpoint-0000");
        fs::create_dir(&selected)?;
        fs::create_dir(&old)?;
        let selected_field = selected.join("continuation-field.bin");
        let old_field = old.join("continuation-field.bin");
        fs::write(&selected_field, b"identical zero field fixture")?;
        fs::write(&old_field, b"identical zero field fixture")?;
        assert_eq!(file_hash(&selected_field)?, file_hash(&old_field)?);
        validate_joint_field_path(&selected_field, &selected)?;
        assert!(validate_joint_field_path(&old_field, &selected).is_err());
        Ok(())
    }
    #[test]
    fn zero_continuation_config_rejects_labels_and_targets() -> Result<()> {
        let config = json!({"model_root":"/fixture/model","expected_model_report_sha256":"a".repeat(64),
            "expected_model_manifest_sha256":"b".repeat(64),"output":"/fixture/new-attempt"});
        let _: ZeroContinuationConfig = serde_json::from_value(config.clone())?;
        let mut with_labels = config.clone();
        with_labels["labels"] = json!("/fixture/labels.json");
        assert!(serde_json::from_value::<ZeroContinuationConfig>(with_labels).is_err());
        let mut with_target = config;
        with_target["target_token_id"] = json!(4);
        assert!(serde_json::from_value::<ZeroContinuationConfig>(with_target).is_err());
        Ok(())
    }
    fn packet() -> Value {
        json!({"id":"fixture","actual_prefix_ids":[],"query_ids":[7],"segments":[
        {"kind":"Context","event":1,"role":1,"token_ids":[4]},
        {"kind":"Source","event":2,"record":9,"commit":3,"scope":"fixture-scope","entity":[4],"relation":2,"view":1,"original_source_ids":[5]}]})
    }
    #[test]
    fn snapshot_preserves_source_occurrences_and_query_with_metadata_only_pin() -> Result<()> {
        let input: Packet = serde_json::from_value(packet())?;
        let bank = snapshot(input)?;
        assert_eq!(bank.query_ids, vec![7]);
        assert_eq!(bank.pin.commit, 3);
        assert_eq!(bank.pin.scope, b"fixture-scope");
        match &bank.segments[1] {
            OwnedBankSegment::Source(s) => {
                assert_eq!(s.original_token_ids, vec![5]);
                assert_eq!(s.record, 9);
            }
            _ => return Err(bad("source absent")),
        }
        let mut malicious = packet();
        malicious["selected_record"] = json!(9);
        assert!(serde_json::from_value::<Packet>(malicious).is_err());
        let mut prefix = packet();
        prefix["actual_prefix_ids"] = json!([2997]);
        assert!(snapshot(serde_json::from_value(prefix)?).is_err());
        Ok(())
    }
    #[test]
    fn different_evaluator_answers_cannot_change_native_snapshot_inputs() -> Result<()> {
        let mut first =
            json!({"id":"fixture","answers":{"intent":"current","accepted":[" first."]}});
        let a: Label = serde_json::from_value(first.clone())?;
        a.answers.validate()?;
        first["answers"]["accepted"] = json!([" second."]);
        let b: Label = serde_json::from_value(first)?;
        b.answers.validate()?;
        assert_ne!(a.answers, b.answers);
        let x = snapshot(serde_json::from_value(packet())?)?;
        let y = snapshot(serde_json::from_value(packet())?)?;
        assert_eq!(x.pin, y.pin);
        assert_eq!(x.query_ids, y.query_ids);
        assert_eq!(x.segments.len(), y.segments.len());
        // Packet deserialization rejects evaluator labels in the runtime input.
        let mut p = packet();
        p["answers"] = json!([" second."]);
        assert!(serde_json::from_value::<Packet>(p).is_err());
        Ok(())
    }
    #[test]
    fn snapshot_rejects_mixed_scope_and_missing_source() -> Result<()> {
        let mut p = packet();
        let mut other = p["segments"][1].clone();
        other["scope"] = json!("other");
        p["segments"]
            .as_array_mut()
            .ok_or_else(|| bad("fixture segments"))?
            .push(other);
        assert!(snapshot(serde_json::from_value(p)?).is_err());
        let mut p = packet();
        p["segments"] = json!([{ "kind":"Context","event":1,"role":1,"token_ids":[4]}]);
        assert!(snapshot(serde_json::from_value(p)?).is_err());
        Ok(())
    }
}
