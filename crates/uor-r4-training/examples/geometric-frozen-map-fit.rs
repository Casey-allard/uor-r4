//! Matched exact-checkpoint restart with a frozen categorical transport map.
//! Targets enter only the token-alias objective; native source selection stays target-free.
#![recursion_limit = "256"]
use candle_core::{Device, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::FrozenAnswers,
    native_geometric::learner::{
        geometric_continuation_field::NativeContinuationField,
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
        geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge},
        native_bank_generate::{
            BankPin, BoundNativeBytes, GenerationLimits, GenerationStop, NativeBankArtifacts,
            NativeBankGenerateStep, NativeBankGenerator, OwnedBankSegment, OwnedBankSource,
            PinnedBankSnapshot, SnapshotSourceStatus,
        },
    },
    report_output,
};
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_cue_carrier::{
        CueAngularConfig, CueAngularQ4, CueCarrierMetadata, CueJointMetadata, CueJointQ4,
    },
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_prefix_transport::{PrefixAngularConfig, PrefixAngularQ4, PrefixTransportMetadata},
    geometric_source_emission_view::SourceEmissionView,
    geometric_source_realizer::{
        NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
    },
    geometric_vocabulary_actions::NativeVocabularyActions,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_bank_generate::{
        FixedContinuationPosition, PreparedBankGenerate, PreparedFixedContinuationBank,
    },
    geometric_continuation_learning::ContinuationLearningWeights,
    geometric_generate_learning::{GenerateLearningWeights, VocabularyScoreAdjoint},
    geometric_occurrence_consumer::{
        source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
        ConsumerIdentity,
    },
    geometric_read_state_bridge::{
        BridgeLearningWeights, CategoricalBridgeLearningWeights, PreparedCategoricalBridge,
    },
    sha256_bytes, sha256_file,
};
#[path = "geometric_frozen_map_fit/categorical_proposals.rs"]
mod categorical_proposals;
#[path = "geometric_frozen_map_fit/constrained_context.rs"]
mod constrained_context;
#[path = "geometric_frozen_map_fit/constrained_emission.rs"]
mod constrained_emission;
#[path = "geometric_frozen_map_fit/context_constraints.rs"]
mod context_constraints;
#[path = "geometric_frozen_map_fit/context_cue_coadapt.rs"]
mod context_cue_coadapt;
#[path = "geometric_frozen_map_fit/context_path_credit.rs"]
mod context_path_credit;
#[path = "geometric_frozen_map_fit/coupled_episode_learning.rs"]
mod coupled_episode_learning;
#[path = "geometric_frozen_map_fit/cross_state_completion.rs"]
mod cross_state_completion;
#[path = "geometric_frozen_map_fit/emission_constraints.rs"]
mod emission_constraints;
#[path = "geometric_frozen_map_fit/frontier.rs"]
mod frontier;
#[path = "geometric_frozen_map_fit/generate_episode_learning.rs"]
mod generate_episode_learning;
#[path = "geometric_frozen_map_fit/native_proposals.rs"]
mod native_proposals;
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
#[path = "geometric_frozen_map_fit/prefix_context_credit.rs"]
mod prefix_context_credit;
#[path = "geometric_frozen_map_fit/prefix_fragment_learning.rs"]
mod prefix_fragment_learning;
#[path = "geometric_frozen_map_fit/prototype_compensation.rs"]
mod prototype_compensation;
#[path = "geometric_frozen_map_fit/reached_u.rs"]
mod reached_u;
#[path = "geometric_frozen_map_fit/readout_coadaptation.rs"]
mod readout_coadaptation;
#[path = "geometric_frozen_map_fit/readout_constraints.rs"]
mod readout_constraints;
#[path = "geometric_frozen_map_fit/reply_completion.rs"]
mod reply_completion;
#[path = "geometric_frozen_map_fit/u_constraints.rs"]
mod u_constraints;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Admission,
    Fit,
    PredictionControl,
    EntryCeiling,
    EntryScorerFit,
    ContinuationOnly,
    CrossStateContinuation,
    JointContinuation,
    ReplyCompletion,
    ReplyQualification,
    ReplyPrototypeQualification,
}

const REPLAY_REPORT_SHA: &str = "9582f56c8d285920cd67977fd23d36e8a96beabe7c5f27ea45ad4b1113d3503c";
const REPLAY_MANIFEST_SHA: &str =
    "45bbcbf2550b6d6a726df09b3c8ad6307ad20b4a8073306be458ca93f3376da5";
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReferenceReplayConfig {
    plan_root: PathBuf,
    reference_root: PathBuf,
    expected_plan_report_sha256: String,
    expected_plan_manifest_sha256: String,
    lambda: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferencePrepareConfig {
    reference_root: PathBuf,
    training_inputs: PathBuf,
    training_labels: PathBuf,
    out: PathBuf,
    maximum_report_bytes: u64,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReferenceRow {
    index: usize,
    id: String,
    packet_sha256: String,
    saved_row_sha256: String,
    targets: Vec<u32>,
    copy_ids: Vec<u32>,
    phases: Vec<usize>,
    eligible: Vec<bool>,
    weight_denominators: Vec<usize>,
    weights: Vec<f64>,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ReferencePlan {
    schema: String,
    reference_root: PathBuf,
    source_binding: NativeArtifactBinding,
    generate_sha256: String,
    initial_receipt_sha256: String,
    reference_report_sha256: String,
    reference_manifest_sha256: String,
    input_sha256: String,
    labels_sha256: String,
    eos_token_id: u32,
    rows: Vec<ReferenceRow>,
}
fn replay_require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(bad(message))
    }
}
fn reference_replay_settings(a: &Args) -> Result<()> {
    coupled_episode_learning::validate_settings(a)?;
    if a.coupled_episode_learning.is_some() {
        return Ok(());
    }
    generate_episode_learning::validate_completion_settings(a)?;
    if a.generate_episode_completion.is_some() {
        return Ok(());
    }
    generate_episode_learning::validate_settings(a)?;
    if a.generate_episode_learning.is_some() {
        return Ok(());
    }
    prefix_fragment_learning::validate_artifact_settings(a)?;
    if a.prefix_artifact_check.is_some() {
        return Ok(());
    }
    prefix_fragment_learning::validate_settings(a)?;
    if a.prefix_fragment_learning.is_some() {
        return Ok(());
    }
    context_cue_coadapt::validate_settings(a)?;
    if a.context_cue_coadapt.is_some() {
        return Ok(());
    }
    context_path_credit::validate_settings(a)?;
    if a.context_path_credit.is_some() {
        return Ok(());
    }
    reached_u::validate_settings(a)?;
    readout_coadaptation::validate_settings(a)?;
    prefix_context_credit::validate_settings(a)?;
    if a.prefix_context_credit.is_some() {
        // The fixed eighteen-frame discriminator owns its objective and pins;
        // historical 89/377 reference-plan admission does not describe this probe.
        return Ok(());
    }
    replay_require(
        if a.prototype_compensation.is_some() {a.constrained_emission_learning && a.retained_context_root.is_none()} else {a.constrained_emission_learning == a.retained_context_root.is_some()},
        "emission needs retained Context root, or compensation needs explicit retained emission/diagnostic roots and no Context root",
    )?;
    replay_require(
        !a.constrained_emission_learning
            || (a.reached_frontier_objective
                && !a.constrained_context_learning
                && !a.categorical_action_learning
                && !a.categorical_action_only),
        "emission requires reached frontier with frozen upstream operators",
    )?;
    replay_require(
        !a.constrained_context_learning
            || (a.reached_frontier_objective
                && !a.categorical_action_learning
                && !a.categorical_action_only),
        "constrained Context requires reached-frontier replay with fixed categorical map",
    )?;
    replay_require(
        !a.categorical_action_only || a.categorical_action_learning,
        "categorical action-only requires explicit categorical action learning",
    )?;
    replay_require(
        !a.categorical_action_learning || a.reached_frontier_objective,
        "categorical action learning requires the reached frontier joint pilot",
    )?;
    if a.reached_frontier_objective && !a.native_code_proposals {
        return Err(bad(
            "reached frontier objective requires native code proposals",
        ));
    }
    if a.native_code_proposals && a.reference_replay.is_none() {
        return Err(bad(
            "native code proposals require the fixed reference replay pilot",
        ));
    }
    if let Some(c) = &a.reference_replay {
        let rates = a
            .joint_continuation
            .as_ref()
            .ok_or_else(|| bad("replay joint rates absent"))?;
        replay_require(
            a.mode == Mode::JointContinuation
                && !a.query_conditioned_read
                && a.seed == 1001
                && a.updates == 1
                && c.lambda == 1.0
                && rates.generate_learning_rate == 0.003
                && rates.context_learning_rate == 0.002
                && rates.potential_learning_rate == 0.003
                && rates.continuation_learning_rate == 0.03
                && [
                    &c.expected_plan_report_sha256,
                    &c.expected_plan_manifest_sha256,
                ]
                .iter()
                .all(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())),
            "reference replay requires one seed1001 proposal, lambda1 and unchanged joint rates",
        )?;
    }
    Ok(())
}

/// Legacy joint rates change four declared families with prototypes/map frozen.
/// Opt-in categorical pilots use their explicit native proposal rules instead.
#[derive(Clone, Copy, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct JointContinuationConfig {
    generate_learning_rate: f64,
    context_learning_rate: f64,
    potential_learning_rate: f64,
    continuation_learning_rate: f64,
}
fn joint_continuation_settings(a: &Args) -> Result<Option<&JointContinuationConfig>> {
    match (a.mode, a.joint_continuation.as_ref()) {
        (Mode::JointContinuation, Some(c))
            if a.credit == Credit::RawIdentity
                && a.read_state_pullback == ReadStatePullback::Categorical
                && a.loss_scope == LossScope::All
                && !a.ceiling_scorer
                && !a.ceiling_float
                && a.baseline.is_none()
                && a.panel_inputs.is_none()
                && a.panel_labels.is_none()
                && a.continuation.is_none()
                && [c.generate_learning_rate, c.context_learning_rate,
                    c.potential_learning_rate, c.continuation_learning_rate]
                    .iter().all(|v| v.is_finite() && *v > 0.) => Ok(Some(c)),
        (Mode::JointContinuation, _) => Err(bad(
            "joint continuation requires categorical/RawIdentity all-answer credit, four positive rates and no old diagnostic/cache options",
        )),
        (_, None) => Ok(None),
        (_, Some(_)) => Err(bad("joint continuation settings require joint_continuation mode")),
    }
}

/// Only this new mode may learn a continuation field on the sealed48/64 parent.
/// All legacy loaders and component-resume guards retain their existing scope.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContinuationConfig {
    expected_model_report_sha256: String,
    expected_model_manifest_sha256: String,
    learning_rate: f64,
    maximum_cache_tensor_bytes: u64,
}
const CONTINUATION_PARENT_REPORT_SHA: &str =
    "a1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278";
const CONTINUATION_PARENT_MANIFEST_SHA: &str =
    "b37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca";
fn continuation_settings(a: &Args) -> Result<Option<&ContinuationConfig>> {
    match (a.mode, a.continuation.as_ref()) {
        (Mode::ContinuationOnly | Mode::CrossStateContinuation, Some(c))
            if a.loss_scope == LossScope::All
                && !a.ceiling_scorer
                && a.baseline.is_none()
                && a.read_state_pullback == ReadStatePullback::Legacy
                && c.expected_model_report_sha256 == CONTINUATION_PARENT_REPORT_SHA
                && c.expected_model_manifest_sha256 == CONTINUATION_PARENT_MANIFEST_SHA
                && c.learning_rate.is_finite()
                && c.learning_rate > 0.
                && c.maximum_cache_tensor_bytes > 0 => Ok(Some(c)),
        (Mode::ContinuationOnly | Mode::CrossStateContinuation, _) => Err(bad(
            "continuation-only requires exact48/64 parent, all-answer loss, positive U rate/cache cap and no old diagnostic/pullback options",
        )),
        (_, None) => Ok(None),
        (_, Some(_)) => Err(bad("continuation settings require continuation_only mode")),
    }
}
#[derive(Clone, Copy, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ControlRates {
    generate: f64,
    prototype: f64,
    context: f64,
    potential: f64,
}
impl Default for ControlRates {
    fn default() -> Self {
        Self {
            generate: 0.003,
            prototype: 0.01,
            context: 0.002,
            potential: 0.003,
        }
    }
}
fn control_settings(a: &Args) -> Result<(usize, ControlRates)> {
    if a.mode != Mode::PredictionControl
        && (a.prediction_control_updates.is_some()
            || a.prediction_control_rates.is_some()
            || a.prediction_control_resume.is_some()
            || a.prediction_control_resume_source_step.is_some()
            || a.prediction_control_resume_generate_step.is_some()
            || a.prediction_control_trainable != ControlTrainable::Joint)
    {
        return Err(bad(
            "prediction control settings cannot change fixed fit/admission",
        ));
    }
    if (a.prediction_control_trainable != ControlTrainable::Joint)
        != a.prediction_control_resume.is_some()
    {
        return Err(bad("Generate-field control requires explicit resume; resume requires Generate-field control"));
    }
    if a.prediction_control_resume.is_none()
        && (a.prediction_control_resume_source_step.is_some()
            || a.prediction_control_resume_generate_step.is_some())
    {
        return Err(bad("component steps require authenticated resume"));
    }
    let n = a.prediction_control_updates.unwrap_or(32);
    let r = a.prediction_control_rates.unwrap_or_default();
    if ![0, 16, 32, 64].contains(&n)
        || [r.generate, r.prototype, r.context, r.potential]
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.)
    {
        return Err(bad(
            "control requires explicit0/16/32/64 updates and finite positive rates",
        ));
    }
    Ok((n, r))
}
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Credit {
    Clipped,
    RawIdentity,
}
impl Credit {
    fn policy(self) -> VocabularyScoreAdjoint {
        match self {
            Self::Clipped => VocabularyScoreAdjoint::Clipped,
            Self::RawIdentity => VocabularyScoreAdjoint::RawIdentity,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Clipped => "clipped",
            Self::RawIdentity => "raw_identity",
        }
    }
}
/// Offline Context transport pullback only. Both variants use the same hard
/// frozen categorical marker artifact and target-free physical Source route.
#[derive(Clone, Copy, Debug, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ReadStatePullback {
    #[default]
    Legacy,
    Categorical,
}
impl ReadStatePullback {
    fn name(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Categorical => "categorical",
        }
    }
}
/// Which supervised phases may carry credit. `all` is the declared
/// phase-balanced objective and stays byte-identical to the retained fits.
/// `entry_only` zeroes every later-position weight and leaves the entry
/// position's own balanced weight untouched, so the entry decision is learned
/// with no later-position credit in the four shared parameter groups. It
/// isolates one variable: the later-phase gradient, not the entry weight.
#[derive(Clone, Copy, Debug, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LossScope {
    #[default]
    All,
    EntryOnly,
}
impl LossScope {
    fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::EntryOnly => "entry_only",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ControlTrainable {
    #[default]
    Joint,
    GenerateField,
    PotentialGenerateField,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: Mode,
    credit: Credit,
    #[serde(default)]
    read_state_pullback: ReadStatePullback,
    seed: u64,
    checkpoint: PathBuf,
    saved_fit: PathBuf,
    categorical: PathBuf,
    parent_config: PathBuf,
    training_inputs: PathBuf,
    training_labels: PathBuf,
    development_inputs: PathBuf,
    development_labels: PathBuf,
    maximum_seconds: u64,
    maximum_report_bytes: u64,
    out: PathBuf,
    #[serde(default = "default_updates")]
    updates: usize,
    #[serde(default)]
    loss_scope: LossScope,
    /// Fit a query-keyed entry scorer on one panel half and apply it at the entry
    /// position through the native forward, reporting the held-out half.
    #[serde(default)]
    ceiling_scorer: bool,
    /// Compare the integer entry decision against the float masters on the same
    /// read states, isolating the export from the learned emission.
    #[serde(default)]
    ceiling_float: bool,
    /// Declared replacement panel for a zero-update entry read. When set, the
    /// pinned 512-row panel is not asserted and the declared file hashes are
    /// recorded in the report instead. Diagnostic only: nothing fits on it.
    #[serde(default)]
    panel_inputs: Option<PathBuf>,
    #[serde(default)]
    panel_labels: Option<PathBuf>,
    /// Select the read occurrence whose cumulative source state is closest to
    /// the query state instead of the earliest raw Copy maximum. Measured
    /// default: the selection is independent of the question and of the answer.
    #[serde(default)]
    query_conditioned_read: bool,
    #[serde(default)]
    baseline: Option<PathBuf>,
    #[serde(default)]
    prediction_control_updates: Option<usize>,
    #[serde(default)]
    prediction_control_rates: Option<ControlRates>,
    #[serde(default)]
    prediction_control_resume: Option<PathBuf>,
    #[serde(default)]
    prediction_control_resume_source_step: Option<usize>,
    #[serde(default)]
    prediction_control_resume_generate_step: Option<usize>,
    #[serde(default)]
    prediction_control_trainable: ControlTrainable,
    #[serde(default)]
    continuation: Option<ContinuationConfig>,
    #[serde(default)]
    cross_state_resume: Option<cross_state_completion::ResumeConfig>,
    #[serde(default)]
    joint_continuation: Option<JointContinuationConfig>,
    #[serde(default)]
    reference_replay: Option<ReferenceReplayConfig>,
    #[serde(default)]
    native_code_proposals: bool,
    #[serde(default)]
    reached_frontier_objective: bool,
    #[serde(default)]
    categorical_action_learning: bool,
    /// Original-parent action attribution only; not joint Context adaptation.
    #[serde(default)]
    categorical_action_only: bool,
    /// One composite shared-Q4 update constructed under native success constraints.
    #[serde(default)]
    constrained_context_learning: bool,
    /// Pinned Context endpoint for one shared Generate-only finite construction.
    #[serde(default)]
    constrained_emission_learning: bool,
    #[serde(default)]
    retained_context_root: Option<PathBuf>,
    #[serde(default)]
    prototype_compensation: Option<prototype_compensation::Config>,
    #[serde(default)]
    reached_u: Option<reached_u::Config>,
    #[serde(default)]
    readout_coadaptation: Option<readout_coadaptation::Config>,
    /// Fixed native-forward OFF/ON Prefix temporal Context-credit discriminator.
    #[serde(default)]
    prefix_context_credit: Option<prefix_context_credit::Config>,
    #[serde(default)]
    context_path_credit: Option<context_path_credit::Config>,
    #[serde(default)]
    context_cue_coadapt: Option<context_cue_coadapt::Config>,
    #[serde(default)]
    prefix_fragment_learning: Option<prefix_fragment_learning::Config>,
    #[serde(default)]
    generate_episode_learning: Option<generate_episode_learning::Config>,
    #[serde(default)]
    coupled_episode_learning: Option<coupled_episode_learning::Config>,
    #[serde(default)]
    generate_episode_completion: Option<generate_episode_learning::CompletionConfig>,
    #[serde(default)]
    prefix_artifact_check: Option<prefix_fragment_learning::ArtifactConfig>,
}
const CONTROL_INDICES: [usize; 8] = [0, 1, 4, 5, 8, 9, 12, 13];
fn default_updates() -> usize {
    UPDATES
}
const INPUT_SHA: &str = "b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a";
const LABEL_SHA: &str = "84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155";
const CP_RECEIPT_SHA: &str = "9196520209cb8ed172fea65a60be0a1f2fa788e40d5857c65bf93eccb658b816";
const CAT_SHA: &str = "af5039e74c6cd92e2fa0bf4c72c315ef34f912d77b09fc607215a18f86cd1d0e";
const QUARTER_SHA: &str = "d7e56ed124ad80619dad5b6bfb90dec5f78fe855f43ef4bfd9011e2e31f5fccf";
const GENERATE_SHA: &str = "41c7beae25e98fdc658ff667959a7c02a6b916b04298a425f7f7f48285d21511";
const UPDATES: usize = 128;
const BATCH: usize = 8;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize, serde::Serialize)]
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
impl Segment {
    fn frame(&self) -> Option<SelectedRecordFrame<'_>> {
        match self {
            Self::Source {
                record,
                commit,
                scope,
                entity,
                relation,
                view,
                original_source_ids,
                ..
            } => Some(SelectedRecordFrame {
                identity: SourceIdentity {
                    record: *record,
                    commit: *commit,
                },
                metadata: FrameMetadata {
                    scope: scope.as_bytes(),
                    entity,
                    relation: *relation,
                    view: *view,
                    status: FrameStatus::Found,
                },
                token_ids: original_source_ids,
            }),
            _ => None,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    schema: String,
    protocol: String,
    membership_only: bool,
    cases: Vec<Label>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Label {
    id: String,
    answers: FrozenAnswers,
    /// Transfer-panel pair linkage, declared so the strict schema still accepts
    /// panels that carry it. The entry read does not use it.
    #[serde(default)]
    pair_id: Option<String>,
}
struct Episode {
    packet: Packet,
    answers: FrozenAnswers,
    target: Vec<u32>,
    views: Vec<Option<SourceEmissionView>>,
}
impl Episode {
    fn segments(&self) -> Result<Vec<SourceBankSegment<'_>>> {
        self.packet
            .segments
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Ok(match s {
                    Segment::Source { event, .. } => SourceBankSegment::Source {
                        frame: s.frame().ok_or_else(|| bad("source frame missing"))?,
                        view: self.views[i]
                            .as_ref()
                            .ok_or_else(|| bad("source view missing"))?,
                        event: *event,
                    },
                    Segment::Context {
                        event,
                        role,
                        token_ids,
                    } => SourceBankSegment::Context {
                        event: *event,
                        role: *role,
                        token_ids,
                    },
                })
            })
            .collect()
    }
    fn base_len(&self) -> usize {
        self.packet.query_ids.len()
            + self
                .packet
                .segments
                .iter()
                .zip(&self.views)
                .map(|(s, v)| match (s, v) {
                    (Segment::Source { .. }, Some(v)) => v.emitted_token_ids().len(),
                    (Segment::Context { token_ids, .. }, _) => token_ids.len(),
                    _ => 0,
                })
                .sum::<usize>()
    }
    fn has_source(&self) -> bool {
        self.packet
            .segments
            .iter()
            .any(|s| matches!(s, Segment::Source { .. }))
    }
    fn causal_no_source(&self, prefix: &[u32]) -> Result<Vec<u32>> {
        let mut ids = Vec::new();
        for s in &self.packet.segments {
            match s {
                Segment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
                _ => return Err(bad("no-source branch contains Source")),
            }
        }
        ids.extend_from_slice(&self.packet.query_ids);
        ids.extend_from_slice(prefix);
        if ids.is_empty() || ids.len() > 128 {
            return Err(bad("no-source causal context outside1..128"));
        }
        Ok(ids)
    }
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn size(p: &Path) -> Result<u64> {
    let mut total = 0;
    for e in fs::read_dir(p)? {
        let p = e?.path();
        total += if p.is_dir() {
            size(&p)?
        } else {
            fs::metadata(p)?.len()
        };
    }
    Ok(total)
}
fn write(a: &Args, name: &str, v: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(v)?;
    let p = a.out.join(name);
    let previous = fs::metadata(&p).map_or(0, |m| m.len());
    if size(&a.out)? - previous + bytes.len() as u64 > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("report cap"));
    }
    fs::write(p, bytes)?;
    Ok(())
}
fn deadline(a: &Args, start: Instant) -> Result<()> {
    let elapsed = start.elapsed().as_secs_f64();
    if elapsed >= a.maximum_seconds as f64 && !a.out.join("wall-estimate-overrun.json").exists() {
        // Owner rule (6 October): a self-set estimate must not terminate a
        // progressing run. Resource/lease extensions still need ledger visibility
        // from the controlling lab; this notice does not renew a GPU lease.
        write(
            a,
            "wall-estimate-overrun.json",
            &json!({
                "schema":"uor-r4.wall-estimate-overrun/1",
                "declared_estimate_seconds":a.maximum_seconds,
                "first_observed_elapsed_seconds":elapsed,
                "action":"continue; self-set estimate is not a cancellation rule",
                "operator_obligation":"record extension on owning issue and renew active lease; owner spending caps, disk floor and failed/diverged work remain boundaries",
                "preflight_projection_admission":"unchanged"
            }),
        )?;
        eprintln!("Wall estimate exceeded after {elapsed:.3}s; continuing under owner rule. Record extension and renew the active lease.");
    }
    Ok(())
}
fn seal_for(path: &Path) -> Result<PathBuf> {
    for p in path.ancestors() {
        if p.join("manifest.json").is_file() {
            return Ok(p.to_path_buf());
        }
    }
    Err(bad("input has no enclosing seal"))
}

fn args() -> Result<(Args, Vec<u8>)> {
    let mut it = std::env::args().skip(1);
    let path = it.next().ok_or_else(|| bad("one JSON config required"))?;
    if it.next().is_some() {
        return Err(bad("one JSON config only"));
    }
    let raw = fs::read(path)?;
    let a: Args = serde_json::from_slice(&raw)?;
    reply_completion::settings(&a)?;
    cross_state_completion::settings(&a)?;
    control_settings(&a)?;
    continuation_settings(&a)?;
    joint_continuation_settings(&a)?;
    reference_replay_settings(&a)?;
    if ![1001, 1002, 1003].contains(&a.seed)
        || a.maximum_seconds == 0
        || a.maximum_report_bytes < (64 << 20)
        || a.maximum_report_bytes > if a.native_code_proposals { 4 << 30 } else { 2 << 30 }
        || (a.baseline.is_some() && a.mode != Mode::Fit)
        // A declared panel is a zero-update diagnostic read, never a fit input.
        || ((a.panel_inputs.is_some() || a.panel_labels.is_some())
            && a.mode != Mode::EntryCeiling)
        // Declared doses stay schedule-comparable with the retained campaign
        // and land on a written checkpoint, because recovery checkpoints are
        // written every 32 updates and the final reload reads the last one.
        || a.updates > if a.mode == Mode::CrossStateContinuation && a.cross_state_resume.is_some() {256} else {UPDATES}
        || (a.mode != Mode::JointContinuation && (a.updates == 0 || a.updates % 32 != 0))
    {
        return Err(bad("fixed order seeds/resource admission"));
    }
    validate_input_output_paths(&a)?;
    Ok((a, raw))
}

fn validate_input_output_paths(a: &Args) -> Result<()> {
    let output = output_support::prospective_output(&a.out)?;
    for path in [
        &a.checkpoint,
        &a.saved_fit,
        &a.categorical,
        &a.parent_config,
        &a.training_inputs,
        &a.training_labels,
        &a.development_inputs,
        &a.development_labels,
    ]
    .into_iter()
    .chain(a.baseline.iter())
    .chain(a.prediction_control_resume.iter())
    .chain(a.cross_state_resume.iter().map(|c| &c.root))
    .chain(a.retained_context_root.iter())
    .chain(
        a.prefix_context_credit
            .iter()
            .flat_map(|c| [&c.retained_intermediate_root, &c.retained_capture_root]),
    )
    .chain(
        a.prefix_context_credit
            .iter()
            .filter_map(|c| c.recorded_finite_contrast.as_ref())
            .flat_map(|c| [&c.retained_path_root, &c.retained_probe_root]),
    )
    .chain(
        a.generate_episode_completion
            .iter()
            .flat_map(|c| c.input_roots()),
    )
    .chain(
        a.coupled_episode_learning
            .iter()
            .flat_map(|c| c.input_roots()),
    )
    .chain(
        a.generate_episode_learning
            .iter()
            .flat_map(|c| c.input_roots()),
    )
    .chain(
        a.prefix_artifact_check
            .iter()
            .flat_map(|c| [&c.retained_candidate_root, &c.retained_intermediate_root]),
    )
    .chain(
        a.prefix_fragment_learning
            .iter()
            .flat_map(|c| [&c.retained_intermediate_root, &c.retained_probe_root]),
    )
    .chain(
        a.prefix_fragment_learning
            .iter()
            .filter_map(|c| c.trajectory.as_ref())
            .flat_map(|t| {
                [
                    &t.retained_prefix_learning_root,
                    &t.retained_supplement_root,
                ]
            }),
    )
    .chain(
        a.prefix_fragment_learning
            .iter()
            .filter_map(|c| c.joint.as_ref())
            .flat_map(|j| {
                [
                    &j.p5_capture.root,
                    &j.p6_conditional_capture.root,
                    &j.retained_supplement_root,
                ]
            }),
    )
    .chain(
        a.prefix_fragment_learning
            .iter()
            .filter_map(|c| c.episode.as_ref())
            .flat_map(|e| {
                let mut roots = vec![
                    &e.typed_authority,
                    &e.retained_projection.root,
                    &e.retained_supplement_root,
                ];
                roots.extend(e.phases.iter().map(|p| &p.capture.root));
                roots
            }),
    )
    .chain(a.context_cue_coadapt.iter().flat_map(|c| {
        [
            &c.retained_intermediate_root,
            &c.retained_probe_root,
            &c.retained_finite_root,
        ]
    }))
    .chain(a.context_path_credit.iter().flat_map(|c| {
        [
            &c.retained_decomposition_root,
            &c.retained_probe_root,
            &c.retained_intermediate_root,
            &c.retained_context_root,
        ]
    }))
    .chain(a.reached_u.iter().map(|c| &c.retained_compensation_root))
    .chain(
        a.readout_coadaptation
            .iter()
            .map(|c| &c.retained_reached_u_root),
    )
    .chain(
        a.readout_coadaptation
            .iter()
            .filter_map(|c| c.intermediate_candidate.as_ref())
            .flat_map(|c| [&c.original_readout_root, &c.margin_root]),
    )
    .chain(
        a.prototype_compensation
            .iter()
            .flat_map(|c| [&c.retained_emission_root, &c.prototype_diagnostic_root]),
    )
    .chain(
        a.reference_replay
            .iter()
            .flat_map(|r| [&r.plan_root, &r.reference_root]),
    ) {
        let input = fs::canonicalize(path)?;
        if output.starts_with(&input) || input.starts_with(&output) {
            return Err(bad("output/input overlap"));
        }
        for ancestor in input.ancestors() {
            if ancestor.join("manifest.json").exists() && output.starts_with(ancestor) {
                return Err(bad("output beneath input seal"));
            }
        }
    }
    Ok(())
}
#[cfg(feature = "cuda")]
fn cuda() -> Result<Device> {
    Ok(Device::new_cuda(0)?)
}
#[cfg(not(feature = "cuda"))]
fn cuda() -> Result<Device> {
    Err(bad("CUDA feature/device required; no CPU fallback"))
}
fn cue_payload(root: &Path) -> Result<CueAngularQ4> {
    let meta = read(&root.join("native-metadata.json"))?;
    let config: CueAngularConfig = serde_json::from_value(meta["potential"].clone())?;
    let packed = fs::read(root.join("cue-q4.bin"))?;
    let mut q = CueAngularQ4::new(config, &packed)?;
    if let Some(j) = meta.get("joint") {
        let m: CueJointMetadata = serde_json::from_value(j.clone())?;
        let joint = CueJointQ4::new(m.config, fs::read(root.join("cue-joint-q4.bin"))?)?;
        if joint.metadata() != m {
            return Err(bad("cue joint payload receipt"));
        }
        q = q.with_joint(joint)?;
    } else if root.join("cue-joint-q4.bin").exists() {
        return Err(bad("unreceipted cue joint"));
    }
    Ok(q)
}
fn prefix_payload(root: &Path) -> Result<PrefixAngularQ4> {
    let m = read(&root.join("native-metadata.json"))?;
    let c: PrefixAngularConfig = serde_json::from_value(m["potential"].clone())?;
    Ok(PrefixAngularQ4::new(
        c,
        &fs::read(root.join("prefix-q4.bin"))?,
    )?)
}
fn load_panel(
    inputs: &Path,
    labels: &Path,
    integer: &IntegerRealizer,
    tok: &ByteBpeTokenizer,
    legal: &BTreeSet<u32>,
    max: usize,
) -> Result<Vec<Episode>> {
    let i: Inputs = serde_json::from_slice(&fs::read(inputs)?)?;
    let l: Labels = serde_json::from_slice(&fs::read(labels)?)?;
    if i.schema != "uor-r4.native-source-bank-probe-input/1"
        || l.schema != "uor-r4.native-source-bank-labels/1"
        || l.protocol != "uor-r4.literal-role-dialogue/2"
        || !l.membership_only
        || i.cases.is_empty()
        || i.cases.len() > max
        || i.cases.len() != l.cases.len()
    {
        return Err(bad("complete panel/schema/count contract"));
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (p, l) in i.cases.into_iter().zip(l.cases) {
        l.answers.validate()?;
        if p.id.is_empty()
            || p.id != l.id
            || !seen.insert(p.id.clone())
            || !p.actual_prefix_ids.is_empty()
            || p.query_ids.is_empty()
            || p.segments.len() > 128
            || l.answers.accepted.is_empty()
        {
            return Err(bad("case identity or complete-answer prefix contract"));
        }
        let mut target = tok.encode(&l.answers.accepted[0]);
        if tok.decode_bytes(&target) != l.answers.accepted[0].as_bytes() {
            return Err(bad("canonical answer roundtrip"));
        }
        target.push(integer.binding().eos_token_id());
        if target.len() > 32 {
            return Err(bad("answer exceeds32 token budget"));
        }
        let views = p
            .segments
            .iter()
            .map(|s| {
                s.frame()
                    .map(|f| integer.compile_view(f.token_ids))
                    .transpose()
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let e = Episode {
            packet: p,
            answers: l.answers,
            target,
            views,
        };
        let mut base = e.packet.query_ids.len();
        for (s, v) in e.packet.segments.iter().zip(&e.views) {
            base += match (s, v) {
                (Segment::Source { .. }, Some(v)) => v.emitted_token_ids().len(),
                (Segment::Context { token_ids, .. }, _) => token_ids.len(),
                _ => return Err(bad("view/source mismatch")),
            };
        }
        if base + e.target.len() > 128 {
            return Err(bad("complete teacher-prefix context exceeds128"));
        } // Eligibility is public token membership, never model-quality filtering.
        let mut ids = e.packet.query_ids.clone();
        ids.extend_from_slice(&e.target);
        for (s, v) in e.packet.segments.iter().zip(&e.views) {
            match s {
                Segment::Source {
                    original_source_ids,
                    ..
                } => {
                    ids.extend_from_slice(original_source_ids);
                    ids.extend_from_slice(
                        v.as_ref().ok_or_else(|| bad("view"))?.emitted_token_ids(),
                    );
                }
                Segment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
            }
        }
        if ids.iter().any(|id| !legal.contains(id)) {
            return Err(bad("panel token outside actual public action binding"));
        }
        out.push(e);
    }
    Ok(out)
}

fn prefix_clone(q: &PrefixAngularQ4) -> Result<PrefixAngularQ4> {
    Ok(PrefixAngularQ4::new(q.config(), q.packed_coefficients())?)
}
fn restore(
    root: &Path,
    inventory: &Value,
    vars: &BTreeMap<String, Var>,
    device: &Device,
) -> Result<()> {
    let map = inventory
        .as_object()
        .ok_or_else(|| bad("master inventory absent"))?;
    if map.keys().collect::<BTreeSet<_>>() != vars.keys().collect::<BTreeSet<_>>() {
        return Err(bad("master parameter inventory differs"));
    }
    for (name, var) in vars {
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
        {
            return Err(bad("unsafe master name"));
        }
        let m = &map[name];
        let shape: Vec<usize> = serde_json::from_value(m["shape"].clone())?;
        let bytes = fs::read(root.join(format!("{name}.f32le")))?;
        if shape != var.dims()
            || bytes.len()
                != var
                    .elem_count()
                    .checked_mul(4)
                    .ok_or_else(|| bad("master size overflow"))?
            || m["bytes"].as_u64() != Some(bytes.len() as u64)
            || m["sha256"].as_str() != Some(sha256_bytes(&bytes).as_str())
        {
            return Err(bad("master shape/hash/bytes differ"));
        }
        let values = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        if values
            .iter()
            .any(|x| !x.is_finite() || (!name.ends_with("prototype_choices") && x.abs() > 1.75))
        {
            return Err(bad("master finite/range admission"));
        }
        var.set(&Tensor::from_vec(values, shape.as_slice(), device)?)?;
    }
    Ok(())
}
fn identities(vars: &BTreeMap<String, Var>) -> Result<BTreeMap<String, String>> {
    vars.iter()
        .map(|(n, v)| {
            let b = v
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>();
            Ok((n.clone(), sha256_bytes(&b)))
        })
        .collect()
}
/// Which occurrence the bank read reads. The retained default takes the earliest
/// raw Copy maximum, which the entry probe measured to be independent of the
/// question and of the answer.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BankSelection {
    EarliestCopyMax,
    QueryConditioned,
}

fn bank_selection(a: &Args) -> BankSelection {
    if a.query_conditioned_read {
        BankSelection::QueryConditioned
    } else {
        BankSelection::EarliestCopyMax
    }
}

/// Pick the occurrence whose cumulative source state is closest to the query
/// state, so what was asked decides what is read. Integer-only: an L1 sum over
/// H4 code indices, with no product of runtime values. Ties fall back to the
/// summed Copy score and then to the earliest occurrence.
fn query_conditioned_copy_choice(
    query: &[uor_r4_integer::h4_tables::H4Code],
    bank: &uor_r4_integer::geometric_source_realizer::BankRealizerTrace,
    scores: &[i64],
) -> Result<usize> {
    if bank.candidates.is_empty() {
        return Err(bad("bank read has no candidates"));
    }
    let mut best: Option<(i64, i64, usize)> = None;
    for (index, candidate) in bank.candidates.iter().enumerate() {
        let state = bank
            .context
            .states
            .get(candidate.context_position)
            .ok_or_else(|| bad("candidate cumulative source state missing"))?;
        if state.len() != query.len() {
            return Err(bad("source and query state widths differ"));
        }
        let mut distance = 0i64;
        for (q, &s) in query.iter().zip(state.iter()) {
            distance = distance
                .checked_add((q.index() as i64 - i64::from(s)).abs())
                .ok_or_else(|| bad("query/source distance overflow"))?;
        }
        let key = (
            distance,
            scores
                .get(index)
                .copied()
                .unwrap_or(i64::MIN)
                .saturating_neg(),
            index,
        );
        if best.is_none_or(|current| key < current) {
            best = Some(key);
        }
    }
    Ok(best.map(|(_, _, index)| index).unwrap_or(0))
}

fn earliest_raw_copy_max(scores: &[i64]) -> Result<usize> {
    let mut selected = 0;
    let mut best = *scores
        .first()
        .ok_or_else(|| bad("bridge has no Copy candidates"))?;
    for (i, &score) in scores.iter().enumerate().skip(1) {
        if score > best {
            selected = i;
            best = score;
        }
    }
    Ok(selected)
}
fn native_step(
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: Option<&NativeGeometricReadStateBridge>,
    pool: &mut NativeVocabularyActions,
    e: &Episode,
    actual: &[u32],
    cueq: &CueAngularQ4,
    prefixq: &PrefixAngularQ4,
    scorer: Option<&EntryScorer>,
    selection: BankSelection,
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let (codes, copy_ids, copy_scores, provenance) = if e.has_source() {
        let cue = model.compile_cue_carrier(cueq.clone())?;
        let prefix = model.compile_prefix_transport(&cue, prefix_clone(prefixq)?)?;
        let trace = model.read_bank_with_prefix_transport(
            &e.segments()?,
            &e.packet.query_ids,
            actual,
            &cue,
            &prefix,
        )?;
        let b = &trace.cue_bank.bank;
        let mut codes = b
            .context
            .states
            .last()
            .ok_or_else(|| bad("bank final state absent"))?
            .iter()
            .copied()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let ids = b
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        let mut scores = vec![0i64; ids.len()];
        for h in &b.heads {
            for (i, &s) in h.scores_q24.iter().enumerate() {
                scores[i] = scores[i]
                    .checked_add(s)
                    .ok_or_else(|| bad("Copy score overflow"))?;
            }
        }
        let bridge_trace = if let Some(bridge) = bridge {
            let selected = match selection {
                BankSelection::EarliestCopyMax => earliest_raw_copy_max(&scores)?,
                BankSelection::QueryConditioned => {
                    query_conditioned_copy_choice(&codes, b, &scores)?
                }
            };
            let candidate = &b.candidates[selected];
            let source = b
                .context
                .states
                .get(candidate.context_position)
                .ok_or_else(|| bad("bridge selected source state missing"))?
                .iter()
                .copied()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let mut distinct_source_states = BTreeSet::new();
            for candidate in &b.candidates {
                let state = b
                    .context
                    .states
                    .get(candidate.context_position)
                    .ok_or_else(|| bad("alternative source state missing"))?;
                distinct_source_states.insert(state.clone());
            }
            let query = codes.clone();
            let mut actions = vec![H4Code::IDENTITY; codes.len()];
            let mut action_scores = vec![0i64; codes.len() * 120];
            let mut counts = BridgeReadCounts::default();
            bridge.apply_into(
                &query,
                &source,
                &mut codes,
                &mut actions,
                &mut action_scores,
                &mut counts,
            )?;
            json!({"distinct_candidate_source_states":distinct_source_states.len(),"selected_ordinal":selected,"selected_candidate":candidate,"query_state_codes":query.iter().map(|c|c.index()).collect::<Vec<_>>(),"selected_source_state_codes":source.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_codes":actions.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_scores_q24":action_scores,"native_costs":counts,"payload_sha256":bridge.metadata().payload_sha256})
        } else {
            Value::Null
        };
        let mut components = BTreeMap::new();
        for (name, source) in [
            ("cue", &trace.cue_bank.carrier.copy_q24),
            ("prefix", &trace.prefix.copy_q24),
        ] {
            let mut totals = vec![0i64; ids.len()];
            if source.len() != b.heads.len() {
                return Err(bad("Copy component head count differs"));
            }
            for head in source {
                if head.len() != ids.len() {
                    return Err(bad("Copy component candidate count differs"));
                }
                for (j, &score) in head.iter().enumerate() {
                    totals[j] = totals[j]
                        .checked_add(score)
                        .ok_or_else(|| bad("Copy component overflow"))?;
                }
            }
            components.insert(name, totals);
        }
        let contextual = scores
            .iter()
            .enumerate()
            .map(|(j, &score)| {
                score
                    .checked_sub(components["cue"][j])
                    .and_then(|s| s.checked_sub(components["prefix"][j]))
                    .ok_or_else(|| bad("Copy contextual attribution overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        components.insert("contextual", contextual);
        let mut provenance = json!({"copy_components_q24":components,"candidates":b.candidates,"causal_tokens":b.context.tokens,"legacy_terminal_actions_discarded":true});
        if bridge.is_some() {
            provenance["read_state_bridge"] = bridge_trace;
        }
        (codes, ids, scores, provenance)
    } else {
        let ids = e.causal_no_source(actual)?;
        let cfg = model.context_config();
        let (tables, geometry) = model.context_encoder_parts();
        let mut st = NativeContextState::new(cfg.heads, cfg.lanes_per_head)?;
        for &id in &ids {
            st.step(id as usize, tables, geometry)?;
        }
        (
            st.states().to_vec(),
            Vec::new(),
            Vec::new(),
            json!({"causal_tokens":ids,"no_source":true}),
        )
    };
    let mut gs = vec![0i64; g.vocab_size()];
    let mut counts = GenerateReadCounts::default();
    g.score_into(&codes, &mut gs, &mut counts)?;
    let actions = pool.reduce_trace(&gs, &copy_ids, &copy_scores)?;
    let mut summary = serde_json::to_value(&actions.summary)?;
    // The boundary decision is owned by the entry scorer when it has seen this
    // feature. At an empty prefix the answer is never a Copy candidate, so the
    // mixed pool cannot decide it; the override is recorded next to the pool's
    // own choice rather than applied silently.
    let mut entry_decision = json!({"applied":false});
    if actual.is_empty() {
        if let Some(scorer) = scorer {
            let feature = EntryScorer::feature_of(e);
            if let Some(chosen) = scorer.choose(&feature) {
                summary["chosen_token_id"] = json!(chosen);
                entry_decision = json!({"applied":true,"feature":feature,"token":chosen,
                    "pool_choice":actions.summary.chosen_token_id});
            }
        }
    }
    Ok(
        json!({"actual_prefix_ids":actual,"retained_state_codes":codes.iter().map(|s|s.index()).collect::<Vec<_>>(),"copy_token_ids":copy_ids,"copy_raw_scores_q24":copy_scores,"generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&gs)?),"pool":{"summary":summary,"token_masses":actions.token_masses.iter().map(|m|[m.token_id as u64,m.weight_q31,m.generate_weight_q31,m.copy_weight_q31]).collect::<Vec<_>>()},"entry_scorer":entry_decision,"source_provenance":provenance,"generate_costs":counts}),
    )
}
fn evaluate(
    a: &Args,
    name: &str,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: Option<&NativeGeometricReadStateBridge>,
    exp: &[u8],
    eps: &[Episode],
    tok: &ByteBpeTokenizer,
    cue: &CueAngularQ4,
    prefix: &PrefixAngularQ4,
    start: Instant,
    scorer: Option<&EntryScorer>,
) -> Result<Value> {
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), exp)?;
    let mut rows = Vec::new();
    let mut complete = 0;
    let mut ce = 0.;
    let mut tokens = 0;
    for e in eps {
        deadline(a, start)?;
        let mut canonical = Vec::new();
        let mut rowce = 0.;
        for (t, &target) in e.target.iter().enumerate() {
            let mut step = native_step(
                model,
                g,
                bridge,
                &mut pool,
                e,
                &e.target[..t],
                cue,
                prefix,
                scorer,
                bank_selection(a),
            )?;
            let total = step["pool"]["summary"]["total_weight_q31"]
                .as_u64()
                .ok_or_else(|| bad("native denominator missing"))?;
            let mass = step["pool"]["token_masses"]
                .as_array()
                .ok_or_else(|| bad("native token masses"))?
                .iter()
                .find(|m| m[0].as_u64() == Some(target as u64))
                .and_then(|m| m[1].as_u64())
                .ok_or_else(|| bad("full Generate target mass missing"))?;
            if total == 0 || mass == 0 {
                return Err(bad("positive native support violated"));
            }
            step["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("pool object"))?
                .remove("token_masses");
            let loss = -(mass as f64 / total as f64).ln();
            rowce += loss;
            canonical.push(json!({"target_label_only":target,"native_ce":loss,"native_target_mass":mass,"native_denominator":total,"native":step}));
            tokens += 1;
        }
        ce += rowce / e.target.len() as f64;
        let mut ids = Vec::new();
        let mut generated = Vec::new();
        let mut eos = false;
        let generation_allowance = 32usize.min(128usize.saturating_sub(e.base_len()));
        for _ in 0..generation_allowance {
            deadline(a, start)?;
            let mut step = native_step(
                model,
                g,
                bridge,
                &mut pool,
                e,
                &ids,
                cue,
                prefix,
                scorer,
                bank_selection(a),
            )?;
            let id = step["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("native chosen ID"))? as u32;
            step["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("pool object"))?
                .remove("token_masses");
            generated.push(step);
            ids.push(id);
            if id == model.binding().eos_token_id() {
                eos = true;
                break;
            }
        }
        let text = tok.decode(&ids[..ids.len() - usize::from(eos)]);
        let accepted = eos && e.answers.accepts(&text);
        complete += usize::from(accepted);
        let row = json!({"id":e.packet.id,"canonical_target_ids_labels_only":e.target,"native_equal_episode_ce":rowce/e.target.len()as f64,"canonical":canonical,"generation":generated,"generated_ids":ids,"decoded":text,"eos":eos,"generation_allowance":generation_allowance,"cutoff":if eos{None}else if generation_allowance<32{Some("context-cap")}else{Some("generation-cap")},"complete":accepted});
        let filename = format!("{name}-row-{:04}.json", rows.len());
        write(a, &filename, &row)?;
        rows.push(json!({"id":e.packet.id,"native_equal_episode_ce":rowce/e.target.len()as f64,"complete":accepted,"eos":eos,"generated_ids":ids,"row_file":filename,"row_sha256":sha256_file(&a.out.join(filename))?}));
    }
    let result = json!({"cases":eps.len(),"target_positions":tokens,"complete":complete,"native_equal_episode_ce":ce/eps.len()as f64,"rows":rows,"runtime":"integer context/Copy/Generate; one full-vocabulary pool; old terminals excluded; actual emitted feedback","selection_primary":"complete own-prefix answers","selection_tie":"native equal-episode CE; earliest checkpoint"});
    write(a, &format!("{name}.json"), &result)?;
    Ok(result)
}
const LOSS_PHASE_NAMES: [&str; 3] = ["entry", "later_copy_covered", "later_generate_only"];
fn loss_weight_policy(enabled: bool, scope: LossScope) -> &'static str {
    if !enabled {
        "legacy-equal-episode-token-mean/1"
    } else if scope == LossScope::EntryOnly {
        "entry-phase-only/1;entry-position0;later-phases-zero-weight;entry-weight-unchanged;empty-phase-zero;no-runtime-gate"
    } else {
        "equal-nonempty-phases-per-episode/1;entry-position0;later-exact-allsource-token-membership;empty-phase-zero;remaining-phases-equal;no-runtime-gate"
    }
}
struct EpisodeLossWeights {
    counts: [usize; 3],
    phases: Vec<usize>,
    weights: Vec<f64>,
}
/// Offline label weighting only, called after actual target-free action admission.
/// Candidate membership does not imply correct-source identity or force Copy.
/// Under `balanced` each nonempty phase receives the same total weight per
/// episode; `LossScope::EntryOnly` then zeroes every non-entry phase, leaving
/// the entry position's own balanced weight untouched.
fn episode_loss_weights(
    target: &[u32],
    admitted_copy_ids: &BTreeSet<u32>,
    batch_episodes: usize,
    balanced: bool,
    scope: LossScope,
) -> Result<EpisodeLossWeights> {
    if target.is_empty() || batch_episodes == 0 {
        return Err(bad("loss weighting requires nonempty episode and batch"));
    }
    let phases = target
        .iter()
        .enumerate()
        .map(|(t, id)| {
            if t == 0 {
                0
            } else if admitted_copy_ids.contains(id) {
                1
            } else {
                2
            }
        })
        .collect::<Vec<_>>();
    let mut counts = [0usize; 3];
    for &phase in &phases {
        counts[phase] += 1;
    }
    let nonempty = counts.iter().filter(|&&n| n != 0).count();
    let weights = phases
        .iter()
        .map(|&phase| {
            if !balanced {
                // Keep exact prior floating arithmetic and operation order.
                1. / target.len() as f64 / batch_episodes as f64
            } else if scope == LossScope::EntryOnly && phase != 0 {
                0.
            } else {
                1. / nonempty as f64 / counts[phase] as f64 / batch_episodes as f64
            }
        })
        .collect();
    Ok(EpisodeLossWeights {
        counts,
        phases,
        weights,
    })
}

fn reference_weights(rows: &mut [ReferenceRow]) -> Result<()> {
    let episodes = rows
        .iter()
        .filter(|r| r.eligible.iter().any(|&v| v))
        .count();
    replay_require(episodes > 0, "empty reference population")?;
    for row in rows {
        replay_require(
            row.targets.len() == row.phases.len()
                && row.targets.len() == row.eligible.len()
                && row.phases.iter().all(|&p| p < 3),
            "reference phase/eligibility shape",
        )?;
        let mut counts = [0usize; 3];
        for (&phase, &eligible) in row.phases.iter().zip(&row.eligible) {
            if eligible {
                counts[phase] += 1;
            }
        }
        let phases = counts.iter().filter(|&&n| n > 0).count();
        row.weight_denominators = row
            .phases
            .iter()
            .zip(&row.eligible)
            .map(|(&p, &eligible)| {
                if eligible {
                    episodes * phases * counts[p]
                } else {
                    0
                }
            })
            .collect();
        row.weights = row
            .weight_denominators
            .iter()
            .map(|&n| if n == 0 { 0. } else { 1. / n as f64 })
            .collect();
    }
    Ok(())
}
fn reference_saved_row(root: &Path, reference: &Value) -> Result<Value> {
    let name = reference["row_file"]
        .as_str()
        .ok_or_else(|| bad("reference row path absent"))?;
    let path = Path::new(name);
    replay_require(
        !path.is_absolute()
            && path
                .components()
                .all(|p| matches!(p, std::path::Component::Normal(_))),
        "unsafe reference row path",
    )?;
    let bytes = fs::read(root.join(path))?;
    replay_require(
        reference["row_sha256"] == sha256_bytes(&bytes),
        "reference row digest differs",
    )?;
    let value: Value = serde_json::from_slice(&bytes)?;
    replay_require(value["id"] == reference["id"], "reference row ID differs")?;
    Ok(value)
}
fn authenticate_reference_root(root: &Path) -> Result<(Value, Value)> {
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == REPLAY_REPORT_SHA
            && sha256_file(&root.join("manifest.json"))? == REPLAY_MANIFEST_SHA,
        "reference donor report/seal differs",
    )?;
    let report = read(&root.join("report.json"))?;
    let initial = read(&root.join("checkpoint-0000/receipt.json"))?;
    replay_require(
        report["status"] == "COMPLETED"
            && report["mode"] == "joint_continuation"
            && report["updates"] == 1
            && initial == report["initial_receipt"]
            && initial["step"] == 0
            && initial["parent_report_sha256"] == CONTINUATION_PARENT_REPORT_SHA
            && initial["parent_manifest_sha256"] == CONTINUATION_PARENT_MANIFEST_SHA
            && initial["training_input_sha256"] == INPUT_SHA
            && initial["training_labels_sha256"] == LABEL_SHA,
        "reference initial checkpoint provenance differs",
    )?;
    Ok((report, initial))
}
fn build_reference_plan(c: &ReferencePrepareConfig) -> Result<(ReferencePlan, Value)> {
    let (report, initial) = authenticate_reference_root(&c.reference_root)?;
    for (path, sha) in [
        (&c.training_inputs, INPUT_SHA),
        (&c.training_labels, LABEL_SHA),
    ] {
        report_output::verify(&seal_for(path)?)?;
        replay_require(sha256_file(path)? == sha, "reference input/labels differ")?;
    }
    let inputs: Inputs = serde_json::from_slice(&fs::read(&c.training_inputs)?)?;
    let labels: Labels = serde_json::from_slice(&fs::read(&c.training_labels)?)?;
    replay_require(
        inputs.schema == "uor-r4.native-source-bank-probe-input/1"
            && labels.schema == "uor-r4.native-source-bank-labels/1"
            && labels.protocol == "uor-r4.literal-role-dialogue/2"
            && labels.membership_only
            && inputs.cases.len() == 512
            && labels.cases.len() == 512,
        "reference complete512 schema differs",
    )?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
        c.reference_root
            .join("checkpoint-0000/native/tokenizer.json"),
    )?)
    .ok_or_else(|| bad("reference tokenizer absent"))?;
    let protocol = uor_r4_tokenizer::dialogue::DialogueProtocol::literal_roles_v2(&tok)?;
    let summary = read(&c.reference_root.join("development-0000.json"))?;
    replay_require(
        summary == report["initial_evaluation"],
        "reference initial evaluation differs",
    )?;
    let refs = summary["rows"]
        .as_array()
        .ok_or_else(|| bad("reference rows absent"))?;
    replay_require(refs.len() == 512, "reference saved population differs")?;
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut total_positions = 0;
    let mut phase_population = [0usize; 3];
    let mut phase_eligible = [0usize; 3];
    let mut eos_eligible = 0;
    for (index, (packet, label)) in inputs.cases.into_iter().zip(labels.cases).enumerate() {
        replay_require(
            !packet.id.is_empty()
                && seen.insert(packet.id.clone())
                && label.id == packet.id
                && refs[index]["id"] == packet.id
                && packet.actual_prefix_ids.is_empty(),
            "reference packet/label identity differs",
        )?;
        label.answers.validate()?;
        let answer = label
            .answers
            .accepted
            .first()
            .ok_or_else(|| bad("reference accepted answer absent"))?;
        let mut targets = tok.encode(answer);
        replay_require(
            tok.decode_bytes(&targets) == answer.as_bytes(),
            "reference answer roundtrip differs",
        )?;
        targets.push(protocol.eos_id);
        let saved = reference_saved_row(&c.reference_root, &refs[index])?;
        replay_require(
            saved["canonical_target_ids_labels_only"] == json!(targets),
            "reference canonical target sequence differs",
        )?;
        let canonical = saved["canonical"]
            .as_array()
            .ok_or_else(|| bad("reference canonical positions absent"))?;
        replay_require(
            canonical.len() == targets.len() && !targets.is_empty(),
            "reference canonical coverage differs",
        )?;
        let copy_ids: Vec<u32> =
            serde_json::from_value(canonical[0]["native"]["copy_token_ids"].clone())?;
        let union = copy_ids.iter().copied().collect();
        let phases = episode_loss_weights(&targets, &union, 1, true, LossScope::All)?.phases;
        let mut eligible = Vec::new();
        for (t, ((&target, &phase), step)) in targets.iter().zip(&phases).zip(canonical).enumerate()
        {
            let native = &step["native"];
            let pool = &native["pool"]["summary"];
            replay_require(
                step["target_label_only"] == target
                    && native["copy_token_ids"] == json!(copy_ids)
                    && native["continuation"]["actual_prefix_tokens"] == t
                    && pool["legal_generate_actions"] == 4096
                    && pool["copy_actions"] == copy_ids.len(),
                "reference target/prefix-count/support differs",
            )?;
            let correct = pool["chosen_token_id"] == target;
            eligible.push(correct);
            total_positions += 1;
            phase_population[phase] += 1;
            if correct {
                phase_eligible[phase] += 1;
                eos_eligible += usize::from(target == protocol.eos_id);
            }
        }
        rows.push(ReferenceRow {
            index,
            id: packet.id.clone(),
            packet_sha256: sha256_bytes(&serde_json::to_vec(&packet)?),
            saved_row_sha256: refs[index]["row_sha256"]
                .as_str()
                .ok_or_else(|| bad("row digest absent"))?
                .into(),
            targets,
            copy_ids,
            phases,
            eligible,
            weight_denominators: Vec::new(),
            weights: Vec::new(),
        });
    }
    reference_weights(&mut rows)?;
    let total_weight: f64 = rows.iter().flat_map(|r| &r.weights).sum();
    replay_require(
        (total_weight - 1.).abs() < 1e-12,
        "reference weights do not sum to one",
    )?;
    let eligible_episodes = rows
        .iter()
        .filter(|r| r.eligible.iter().any(|&v| v))
        .count();
    let eligible_positions: usize = phase_eligible.iter().sum();
    let mut phase_weights = [0.; 3];
    for row in &rows {
        for (&phase, &weight) in row.phases.iter().zip(&row.weights) {
            phase_weights[phase] += weight;
        }
    }
    let counts = json!({"episodes":512,"eligible_episodes":eligible_episodes,"positions":total_positions,"eligible_positions":eligible_positions,
        "excluded_positions":total_positions-eligible_positions,"phase_population":phase_population,"phase_eligible":phase_eligible,
        "eos_eligible":eos_eligible,"phase_weights":phase_weights,"total_reference_weight":total_weight});
    Ok((
        ReferencePlan {
            schema: "uor-r4.supervised-reference-plan/1".into(),
            reference_root: fs::canonicalize(&c.reference_root)?,
            source_binding: serde_json::from_value(initial["parent"].clone())?,
            generate_sha256: initial["generate_sha256"]
                .as_str()
                .ok_or_else(|| bad("initial Generate SHA absent"))?
                .into(),
            initial_receipt_sha256: sha256_file(
                &c.reference_root.join("checkpoint-0000/receipt.json"),
            )?,
            reference_report_sha256: REPLAY_REPORT_SHA.into(),
            reference_manifest_sha256: REPLAY_MANIFEST_SHA.into(),
            input_sha256: INPUT_SHA.into(),
            labels_sha256: LABEL_SHA.into(),
            eos_token_id: protocol.eos_id,
            rows,
        },
        counts,
    ))
}
fn prepare_reference_replay(path: &Path, reached: bool) -> Result<()> {
    let raw = fs::read(path)?;
    let mut c: ReferencePrepareConfig = serde_json::from_slice(&raw)?;
    replay_require(
        c.maximum_report_bytes == 64 << 20,
        "reference preparation requires64MiB cap",
    )?;
    c.out = output_support::prospective_output(&c.out)?;
    for input in [
        &mut c.reference_root,
        &mut c.training_inputs,
        &mut c.training_labels,
    ] {
        replay_require(
            input.is_absolute()
                && !input
                    .components()
                    .any(|v| matches!(v, std::path::Component::ParentDir)),
            "reference absolute nontraversing paths required",
        )?;
        *input = fs::canonicalize(&*input)?;
        let seal = seal_for(input)?;
        replay_require(
            !c.out.starts_with(&*input) && !input.starts_with(&c.out) && !c.out.starts_with(seal),
            "reference output overlaps sealed input",
        )?;
    }
    report_output::claim(&c.out)?;
    let started = Instant::now();
    let result = (|| -> Result<Value> {
        fs::write(c.out.join("config.json"), &raw)?;
        let (encoded, counts) = if reached {
            let (plan, counts) = frontier::build(&c)?;
            (serde_json::to_vec_pretty(&plan)?, counts)
        } else {
            let (plan, counts) = build_reference_plan(&c)?;
            (serde_json::to_vec_pretty(&plan)?, counts)
        };
        replay_require(
            (encoded.len() as u64 + raw.len() as u64) < c.maximum_report_bytes - (1 << 20),
            "reference preparation storage cap",
        )?;
        fs::write(c.out.join("plan.json"), encoded)?;
        let mut report = json!({"schema":"uor-r4.supervised-reference-preparation/1","status":"COMPLETED","plan_sha256":sha256_file(&c.out.join("plan.json"))?,
            "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "reference_report_sha256":REPLAY_REPORT_SHA,"reference_manifest_sha256":REPLAY_MANIFEST_SHA,"counts":counts,
            "eligibility":"frozen authenticated parent native winner equals supervised target at canonical position; all512/allphases",
            "weighting":"mean eligible episodes, then nonempty eligible phases, then eligible positions; total weight1",
            "prefix_evidence":"saved prefix lengths plus canonical targets; full prefix IDs source-bound to pinned producer, not stored independently",
            "native_calls":0,"optimizer_steps":0,"elapsed_seconds":started.elapsed().as_secs_f64()});
        if reached {
            report["schema"] = json!("uor-r4.reached-frontier-preparation/1");
            report["eligibility"] = json!("one first canonical divergence at actual prefix for each noncomplete episode, plus every actual accepted-complete trajectory position through EOS; alternate accepted token sequence explicitly unsupported in this pilot");
            report["weighting"] = json!(frontier::OBJECTIVE);
            report["prefix_evidence"] = json!("authenticated full saved emitted prefix IDs; all trace/prediction/EOS/decoded membership checks; no native forward");
        }
        Ok(report)
    })();
    let report = match &result {
        Ok(r) => r.clone(),
        Err(e) => {
            json!({"schema":if reached {"uor-r4.reached-frontier-preparation/1"} else {"uor-r4.supervised-reference-preparation/1"},"status":"FAILED","error":e.to_string(),"native_calls":0,"optimizer_steps":0})
        }
    };
    fs::write(
        c.out.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    report_output::seal(&c.out)?;
    report_output::verify(&c.out)?;
    result.map(|_| ())
}
fn load_reference_plan(
    a: &Args,
    parent: &ContinuationParent,
    eps: &[Episode],
) -> Result<Option<ReferencePlan>> {
    let Some(c) = &a.reference_replay else {
        return Ok(None);
    };
    reference_replay_settings(a)?;
    report_output::verify(&c.plan_root)?;
    replay_require(
        sha256_file(&c.plan_root.join("report.json"))? == c.expected_plan_report_sha256
            && sha256_file(&c.plan_root.join("manifest.json"))? == c.expected_plan_manifest_sha256,
        "reference preparation seal differs",
    )?;
    let report = read(&c.plan_root.join("report.json"))?;
    let plan_bytes = fs::read(c.plan_root.join("plan.json"))?;
    replay_require(
        report["schema"] == "uor-r4.supervised-reference-preparation/1"
            && report["status"] == "COMPLETED"
            && report["plan_sha256"] == sha256_bytes(&plan_bytes),
        "reference plan identity differs",
    )?;
    let plan: ReferencePlan = serde_json::from_slice(&plan_bytes)?;
    replay_require(
        plan.schema == "uor-r4.supervised-reference-plan/1"
            && plan.source_binding == parent.binding
            && plan.generate_sha256 == parent.generate_sha256
            && plan.reference_report_sha256 == REPLAY_REPORT_SHA
            && plan.reference_manifest_sha256 == REPLAY_MANIFEST_SHA
            && plan.input_sha256 == INPUT_SHA
            && plan.labels_sha256 == LABEL_SHA
            && plan.rows.len() == eps.len(),
        "reference/model/data identity differs",
    )?;
    let (mut rebuilt, _) = build_reference_plan(&ReferencePrepareConfig {
        reference_root: c.reference_root.clone(),
        training_inputs: a.training_inputs.clone(),
        training_labels: a.training_labels.clone(),
        out: a.out.clone(),
        maximum_report_bytes: 64 << 20,
    })?;
    // Location is provenance, not artifact identity. A fresh pod may mount the
    // identical sealed donor elsewhere; all content pins remain exact.
    rebuilt.reference_root = plan.reference_root.clone();
    replay_require(
        serde_json::to_vec(&rebuilt)? == serde_json::to_vec(&plan)?,
        "reference eligibility/weight reconstruction differs",
    )?;
    for (i, (row, e)) in plan.rows.iter().zip(eps).enumerate() {
        let copy = e
            .views
            .iter()
            .flatten()
            .flat_map(|v| v.emitted_token_ids().iter().copied())
            .collect::<BTreeSet<_>>();
        replay_require(
            row.index == i
                && row.id == e.packet.id
                && row.targets == e.target
                && row.packet_sha256 == sha256_bytes(&serde_json::to_vec(&e.packet)?)
                && row.copy_ids.iter().copied().collect::<BTreeSet<_>>() == copy
                && row.phases
                    == episode_loss_weights(&e.target, &copy, 1, true, LossScope::All)?.phases,
            "reference live episode/prefix/phase binding differs",
        )?;
    }
    write(a, "reference-plan.json", &serde_json::to_value(&plan)?)?;
    Ok(Some(plan))
}
fn combine_reference_gradients(
    task: &BTreeMap<String, Tensor>,
    reference: &BTreeMap<String, Tensor>,
    lambda: f64,
) -> Result<BTreeMap<String, Tensor>> {
    replay_require(
        lambda.is_finite() && lambda > 0.,
        "reference gradient coefficient invalid",
    )?;
    let mut combined = task
        .iter()
        .map(|(n, g)| (n.clone(), g.detach()))
        .collect::<BTreeMap<_, _>>();
    for (name, gradient) in reference {
        let weighted = gradient.affine(lambda, 0.)?;
        let value = match combined.get(name) {
            Some(current) => current.add(&weighted)?.detach(),
            None => weighted.detach(),
        };
        combined.insert(name.clone(), value);
    }
    Ok(combined)
}

fn reference_binding(a: &Args) -> Result<Value> {
    let c = a
        .reference_replay
        .as_ref()
        .ok_or_else(|| bad("reference config absent"))?;
    let mut binding = json!({"configuration":c,"plan_sha256":sha256_file(&c.plan_root.join("plan.json"))?,
        "copied_plan_sha256":sha256_file(&a.out.join(if a.reached_frontier_objective {"frontier-plan.json"} else {"reference-plan.json"}))?,
        "objective":"unchanged currentB8 phase-balanced CE + 1.0*hierarchical supervised parent-correct reference CE",
        "updates":1,"reference_passes":1,"eligibility_frozen_before_fit":true,
        "acceptance":"all8 parent complete replies retained and more than8/512 complete; teacher-position retention reported separately, never an own-prefix evaluation veto"});
    if a.reached_frontier_objective {
        binding["objective"] = json!(frontier::OBJECTIVE);
        binding["schema"] = json!("uor-r4.reached-frontier-plan/1");
        binding["legacy398_scope"] =
            json!("separate stability diagnostic only; not included in either training component");
    }
    if a.readout_coadaptation.is_some() {
        binding["objective"] = json!("five actual reached factual terms weight.2 + unchanged original84 successful trajectory terms;89positions");
        binding["schema"] = json!("uor-r4.readout-coadaptation-plan/1");
    }
    Ok(binding)
}

fn reference_gradient_norms(
    grads: &BTreeMap<String, Tensor>,
    groups: &[(&str, &BTreeMap<String, Var>)],
    d: &Device,
) -> Result<Value> {
    let mut result = BTreeMap::new();
    for (name, params) in groups {
        let subset = params
            .keys()
            .filter_map(|key| grads.get(key).map(|g| (key.clone(), g.clone())))
            .collect::<BTreeMap<_, _>>();
        let norm = if subset.is_empty() {
            0.
        } else {
            clip_denominator(&subset, d)?.1
        };
        result.insert(*name, json!({"l2":norm,"gradient_tensors":subset.len()}));
    }
    Ok(serde_json::to_value(result)?)
}
fn reference_native_loss(step: &Value, target: u32) -> Result<f64> {
    let mass = step["native_target_mass"]
        .as_u64()
        .ok_or_else(|| bad("saved target mass absent"))?;
    let total = step["native_denominator"]
        .as_u64()
        .ok_or_else(|| bad("saved denominator absent"))?;
    replay_require(
        step["target_label_only"] == target
            && mass > 0
            && mass <= total
            && step["native"]["pool"]["summary"]["total_weight_q31"] == total,
        "reference native target/pool invalid",
    )?;
    Ok(-(mass as f64 / total as f64).ln())
}
fn reference_outcomes(
    a: &Args,
    plan: &ReferencePlan,
    before: &Value,
    after: &Value,
    eps: &[Episode],
    task_indices: &[usize],
) -> Result<Value> {
    let old = before["rows"]
        .as_array()
        .ok_or_else(|| bad("initial reference rows absent"))?;
    let new = after["rows"]
        .as_array()
        .ok_or_else(|| bad("final reference rows absent"))?;
    replay_require(
        old.len() == plan.rows.len() && new.len() == plan.rows.len(),
        "reference evaluation population differs",
    )?;
    let mut reference_losses = [0.; 2];
    let mut task_losses = [0.; 2];
    let mut lost = Vec::new();
    let mut historical_initial_lost = Vec::new();
    let mut recovered_during_emission = Vec::new();
    let mut emission_lost = Vec::new();
    let mut emission_retained = Vec::new();
    let mut gains = Vec::new();
    let mut retained = [0usize; 3];
    let mut lost_by_phase = [0usize; 3];
    let mut gain_by_phase = [0usize; 3];
    let mut positions = Vec::new();
    let mut factual_state_changed = 0;
    let mut local_state_changed = 0;
    for (index, row) in plan.rows.iter().enumerate() {
        replay_require(
            old[index]["id"] == row.id && new[index]["id"] == row.id,
            "reference evaluation ID differs",
        )?;
        let initial = reference_saved_row(&a.out, &old[index])?;
        let final_row = reference_saved_row(&a.out, &new[index])?;
        let initial_steps = initial["canonical"]
            .as_array()
            .ok_or_else(|| bad("initial canonical absent"))?;
        let final_steps = final_row["canonical"]
            .as_array()
            .ok_or_else(|| bad("final canonical absent"))?;
        replay_require(
            initial["canonical_target_ids_labels_only"] == json!(row.targets)
                && final_row["canonical_target_ids_labels_only"] == json!(row.targets)
                && initial_steps.len() == row.targets.len()
                && final_steps.len() == row.targets.len(),
            "reference evaluation target coverage differs",
        )?;
        let copy = row.copy_ids.iter().copied().collect();
        let task = episode_loss_weights(
            &eps[index].target,
            &copy,
            task_indices.len().max(1),
            true,
            LossScope::All,
        )?;
        for (t, &target) in row.targets.iter().enumerate() {
            let left = &initial_steps[t];
            let right = &final_steps[t];
            let native_losses = [
                reference_native_loss(left, target)?,
                reference_native_loss(right, target)?,
            ];
            let was = left["native"]["pool"]["summary"]["chosen_token_id"] == target;
            let now = right["native"]["pool"]["summary"]["chosen_token_id"] == target;
            replay_require(
                a.constrained_emission_learning || was == row.eligible[t],
                "actual initial eligibility differs from frozen parent",
            )?;
            let phase = row.phases[t];
            let identity = json!({"index":index,"id":row.id,"position":t,"target":target,"phase":phase,"eos":target==plan.eos_token_id});
            factual_state_changed += usize::from(
                left["native"]["post_state_codes"] != right["native"]["post_state_codes"],
            );
            local_state_changed += usize::from(
                left["native"]["continuation"]["state_codes"]
                    != right["native"]["continuation"]["state_codes"],
            );
            if row.eligible[t] {
                if !was {
                    historical_initial_lost.push(identity.clone());
                }
                if !was && now {
                    recovered_during_emission.push(identity.clone());
                }
                if was && !now {
                    emission_lost.push(identity.clone());
                }
                if was && now {
                    emission_retained.push(identity.clone());
                }
                for arm in 0..2 {
                    reference_losses[arm] += row.weights[t] * native_losses[arm];
                }
                if now {
                    retained[phase] += 1;
                } else {
                    lost_by_phase[phase] += 1;
                    lost.push(identity.clone());
                }
                positions.push(json!({"identity":identity,"weight":row.weights[t],"weight_denominator":row.weight_denominators[t],
                    "native_losses_before_after":native_losses,"historical_eligible_correct_after":now,"correct_at_immediate_input":was,"retained":was && now,
                    "before_mass":left["native_target_mass"],"before_total":left["native_denominator"],
                    "after_mass":right["native_target_mass"],"after_total":right["native_denominator"],
                    "after_chosen":right["native"]["pool"]["summary"]["chosen_token_id"]}));
            } else if now {
                gain_by_phase[phase] += 1;
                gains.push(identity);
            }
            if task_indices.contains(&index) {
                for arm in 0..2 {
                    task_losses[arm] += task.weights[t] * native_losses[arm];
                }
            }
        }
    }
    let retained_complete = old
        .iter()
        .zip(new)
        .filter(|(a, b)| a["complete"] == true && b["complete"] == true)
        .count();
    let initial_complete = old.iter().filter(|a| a["complete"] == true).count();
    let final_complete = new.iter().filter(|a| a["complete"] == true).count();
    let mut result = json!({"reference_losses_before_after":reference_losses,"current_batch_losses_before_after":task_losses,
        "combined_losses_before_after":[task_losses[0]+reference_losses[0],task_losses[1]+reference_losses[1]],"lambda":1.0,
        "reference_positions":positions,"reference_retained_by_phase":retained,"reference_lost_by_phase":lost_by_phase,
        "reference_lost_positions":lost,"all_reference_positions_retained":lost.is_empty(),
        "full_panel_wrong_to_correct_by_phase":gain_by_phase,"full_panel_wrong_to_correct_positions":gains,
        "factual_state_changed_positions":factual_state_changed,"local_context_state_changed_positions":local_state_changed,
        "initial_complete":initial_complete,"retained_parent_complete":retained_complete,"final_complete":final_complete,
        "useful_candidate":initial_complete==8 && retained_complete==8 && final_complete>8,
        "scope":"native saved canonical objectives/retention and independent own-prefix answers; teacher retention is diagnostic, not an output-evaluation veto"});
    if a.constrained_emission_learning {
        result["historical_initial_lost_positions"] = json!(historical_initial_lost);
        result["recovered_during_emission_positions"] = json!(recovered_during_emission);
        result["lost_during_emission_positions"] = json!(emission_lost);
        result["retained_during_emission_positions"] = json!(emission_retained);
        let object = result
            .as_object_mut()
            .ok_or_else(|| bad("reference diagnostic absent"))?;
        if let Some(value) = object.remove("reference_retained_by_phase") {
            object.insert("historical_eligible_correct_after_by_phase".into(), value);
        }
    }
    if a.reached_frontier_objective {
        let object = result
            .as_object_mut()
            .ok_or_else(|| bad("reference diagnostic object absent"))?;
        object.remove("current_batch_losses_before_after");
        object.remove("combined_losses_before_after");
        object.remove("lambda");
        result["scope"] = json!(if a.constrained_emission_learning {
            "original398 donor teacher-position diagnostic with original eligibility weights; current retained-Context initial correctness may differ; not the emission objective; no B8 draw"
        } else {
            "legacy398 canonical teacher-position stability diagnostic only; not the reached-frontier training objective; no B8 draw"
        });
    }
    Ok(result)
}

fn apply(
    optimizer: &mut AdamW,
    params: &BTreeMap<String, Var>,
    gradients: &BTreeMap<String, Tensor>,
    denominator: &Tensor,
) -> Result<()> {
    // A zero scalar graph creates an empty GradStore on the same device. Only the
    // admitted parameter group is inserted; separate optimizers share one clip.
    let first = params
        .values()
        .next()
        .ok_or_else(|| bad("empty optimizer group"))?;
    let mut store = first.as_tensor().sum_all()?.affine(0., 0.)?.backward()?;
    for (name, var) in params {
        if let Some(g) = gradients.get(name) {
            store.insert(var.as_tensor(), g.broadcast_div(denominator)?.detach());
        }
    }
    optimizer.step(&store)?;
    Ok(())
}

struct Loaded {
    source: SourceRealizerWeights,
    frozen: NativeSourceRealizer,
    integer: IntegerRealizer,
    tokenizer: ByteBpeTokenizer,
    generate: GenerateLearningWeights,
    original_bridge: BridgeLearningWeights,
    marker: BridgeLearningWeights,
    categorical: Option<CategoricalBridgeLearningWeights>,
    cue: CueAngularQ4,
    prefix: PrefixAngularQ4,
    exp: Vec<u8>,
    receipt: Value,
    categorical_receipt: Value,
}
impl Loaded {
    fn prepared_categorical(&self, d: &Device) -> Result<PreparedCategoricalBridge> {
        if let Some(weights) = &self.categorical {
            Ok(weights.prepare_native()?)
        } else {
            Ok(PreparedCategoricalBridge::from_bytes(
                &self.marker.export_native()?.to_bytes()?,
                self.generate.binding(),
                CAT_SHA,
                d,
            )?)
        }
    }
    fn categorical_bytes(&self) -> Result<Vec<u8>> {
        match &self.categorical {
            Some(weights) => Ok(weights.export_native()?.to_bytes()?),
            None => Ok(self.marker.export_native()?.to_bytes()?),
        }
    }
}
fn proposal_policy(a: &Args) -> Value {
    if a.prototype_compensation.is_some() {
        prototype_compensation::policy()
    } else if a.constrained_emission_learning {
        constrained_emission::policy()
    } else if a.constrained_context_learning {
        constrained_context::policy()
    } else if a.categorical_action_learning {
        categorical_proposals::policy_for(a.categorical_action_only)
    } else {
        native_proposals::policy(a.reached_frontier_objective)
    }
}

fn authenticate(a: &Args) -> Result<()> {
    report_output::verify(&a.saved_fit)?;
    report_output::verify(&a.categorical)?;
    // A declared panel replaces the pinned one for a zero-update read; its own
    // hashes are recorded in the report rather than asserted against constants.
    let declared_panel = a.panel_inputs.is_some() || a.panel_labels.is_some();
    if !declared_panel {
        for p in [
            &a.training_inputs,
            &a.training_labels,
            &a.development_inputs,
            &a.development_labels,
        ] {
            report_output::verify(&seal_for(p)?)?;
        }
    }
    let fit = read(&a.saved_fit.join("report.json"))?;
    let receipt = read(&a.checkpoint.join("receipt.json"))?;
    if fit["schema"] != "uor-r4.geometric-bank-generate-fit/1"
        || fit["status"] != "COMPLETED"
        || fit["mode"] != "fit"
        || fit["arm"] != "joint-potential"
        || fit["updates"] != 128
        || fs::canonicalize(a.saved_fit.join("checkpoint-0128"))?
            != fs::canonicalize(&a.checkpoint)?
        || sha256_file(&a.checkpoint.join("receipt.json"))? != CP_RECEIPT_SHA
        || fit["stages"]
            .as_array()
            .and_then(|v| v.last())
            .map(|v| &v["checkpoint"])
            != Some(&receipt)
    {
        return Err(bad("exact retained final checkpoint admission"));
    }
    if !declared_panel {
        for (p, expected) in [
            (&a.training_inputs, INPUT_SHA),
            (&a.development_inputs, INPUT_SHA),
            (&a.training_labels, LABEL_SHA),
            (&a.development_labels, LABEL_SHA),
        ] {
            if sha256_file(p)? != expected {
                return Err(bad("fixed 512-row panel hash differs"));
            }
        }
    }
    let parent = read(&a.parent_config)?;
    for (key, expected) in [
        ("updates", json!(128)),
        ("learning_rate", json!(0.003)),
        ("prototype_learning_rate", json!(0.01)),
        ("context_learning_rate", json!(0.002)),
        ("potential_learning_rate", json!(0.003)),
        ("token_backward_chunk", json!(1)),
        ("phase_balanced_token_loss", json!(true)),
        ("prefix_temporal_utility", json!(false)),
        ("arm", json!("joint-potential")),
    ] {
        if parent[key] != expected {
            return Err(bad("retained fit configuration differs"));
        }
    }
    // Parent-config absolute paths are historical. Data/artifact identities above,
    // not guessed replacement ancestry, determine admission.
    Ok(())
}
fn load(a: &Args, d: &Device) -> Result<Loaded> {
    authenticate(a)?;
    let cp = &a.checkpoint;
    let receipt = read(&cp.join("receipt.json"))?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let integer = IntegerRealizer::load_native(&cp.join("native"), &binding)?;
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokbytes)
        .ok_or_else(|| bad("ByteBPE unavailable"))?;
    let source =
        SourceRealizerWeights::load_context_potential_on_device(&cp.join("source"), &tokbytes, d)?;
    let meta = read(&cp.join("native/metadata.json"))?;
    let identity: ConsumerIdentity = serde_json::from_value(meta["identity"].clone())?;
    let frozen = NativeSourceRealizer::load(&cp.join("native"), &source, &identity)?;
    if frozen.artifact_binding()? != binding {
        return Err(bad("source/native binding differs"));
    }
    let genbytes = fs::read(cp.join("generate.bin"))?;
    if sha256_bytes(&genbytes) != GENERATE_SHA {
        return Err(bad("Generate donor identity"));
    }
    let gn = NativeGeometricGenerate::from_bytes(&genbytes, integer.binding())?;
    let generate = GenerateLearningWeights::from_native(integer.binding().clone(), &gn, d)?;
    let gm = read(&cp.join("generate-source/metadata.json"))?;
    if gm["tokenizer_sha256"] != sha256_bytes(&tokbytes)
        || gm["protocol"] != serde_json::to_value(integer.binding().protocol())?
        || gm["lanes"] != json!(generate.lanes())
    {
        return Err(bad("Generate masters binding"));
    }
    restore(
        &cp.join("generate-source"),
        &gm["parameters"],
        &generate.parameters(),
        d,
    )?;
    if generate.export_native()?.to_bytes()? != genbytes {
        return Err(bad("restored Generate export differs"));
    }
    let bbytes = fs::read(cp.join("read-state-bridge.bin"))?;
    if sha256_bytes(&bbytes) != QUARTER_SHA {
        return Err(bad("quarter bridge donor identity"));
    }
    let bn = NativeGeometricReadStateBridge::from_bytes(&bbytes, integer.binding())?;
    let original_bridge = BridgeLearningWeights::from_native(&bn, integer.binding(), d)?;
    let bmbytes = fs::read(cp.join("read-state-bridge-source/metadata.json"))?;
    let bm: Value = serde_json::from_slice(&bmbytes)?;
    if bm["metadata"] != serde_json::to_value(bn.metadata())? || bm != receipt["read_state_bridge"]
    {
        return Err(bad("bridge donor master receipt differs"));
    }
    restore(
        &cp.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &original_bridge.parameters(),
        d,
    )?;
    if original_bridge.export_native()?.to_bytes()? != bbytes {
        return Err(bad("original quarter export differs"));
    }
    let rebuilt = original_bridge.export_categorical_actions()?;
    let cb = fs::read(a.categorical.join("read-state-bridge-categorical.bin"))?;
    let probe = read(&a.categorical.join("probe.json"))?;
    if sha256_bytes(&cb) != CAT_SHA
        || rebuilt.native.to_bytes()? != cb
        || probe["status"] != "COMPLETED"
        || probe["mode"] != "categorical"
        || probe["schema"] != "uor-r4.read-state-export-probe/1"
        || probe["checkpoint_receipt_sha256"] != CP_RECEIPT_SHA
        || probe["parent_manifest_sha256"] != sha256_file(&a.saved_fit.join("manifest.json"))?
        || probe["master_metadata_sha256"] != sha256_bytes(&bmbytes)
        || probe["master_identities"] != bm["source_parameters"]
        || probe["original_native_sha256"] != QUARTER_SHA
        || probe["derived_native_sha256"] != CAT_SHA
        || probe["derived_payload_sha256"] != rebuilt.native.metadata().payload_sha256
        || probe["original_coarse_reproduction"] != "BYTE_IDENTICAL"
        || probe["parent_binding"] != receipt["parent"]
        || probe["all_query_frame_mismatches"] != 0
        || probe["all_query_frame_checks"] != json!(generate.lanes() * 120 * 120)
    {
        return Err(bad(
            "categorical artifact does not bind exact retained masters",
        ));
    }
    // This separate marker object supplies a truthful quarter graph for its
    // categorical native energies. Its Vars are never in an optimizer or clip.
    let marker = BridgeLearningWeights::from_native(&rebuilt.native, integer.binding(), d)?;
    if marker.prepare_native()?.native.to_bytes()? != cb {
        return Err(bad("frozen marker/native energy snapshot differs"));
    }
    let cue = cue_payload(&cp.join("cue"))?;
    let prefix = prefix_payload(&cp.join("prefix"))?;
    let cc = frozen.compile_cue_carrier(cue.clone())?;
    let pp = frozen.compile_prefix_transport(&cc, prefix_clone(&prefix)?)?;
    if serde_json::to_value(cc.metadata())? != read(&cp.join("cue/native-metadata.json"))?
        || serde_json::to_value(pp.metadata())? != read(&cp.join("prefix/native-metadata.json"))?
    {
        return Err(bad("donor cue/prefix receipts differ"));
    }
    let exp = fs::read(cp.join("native/consumer/exp-q31.bin"))?;
    Ok(Loaded {
        source,
        frozen,
        integer,
        tokenizer,
        generate,
        original_bridge,
        marker,
        categorical: None,
        cue,
        prefix,
        exp,
        receipt,
        categorical_receipt: serde_json::to_value(rebuilt.receipt)?,
    })
}
fn active(
    source: &SourceRealizerWeights,
    g: &GenerateLearningWeights,
) -> Result<BTreeMap<String, Var>> {
    let mut vars = g.parameters();
    vars.extend(source.context_state_parameters());
    vars.extend(source.potential_parameters());
    if vars.keys().any(|n| n.starts_with("read_state_bridge.")) {
        return Err(bad("frozen bridge admitted to optimizer"));
    }
    Ok(vars)
}
fn order(seed: u64, n: usize) -> Vec<usize> {
    // Fully specified SplitMix64/Fisher-Yates order only; no model reseeding.
    let mut state = seed;
    let mut out = (0..n).collect::<Vec<_>>();
    for i in (1..n).rev() {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        out.swap(i, (z % (i as u64 + 1)) as usize);
    }
    out
}
fn batch(
    a: &Args,
    l: &Loaded,
    eps: &[Episode],
    indices: &[usize],
    d: &Device,
    start: Instant,
    independent: Option<&IntegerRealizer>,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    batch_live(a, l, eps, indices, d, start, independent, None)
}

fn joint_active(
    l: &Loaded,
    continuation: &ContinuationLearningWeights,
) -> Result<BTreeMap<String, Var>> {
    let mut parameters = active(&l.source, &l.generate)?;
    if parameters.remove("generate.prototype_choices").is_none() {
        return Err(bad("joint frozen prototype family absent"));
    }
    parameters.extend(continuation.parameters());
    if let Some(categorical) = &l.categorical {
        parameters.extend(categorical.parameters());
    }
    Ok(parameters)
}

fn batch_live(
    a: &Args,
    l: &Loaded,
    eps: &[Episode],
    indices: &[usize],
    d: &Device,
    start: Instant,
    independent: Option<&IntegerRealizer>,
    continuation: Option<&ContinuationLearningWeights>,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    batch_live_weighted(
        a,
        l,
        eps,
        indices,
        d,
        start,
        independent,
        continuation,
        None,
    )
}
fn supervised_prefix(targets: &[u32], position: usize) -> Result<&[u32]> {
    replay_require(
        position < targets.len(),
        "supervised position outside target sequence",
    )?;
    Ok(&targets[..position])
}

fn batch_live_weighted(
    a: &Args,
    l: &Loaded,
    eps: &[Episode],
    indices: &[usize],
    d: &Device,
    start: Instant,
    independent: Option<&IntegerRealizer>,
    continuation: Option<&ContinuationLearningWeights>,
    reference: Option<&ReferencePlan>,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    if continuation.is_some() && independent.is_some() {
        return Err(bad(
            "joint native parity requires the bound current full generator",
        ));
    }
    let current = l.source.compile_context_potential_rebound(&l.frozen)?;
    let prepared = l.source.prepare_context_potential_on_device(&current, d)?;
    let cue = current.compile_cue_carrier(l.cue.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let gs = l.generate.prepare_native()?;
    let bs = l.marker.prepare_native()?;
    if sha256_bytes(&bs.native.to_bytes()?) != CAT_SHA {
        return Err(bad("frozen categorical map changed"));
    }
    // The historical marker remains authenticated above; the explicit learned
    // mode prepares a fresh current action map and master snapshot below.
    let categorical = if a.read_state_pullback == ReadStatePullback::Categorical {
        Some(l.prepared_categorical(d)?)
    } else {
        None
    };
    let learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_prefix_temporal_utility(false);
    let mut learner = match a.read_state_pullback {
        ReadStatePullback::Legacy => learner.with_read_state_bridge(&l.marker, &bs)?,
        ReadStatePullback::Categorical => learner.with_categorical_read_state_bridge(
            categorical
                .as_ref()
                .ok_or_else(|| bad("categorical pullback snapshot absent"))?,
        )?,
    }
    .with_read_selector_credit(true);
    // Rebuild against both current upstream artifacts at every batch. No
    // FixedContinuationPosition (states or scores) survives an optimizer step.
    let field = continuation
        .map(|weights| weights.prepare_native(&current.execution_binding()?, &gs.native))
        .transpose()?;
    if let (Some(weights), Some(prepared_field)) = (continuation, field.as_ref()) {
        learner = learner
            .with_continuation_field(weights, prepared_field)?
            .with_continuation_context_credit(true)?;
    }
    let params = if let Some(weights) = continuation {
        joint_active(l, weights)?
    } else {
        active(&l.source, &l.generate)?
    };
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut positions = 0;
    let mut phase_positions = [0usize; 3];
    let mut phase_losses = [0.; 3];
    let mut pool = NativeVocabularyActions::new(l.generate.binding().clone(), &l.exp)?;
    for &index in indices {
        let e = eps.get(index).ok_or_else(|| bad("batch index absent"))?;
        let expected_union = e
            .views
            .iter()
            .flatten()
            .flat_map(|v| v.emitted_token_ids().iter().copied())
            .collect::<BTreeSet<_>>();
        let mut plan = None;
        let mut ep_loss = 0.;
        for (t, &target) in e.target.iter().enumerate() {
            if let Some(plan) = reference {
                let row = plan
                    .rows
                    .get(index)
                    .ok_or_else(|| bad("reference episode absent"))?;
                if !*row
                    .eligible
                    .get(t)
                    .ok_or_else(|| bad("reference position absent"))?
                {
                    continue;
                }
            }
            deadline(a, start)?;
            let actual_prefix = supervised_prefix(&e.target, t)?;
            let out = if e.has_source() {
                learner.forward_bank(
                    &e.segments()?,
                    &e.packet.query_ids,
                    actual_prefix,
                    &cue,
                    &prefix,
                )?
            } else if continuation.is_some() {
                learner.forward_no_source_with_query(
                    &e.causal_no_source(actual_prefix)?,
                    &e.packet.query_ids,
                    actual_prefix,
                )?
            } else {
                learner.forward_no_source(&e.causal_no_source(actual_prefix)?)?
            };
            let union = out.copy_token_ids.iter().copied().collect::<BTreeSet<_>>();
            if union != expected_union {
                return Err(bad("actual all-source Copy union changed"));
            }
            if let Some(model) = independent {
                let expected = native_step(
                    model,
                    &gs.native,
                    Some(&bs.native),
                    &mut pool,
                    e,
                    actual_prefix,
                    &l.cue,
                    &l.prefix,
                    None,
                    BankSelection::EarliestCopyMax,
                )?;
                let masses = out
                    .actions
                    .token_masses
                    .iter()
                    .map(|m| {
                        [
                            m.token_id as u64,
                            m.weight_q31,
                            m.generate_weight_q31,
                            m.copy_weight_q31,
                        ]
                    })
                    .collect::<Vec<_>>();
                if expected["pool"]["summary"] != serde_json::to_value(&out.actions.summary)?
                    || expected["pool"]["token_masses"] != json!(masses)
                    || expected["copy_token_ids"] != json!(out.copy_token_ids)
                    || expected["copy_raw_scores_q24"] != json!(out.copy_scores_q24)
                    || expected["retained_state_codes"]
                        != json!(out
                            .final_state_codes
                            .iter()
                            .map(|c| c.index())
                            .collect::<Vec<_>>())
                    || expected["generate_raw_scores_sha256"]
                        != sha256_bytes(&serde_json::to_vec(&out.generate.scores_q24)?)
                {
                    return Err(bad(
                        "independent native categorical bank/Generate/pool parity differs",
                    ));
                }
                let old = out
                    .loss_with_credit(target, VocabularyScoreAdjoint::Clipped)?
                    .to_scalar::<f32>()?;
                let raw = out
                    .loss_with_credit(target, VocabularyScoreAdjoint::RawIdentity)?
                    .to_scalar::<f32>()?;
                if old.to_bits() != raw.to_bits() || !old.is_finite() {
                    return Err(bad("matched credits have different forward losses"));
                }
                let (_, bridge) = out
                    .read_state_bridge
                    .as_ref()
                    .ok_or_else(|| bad("categorical bridge absent"))?;
                let nb = &expected["source_provenance"]["read_state_bridge"];
                if nb["selected_ordinal"] != json!(out.read_state_bridge.as_ref().map(|v| v.0))
                    || nb["action_codes"]
                        != json!(bridge
                            .action_codes
                            .iter()
                            .map(|c| c.index())
                            .collect::<Vec<_>>())
                    || nb["action_scores_q24"] != json!(bridge.action_scores_q24)
                {
                    return Err(bad(
                        "independent categorical selected/action parity differs",
                    ));
                }
            }
            // Labels enter only after the complete target-free native pool.
            if plan.is_none() {
                plan = Some(episode_loss_weights(
                    &e.target,
                    &union,
                    indices.len(),
                    true,
                    a.loss_scope,
                )?);
            }
            let plan = plan.as_ref().ok_or_else(|| bad("loss phases absent"))?;
            let phase = plan.phases[t];
            let weight = if let Some(reference) = reference {
                let row = &reference.rows[index];
                replay_require(
                    row.id == e.packet.id && row.targets[t] == target && row.phases[t] == phase,
                    "reference target/phase differs at live forward",
                )?;
                row.weights[t]
            } else {
                plan.weights[t]
            };
            let mass = out
                .actions
                .token_masses
                .iter()
                .find(|m| m.token_id == target)
                .ok_or_else(|| bad("target support absent"))?
                .weight_q31;
            let nll = -(mass as f64 / out.actions.summary.total_weight_q31 as f64).ln();
            if !nll.is_finite() {
                return Err(bad("nonfinite native token objective"));
            }
            total += nll * weight;
            ep_loss += nll * weight;
            positions += 1;
            phase_positions[phase] += 1;
            phase_losses[phase] += nll * weight;
            let grads = out
                .loss_with_credit(target, a.credit.policy())?
                .affine(weight, 0.)?
                .backward()?;
            // One token graph at a time; no detached context carrier is inserted.
            for (name, var) in &params {
                if let Some(g) = grads.get(var.as_tensor()) {
                    if !g.device().same_device(d)
                        || !g
                            .abs()?
                            .flatten_all()?
                            .max(0)?
                            .to_scalar::<f32>()?
                            .is_finite()
                    {
                        return Err(bad("gradient device/finite admission"));
                    }
                    let sum = if let Some(old) = sums.get(name) {
                        old.add(g)?.detach()
                    } else {
                        g.detach()
                    };
                    sums.insert(name.clone(), sum);
                }
            }
        }
        rows.push(
            json!({"id":e.packet.id,"index":index,"weighted_native_loss":ep_loss,
            "phase_counts":plan.map(|p|p.counts)}),
        );
    }
    let mut receipt = json!({"indices":indices,"rows":rows,"positions":positions,"weighted_native_loss":total,
        "phase_positions":phase_positions,"phase_losses":phase_losses,
        "independent_native_parity":independent.is_some(),
        "matched_forward_loss_bit_equal":independent.is_some(),
        "credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"token_backward_chunk":1,"frozen_bridge_excluded_from_gradient_accumulation":l.categorical.is_none(),
        "historical_quarter_and_marker_excluded_from_gradient_accumulation":true,
        "current_categorical_sha256":categorical.as_ref().map(|c|c.native_sha256()),
        "current_categorical_master_sha256":categorical.as_ref().and_then(|c|c.action_choices_sha256()),
        "categorical_master_download_bytes":categorical.as_ref().map(|c|c.downloaded_master_bytes()),
        "joint_continuation":continuation.is_some(),"prototype_excluded_from_gradient_accumulation_and_clip":continuation.is_some(),
        "current_source_binding":current.execution_binding()?,
        "current_generate_sha256":sha256_bytes(&gs.native.to_bytes()?),
        "current_continuation_sha256":field.as_ref().map(|f| f.native.to_bytes().map(|b| sha256_bytes(&b))).transpose()?});
    if reference.is_some() {
        receipt["reference_weighting"]=json!("externally normalized fixed episode/eligible-phase/eligible-position weights; no batch divisor");
        receipt["actual_teacher_prefix_policy"] =
            json!("e.target[..t], including preceding positions excluded from replay");
    }
    Ok((sums, receipt))
}
fn save_masters(root: &Path, vars: &BTreeMap<String, Var>) -> Result<Value> {
    fs::create_dir(root)?;
    let mut inventory = BTreeMap::new();
    for (name, var) in vars {
        let values = var.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad("nonfinite checkpoint master"));
        }
        let bytes = values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        fs::write(root.join(format!("{name}.f32le")), &bytes)?;
        inventory.insert(
            name,
            json!({"shape":var.dims(),"bytes":bytes.len(),"sha256":sha256_bytes(&bytes)}),
        );
    }
    Ok(json!(inventory))
}
fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(bad("unexpected donor master subdirectory"));
        }
        fs::copy(entry.path(), to.join(entry.file_name()))?;
    }
    Ok(())
}
fn checkpoint(
    a: &Args,
    step: usize,
    l: &Loaded,
) -> Result<(
    IntegerRealizer,
    NativeGeometricGenerate,
    NativeGeometricReadStateBridge,
    Value,
)> {
    let estimated = size(&a.checkpoint.join("source"))?
        + size(&a.checkpoint.join("native"))?
        + l.generate
            .parameters()
            .values()
            .map(|v| v.elem_count() as u64 * 4)
            .sum::<u64>()
        + 4_194_304;
    if size(&a.out)? + estimated > a.maximum_report_bytes - 1_048_576 {
        return Err(bad("checkpoint storage admission"));
    }
    let root = a.out.join(format!("checkpoint-{step:04}"));
    fs::create_dir(&root)?;
    let native = l.source.compile_context_potential_rebound(&l.frozen)?;
    l.source.save_source(&root.join("source"))?;
    native.save(&root.join("native"))?;
    let binding = native.execution_binding()?;
    let integer = IntegerRealizer::load_native(&root.join("native"), &binding)?;
    let tokbytes = fs::read(root.join("native/tokenizer.json"))?;
    let disk_source = SourceRealizerWeights::load_context_potential_on_device(
        &root.join("source"),
        &tokbytes,
        &Device::Cpu,
    )?;
    let disk_native = NativeSourceRealizer::load(
        &root.join("native"),
        &disk_source,
        &serde_json::from_value::<ConsumerIdentity>(
            read(&root.join("native/metadata.json"))?["identity"].clone(),
        )?,
    )?;
    if disk_native.artifact_binding()? != binding {
        return Err(bad("independent source/native master reload differs"));
    }
    let gm = save_masters(&root.join("generate-source"), &l.generate.parameters())?;
    fs::write(
        root.join("generate-source/metadata.json"),
        serde_json::to_vec_pretty(&json!({
        "parameters":gm,"tokenizer_sha256":integer.binding().tokenizer_sha256(),
        "protocol":integer.binding().protocol(),"lanes":l.generate.lanes(),"order_seed":a.seed}))?,
    )?;
    let genbytes = l.generate.export_native()?.to_bytes()?;
    fs::write(root.join("generate.bin"), &genbytes)?;
    let generate = NativeGeometricGenerate::from_bytes(
        &fs::read(root.join("generate.bin"))?,
        integer.binding(),
    )?;
    let disk_gen =
        GenerateLearningWeights::from_native(integer.binding().clone(), &generate, &Device::Cpu)?;
    restore(
        &root.join("generate-source"),
        &gm,
        &disk_gen.parameters(),
        &Device::Cpu,
    )?;
    if disk_gen.export_native()?.to_bytes()? != genbytes {
        return Err(bad("saved Generate masters/native reload differs"));
    }
    let quarter = l.original_bridge.export_native()?.to_bytes()?;
    if sha256_bytes(&quarter) != QUARTER_SHA {
        return Err(bad("original quarter bridge changed"));
    }
    fs::write(root.join("read-state-bridge.bin"), &quarter)?;
    copy_directory(
        &a.checkpoint.join("read-state-bridge-source"),
        &root.join("read-state-bridge-source"),
    )?;
    let cat = l.categorical_bytes()?;
    if l.categorical.is_none() && sha256_bytes(&cat) != CAT_SHA {
        return Err(bad("categorical frozen map changed"));
    }
    fs::write(root.join("read-state-bridge-categorical.bin"), &cat)?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(
        &fs::read(root.join("read-state-bridge-categorical.bin"))?,
        integer.binding(),
    )?;
    let rebind = NativeGeometricReadStateBridge::compile(
        integer.binding(),
        bridge.lanes(),
        bridge.packed_bias(),
        bridge.packed_relative(),
    )?;
    if rebind.to_bytes()? != cat {
        return Err(bad(
            "source action binding would change frozen categorical artifact",
        ));
    }
    let original_reload = NativeGeometricReadStateBridge::from_bytes(&quarter, integer.binding())?;
    let original_masters =
        BridgeLearningWeights::from_native(&original_reload, integer.binding(), &Device::Cpu)?;
    let bm = read(&root.join("read-state-bridge-source/metadata.json"))?;
    restore(
        &root.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &original_masters.parameters(),
        &Device::Cpu,
    )?;
    if original_masters.export_native()?.to_bytes()? != quarter
        || original_masters
            .export_categorical_actions()?
            .native
            .to_bytes()?
            != l.marker.export_native()?.to_bytes()?
    {
        return Err(bad(
            "independently restored historical original/map differs",
        ));
    }
    // Historical quarter masters retain their original map provenance. Learned
    // categorical choices have a separate current artifact and master receipt.
    let categorical_masters = if let Some(weights) = &l.categorical {
        let dir = root.join("categorical-action-source");
        let masters = save_masters(&dir, &weights.parameters())?;
        let restored = CategoricalBridgeLearningWeights::from_bytes(
            &cat,
            integer.binding(),
            &sha256_bytes(&cat),
            &Device::Cpu,
        )?;
        restore(&dir, &masters, &restored.parameters(), &Device::Cpu)?;
        replay_require(
            restored.export_native()?.to_bytes()? == cat,
            "categorical action masters/native independent reload differs",
        )?;
        replay_require(
            identities(&restored.parameters())? == identities(&weights.parameters())?,
            "categorical action restored master identities differ",
        )?;
        fs::write(
            dir.join("metadata.json"),
            serde_json::to_vec_pretty(&json!({
                "parameters":masters,"categorical_sha256":sha256_bytes(&cat),
                "original_categorical_sha256":CAT_SHA,"original_quarter_sha256":QUARTER_SHA,
                "lanes":weights.lanes(),"tokenizer_sha256":integer.binding().tokenizer_sha256()
            }))?,
        )?;
        Some(masters)
    } else {
        None
    };
    let cue = integer.compile_cue_carrier(l.cue.clone())?;
    let prefix = integer.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    fs::create_dir(root.join("cue"))?;
    fs::create_dir(root.join("prefix"))?;
    fs::write(
        root.join("cue/native-metadata.json"),
        serde_json::to_vec_pretty(cue.metadata())?,
    )?;
    fs::write(root.join("cue/cue-q4.bin"), l.cue.packed_coefficients())?;
    if let Some(j) = l.cue.joint() {
        fs::write(root.join("cue/cue-joint-q4.bin"), j.packed_coefficients())?;
    }
    fs::write(
        root.join("prefix/native-metadata.json"),
        serde_json::to_vec_pretty(prefix.metadata())?,
    )?;
    fs::write(
        root.join("prefix/prefix-q4.bin"),
        l.prefix.packed_coefficients(),
    )?;
    let dc = integer.compile_cue_carrier(cue_payload(&root.join("cue"))?)?;
    let dp = integer.compile_prefix_transport(&dc, prefix_payload(&root.join("prefix"))?)?;
    if serde_json::to_value(dc.metadata())? != read(&root.join("cue/native-metadata.json"))?
        || serde_json::to_value(dp.metadata())? != read(&root.join("prefix/native-metadata.json"))?
        || fs::read(root.join("native/consumer/exp-q31.bin"))? != l.exp
    {
        return Err(bad("frozen sidecar/exp payload reload differs"));
    }
    let old_binding = &l.receipt["parent"];
    let mut receipt = json!({"step":step,"parent":binding,"original_parent":old_binding,
        "source_metadata_rebound":serde_json::to_value(&binding)?!=*old_binding,
        "binding_scope":"source context/potential metadata honestly rebound; typed action bindings checked against current artifact; unchanged bridge coefficients, original masters and map bytes independently verified",
        "generate_sha256":sha256_bytes(&genbytes),"original_quarter_sha256":QUARTER_SHA,"categorical_sha256":sha256_bytes(&cat),
        "categorical_receipt":l.categorical_receipt,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"order_seed":a.seed,
        "native_independently_reloaded":true,"masters_independently_reloaded":true,
        "fresh_adam":"moments zero-initialized; not historical optimizer continuation",
        "frozen_bridge_training":"marker parameters excluded from Adam and clip; original masters unchanged"});
    if let Some(masters) = categorical_masters {
        receipt["categorical_action_learning"] = json!({"parameters":masters,
            "original_categorical_sha256":CAT_SHA,"current_categorical_sha256":sha256_bytes(&cat),
            "independent_master_and_native_reload":true});
        receipt["binding_scope"] = json!("current source metadata and current categorical action artifact independently rebound and reloaded; historical quarter masters and their parent map preserved separately");
        receipt["frozen_bridge_training"] = json!("historical quarter/marker masters unchanged; separately trained categorical action choices exported as one-marker zero-bias map");
    }
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok((integer, generate, bridge, receipt))
}

fn scoped_pairs(eps: &[Episode]) -> Result<Vec<(usize, usize, usize)>> {
    let ids = eps
        .iter()
        .enumerate()
        .map(|(i, e)| (e.packet.id.clone(), i))
        .collect::<BTreeMap<_, _>>();
    let mut covered = BTreeSet::new();
    let mut pairs = Vec::new();
    for (i, e) in eps.iter().enumerate() {
        if e.packet.id.contains("-swap0-") {
            let other = e.packet.id.replace("-swap0-", "-swap1-");
            let j = *ids
                .get(&other)
                .ok_or_else(|| bad("paired source-swap row absent"))?;
            let f = &eps[j];
            let pos = e
                .target
                .iter()
                .zip(&f.target)
                .position(|(a, b)| a != b)
                .ok_or_else(|| bad("source-swap targets do not diverge"))?;
            if pos == 0
                || e.packet.query_ids != f.packet.query_ids
                || !covered.insert(i)
                || !covered.insert(j)
            {
                return Err(bad("source-swap pairing/query/prefix admission"));
            }
            pairs.push((i, j, pos));
        }
    }
    if pairs.len() * 2 != eps.len() || covered.len() != eps.len() {
        return Err(bad("complete source-swap pair coverage required"));
    }
    Ok(pairs)
}
fn pairs(eps: &[Episode]) -> Result<Vec<(usize, usize, usize)>> {
    let p = scoped_pairs(eps)?;
    if p.len() != 256 {
        return Err(bad("complete256 source-swap pairs required"));
    }
    Ok(p)
}
fn metrics(a: &Args, eval: &Value, eps: &[Episode]) -> Result<Value> {
    let refs = eval["rows"]
        .as_array()
        .ok_or_else(|| bad("evaluation rows missing"))?;
    if refs.len() != eps.len() {
        return Err(bad("metric row coverage differs"));
    }
    let mut rows = Vec::new();
    let mut entry_teacher = 0;
    let mut entry_own = 0;
    let mut copy_total = 0;
    let mut copy_correct = 0;
    for (r, e) in refs.iter().zip(eps) {
        if r["id"] != e.packet.id {
            return Err(bad("metric row IDs differ"));
        }
        let name = r["row_file"]
            .as_str()
            .ok_or_else(|| bad("metric row filename absent"))?;
        let path = a.out.join(name);
        if sha256_file(&path)? != r["row_sha256"] {
            return Err(bad("metric row hash differs"));
        }
        let row = read(&path)?;
        let canonical = row["canonical"]
            .as_array()
            .ok_or_else(|| bad("canonical steps absent"))?;
        let generated: Vec<u32> = serde_json::from_value(row["generated_ids"].clone())?;
        if canonical.len() != e.target.len() {
            return Err(bad("complete canonical metric coverage"));
        }
        let chosen = |t: usize| -> Result<u32> {
            Ok(canonical[t]["native"]["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("canonical chosen ID absent"))? as u32)
        };
        entry_teacher += usize::from(chosen(0)? == e.target[0]);
        entry_own += usize::from(generated.first() == e.target.first());
        for (t, &target) in e.target.iter().enumerate().skip(1) {
            let copy: Vec<u32> =
                serde_json::from_value(canonical[t]["native"]["copy_token_ids"].clone())?;
            if copy.contains(&target) {
                copy_total += 1;
                copy_correct += usize::from(chosen(t)? == target);
            }
        }
        rows.push((canonical.clone(), generated));
    }
    let mut teacher_count = 0;
    let mut own_count = 0;
    let mut reached_count = 0;
    let mut reached_correct_count = 0;
    let mut teacher_both = 0;
    let mut own_both = 0;
    let mut reached_correct_both = 0;
    let mut complete_both = 0;
    let mut receipts = Vec::new();
    let paired = if a.mode == Mode::PredictionControl {
        scoped_pairs(eps)?
    } else {
        pairs(eps)?
    };
    for (i, j, pos) in paired {
        let mut arms = Vec::new();
        for k in [i, j] {
            let target = eps[k].target[pos];
            let canonical = &rows[k].0;
            let generated = &rows[k].1;
            let chosen = canonical[pos]["native"]["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("source-word chosen ID absent"))?
                as u32;
            let teacher = chosen == target;
            let own = generated.get(pos) == Some(&target);
            let reached = generated.len() > pos && generated[..pos] == eps[k].target[..pos];
            teacher_count += usize::from(teacher);
            own_count += usize::from(own);
            reached_count += usize::from(reached);
            reached_correct_count += usize::from(reached && own);
            let mut arm = json!({"index":k,"id":eps[k].packet.id,"position":pos,"target":target,
                "teacher_prefix_correct":teacher,"own_position_correct":own,
                "own_prefix_reached_correctly":reached,
                "target_mass":canonical[pos]["native_target_mass"],
                "denominator":canonical[pos]["native_denominator"],
                "chosen_token":chosen});
            if a.mode == Mode::JointContinuation {
                arm["own_prefix_reached_and_answered_correctly"] = json!(reached && own);
            }
            arms.push(arm);
        }
        teacher_both += usize::from(arms.iter().all(|v| v["teacher_prefix_correct"] == true));
        own_both += usize::from(arms.iter().all(|v| v["own_position_correct"] == true));
        reached_correct_both += usize::from(
            arms.iter()
                .all(|v| v["own_prefix_reached_and_answered_correctly"] == true),
        );
        complete_both += usize::from(refs[i]["complete"] == true && refs[j]["complete"] == true);
        receipts.push(json!({"indices":[i,j],"first_divergent_position":pos,"arms":arms}));
    }
    let mut result = json!({"rows":eps.len(),"missing_rows":0,"entry_teacher_correct":entry_teacher,"entry_own_correct":entry_own,
        "first_divergent_teacher_correct":teacher_count,"first_divergent_own_position_correct":own_count,
        "first_divergent_own_prefix_reached":reached_count,"paired_teacher_both_correct":teacher_both,
        "paired_own_position_both_correct":own_both,"pairs":receipts,
        "later_copy_covered_positions":copy_total,"later_copy_covered_teacher_correct":copy_correct,
        "scope":"complete native own-prefix outputs and separately canonical teacher-prefix metrics; own position matches without a correct preceding prefix are not successful source-dependent continuation"});
    // Legacy baseline reuse compares the complete historical metric object.
    // New diagnostics must not invalidate a verified old evaluation receipt.
    if a.mode == Mode::JointContinuation {
        result["first_divergent_own_prefix_reached_and_answered_correctly"] =
            json!(reached_correct_count);
        result["paired_own_prefix_reached_and_answered_both_correct"] = json!(reached_correct_both);
        result["complete_swap_pairs"] = json!(complete_both);
        result["any_emitted_eos"] = json!(refs.iter().filter(|r| r["eos"] == true).count());
    }
    Ok(result)
}

// f63's native_step/evaluate semantics are byte-identical here. This explicit
// compatibility allowance must be revisited if initialized native evaluation
// semantics change; a matching parameter count alone never permits reuse.
const BASELINE_PRODUCER: &str = "f63ed81fd4f74d2501ba3d9888420662492416df";
fn baseline_identity(base: &Value, current: &Value, before: &Value, now: &Value) -> Result<()> {
    let producer = base["source_commit"]
        .as_str()
        .ok_or_else(|| bad("baseline producer absent"))?;
    if producer != BASELINE_PRODUCER && Some(producer) != option_env!("UOR_BUILD_SOURCE_COMMIT") {
        return Err(bad(
            "baseline producer has no explicit native-evaluation compatibility",
        ));
    }
    for field in [
        "checkpoint_receipt_sha256",
        "parent_config_sha256",
        "training_input_sha256",
        "training_labels_sha256",
        "original_bridge_masters",
        "marker_masters",
        "categorical",
        "initial_active_masters",
        "active_parameter_names",
        "fresh_adam",
        "rates",
        "phase_policy",
    ] {
        if base.get(field).is_none() || base[field] != current[field] {
            return Err(bad(&format!("baseline initial identity differs: {field}")));
        }
    }
    for field in [
        "step",
        "parent",
        "original_parent",
        "source_metadata_rebound",
        "generate_sha256",
        "original_quarter_sha256",
        "categorical_sha256",
        "categorical_receipt",
        "native_independently_reloaded",
        "masters_independently_reloaded",
    ] {
        if before.get(field).is_none() || before[field] != now[field] {
            return Err(bad(&format!(
                "baseline checkpoint identity differs: {field}"
            )));
        }
    }
    if before["step"] != 0
        || before["native_independently_reloaded"] != true
        || before["masters_independently_reloaded"] != true
        || base["fresh_adam"] != true
    {
        return Err(bad("baseline initialization/reload admission"));
    }
    // Credit, read-state pullback, data-order seed, host and CUDA device are
    // not initialized integer evaluation inputs. Do not require the new
    // non-forward pullback field from an older authenticated baseline.
    // Each fit still executes fresh seed/arm-specific graph/native parity.
    Ok(())
}
fn tree_identity(root: &Path) -> Result<String> {
    fn walk(root: &Path, path: &Path, rows: &mut Vec<(String, u64, String)>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                walk(root, &entry.path(), rows)?;
            } else if kind.is_file() {
                rows.push((
                    entry.path().strip_prefix(root)?.display().to_string(),
                    entry.metadata()?.len(),
                    sha256_file(&entry.path())?,
                ));
            } else {
                return Err(bad("baseline artifact has a nonregular entry"));
            }
        }
        Ok(())
    }
    let mut rows = Vec::new();
    walk(root, root, &mut rows)?;
    rows.sort();
    Ok(sha256_bytes(&serde_json::to_vec(&rows)?))
}
fn reuse_baseline(
    a: &Args,
    root: &Path,
    dev: &[Episode],
    current_receipt: &Value,
) -> Result<(Value, Value)> {
    report_output::verify(root)?;
    let report = read(&root.join("report.json"))?;
    let admission = read(&root.join("admission.json"))?;
    let zero = read(&root.join("zero-update-admission.json"))?;
    if report["schema"] != "uor-r4.geometric-frozen-map-fit/1"
        || report["status"] != "COMPLETED"
        || report["mode"] != "admission"
        || report["updates"] != 0
        || report["admission"] != zero
        || zero["independent_native_parity"] != true
        || zero["matched_forward_loss_bit_equal"] != true
    {
        return Err(bad(
            "baseline must be a completed sealed zero-update native admission",
        ));
    }
    let current = read(&a.out.join("admission.json"))?;
    let previous_receipt = read(&root.join("checkpoint-0000/receipt.json"))?;
    baseline_identity(&admission, &current, &previous_receipt, current_receipt)?;
    let mut artifact_receipts = BTreeMap::new();
    for name in [
        "source",
        "native",
        "read-state-bridge-source",
        "cue",
        "prefix",
    ] {
        let expected = tree_identity(&root.join("checkpoint-0000").join(name))?;
        if expected != tree_identity(&a.out.join("checkpoint-0000").join(name))? {
            return Err(bad(
                "baseline initialized source/native/frozen master bytes differ",
            ));
        }
        artifact_receipts.insert(name, expected);
    }
    for name in [
        "generate.bin",
        "read-state-bridge.bin",
        "read-state-bridge-categorical.bin",
    ] {
        if sha256_file(&root.join("checkpoint-0000").join(name))?
            != sha256_file(&a.out.join("checkpoint-0000").join(name))?
        {
            return Err(bad("baseline initialized native decoder/map bytes differ"));
        }
    }
    let evaluation = read(&root.join("development-0000.json"))?;
    let metric = read(&root.join("metrics-0000.json"))?;
    if report["initial_metrics"] != metric
        || evaluation["cases"] != 512
        || evaluation["target_positions"] != 6664
        || dev.len() != 512
    {
        return Err(bad("baseline complete evaluation/metric scope differs"));
    }
    let refs = evaluation["rows"]
        .as_array()
        .ok_or_else(|| bad("baseline rows absent"))?;
    if refs.len() != dev.len() {
        return Err(bad("baseline missing row references"));
    }
    for (index, (reference, e)) in refs.iter().zip(dev).enumerate() {
        let name = format!("development-0000-row-{index:04}.json");
        if reference["id"] != e.packet.id || reference["row_file"] != name {
            return Err(bad("baseline ordered row ID/file differs"));
        }
        let path = root.join(&name);
        let bytes = fs::read(&path)?;
        if reference["row_sha256"] != sha256_bytes(&bytes) {
            return Err(bad("baseline row hash differs"));
        }
        let row: Value = serde_json::from_slice(&bytes)?;
        if row["id"] != e.packet.id
            || row["canonical_target_ids_labels_only"] != json!(e.target)
            || row["generated_ids"] != reference["generated_ids"]
            || row["complete"] != reference["complete"]
        {
            return Err(bad("baseline row label/output reference differs"));
        }
        let canonical = row["canonical"]
            .as_array()
            .ok_or_else(|| bad("baseline canonical rows absent"))?;
        if canonical.len() != e.target.len()
            || canonical
                .iter()
                .zip(&e.target)
                .any(|(step, target)| step["target_label_only"] != *target)
        {
            return Err(bad("baseline canonical target coverage differs"));
        }
        if size(&a.out)? + bytes.len() as u64 > a.maximum_report_bytes - 1_048_576 {
            return Err(bad("baseline evidence copy exceeds report cap"));
        }
        // Keep the fit report self-contained. Never write into the sealed source.
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(a.out.join(name))?
            .write_all(&bytes)?;
    }
    let reproduced = metrics(a, &evaluation, dev)?;
    if reproduced != metric {
        return Err(bad(
            "baseline metrics do not reproduce from complete copied rows",
        ));
    }
    write(a, "development-0000.json", &evaluation)?;
    let provenance = json!({"reused":true,"execution":"NOT_RUN in this fit; copied authenticated initial native evaluation",
        "producer_source_commit":admission["source_commit"],"compatible_consumer_source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "baseline_root":root,"baseline_manifest_sha256":sha256_file(&root.join("manifest.json"))?,
        "baseline_report_sha256":sha256_file(&root.join("report.json"))?,
        "baseline_evaluation_sha256":sha256_file(&root.join("development-0000.json"))?,
        "baseline_metrics_sha256":sha256_file(&root.join("metrics-0000.json"))?,
        "initialized_artifact_tree_identities":artifact_receipts,"rows_verified_and_copied":refs.len(),
        "producer_elapsed_seconds":report["elapsed_seconds"],
        "compatibility_scope":"same initialized integer source/native/decoder/categorical map, full input/labels and master identities; native_step/evaluate unchanged; order/credit do not enter native inference; each fit executes its own graph/native admission"});
    write(a, "baseline-reuse.json", &provenance)?;
    Ok((evaluation, provenance))
}

fn optimizer(vars: &BTreeMap<String, Var>, lr: f64) -> Result<AdamW> {
    Ok(AdamW::new(
        vars.values().cloned().collect(),
        ParamsAdamW {
            lr,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?)
}
fn clip_denominator(grads: &BTreeMap<String, Tensor>, d: &Device) -> Result<(Tensor, f64)> {
    if grads.is_empty() {
        return Err(bad("no active parameter gradients"));
    }
    let mut scale = 0f64;
    for g in grads.values() {
        let value = g.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()? as f64;
        if !value.is_finite() {
            return Err(bad("nonfinite accumulated gradient"));
        }
        scale = scale.max(value);
    }
    let norm = if scale == 0. {
        0.
    } else {
        let terms = grads
            .values()
            .map(|g| g.affine(1. / scale, 0.)?.sqr()?.sum_all())
            .collect::<candle_core::Result<Vec<_>>>()?;
        scale * (Tensor::stack(&terms, 0)?.sum_all()?.to_scalar::<f32>()? as f64).sqrt()
    };
    if !norm.is_finite() {
        return Err(bad("nonfinite global gradient norm"));
    }
    let denominator = norm.max(1.) as f32;
    if !denominator.is_finite() {
        return Err(bad("gradient clip denominator exceeds F32"));
    }
    Ok((Tensor::new(denominator, d)?, norm))
}
fn disk_floor(a: &Args) -> Result<()> {
    let output = std::process::Command::new("df")
        .arg("-Pk")
        .arg(&a.out)
        .output()?;
    if !output.status.success() {
        return Err(bad("disk-floor observation unavailable"));
    }
    let text = std::str::from_utf8(&output.stdout)?;
    let available = text
        .lines()
        .last()
        .and_then(|l| l.split_whitespace().nth(3))
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or_else(|| bad("disk-floor parse"))?;
    if available < 128 * 1024 {
        return Err(bad("128MiB disk stop margin reached"));
    }
    Ok(())
}
fn control_panel_admit(eps: &[Episode]) -> Result<Value> {
    if eps.len() != 8 {
        return Err(bad("prediction control requires predetermined eight cases"));
    }
    let p = scoped_pairs(eps)?;
    if p != vec![(0, 4, 3), (1, 5, 3), (2, 6, 3), (3, 7, 3)] {
        return Err(bad("control source-swap pairs/common prefix differ"));
    }
    let mut entries = BTreeSet::new();
    let mut phases = [0usize; 3];
    let mut rows = Vec::new();
    for (local, e) in eps.iter().enumerate() {
        if !e.has_source() || !e.packet.actual_prefix_ids.is_empty() {
            return Err(bad(
                "control requires authentic full Source bank and empty entry prefix",
            ));
        }
        entries.insert(e.target[0]);
        let ids = e
            .views
            .iter()
            .flatten()
            .flat_map(|v| v.emitted_token_ids().iter().copied())
            .collect::<Vec<_>>();
        if ids.len() != 11 || ids.contains(&e.target[0]) {
            return Err(bad(
                "control full eleven occurrences/Generate-only entry differ",
            ));
        }
        // Only the phase classification is consumed here, and the loss scope
        // changes weights rather than counts, so this admission receipt is
        // scope-independent by construction.
        let plan = episode_loss_weights(
            &e.target,
            &ids.iter().copied().collect(),
            8,
            true,
            LossScope::All,
        )?;
        for k in 0..3 {
            phases[k] += plan.counts[k];
        }
        rows.push(json!({"local_index":local,"original_index":CONTROL_INDICES[local],"id":e.packet.id,
            "target_ids_labels_only":e.target,"physical_copy_occurrences":ids.len(),"phase_counts":plan.counts}));
    }
    if entries != BTreeSet::from([617, 2997]) || phases != [8, 44, 32] {
        return Err(bad("control entry roles/phase coverage differ"));
    }
    Ok(
        json!({"original_indices":CONTROL_INDICES,"local_pairs":p,"phase_counts":phases,"rows":rows,
        "scope":"eight construction cases; four swapped-source pairs, two entry roles; no held-out or temporal-update claim"}),
    )
}

fn control_entry_diagnostics(
    a: &Args,
    l: &Loaded,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    eps: &[Episode],
    step: usize,
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut only = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let params = l.generate.parameters();
    let biases = params
        .get("generate.bias")
        .ok_or_else(|| bad("control bias masters absent"))?;
    let mut rows = Vec::new();
    for (local, e) in eps.iter().enumerate() {
        // Complete target-free native bank selection first. Labels cannot choose
        // a source occurrence, frame, route, state or vocabulary candidate.
        let native = native_step(
            model,
            g,
            Some(bridge),
            &mut pool,
            e,
            &[],
            &l.cue,
            &l.prefix,
            None,
            bank_selection(a),
        )?;
        let state: Vec<u8> = serde_json::from_value(native["retained_state_codes"].clone())?;
        let state = state
            .into_iter()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut scores = vec![0i64; g.vocab_size()];
        g.score_into(&state, &mut scores, &mut GenerateReadCounts::default())?;
        if native["generate_raw_scores_sha256"] != sha256_bytes(&serde_json::to_vec(&scores)?) {
            return Err(bad("entry diagnostic native score digest differs"));
        }
        let ids: Vec<u32> = serde_json::from_value(native["copy_token_ids"].clone())?;
        let copies: Vec<i64> = serde_json::from_value(native["copy_raw_scores_q24"].clone())?;
        let full = pool.reduce_trace(&scores, &ids, &copies)?;
        if native["pool"]["summary"] != serde_json::to_value(&full.summary)? {
            return Err(bad("entry diagnostic full pool differs"));
        }
        let generate_only = only.reduce_trace(&scores, &[], &[])?;
        let gold = e.target[0];
        let gold_score = *scores
            .get(gold as usize)
            .ok_or_else(|| bad("entry gold outside vocabulary"))?;
        let gm = full
            .token_masses
            .iter()
            .find(|m| m.token_id == gold)
            .ok_or_else(|| bad("entry native gold mass absent"))?;
        let wrong = full
            .actions
            .iter()
            .filter(|x| {
                matches!(
                    x.action,
                    uor_r4_integer::geometric_vocabulary_actions::VocabularyAction::Copy { .. }
                ) && x.token_id != gold
            })
            .max_by(|x, y| {
                x.raw_score_q24
                    .cmp(&y.raw_score_q24)
                    .then_with(|| y.action_offset.cmp(&x.action_offset))
            })
            .ok_or_else(|| bad("entry wrong Copy absent"))?;
        let wrong_alias = full
            .token_masses
            .iter()
            .filter(|m| m.token_id != gold && m.copy_weight_q31 > 0)
            .max_by(|x, y| {
                x.copy_weight_q31
                    .cmp(&y.copy_weight_q31)
                    .then_with(|| y.token_id.cmp(&x.token_id))
            })
            .ok_or_else(|| bad("entry wrong Copy alias absent"))?;
        let bias = biases
            .as_tensor()
            .narrow(0, gold as usize, 1)?
            .reshape(())?
            .to_scalar::<f32>()?;
        let code = g.token_bias(gold as usize)?;
        let next_up = if code < 7 {
            Some((f64::from(code) + 0.5) / 4.)
        } else {
            None
        };
        let next_down = if code > -7 {
            Some((f64::from(code) - 0.5) / 4.)
        } else {
            None
        };
        rows.push(json!({"local_index":local,"original_index":CONTROL_INDICES[local],"id":e.packet.id,
            "gold_entry_label_only":gold,"generate_only_summary":generate_only.summary,"full_summary":full.summary,
            "gold_generate_raw_q24":gold_score,"gold_generate_clipped_q24":gold_score.clamp(-(8<<24),8<<24),
            "gold_generate_weight_q31":gm.generate_weight_q31,"gold_copy_weight_q31":gm.copy_weight_q31,"gold_total_weight_q31":gm.weight_q31,
            "highest_raw_wrong_copy_action":wrong,"highest_copy_component_wrong_token_alias":wrong_alias,
            "raw_gold_minus_wrong_copy_q24":gold_score.checked_sub(wrong.raw_score_q24),
            "clipped_gold_minus_wrong_copy_q24":gold_score.clamp(-(8<<24),8<<24)-wrong.score_q24,
            "score_clip_q24":8i64<<24,"gold_bias_master":bias,"gold_bias_native_code":code,
            "bias_positive_cell_boundary":next_up,"bias_positive_boundary_distance":next_up.map(|x|x-f64::from(bias)),
            "bias_negative_cell_boundary":next_down,"bias_negative_boundary_distance":next_down.map(|x|f64::from(bias)-x),
            "native":native}));
    }
    let v = json!({"step":step,"rows":rows,"generate_sha256":sha256_bytes(&g.to_bytes()?),
        "categorical_sha256":sha256_bytes(&bridge.to_bytes()?),"scope":"native entry replay; all4096 Generate and all physical Copy; labels read only after forward; Generate-only pool is a diagnostic, not serving"});
    write(a, &format!("prediction-entry-{step:04}.json"), &v)?;
    Ok(v)
}

/// Learned entry scorer: the boundary decision owned by its own scorer instead of
/// by the mixed Copy+Generate pool, where the answer is never a Copy candidate
/// and its Generate atom is a bounded-range score. The artifact is data, not
/// floats: per feature, the training counts of the first answer token, exported
/// whole and read back before use.
#[derive(Clone, Debug, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryScorer {
    schema: String,
    feature_law: String,
    rows: Vec<EntryScorerRow>,
}

#[derive(Clone, Debug, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryScorerRow {
    feature: String,
    /// (token, count), ordered by token.
    scores: Vec<(u32, u64)>,
}

const ENTRY_SCORER_SCHEMA: &str = "uor-r4.native-entry-scorer/1";

impl EntryScorer {
    fn feature_of(e: &Episode) -> String {
        format!("q{:?}", e.packet.query_ids)
    }

    fn fit(eps: &[Episode]) -> Self {
        let mut counts = BTreeMap::<String, BTreeMap<u32, u64>>::new();
        for e in eps {
            if panel_split(&e.packet.id) != 0 {
                continue;
            }
            *counts
                .entry(Self::feature_of(e))
                .or_default()
                .entry(e.target[0])
                .or_default() += 1;
        }
        let rows = counts
            .into_iter()
            .map(|(feature, scores)| EntryScorerRow {
                feature,
                scores: scores.into_iter().collect(),
            })
            .collect();
        Self {
            schema: ENTRY_SCORER_SCHEMA.into(),
            feature_law: "packet query_ids verbatim; an unseen query falls back to the pool".into(),
            rows,
        }
    }

    fn choose(&self, feature: &str) -> Option<u32> {
        self.rows
            .iter()
            .find(|row| row.feature == feature)
            .and_then(|row| {
                row.scores
                    .iter()
                    .max_by(|x, y| x.1.cmp(&y.1).then_with(|| y.0.cmp(&x.0)))
                    .map(|(token, _)| *token)
            })
    }

    fn sha256(&self) -> Result<String> {
        Ok(sha256_bytes(&serde_json::to_vec(self)?))
    }
}

/// Entry correctness on the live evaluation, split by the label-independent
/// halves, so in-sample and held-out are never merged into one number.
fn entry_correct_by_split(
    a: &Args,
    eval: &Value,
    eps: &[Episode],
) -> Result<(usize, usize, usize, usize)> {
    let refs = eval["rows"]
        .as_array()
        .ok_or_else(|| bad("evaluation rows missing"))?;
    if refs.len() != eps.len() {
        return Err(bad("row coverage differs"));
    }
    let (mut fit_ok, mut fit_n, mut held_ok, mut held_n) = (0usize, 0usize, 0usize, 0usize);
    for (row_ref, e) in refs.iter().zip(eps) {
        let name = row_ref["row_file"]
            .as_str()
            .ok_or_else(|| bad("row file absent"))?;
        let path = a.out.join(name);
        if sha256_file(&path)? != row_ref["row_sha256"] {
            return Err(bad("row hash differs"));
        }
        let row = read(&path)?;
        let chosen = row["canonical"][0]["native"]["pool"]["summary"]["chosen_token_id"]
            .as_u64()
            .ok_or_else(|| bad("entry chosen token absent"))? as u32;
        let correct = usize::from(chosen == e.target[0]);
        if panel_split(&e.packet.id) == 0 {
            fit_n += 1;
            fit_ok += correct;
        } else {
            held_n += 1;
            held_ok += correct;
        }
    }
    Ok((fit_ok, fit_n, held_ok, held_n))
}

/// Fit the entry scorer on one panel half, export it, read it back, and evaluate
/// the whole panel twice -- once with the pool owning the entry decision and once
/// with the scorer owning it -- reporting the halves separately.
fn run_entry_scorer_fit(a: &Args, start: Instant, l: &Loaded, dev: &[Episode]) -> Result<Value> {
    let scorer = EntryScorer::fit(dev);
    let fit_rows = dev
        .iter()
        .filter(|e| panel_split(&e.packet.id) == 0)
        .count();
    let held_rows = dev.len() - fit_rows;
    if scorer.rows.is_empty() || fit_rows == 0 || held_rows == 0 {
        return Err(bad("entry scorer fit needs both panel halves"));
    }
    // Export, then use only the copy read back from disk.
    let artifact = a.out.join("entry-scorer.json");
    fs::write(&artifact, serde_json::to_vec_pretty(&scorer)?)?;
    let reloaded_bytes = fs::read(&artifact)?;
    let reloaded: EntryScorer = serde_json::from_slice(&reloaded_bytes)?;
    // Compare like with like: the file is pretty-printed, so hash the canonical
    // serialization of the reloaded copy rather than the file bytes.
    let expected = scorer.sha256()?;
    let reloaded_sha = sha256_bytes(&serde_json::to_vec(&reloaded)?);
    if reloaded.schema != ENTRY_SCORER_SCHEMA
        || reloaded_sha != expected
        || reloaded.rows.len() != scorer.rows.len()
    {
        return Err(bad("entry scorer export/reload identity differs"));
    }
    let (initial_native, initial_generate, initial_bridge, _) = checkpoint(a, 0, l)?;
    let base_eval = evaluate(
        a,
        "entry-scorer-base",
        &initial_native,
        &initial_generate,
        Some(&initial_bridge),
        &l.exp,
        dev,
        &l.tokenizer,
        &l.cue,
        &l.prefix,
        start,
        None,
    )?;
    let scored_eval = evaluate(
        a,
        "entry-scorer-applied",
        &initial_native,
        &initial_generate,
        Some(&initial_bridge),
        &l.exp,
        dev,
        &l.tokenizer,
        &l.cue,
        &l.prefix,
        start,
        Some(&reloaded),
    )?;
    let (base_fit, base_fit_n, base_held, base_held_n) =
        entry_correct_by_split(a, &base_eval, dev)?;
    let (fit_ok, fit_n, held_ok, held_n) = entry_correct_by_split(a, &scored_eval, dev)?;
    let v = json!({"schema":"uor-r4.entry-scorer-fit/1","status":"COMPLETED","mode":"entry_scorer_fit",
        "artifact":"entry-scorer.json","artifact_sha256":reloaded_sha.clone(),
        "feature_law":reloaded.feature_law,"distinct_features":reloaded.rows.len(),
        "fit_rows":fit_rows,"held_out_rows":held_rows,
        "pool_entry_correct":{"in_sample":base_fit,"in_sample_rows":base_fit_n,"held_out":base_held,"held_out_rows":base_held_n},
        "scorer_entry_correct":{"in_sample":fit_ok,"in_sample_rows":fit_n,"held_out":held_ok,"held_out_rows":held_n},
        "update_state":"no optimizer step; the scorer is fitted from the training half and owns the entry decision only",
        "elapsed_seconds":start.elapsed().as_secs_f64(),
        "scope":"plumbing proof through the live evaluation path: the entry decision is owned by a fitted, exported and reloaded table applied at the empty prefix; the feature is the prompt, so the held-out number is memorisation of the panel query signatures and is not generalisation evidence"});
    write(a, "entry-scorer-fit.json", &v)?;
    Ok(v)
}

/// Zero-update entry-position ceiling over every development row.
///
/// The same target-free native forward as the control diagnostic, run over the
/// whole panel and reported twice: once with the physical Copy channel present
/// (the live mixed pool) and once with it removed (the Generate-only pool).
/// Labels are read only after each forward, so this is a capability bound on
/// the frozen artifact: not a fit, not a selection, not a serving change.
/// Stable, label-independent half of the panel. Any positional split on this
/// panel is a split on the answer -- the two answers alternate with row parity --
/// so the split hashes the case id.
fn panel_split(id: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash % 2
}

/// The boundary feature a dedicated entry scorer keys on: the query the model
/// was given. This is the plumbing proof -- it asks whether a learned table,
/// fitted on one half of the panel and applied at the entry position through the
/// same native forward and the same pool, moves the entry decision, and whether
/// it survives export to integer scores. The feature is deliberately the weakest
/// interesting one: it cannot generalise to an unseen wording, which the
/// feasibility read on the same panel already established, and that limit is
/// reported with the number rather than hidden behind it.
fn entry_scorer_read(
    a: &Args,
    l: &Loaded,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    eps: &[Episode],
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut only = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let feature = |e: &Episode| format!("q{:?}", e.packet.query_ids);
    let mut table = BTreeMap::<String, BTreeMap<u32, u64>>::new();
    let mut fit_rows = 0usize;
    for e in eps {
        if panel_split(&e.packet.id) != 0 {
            continue;
        }
        fit_rows += 1;
        *table
            .entry(feature(e))
            .or_default()
            .entry(e.target[0])
            .or_default() += 1;
    }
    let (mut test_rows, mut covered) = (0usize, 0usize);
    let (mut base_correct, mut full_correct, mut scored_correct, mut scored_full_correct) =
        (0usize, 0usize, 0usize, 0usize);
    let mut records = Vec::new();
    for e in eps {
        if panel_split(&e.packet.id) != 1 {
            continue;
        }
        test_rows += 1;
        let native = native_step(
            model,
            g,
            Some(bridge),
            &mut pool,
            e,
            &[],
            &l.cue,
            &l.prefix,
            None,
            bank_selection(a),
        )?;
        let state: Vec<u8> = serde_json::from_value(native["retained_state_codes"].clone())?;
        let state = state
            .into_iter()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut scores = vec![0i64; g.vocab_size()];
        g.score_into(&state, &mut scores, &mut GenerateReadCounts::default())?;
        let ids: Vec<u32> = serde_json::from_value(native["copy_token_ids"].clone())?;
        let copies: Vec<i64> = serde_json::from_value(native["copy_raw_scores_q24"].clone())?;
        let gold = e.target[0];
        let base = only.reduce_trace(&scores, &[], &[])?;
        let base_full = pool.reduce_trace(&scores, &ids, &copies)?;
        base_correct += usize::from(base.summary.chosen_token_id == gold);
        full_correct += usize::from(base_full.summary.chosen_token_id == gold);
        // Apply the fitted table the way an exported scorer would: a bounded
        // integer bonus on the learned token, inside the same score clip the
        // native pool already enforces.
        let mut scored = scores.clone();
        let mut bonus = None;
        if let Some(counts) = table.get(&feature(e)) {
            covered += 1;
            let total = counts.values().sum::<u64>().max(1);
            if let Some((token, count)) = counts
                .iter()
                .max_by(|x, y| x.1.cmp(y.1).then_with(|| y.0.cmp(x.0)))
            {
                let scaled = (*count as f64 / total as f64) * (6.0 * f64::from(1 << 24));
                scored[*token as usize] =
                    scored[*token as usize].saturating_add(scaled.round() as i64);
                bonus = Some((*token, scaled.round() as i64));
            }
        }
        let with = pool.reduce_trace(&scored, &ids, &copies)?;
        let with_only = only.reduce_trace(&scored, &[], &[])?;
        scored_correct += usize::from(with_only.summary.chosen_token_id == gold);
        scored_full_correct += usize::from(with.summary.chosen_token_id == gold);
        records.push(json!({"id":e.packet.id,"gold_entry_label_only":gold,
            "base_generate_only":base.summary.chosen_token_id,
            "base_mixed":base_full.summary.chosen_token_id,
            "scored_generate_only":with_only.summary.chosen_token_id,
            "scored_mixed":with.summary.chosen_token_id,
            "scorer_bonus":bonus}));
    }
    let v = json!({"fit_rows":fit_rows,"test_rows":test_rows,"test_rows_covered_by_table":covered,
        "distinct_features":table.len(),
        "base_generate_only_correct":base_correct,"base_mixed_correct":full_correct,
        "scored_generate_only_correct":scored_correct,"scored_mixed_correct":scored_full_correct,
        "row_records":records,
        "scope":"plumbing proof on the native path: query-keyed table fitted on one panel half and applied as a bounded integer bonus at the entry position before the pool reduction; the feature cannot generalise to an unseen wording and the covered-row count says how much of the number is memorisation; no checkpoint export yet"});
    write(a, "entry-scorer-read.json", &v)?;
    Ok(v)
}

fn entry_ceiling_panel(
    a: &Args,
    l: &Loaded,
    model: &IntegerRealizer,
    g: &NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    eps: &[Episode],
) -> Result<Value> {
    use uor_r4_integer::h4_tables::H4Code;
    let mut pool = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut only = NativeVocabularyActions::new(model.binding().clone(), &l.exp)?;
    let mut rows = Vec::new();
    let (mut full_correct, mut generate_only_correct) = (0usize, 0usize);
    let (mut full_copy_dominated, mut gold_in_copy) = (0usize, 0usize);
    let mut gold_ranks: Vec<u64> = Vec::new();
    let mut next_gold_ranks: Vec<u64> = Vec::new();
    let mut gold_mass_fraction: Vec<f64> = Vec::new();
    // Export control: score the same read state with the float masters as well as
    // with the exported integer table, so "the learned emission is uninformative"
    // is separated from "the export destroyed what the masters carried".
    let float_prepared = if a.ceiling_float {
        Some(l.generate.prepare_native()?)
    } else {
        None
    };
    let mut float_gold_ranks: Vec<u64> = Vec::new();
    let mut float_rank_delta: Vec<i64> = Vec::new();
    let mut float_top1_agree = 0usize;
    let mut tie_groups: Vec<u64> = Vec::new();
    let mut top1_shares: Vec<f64> = Vec::new();
    let mut target_counts = BTreeMap::<u32, u64>::new();
    for (index, e) in eps.iter().enumerate() {
        // Complete target-free native bank selection first. Labels cannot choose
        // a source occurrence, frame, route, state or vocabulary candidate.
        let native = native_step(
            model,
            g,
            Some(bridge),
            &mut pool,
            e,
            &[],
            &l.cue,
            &l.prefix,
            None,
            bank_selection(a),
        )?;
        let state: Vec<u8> = serde_json::from_value(native["retained_state_codes"].clone())?;
        let state = state
            .into_iter()
            .map(H4Code::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut scores = vec![0i64; g.vocab_size()];
        g.score_into(&state, &mut scores, &mut GenerateReadCounts::default())?;
        let ids: Vec<u32> = serde_json::from_value(native["copy_token_ids"].clone())?;
        let copies: Vec<i64> = serde_json::from_value(native["copy_raw_scores_q24"].clone())?;
        let full = pool.reduce_trace(&scores, &ids, &copies)?;
        let generate_only = only.reduce_trace(&scores, &[], &[])?;
        let gold = e.target[0];
        let full_chosen = full.summary.chosen_token_id;
        let only_chosen = generate_only.summary.chosen_token_id;
        // Where the answer sits in the fully ordered emission, so "the emission
        // cannot do it" and "the emission nearly does it" are distinguishable.
        let mut ordered = generate_only
            .token_masses
            .iter()
            .map(|m| (m.token_id, m.weight_q31))
            .collect::<Vec<_>>();
        ordered.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
        let gold_rank = ordered
            .iter()
            .position(|m| m.0 == gold)
            .map(|position| position as u64 + 1);
        gold_ranks.push(gold_rank.unwrap_or(ordered.len() as u64 + 1));
        *target_counts.entry(gold).or_default() += 1;
        // Tie group at the gold's mass: the pool quantizes at 256 steps/octave, so
        // a rank is only interpretable next to how many tokens share the mass.
        if let Some(gold_mass) = ordered.iter().find(|m| m.0 == gold).map(|m| m.1) {
            tie_groups.push(ordered.iter().filter(|m| m.1 == gold_mass).count() as u64);
        }
        top1_shares.push(
            ordered
                .first()
                .map(|m| m.1 as f64 / generate_only.summary.total_weight_q31.max(1) as f64)
                .unwrap_or(0.),
        );
        if let Some(prepared) = &float_prepared {
            let out = l
                .generate
                .forward_prepared_coefficients_only(prepared, &state)?;
            let mut float_order = out
                .scores_q24
                .iter()
                .enumerate()
                .map(|(token, &score)| (token as u32, score))
                .collect::<Vec<_>>();
            float_order.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
            let float_rank = float_order
                .iter()
                .position(|m| m.0 == gold)
                .map(|position| position as u64 + 1)
                .unwrap_or(float_order.len() as u64 + 1);
            float_gold_ranks.push(float_rank);
            float_rank_delta.push(gold_rank.unwrap_or(0) as i64 - float_rank as i64);
            if float_order.first().map(|m| m.0) == ordered.first().map(|m| m.0) {
                float_top1_agree += 1;
            }
        }
        // The same read one token later, after observing the correct first
        // token. This separates "the boundary state carries nothing" from "the
        // emission cannot rank at all": the continuation is given its prefix,
        // while the entry position has to select the answer from the query.
        if e.target.len() > 1 {
            let native_next = native_step(
                model,
                g,
                Some(bridge),
                &mut pool,
                e,
                &[gold],
                &l.cue,
                &l.prefix,
                None,
                bank_selection(a),
            )?;
            let state_next: Vec<u8> =
                serde_json::from_value(native_next["retained_state_codes"].clone())?;
            let state_next = state_next
                .into_iter()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let mut scores_next = vec![0i64; g.vocab_size()];
            g.score_into(
                &state_next,
                &mut scores_next,
                &mut GenerateReadCounts::default(),
            )?;
            let only_next = only.reduce_trace(&scores_next, &[], &[])?;
            let gold_next = e.target[1];
            let mut ordered_next = only_next
                .token_masses
                .iter()
                .map(|m| (m.token_id, m.weight_q31))
                .collect::<Vec<_>>();
            ordered_next.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
            next_gold_ranks.push(
                ordered_next
                    .iter()
                    .position(|m| m.0 == gold_next)
                    .map(|position| position as u64 + 1)
                    .unwrap_or(ordered_next.len() as u64 + 1),
            );
        }
        gold_mass_fraction.push(
            ordered
                .iter()
                .find(|m| m.0 == gold)
                .map(|m| m.1 as f64 / generate_only.summary.total_weight_q31.max(1) as f64)
                .unwrap_or(0.),
        );
        full_correct += usize::from(full_chosen == gold);
        generate_only_correct += usize::from(only_chosen == gold);
        full_copy_dominated += usize::from(
            full.summary.chosen_copy_weight_q31 > full.summary.chosen_generate_weight_q31,
        );
        gold_in_copy += usize::from(ids.contains(&gold));
        rows.push(
            json!({"index":index,"id":e.packet.id,"gold_entry_label_only":gold,
            "full_chosen":full_chosen,"generate_only_chosen":only_chosen,
            "full_correct":full_chosen==gold,"generate_only_correct":only_chosen==gold,
            "gold_in_copy_candidates":ids.contains(&gold),"copy_candidates":ids.len(),
            "gold_generate_rank":gold_rank,
            // The boundary state itself, so a readout can be fitted against it
            // offline without any model: does the read encode what was asked?
            "retained_state_codes":native["retained_state_codes"].clone(),
            // What the read selected, which the state does not expose: if the
            // selection is question-determined and content-invariant, it is the
            // channel the state is missing.
            "source_provenance":native["source_provenance"].clone(),
            "full_summary":full.summary,"generate_only_summary":generate_only.summary}),
        );
    }
    let mut ranks = gold_ranks.clone();
    let mut next_ranks = next_gold_ranks.clone();
    let mut fractions = gold_mass_fraction.clone();
    let mut ties = tie_groups.clone();
    let mut shares = top1_shares.clone();
    let mut float_ranks = float_gold_ranks.clone();
    let mut deltas = float_rank_delta.clone();
    ranks.sort_unstable();
    next_ranks.sort_unstable();
    ties.sort_unstable();
    float_ranks.sort_unstable();
    deltas.sort_unstable();
    fractions.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    shares.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let pick = |v: &[u64], q: f64| -> u64 {
        if v.is_empty() {
            0
        } else {
            v[((v.len() as f64 - 1.) * q).round() as usize]
        }
    };
    let pick_fraction = |v: &[f64], q: f64| -> f64 {
        if v.is_empty() {
            0.
        } else {
            v[((v.len() as f64 - 1.) * q).round() as usize]
        }
    };
    let v = json!({"rows":eps.len(),
        "full_pool_entry_correct":full_correct,
        "generate_only_entry_correct":generate_only_correct,
        "full_pool_winners_copy_dominated":full_copy_dominated,
        "rows_with_gold_in_copy_candidates":gold_in_copy,
        "generate_only_gold_rank_min":ranks.first().copied().unwrap_or(0),
        "generate_only_gold_rank_median":pick(&ranks, 0.5),
        "generate_only_gold_rank_p90":pick(&ranks, 0.9),
        "generate_only_gold_rank_max":ranks.last().copied().unwrap_or(0),
        "generate_only_gold_mass_fraction_median":pick_fraction(&fractions, 0.5),
        "next_token_gold_rank_median":pick(&next_ranks, 0.5),
        "next_token_gold_rank_p90":pick(&next_ranks, 0.9),
        "next_token_gold_rank_max":next_ranks.last().copied().unwrap_or(0),
        "next_token_rows":next_ranks.len(),
        "generate_actions":eps
            .first()
            .map_or(0, |_| only.legal_token_ids().len()),
        // Baselines every rank and share has to be read against, and the export
        // control: the same read state scored by the float masters.
        "majority_baseline":target_counts.values().copied().max().unwrap_or(0) as f64
            / eps.len().max(1) as f64,
        "uniform_baseline":1.0 / only.legal_token_ids().len().max(1) as f64,
        "integer_tie_group_at_gold_median":pick(&ties, 0.5),
        "integer_tie_group_at_gold_max":ties.last().copied().unwrap_or(0),
        "integer_top1_mass_share_median":pick_fraction(&shares, 0.5),
        "float_compare":float_prepared.is_some(),
        "float_gold_rank_median":pick(&float_ranks, 0.5),
        "float_gold_rank_p90":pick(&float_ranks, 0.9),
        "float_gold_rank_max":float_ranks.last().copied().unwrap_or(0),
        "float_minus_integer_rank_median":deltas.get(deltas.len() / 2).copied().unwrap_or(0),
        "float_top1_agrees_with_integer":float_top1_agree,
        "row_records":rows,
        "scope":"zero-update entry ceiling: target-free native forward, mixed pool versus Generate-only pool, labels read after forward; gold rank is its position in the fully ordered emission; float_compare scores the same read state with the float masters so the export is separated from the learned emission; a capability bound on one artifact, not a fit and not held-out language evidence"});
    write(a, "entry-ceiling.json", &v)?;
    Ok(v)
}

fn control_crossings(
    a: &Args,
    initial: &NativeGeometricGenerate,
    g: &NativeGeometricGenerate,
    step: usize,
) -> Result<Value> {
    let prototypes = initial
        .prototypes()
        .iter()
        .zip(g.prototypes())
        .filter(|(x, y)| x != y)
        .count();
    let mut bias = 0;
    let mut unary = 0;
    let mut pair = 0;
    for id in 0..g.vocab_size() {
        bias += usize::from(initial.token_bias(id)? != g.token_bias(id)?);
    }
    for lane in 0..g.lanes() {
        for r in 0..120 {
            unary += usize::from(
                initial.energy().get_unary(lane as u8, r)?
                    != g.energy().get_unary(lane as u8, r)?,
            );
        }
    }
    for e in 0..g.energy().edges().len() {
        for x in 0..120 {
            for y in 0..120 {
                pair += usize::from(
                    initial.energy().get_pair(e, x, y)? != g.energy().get_pair(e, x, y)?,
                );
            }
        }
    }
    let old = tree_identity(&a.out.join("checkpoint-0000/native/consumer"))?;
    let current = tree_identity(&a.out.join(format!("checkpoint-{step:04}/native/consumer")))?;
    Ok(
        json!({"step":step,"generate_prototype_lane_code_crossings":prototypes,"generate_bias_code_crossings":bias,
        "generate_unary_code_crossings":unary,"generate_pair_code_crossings":pair,
        "native_consumer_tree_changed":old!=current,"initial_native_consumer_tree_sha256":old,"current_native_consumer_tree_sha256":current,
        "consumer_identity_scope":"tree identity changes include native Context/Potential metadata; not a count of categorical state crossings"}),
    )
}

const CONTROL_RESUME_PRODUCER: &str = "3fc63bec66e03a3148c83c48c795667582d258c2";
fn restore_control_resume(a: &Args, l: &Loaded, d: &Device) -> Result<Option<Value>> {
    let Some(root) = &a.prediction_control_resume else {
        return Ok(None);
    };
    report_output::verify(root)?;
    let r = read(&root.join("report.json"))?;
    let admission = read(&root.join("admission.json"))?;
    let source_step = a.prediction_control_resume_source_step.unwrap_or(32);
    let generate_step = a.prediction_control_resume_generate_step.unwrap_or(32);
    let cross = source_step == 48
        && generate_step == 64
        && r["source_commit"] == "b451571aa638d1f93e3ccf25f718f27ed2ea731f";
    let original =
        source_step == 32 && generate_step == 32 && r["source_commit"] == CONTROL_RESUME_PRODUCER;
    if !original && !cross {
        return Err(bad("unreviewed component-resume producer/step combination"));
    }
    if cross && control_settings(a)?.0 != 0 {
        return Err(bad("component recomposition is zero-update only"));
    }
    let final_step = if cross { 64 } else { 32 };
    let cp = root.join(format!("checkpoint-{source_step:04}"));
    let gp = root.join(format!("checkpoint-{generate_step:04}"));
    let final_receipt = read(&root.join(format!("checkpoint-{final_step:04}/receipt.json")))?;
    let generate_receipt = read(&gp.join("receipt.json"))?;
    let receipt = read(&cp.join("receipt.json"))?;
    if r["schema"] != "uor-r4.geometric-prediction-control/1"
        || r["status"] != "COMPLETED"
        || r["mode"] != "prediction_control"
        || r["updates"] != final_step
        || r["original_indices"] != json!(CONTROL_INDICES)
        || r["final_receipt"] != final_receipt
        || final_receipt["step"] != final_step
        || receipt["step"] != source_step
        || generate_receipt["step"] != generate_step
        || generate_receipt["generate_sha256"] != sha256_file(&gp.join("generate.bin"))?
        || generate_receipt["credit"] != a.credit.name()
        || generate_receipt["read_state_pullback"] != a.read_state_pullback.name()
        || !r["blocks"]
            .as_array()
            .is_some_and(|rows| rows.iter().any(|v| v["step"] == source_step))
        || receipt["credit"] != a.credit.name()
        || receipt["read_state_pullback"] != a.read_state_pullback.name()
        || admission["input_sha256"] != INPUT_SHA
        || admission["labels_sha256"] != LABEL_SHA
        || admission["checkpoint_receipt_sha256"] != CP_RECEIPT_SHA
        || admission["credit"] != a.credit.name()
        || admission["read_state_pullback"] != a.read_state_pullback.name()
        || admission["panel"]["original_indices"] != json!(CONTROL_INDICES)
    {
        return Err(bad("resume must be an authenticated completed supported control with matching components/data/geometry/credit"));
    }
    if tree_identity(&cp.join("read-state-bridge-source"))?
        != tree_identity(&a.checkpoint.join("read-state-bridge-source"))?
    {
        return Err(bad("resume changed original frozen bridge masters"));
    }
    let cue = cue_payload(&cp.join("cue"))?;
    let prefix = prefix_payload(&cp.join("prefix"))?;
    if cue.config() != l.cue.config()
        || cue.packed_coefficients() != l.cue.packed_coefficients()
        || cue.joint().map(|v| v.packed_coefficients())
            != l.cue.joint().map(|v| v.packed_coefficients())
        || prefix.config() != l.prefix.config()
        || prefix.packed_coefficients() != l.prefix.packed_coefficients()
    {
        return Err(bad(
            "resume changed fixed cue/prefix coefficient/configuration payloads",
        ));
    }
    if sha256_file(&cp.join("read-state-bridge.bin"))? != QUARTER_SHA
        || sha256_file(&cp.join("read-state-bridge-categorical.bin"))? != CAT_SHA
    {
        return Err(bad("resume changed frozen bridge"));
    }
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let resumed_native = IntegerRealizer::load_native(&cp.join("native"), &binding)?;
    if resumed_native.binding() != l.integer.binding() {
        return Err(bad("resume tokenizer/protocol binding differs"));
    }
    let restored =
        SourceRealizerWeights::load_context_potential_on_device(&cp.join("source"), &tokbytes, d)?;
    let mut source = l.source.context_state_parameters();
    source.extend(l.source.potential_parameters());
    let mut donor = restored.context_state_parameters();
    donor.extend(restored.potential_parameters());
    if source.keys().collect::<Vec<_>>() != donor.keys().collect::<Vec<_>>() {
        return Err(bad("resume source master families differ"));
    }
    for (name, var) in &source {
        var.set(donor[name].as_tensor())?;
    }
    let gm = read(&gp.join("generate-source/metadata.json"))?;
    if gm["tokenizer_sha256"] != sha256_bytes(&tokbytes)
        || gm["protocol"] != serde_json::to_value(l.integer.binding().protocol())?
        || gm["lanes"] != json!(l.generate.lanes())
    {
        return Err(bad("resume Generate metadata binding differs"));
    }
    restore(
        &gp.join("generate-source"),
        &gm["parameters"],
        &l.generate.parameters(),
        d,
    )?;
    if l.generate.export_native()?.to_bytes()? != fs::read(gp.join("generate.bin"))? {
        return Err(bad("resumed Generate masters do not export saved artifact"));
    }
    if cross {
        let other_binding: NativeArtifactBinding =
            serde_json::from_value(generate_receipt["parent"].clone())?;
        let other_native = IntegerRealizer::load_native(&gp.join("native"), &other_binding)?;
        if other_native.binding() != resumed_native.binding() {
            return Err(bad("component tokenizer/protocol binding differs"));
        }
        let other_source = SourceRealizerWeights::load_context_potential_on_device(
            &gp.join("source"),
            &tokbytes,
            &Device::Cpu,
        )?;
        if identities(&other_source.context_state_parameters())?
            != identities(&restored.context_state_parameters())?
        {
            return Err(bad("component Context masters differ"));
        }
        let source_gm = read(&cp.join("generate-source/metadata.json"))?;
        if source_gm["parameters"]["generate.prototype_choices"]
            != gm["parameters"]["generate.prototype_choices"]
        {
            return Err(bad("component prototype masters differ"));
        }
        for name in ["read-state-bridge-source"] {
            if tree_identity(&cp.join(name))? != tree_identity(&gp.join(name))? {
                return Err(bad("component frozen bridge masters differ"));
            }
        }
        let gc = cue_payload(&gp.join("cue"))?;
        let pc = prefix_payload(&gp.join("prefix"))?;
        if gc.config() != l.cue.config()
            || gc.packed_coefficients() != l.cue.packed_coefficients()
            || gc.joint().map(|v| v.packed_coefficients())
                != l.cue.joint().map(|v| v.packed_coefficients())
            || pc.config() != l.prefix.config()
            || pc.packed_coefficients() != l.prefix.packed_coefficients()
            || sha256_file(&gp.join("read-state-bridge.bin"))? != QUARTER_SHA
            || sha256_file(&gp.join("read-state-bridge-categorical.bin"))? != CAT_SHA
        {
            return Err(bad("component frozen sidecars differ"));
        }
    }
    let n = l.source.compile_context_potential_rebound(&l.frozen)?;
    // Provenance belongs to the independently saved/reloaded parent; the
    // freshly compiled object supplies only a would-export execution identity.
    let identity: ConsumerIdentity =
        serde_json::from_value(read(&cp.join("native/metadata.json"))?["identity"].clone())?;
    let reloaded = NativeSourceRealizer::load(&cp.join("native"), &l.source, &identity)?;
    let loaded_binding = reloaded.artifact_binding()?;
    if serde_json::to_value(&loaded_binding)? != receipt["parent"]
        || n.execution_binding()? != loaded_binding
    {
        return Err(bad(
            "resumed source masters do not export saved native identity",
        ));
    }
    Ok(Some(
        json!({"root":fs::canonicalize(root)?,"producer_source_commit":r["source_commit"],"source_step":source_step,"generate_step":generate_step,"component_recomposition":cross,
        "report_sha256":sha256_file(&root.join("report.json"))?,"manifest_sha256":sha256_file(&root.join("manifest.json"))?,
        "checkpoint_receipt_sha256":sha256_file(&cp.join("receipt.json"))?,"generate_checkpoint_receipt_sha256":sha256_file(&gp.join("receipt.json"))?,"final_checkpoint_receipt_sha256":sha256_file(&root.join(format!("checkpoint-{final_step:04}/receipt.json")))?,"producer_completed_block_updates":final_step,"continued_parent_steps":if cross {None} else {Some(32)},"optimizer":if cross {"zero-update component recomposition; no optimizer steps"} else {"fresh field Adam, prior moments not resumed"}}),
    ))
}
fn run_prediction_control(
    a: &Args,
    start: Instant,
    d: &Device,
    l: &Loaded,
    train: &[Episode],
    dev: Vec<Episode>,
) -> Result<Value> {
    let (limit, rates) = control_settings(a)?;
    let resume = restore_control_resume(a, l, d)?;
    let eps = dev
        .into_iter()
        .enumerate()
        .filter_map(|(i, e)| CONTROL_INDICES.contains(&i).then_some(e))
        .collect::<Vec<_>>();
    let panel = control_panel_admit(&eps)?;
    for (local, e) in eps.iter().enumerate() {
        if train[CONTROL_INDICES[local]].packet.id != e.packet.id
            || train[CONTROL_INDICES[local]].target != e.target
        {
            return Err(bad("control train/development cases differ"));
        }
    }
    let original = identities(&l.original_bridge.parameters())?;
    let marker = identities(&l.marker.parameters())?;
    write(
        a,
        "admission.json",
        &json!({"mode":"prediction_control","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "checkpoint_receipt_sha256":CP_RECEIPT_SHA,"input_sha256":INPUT_SHA,"labels_sha256":LABEL_SHA,
        "panel":panel,"updates":limit,"rates":rates,"resume":resume,"trainable":a.prediction_control_trainable,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),
        "native_pool_backend":"host","fresh_adam":true,"initial_active_masters":identities(&active(&l.source,&l.generate)?)?,
        "original_bridge_masters":original,"marker_masters":marker,"loss_scope":a.loss_scope.name(),"phase_policy":loss_weight_policy(true, a.loss_scope)}),
    )?;
    write(
        a,
        "order.json",
        &json!({"order":CONTROL_INDICES,"policy":"same predetermined eight cases every update; no resampling or model reseeding"}),
    )?;
    let (model, g, b, initial_receipt) = checkpoint(a, 0, l)?;
    if g.vocab_size() != 4096 {
        return Err(bad("control requires the full pinned 4096 vocabulary"));
    }
    let initial_parent = if let Some(root) = &a.prediction_control_resume {
        root.join(format!(
            "checkpoint-{:04}",
            a.prediction_control_resume_source_step.unwrap_or(32)
        ))
    } else {
        a.checkpoint.clone()
    };
    let expected_receipt = read(&initial_parent.join("receipt.json"))?;
    let initial_generate = if let Some(root) = &a.prediction_control_resume {
        root.join(format!(
            "checkpoint-{:04}",
            a.prediction_control_resume_generate_step.unwrap_or(32)
        ))
    } else {
        a.checkpoint.clone()
    };
    if g.to_bytes()? != fs::read(initial_generate.join("generate.bin"))?
        || initial_receipt["parent"] != expected_receipt["parent"]
    {
        return Err(bad("control zero-update donor/resume identity differs"));
    }
    if a.prediction_control_resume.is_some() {
        for name in ["source", "native", "cue", "prefix"] {
            if tree_identity(&a.out.join("checkpoint-0000").join(name))?
                != tree_identity(&initial_parent.join(name))?
            {
                return Err(bad(
                    "initial resumed checkpoint is not exact saved native/master input",
                ));
            }
        }
    }
    let (_, admission) = batch(a, l, train, &CONTROL_INDICES, d, start, Some(&model))?;
    write(a, "zero-update-admission.json", &admission)?;
    // Initial entry replay precedes any optimizer construction or update.
    control_entry_diagnostics(a, l, &model, &g, &b, &eps, 0)?;
    // The eight control cases are a selected subset. Measure the same artifact's
    // entry decision on the whole panel it was drawn from, mixed pool versus
    // Generate-only pool, so "works on the eight" and "works on the panel" are
    // separate numbers rather than one number and an assumption. At the declared
    // zero-update recomposition the artifact measured here is the recomposed one.
    let panel_ceiling = {
        let public = NativeVocabularyActions::new(l.integer.binding().clone(), &l.exp)?;
        let legal = public
            .legal_token_ids()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let panel = load_panel(
            &a.development_inputs,
            &a.development_labels,
            &l.integer,
            &l.tokenizer,
            &legal,
            512,
        )?;
        entry_ceiling_panel(a, l, &model, &g, &b, &panel)?
    };
    let initial_eval = evaluate(
        a,
        "prediction-0000",
        &model,
        &g,
        Some(&b),
        &l.exp,
        &eps,
        &l.tokenizer,
        &l.cue,
        &l.prefix,
        start,
        None,
    )?;
    let initial_metrics = metrics(a, &initial_eval, &eps)?;
    write(a, "prediction-metrics-0000.json", &initial_metrics)?;
    let mut blocks = vec![
        json!({"step":0,"evaluation":initial_eval,"metrics":initial_metrics,"native_crossings":control_crossings(a,&g,&g,0)?}),
    ];
    let gp = l.generate.parameters();
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = gp
        .into_iter()
        .partition(|(name, _)| name == "generate.prototype_choices");
    let context = l.source.context_state_parameters();
    let potential = l.source.potential_parameters();
    let field_only = a.prediction_control_trainable != ControlTrainable::Joint;
    let potential_active = a.prediction_control_trainable != ControlTrainable::GenerateField;
    let mut frozen_groups = prototype.clone();
    frozen_groups.extend(context.clone());
    if !potential_active {
        frozen_groups.extend(potential.clone());
    }
    let frozen_groups_identity = identities(&frozen_groups)?;
    let mut go = optimizer(&coefficients, rates.generate)?;
    let mut po = optimizer(&prototype, rates.prototype)?;
    let mut co = optimizer(&context, rates.context)?;
    let mut vo = optimizer(&potential, rates.potential)?;
    let mut updates = Vec::new();
    let mut final_receipt = initial_receipt.clone();
    for update in 0..limit {
        disk_floor(a)?;
        deadline(a, start)?;
        let (grads, receipt) = batch(a, l, train, &CONTROL_INDICES, d, start, None)?;
        let grads = if field_only {
            grads
                .into_iter()
                .filter(|(name, _)| {
                    coefficients.contains_key(name)
                        || (potential_active && potential.contains_key(name))
                })
                .collect()
        } else {
            grads
        };
        let (denominator, norm) = clip_denominator(&grads, d)?;
        apply(&mut go, &coefficients, &grads, &denominator)?;
        if !field_only {
            apply(&mut po, &prototype, &grads, &denominator)?;
            apply(&mut co, &context, &grads, &denominator)?;
            for var in context.values() {
                var.set(&var.as_tensor().clamp(-1.75, 1.75)?)?;
            }
        }
        if potential_active {
            apply(&mut vo, &potential, &grads, &denominator)?;
            l.source.project_potential_range()?;
        }
        l.generate.project_shadow_range()?;
        if field_only && identities(&frozen_groups)? != frozen_groups_identity {
            return Err(bad(
                "field control changed frozen Context/prototype or inactive Potential masters",
            ));
        }
        if identities(&l.original_bridge.parameters())? != original
            || identities(&l.marker.parameters())? != marker
        {
            return Err(bad("control mutated frozen bridge masters"));
        }
        d.synchronize()?;
        updates.push(json!({"step":update+1,"before_update":receipt,"rates":rates,"global_active_gradient_norm":norm}));
        write(a, "updates.json", &json!(updates))?;
        if (update + 1) % 8 == 0 {
            let step = update + 1;
            let (model, current, bridge, rec) = checkpoint(a, step, l)?;
            control_entry_diagnostics(a, l, &model, &current, &bridge, &eps, step)?;
            let eval = evaluate(
                a,
                &format!("prediction-{step:04}"),
                &model,
                &current,
                Some(&bridge),
                &l.exp,
                &eps,
                &l.tokenizer,
                &l.cue,
                &l.prefix,
                start,
                None,
            )?;
            let m = metrics(a, &eval, &eps)?;
            write(a, &format!("prediction-metrics-{step:04}.json"), &m)?;
            blocks.push(json!({"step":step,"evaluation":eval,"metrics":m,"native_crossings":control_crossings(a,&g,&current,step)?}));
            final_receipt = rec;
        }
        write(a, "prediction-blocks.json", &json!(blocks))?;
    }
    let last = blocks
        .last()
        .ok_or_else(|| bad("control final evaluation absent"))?;
    let win = last["evaluation"]["complete"] == 8
        && last["metrics"]["entry_own_correct"] == 8
        && last["metrics"]["paired_own_position_both_correct"] == 4
        && last["metrics"]["first_divergent_own_prefix_reached"] == 8;
    write(a, "prediction-blocks.json", &json!(blocks))?;
    Ok(
        json!({"schema":"uor-r4.geometric-prediction-control/1","status":"COMPLETED","mode":"prediction_control",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"updates":limit,"rates":rates,"native_pool_backend":"host",
        "resume":resume,"trainable":a.prediction_control_trainable,"source_routing_scope":"Potential updates may change selected physical Source and transported state; no fixed-coordinate claim","original_indices":CONTROL_INDICES,"training_row_draws":limit*8,"target_position_draws":limit*84,
        "initial_receipt":initial_receipt,"final_receipt":final_receipt,"blocks":blocks,"panel_ceiling_step0":panel_ceiling,
        "native_prediction_control_win":win,"elapsed_seconds":start.elapsed().as_secs_f64(),
        "scope":"construction learning control on8 retained cases; complete accepted own-prefix replies+EOS and four source-swap pairs required; no generalization/chat/attention qualification; initial diagnostic0 can be run before choosing prospective control rates"}),
    )
}

/// Frozen native bytes only; no old Source/Context/Generate Vars are loaded.
struct ContinuationParent {
    binding: NativeArtifactBinding,
    integer: IntegerRealizer,
    tokenizer: ByteBpeTokenizer,
    native_directory: PathBuf,
    generate: Vec<u8>,
    generate_sha256: String,
    bridge: Vec<u8>,
    bridge_sha256: String,
    cue: Vec<u8>,
    joint: Option<Vec<u8>>,
    cue_metadata: CueCarrierMetadata,
    prefix: Vec<u8>,
    prefix_metadata: PrefixTransportMetadata,
    exp: Vec<u8>,
    exp_sha256: String,
    receipt: Value,
}
impl ContinuationParent {
    fn load(a: &Args) -> Result<Self> {
        if reply_completion::is_mode(a.mode) {
            reply_completion::settings(a)?;
        } else if a.mode == Mode::JointContinuation {
            joint_continuation_settings(a)?
                .ok_or_else(|| bad("joint continuation config absent"))?;
        } else {
            continuation_settings(a)?.ok_or_else(|| bad("continuation config absent"))?;
        }
        report_output::verify(&a.saved_fit)?;
        if sha256_file(&a.saved_fit.join("report.json"))? != CONTINUATION_PARENT_REPORT_SHA
            || sha256_file(&a.saved_fit.join("manifest.json"))? != CONTINUATION_PARENT_MANIFEST_SHA
            || fs::canonicalize(&a.checkpoint)?
                != fs::canonicalize(a.saved_fit.join("checkpoint-0000"))?
        {
            return Err(bad(
                "exact selected continuation parent report/seal/checkpoint differs",
            ));
        }
        let r = read(&a.saved_fit.join("report.json"))?;
        let receipt = read(&a.checkpoint.join("receipt.json"))?;
        if r["schema"] != "uor-r4.geometric-prediction-control/1"
            || r["status"] != "COMPLETED"
            || r["updates"] != 0
            || r["native_prediction_control_win"] != true
            || r["resume"]["component_recomposition"] != true
            || r["resume"]["source_step"] != 48
            || r["resume"]["generate_step"] != 64
            || r["resume"]["producer_source_commit"] != "b451571aa638d1f93e3ccf25f718f27ed2ea731f"
            || receipt != r["initial_receipt"]
            || receipt != r["final_receipt"]
            || receipt["step"] != 0
            || receipt["native_independently_reloaded"] != true
            || receipt["masters_independently_reloaded"] != true
        {
            return Err(bad(
                "continuation parent is not the authenticated successful48/64 recomposition",
            ));
        }
        for (p, expected) in [
            (&a.training_inputs, INPUT_SHA),
            (&a.development_inputs, INPUT_SHA),
            (&a.training_labels, LABEL_SHA),
            (&a.development_labels, LABEL_SHA),
        ] {
            report_output::verify(&seal_for(p)?)?;
            if sha256_file(p)? != expected {
                return Err(bad("fixed512 continuation panel identity differs"));
            }
        }
        Self::from_checkpoint(&a.checkpoint)
    }
    /// Current checkpoints are owned by this still-open report. Their receipt
    /// and every native sidecar are authenticated here before any evaluation.
    /// Only load() authorizes an initial parent from the sealed retained run.
    fn from_checkpoint(cp: &Path) -> Result<Self> {
        let receipt = read(&cp.join("receipt.json"))?;
        let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
        let native_directory = cp.join("native");
        let integer = IntegerRealizer::load_native(&native_directory, &binding)?;
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
            native_directory.join("tokenizer.json"),
        )?)
        .ok_or_else(|| bad("continuation ByteBPE unavailable"))?;
        let exp = fs::read(native_directory.join("consumer/exp-q31.bin"))?;
        let loaded = Self {
            binding,
            integer,
            tokenizer,
            native_directory,
            generate: fs::read(cp.join("generate.bin"))?,
            generate_sha256: receipt["generate_sha256"]
                .as_str()
                .ok_or_else(|| bad("parent Generate hash absent"))?
                .into(),
            bridge: fs::read(cp.join("read-state-bridge-categorical.bin"))?,
            bridge_sha256: receipt["categorical_sha256"]
                .as_str()
                .ok_or_else(|| bad("parent bridge hash absent"))?
                .into(),
            cue: fs::read(cp.join("cue/cue-q4.bin"))?,
            joint: if cp.join("cue/cue-joint-q4.bin").is_file() {
                Some(fs::read(cp.join("cue/cue-joint-q4.bin"))?)
            } else {
                None
            },
            cue_metadata: serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?,
            prefix: fs::read(cp.join("prefix/prefix-q4.bin"))?,
            prefix_metadata: serde_json::from_value(read(
                &cp.join("prefix/native-metadata.json"),
            )?)?,
            exp_sha256: sha256_bytes(&exp),
            exp,
            receipt,
        };
        // The public native loader verifies the full binding and every sidecar;
        // no private generalizer loader or caller-selected endpoint is needed.
        loaded.generator()?;
        Ok(loaded)
    }
    fn generator(&self) -> Result<NativeBankGenerator> {
        Ok(NativeBankGenerator::load(NativeBankArtifacts {
            native_directory: &self.native_directory,
            source_binding: &self.binding,
            generate: BoundNativeBytes {
                bytes: &self.generate,
                sha256: &self.generate_sha256,
            },
            bridge: Some(BoundNativeBytes {
                bytes: &self.bridge,
                sha256: &self.bridge_sha256,
            }),
            cue_packed: &self.cue,
            cue_joint_packed: self.joint.as_deref(),
            cue_metadata: &self.cue_metadata,
            prefix_packed: &self.prefix,
            prefix_metadata: &self.prefix_metadata,
            exp: BoundNativeBytes {
                bytes: &self.exp,
                sha256: &self.exp_sha256,
            },
        })?)
    }
}

/// Input-only conversion. The complete packet is retained, not a gold Source.
fn continuation_snapshot(p: &Packet) -> Result<PinnedBankSnapshot> {
    if !p.actual_prefix_ids.is_empty() {
        return Err(bad("continuation packet has supplied prefix"));
    }
    let mut scope: Option<&String> = None;
    let mut commit = 0;
    for s in &p.segments {
        if let Segment::Source {
            scope: q,
            commit: c,
            ..
        } = s
        {
            if q.is_empty() || scope.is_some_and(|old| old != q) {
                return Err(bad("continuation bank mixed/empty scope"));
            }
            scope = Some(q);
            commit = commit.max(*c);
        }
    }
    let scope = scope.ok_or_else(|| bad("fixed continuation panel requires actual Source bank"))?;
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: 0,
            commit,
            scope: scope.as_bytes().to_vec(),
        },
        query_ids: p.query_ids.clone(),
        segments: p
            .segments
            .iter()
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
                    event: *event,
                    record: *record,
                    commit: *commit,
                    scope: scope.as_bytes().to_vec(),
                    entity: entity.clone(),
                    relation: *relation,
                    view: *view,
                    status: SnapshotSourceStatus::Found,
                    original_token_ids: original_source_ids.clone(),
                }),
                Segment::Context {
                    event,
                    role,
                    token_ids,
                } => OwnedBankSegment::Context {
                    event: *event,
                    role: *role,
                    token_ids: token_ids.clone(),
                },
            })
            .collect(),
    })
}

fn continuation_checkpoint(
    a: &Args,
    step: usize,
    p: &ContinuationParent,
    weights: &ContinuationLearningWeights,
    resume: Option<&cross_state_completion::ResumeState>,
) -> Result<(NativeContinuationField, Value)> {
    disk_floor(a)?;
    if size(&a.out)?.saturating_add(1 << 20) > a.maximum_report_bytes - (1 << 20) {
        return Err(bad("continuation checkpoint storage admission"));
    }
    // Check the source seal again before independently loading the native parent.
    report_output::verify(&a.saved_fit)?;
    if sha256_file(&a.saved_fit.join("report.json"))? != CONTINUATION_PARENT_REPORT_SHA
        || sha256_file(&a.saved_fit.join("manifest.json"))? != CONTINUATION_PARENT_MANIFEST_SHA
    {
        return Err(bad("frozen continuation upstream changed"));
    }
    let reloaded = p.generator()?;
    let field = weights.export_native(reloaded.source_binding(), reloaded.generate_model())?;
    let bytes = field.to_bytes()?;
    let root = a.out.join(format!("checkpoint-{step:04}"));
    fs::create_dir(&root)?;
    let masters = save_masters(&root.join("continuation-source"), &weights.parameters())?;
    fs::write(root.join("continuation-field.bin"), &bytes)?;
    let disk = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        reloaded.source_binding(),
        reloaded.generate_model(),
    )?;
    let cpu = ContinuationLearningWeights::from_native(
        &disk,
        reloaded.generate_model(),
        p.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &root.join("continuation-source"),
        &masters,
        &cpu.parameters(),
        &Device::Cpu,
    )?;
    if disk.to_bytes()? != bytes
        || cpu
            .export_native(reloaded.source_binding(), reloaded.generate_model())?
            .to_bytes()?
            != bytes
    {
        return Err(bad(
            "continuation independent masters/native reload differs",
        ));
    }
    let receipt = json!({"step":step,"lineage_step":step+if resume.is_some(){64}else{0},
        "cross_state_resume":resume.map(|r|&r.provenance),
        "parent":reloaded.source_binding(),"generate_sha256":p.generate_sha256,
        "frozen_model_root":fs::canonicalize(&a.saved_fit)?,"frozen_model_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
        "frozen_model_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,"frozen_parent_receipt":p.receipt,
        "continuation_sha256":sha256_bytes(&bytes),"parameters":masters,"active_parameter_names":weights.parameters().keys().collect::<Vec<_>>(),
        "shared_coefficients":weights.shared_coefficients(),"loss_scope":"all","credit":a.credit.name(),"order_seed":a.seed,
        "native_independently_reloaded":true,"masters_independently_reloaded":true,
        "upstream_training":"all Context/Source/Potential/Generate/prototype/bridge/cue/prefix frozen; no old Vars loaded",
        "fresh_adam":"zero moments; not optimizer-state continuation"});
    fs::write(
        root.join("continuation-source/metadata.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    Ok((disk, receipt))
}

/// Uses the production generator independently of the training cache. Labels
/// supply only previous teacher tokens and evaluator membership after emission.
fn continuation_witness(step: &NativeBankGenerateStep) -> Result<Value> {
    let witness = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("native continuation witness absent"))?;
    Ok(
        json!({"query_tokens":witness.query_tokens,"actual_prefix_tokens":witness.actual_prefix_tokens,
        "state_codes":witness.state_codes.iter().map(|v|v.index()).collect::<Vec<_>>(),
        "factual_post_state_codes":step.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),
        "delta_scores_q24_sha256":sha256_bytes(&serde_json::to_vec(&witness.delta_scores_q24)?),
        "delta_scores":witness.delta_scores_q24.len(),"minimum_delta_q24":witness.delta_scores_q24.iter().min(),
        "maximum_delta_q24":witness.delta_scores_q24.iter().max(),"encoding_coefficient_reads":witness.encoding_coefficient_reads,
        "field_counts":witness.counts,"full_delta_trace":"existing native-bank-generalization evaluator; fitter stores digest and bound"}),
    )
}

fn continuation_evaluate(
    a: &Args,
    name: &str,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    start: Instant,
) -> Result<Value> {
    continuation_evaluate_rows(a, name, p, field, &eps.iter().collect::<Vec<_>>(), start)
}
fn continuation_evaluate_rows(
    a: &Args,
    name: &str,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[&Episode],
    start: Instant,
) -> Result<Value> {
    continuation_evaluate_rows_impl(a, name, p, field, eps, start, true)
}
fn continuation_evaluate_rows_impl(
    a: &Args,
    name: &str,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[&Episode],
    start: Instant,
    include_canonical: bool,
) -> Result<Value> {
    let bytes = field.to_bytes()?;
    let field_sha = sha256_bytes(&bytes);
    let mut generator = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &field_sha,
    })?;
    let mut rows = Vec::new();
    let mut complete = 0;
    let mut ce = 0.;
    let mut tokens = 0;
    for e in eps {
        deadline(a, start)?;
        let bank = generator.admit_bank(continuation_snapshot(&e.packet)?)?;
        let mut canonical = Vec::new();
        let mut rowce = 0.;
        if include_canonical {
            for (t, &target) in e.target.iter().enumerate() {
                deadline(a, start)?;
                let step = generator.step(&bank, &e.target[..t])?;
                let total = step.actions.summary.total_weight_q31;
                let mass = step
                    .actions
                    .token_masses
                    .iter()
                    .find(|m| m.token_id == target)
                    .ok_or_else(|| bad("continuation full-pool target support missing"))?
                    .weight_q31;
                if total == 0 || mass == 0 {
                    return Err(bad("continuation positive native support violated"));
                }
                let loss = -(mass as f64 / total as f64).ln();
                rowce += loss;
                tokens += 1;
                canonical.push(json!({"target_label_only":target,"native_ce":loss,"native_target_mass":mass,
                "native_denominator":total,"native":{"pool":{"summary":step.actions.summary},
                    "copy_token_ids":step.copy_token_ids,"post_state_codes":step.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),
                    "generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&step.generate_raw_scores_q24)?),
                    "continuation":continuation_witness(&step)?}}));
            }
        }
        // No target, accepted answer or cache is passed to own-feedback serving.
        let generated = generator.generate(
            &bank,
            GenerationLimits {
                maximum_tokens: 32,
                maximum_retained_steps: 32,
            },
        )?;
        let eos = generated.stop == GenerationStop::Eos;
        let text = p
            .tokenizer
            .decode(&generated.generated_ids[..generated.generated_ids.len() - usize::from(eos)]);
        let accepted = eos && e.answers.accepts(&text);
        complete += usize::from(accepted);
        ce += rowce / e.target.len() as f64;
        let filename = format!("{name}-row-{:04}.json", rows.len());
        let generation = generated.steps.iter().map(|s| -> Result<Value> {
            Ok(json!({"actual_prefix_ids":s.actual_prefix_ids,"pool":{"summary":s.actions.summary},
                "continuation":continuation_witness(s)?}))
        }).collect::<Result<Vec<_>>>()?;
        write(
            a,
            &filename,
            &json!({"id":e.packet.id,"canonical_target_ids_labels_only":e.target,"canonical":if include_canonical {json!(canonical)} else {json!("NOT_RUN")},
            "generation":generation,"generated_ids":generated.generated_ids,"decoded":text,"eos":eos,"complete":accepted,
            "native_equal_episode_ce":if include_canonical {json!(rowce/e.target.len()as f64)} else {Value::Null},"continuation_sha256":field_sha}),
        )?;
        rows.push(json!({"id":e.packet.id,"native_equal_episode_ce":if include_canonical {json!(rowce/e.target.len()as f64)} else {Value::Null},"complete":accepted,
            "eos":eos,"generated_ids":generated.generated_ids,"row_file":filename,"row_sha256":sha256_file(&a.out.join(&filename))?}));
    }
    let result = json!({"cases":eps.len(),"target_positions":tokens,"complete":complete,"native_equal_episode_ce":if include_canonical {json!(ce/eps.len()as f64)} else {Value::Null},
        "rows":rows,"continuation_sha256":field_sha,"runtime":"production native integer generator; full Copy/Generate common pool; own emitted feedback",
        "scope":if include_canonical {"retained exposed512 construction panel; teacher-prefix metrics separate from complete own-prefix answers; no transfer/chat qualification"} else {"selected exposed-panel actual ownprefix rows; canonical metrics NOT_RUN; no fullpanel/transfer/chat qualification"}});
    write(a, &format!("{name}.json"), &result)?;
    Ok(result)
}

/// Restore the accepted recomposition's floating masters as well as native
/// bytes. The old load() authenticates a different historical parent and must
/// never be weakened to admit this joint rung.
fn load_joint_continuation(a: &Args, p: &ContinuationParent, d: &Device) -> Result<Loaded> {
    let cp = &a.checkpoint;
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let source =
        SourceRealizerWeights::load_context_potential_on_device(&cp.join("source"), &tokbytes, d)?;
    let identity: ConsumerIdentity =
        serde_json::from_value(read(&cp.join("native/metadata.json"))?["identity"].clone())?;
    let frozen = NativeSourceRealizer::load(&cp.join("native"), &source, &identity)?;
    if frozen.artifact_binding()? != p.binding {
        return Err(bad("joint source masters/native parent differ"));
    }
    let integer = IntegerRealizer::load_native(&cp.join("native"), &p.binding)?;
    let native_generate = NativeGeometricGenerate::from_bytes(&p.generate, integer.binding())?;
    let generate =
        GenerateLearningWeights::from_native(integer.binding().clone(), &native_generate, d)?;
    let gm = read(&cp.join("generate-source/metadata.json"))?;
    if gm["tokenizer_sha256"] != sha256_bytes(&tokbytes)
        || gm["protocol"] != serde_json::to_value(integer.binding().protocol())?
        || gm["lanes"] != json!(generate.lanes())
        || generate.lanes() != 8
        || generate.vocab_size() != 4096
    {
        return Err(bad("joint Generate master/tokenizer/shape binding differs"));
    }
    restore(
        &cp.join("generate-source"),
        &gm["parameters"],
        &generate.parameters(),
        d,
    )?;
    if generate.export_native()?.to_bytes()? != p.generate {
        return Err(bad(
            "joint restored Generate differs from exact48/64 native parent",
        ));
    }
    let original_bytes = fs::read(cp.join("read-state-bridge.bin"))?;
    if sha256_bytes(&original_bytes) != QUARTER_SHA || p.bridge_sha256 != CAT_SHA {
        return Err(bad("joint frozen bridge identity differs"));
    }
    let original = NativeGeometricReadStateBridge::from_bytes(&original_bytes, integer.binding())?;
    let original_bridge = BridgeLearningWeights::from_native(&original, integer.binding(), d)?;
    let bm = read(&cp.join("read-state-bridge-source/metadata.json"))?;
    if bm["metadata"] != serde_json::to_value(original.metadata())? {
        return Err(bad("joint original bridge metadata differs"));
    }
    restore(
        &cp.join("read-state-bridge-source"),
        &bm["source_parameters"],
        &original_bridge.parameters(),
        d,
    )?;
    let rebuilt = original_bridge.export_categorical_actions()?;
    if original_bridge.export_native()?.to_bytes()? != original_bytes
        || rebuilt.native.to_bytes()? != p.bridge
    {
        return Err(bad("joint frozen bridge masters/native/map differ"));
    }
    let marker = BridgeLearningWeights::from_native(&rebuilt.native, integer.binding(), d)?;
    if marker.export_native()?.to_bytes()? != p.bridge {
        return Err(bad("joint categorical marker differs"));
    }
    let cue = cue_payload(&cp.join("cue"))?;
    let prefix = prefix_payload(&cp.join("prefix"))?;
    let cc = frozen.compile_cue_carrier(cue.clone())?;
    let pp = frozen.compile_prefix_transport(&cc, prefix_clone(&prefix)?)?;
    if cc.metadata() != &p.cue_metadata || pp.metadata() != &p.prefix_metadata {
        return Err(bad("joint frozen cue/prefix binding differs"));
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokbytes)
        .ok_or_else(|| bad("joint tokenizer unavailable"))?;
    Ok(Loaded {
        source,
        frozen,
        integer,
        tokenizer,
        generate,
        original_bridge,
        marker,
        categorical: None,
        cue,
        prefix,
        exp: p.exp.clone(),
        receipt: p.receipt.clone(),
        categorical_receipt: serde_json::to_value(rebuilt.receipt)?,
    })
}

fn joint_checkpoint(
    a: &Args,
    step: usize,
    l: &Loaded,
    weights: &ContinuationLearningWeights,
) -> Result<(ContinuationParent, NativeContinuationField, Value)> {
    // This saves/reloads current Source, Generate, their floating masters and
    // all rebound cue/prefix sidecars before U is bound to them.
    let (_, _, _, mut receipt) = checkpoint(a, step, l)?;
    let root = a.out.join(format!("checkpoint-{step:04}"));
    let current = ContinuationParent::from_checkpoint(&root)?;
    let generator = current.generator()?;
    let field = weights.export_native(generator.source_binding(), generator.generate_model())?;
    let bytes = field.to_bytes()?;
    let masters = save_masters(&root.join("continuation-source"), &weights.parameters())?;
    fs::write(root.join("continuation-field.bin"), &bytes)?;
    let disk = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        generator.source_binding(),
        generator.generate_model(),
    )?;
    let cpu = ContinuationLearningWeights::from_native(
        &disk,
        generator.generate_model(),
        current.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &root.join("continuation-source"),
        &masters,
        &cpu.parameters(),
        &Device::Cpu,
    )?;
    if disk.to_bytes()? != bytes
        || cpu
            .export_native(generator.source_binding(), generator.generate_model())?
            .to_bytes()?
            != bytes
    {
        return Err(bad("joint U masters/native independent reload differs"));
    }
    receipt["mode"] = json!("joint_continuation");
    receipt["continuation_sha256"] = json!(sha256_bytes(&bytes));
    receipt["continuation_parameters"] = masters;
    receipt["active_parameter_names"] = json!(joint_active(l, weights)?.keys().collect::<Vec<_>>());
    receipt["frozen_prototype_masters"] = json!(identities(&BTreeMap::from([(
        "generate.prototype_choices".into(),
        l.generate.prototype_choices.clone(),
    )]))?);
    receipt["parent_report_sha256"] = json!(CONTINUATION_PARENT_REPORT_SHA);
    receipt["parent_manifest_sha256"] = json!(CONTINUATION_PARENT_MANIFEST_SHA);
    receipt["training_input_sha256"] = json!(INPUT_SHA);
    receipt["training_labels_sha256"] = json!(LABEL_SHA);
    receipt["source_commit"] = json!(option_env!("UOR_BUILD_SOURCE_COMMIT"));
    receipt["learning_rates_sha256"] = json!(sha256_bytes(&serde_json::to_vec(
        joint_continuation_settings(a)?.ok_or_else(|| bad("joint config absent"))?,
    )?));
    if a.reference_replay.is_some() {
        receipt["reference_replay"] = reference_binding(a)?;
    }
    receipt["credit_scope"] = json!("Context/Potential + Generate unary/pair/bias + v2 U; local conditional full120 Context utility and factual selector credit; frozen prototype choices and categorical map; RawIdentity surrogate, not a hard-runtime derivative");
    if a.native_code_proposals {
        receipt["native_code_proposals"] = proposal_policy(a);
        receipt["optimizer_updates"] = json!(0);
        receipt["credit_scope"] = json!("parent RawIdentity gradients rank fixed legal native-code Context basis/Potential/Generate unary-pair/U proposals; native objective selects at most one; no Adam or global clipping applied; token coefficients, Generate bias/prototypes and other source masters frozen");
    }
    if a.constrained_context_learning {
        receipt["constrained_context_learning"] = json!(true);
        receipt["fresh_adam"] = json!(false);
        receipt["credit_scope"] = json!("one complete original-parent all-channel Context gradient ranks adjacent Q4 moves across six shared bases; one composite preserves exact protected native decisions; Potential/Generate/U and categorical map fixed; no Adam or global clipping; independent native CE acceptance; greedy feasible-prefix construction, not an exact recurrent gradient");
    }
    if a.categorical_action_learning {
        receipt["credit_scope"] = json!("parent Context row credit followed by recomputed full120 shared categorical action contrasts at changed Context; one conditional two-block native candidate; Potential/Generate/U unchanged, no Adam; independent native CE acceptance");
    }
    if a.categorical_action_only {
        receipt["categorical_action_only"] = json!(true);
        receipt["fresh_adam"] = json!(false);
        receipt["credit_scope"] = json!("one original-parent full120 shared categorical action contrast; only one action row may change; Context/Potential/Generate/U fixed for attribution, no Adam; independent native CE acceptance; not joint Context adaptation");
    }
    if a.prototype_compensation.is_some() {
        receipt["prototype_compensation"] = prototype_compensation::input_identity(a)?;
        receipt["fresh_adam"] = json!(false);
        receipt["credit_scope"] = prototype_compensation::policy();
    } else if a.constrained_emission_learning {
        receipt["constrained_emission_learning"] = json!(true);
        receipt["immediate_input"] = constrained_emission::input_identity(a)?;
        receipt["fresh_adam"] = json!(false);
        receipt["credit_scope"] = json!("fresh retained-Context gradient; only shared Generate unary/pair adjacent Q4 codes accumulate under exact pooled successful-output constraints; all upstream, bias, prototypes and U numerical masters fixed; one composite, no Adam");
    }

    fs::write(
        root.join("continuation-source/metadata.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    let reloaded = ContinuationParent::from_checkpoint(&root)?;
    let _ = reloaded
        .generator()?
        .with_continuation_field(BoundNativeBytes {
            bytes: &bytes,
            sha256: &sha256_bytes(&bytes),
        })?;
    Ok((reloaded, disk, receipt))
}

/// New interface guard: actual live learning graph versus the separately
/// loaded production generator, across entry, continuation, factual Copy and
/// EOS positions. No cache from the frozen-only fitter participates.
fn joint_native_parity(
    l: &Loaded,
    weights: &ContinuationLearningWeights,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    indices: &[usize],
    d: &Device,
    reached: Option<&frontier::Plan>,
) -> Result<Value> {
    let clock = Instant::now();
    let mask = reached.map(|plan| {
        plan.terms
            .iter()
            .map(|t| ((t.index, t.position), t.parent_actual_prefix_ids.as_slice()))
            .collect::<BTreeMap<_, _>>()
    });
    let current = l.source.compile_context_potential_rebound(&l.frozen)?;
    let prepared = l.source.prepare_context_potential_on_device(&current, d)?;
    let cue = current.compile_cue_carrier(l.cue.clone())?;
    let prefix = current.compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let gs = l.generate.prepare_native()?;
    let categorical = l.prepared_categorical(d)?;
    let us = weights.prepare_native(&current.execution_binding()?, &gs.native)?;
    if us.native.to_bytes()? != field.to_bytes()? || current.execution_binding()? != p.binding {
        return Err(bad("joint current graph/checkpoint bindings differ"));
    }
    let mut learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_prefix_temporal_utility(false)
        .with_categorical_read_state_bridge(&categorical)?
        .with_read_selector_credit(true)
        .with_continuation_field(weights, &us)?
        .with_continuation_context_credit(true)?;
    let bytes = field.to_bytes()?;
    let mut generator = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &sha256_bytes(&bytes),
    })?;
    let mut positions = 0usize;
    for &index in indices {
        let e = eps
            .get(index)
            .ok_or_else(|| bad("joint parity index absent"))?;
        let bank = generator.admit_bank(continuation_snapshot(&e.packet)?)?;
        for t in 0..e.target.len() {
            if mask.as_ref().is_some_and(|m| !m.contains_key(&(index, t))) {
                continue;
            }
            let actual = mask
                .as_ref()
                .and_then(|m| m.get(&(index, t)).copied())
                .unwrap_or(&e.target[..t]);
            replay_require(
                actual == &e.target[..t],
                "parity actual prefix admission differs",
            )?;
            let out =
                learner.forward_bank(&e.segments()?, &e.packet.query_ids, actual, &cue, &prefix)?;
            let native = generator.step(&bank, actual)?;
            let local = out
                .continuation
                .as_ref()
                .ok_or_else(|| bad("joint local graph missing"))?;
            let witness = native
                .continuation
                .as_ref()
                .ok_or_else(|| bad("joint native local witness missing"))?;
            if out.generate.scores_q24 != native.generate_raw_scores_q24
                || out.copy_token_ids != native.copy_token_ids
                || out.copy_scores_q24 != native.copy_raw_scores_q24
                || out.final_state_codes != native.post_state
                || local.final_state_codes != witness.state_codes
                || local.field.delta_scores_q24 != witness.delta_scores_q24
                || serde_json::to_value(&out.actions)? != serde_json::to_value(&native.actions)?
            {
                return Err(bad(
                    "joint live graph/native current checkpoint parity differs",
                ));
            }
            positions += 1;
        }
    }
    if let Some(mask) = mask {
        replay_require(
            positions == mask.len(),
            "reached parity position coverage differs",
        )?;
    }
    let mut receipt = json!({"indices":indices,"positions":positions,"current_source_binding":p.binding,
        "generate_sha256":p.generate_sha256,"continuation_sha256":sha256_bytes(&bytes),
        "exact_scores_states_alias_pool":true,"labels":"previous teacher prefix only; current target never enters forward"});
    if reached.is_some() {
        receipt["labels"] =
            json!("frozen authenticated parent actual prefix; current target never enters forward");
        receipt["elapsed_seconds"] = json!(clock.elapsed().as_secs_f64());
    }
    Ok(receipt)
}

fn joint_row_comparison(before: &Value, after: &Value) -> Result<Value> {
    let a = before["rows"]
        .as_array()
        .ok_or_else(|| bad("initial joint rows absent"))?;
    let b = after["rows"]
        .as_array()
        .ok_or_else(|| bad("final joint rows absent"))?;
    if a.len() != b.len() {
        return Err(bad("joint row populations differ"));
    }
    let mut retained = Vec::new();
    let mut gained = Vec::new();
    let mut lost = Vec::new();
    let mut changed = Vec::new();
    for (old, new) in a.iter().zip(b) {
        if old["id"] != new["id"] {
            return Err(bad("joint row identity/order differs"));
        }
        let was = old["complete"]
            .as_bool()
            .ok_or_else(|| bad("initial complete missing"))?;
        let now = new["complete"]
            .as_bool()
            .ok_or_else(|| bad("final complete missing"))?;
        match (was, now) {
            (true, true) => retained.push(old["id"].clone()),
            (false, true) => gained.push(old["id"].clone()),
            (true, false) => lost.push(old["id"].clone()),
            _ => (),
        }
        if old["generated_ids"] != new["generated_ids"] {
            changed.push(old["id"].clone());
        }
    }
    Ok(
        json!({"retained_complete_ids":retained,"gained_complete_ids":gained,
        "lost_complete_ids":lost,"changed_output_ids":changed}),
    )
}

fn run_joint_continuation(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    if a.coupled_episode_learning.is_some() {
        return coupled_episode_learning::run(a, start, d);
    }
    if a.generate_episode_learning.is_some() {
        return generate_episode_learning::run(a, start, d);
    }
    if a.prefix_fragment_learning.is_some() {
        return prefix_fragment_learning::run(a, start, d);
    }
    if a.context_cue_coadapt.is_some() {
        return context_cue_coadapt::run(a, start, d);
    }
    if a.prefix_context_credit.is_some() {
        return prefix_context_credit::run(a, start, d);
    }
    if a.readout_coadaptation.is_some() {
        return readout_coadaptation::run(a, start, d);
    }
    if a.reached_u.is_some() {
        return reached_u::run(a, start, d);
    }
    if a.prototype_compensation.is_some() {
        return prototype_compensation::run(a, start, d);
    }
    let rates = joint_continuation_settings(a)?.ok_or_else(|| bad("joint config absent"))?;
    let original_parent = ContinuationParent::load(a)?;
    let parent = if a.constrained_emission_learning {
        constrained_emission::load_parent(a, &original_parent)?
    } else {
        ContinuationParent::from_checkpoint(&a.checkpoint)?
    };
    let mut load_args = a.clone();
    if let Some(root) = &a.retained_context_root {
        load_args.checkpoint = root.join("checkpoint-0001");
    }
    let mut l = load_joint_continuation(&load_args, &parent, d)?;
    if a.categorical_action_learning {
        l.categorical = Some(CategoricalBridgeLearningWeights::from_bytes(
            &parent.bridge,
            l.generate.binding(),
            CAT_SHA,
            d,
        )?);
        replay_require(
            l.categorical_bytes()? == parent.bridge,
            "categorical learning initialization changed parent bytes",
        )?;
    }
    let legal = NativeVocabularyActions::new(parent.integer.binding().clone(), &parent.exp)?
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &parent.integer,
        &parent.tokenizer,
        &legal,
        512,
    )?;
    if eps.len() != 512 {
        return Err(bad("joint requires complete512 construction panel"));
    }
    pairs(&eps)?;
    // Historical eligibility remains pinned to its original donor, even when the immediate input is the retained Context endpoint.
    let reached = frontier::load(a, &original_parent, &eps)?;
    let reference = if let Some(plan) = &reached {
        Some(plan.canonical_reference.clone())
    } else {
        load_reference_plan(a, &parent, &eps)?
    };
    let reached_components = reached.as_ref().map(frontier::components).transpose()?;
    let weights = ContinuationLearningWeights::zeroed_shared_action(
        l.integer.binding(),
        &parent.binding,
        l.generate.lanes(),
        d,
    )?;
    if a.constrained_emission_learning {
        constrained_emission::restore_field(a, &parent, &weights, d)?;
    }
    let params = joint_active(&l, &weights)?;
    let prototype = BTreeMap::from([(
        "generate.prototype_choices".into(),
        l.generate.prototype_choices.clone(),
    )]);
    let frozen_prototype = identities(&prototype)?;
    let frozen_bridge = identities(&l.original_bridge.parameters())?;
    let frozen_marker = identities(&l.marker.parameters())?;
    let schedule = if reached.is_some() {
        (0..eps.len()).collect()
    } else {
        order(a.seed, eps.len())
    };
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,"batch":BATCH,"updates":a.updates,
        "policy":"full512 SplitMix64/Fisher-Yates cyclic batches; actual input plus prior supervised prefix"}),
    )?;
    write(
        a,
        "admission.json",
        &json!({"mode":"joint_continuation","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "parent":parent.receipt,"parent_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
        "parent_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,"training_input_sha256":INPUT_SHA,"training_labels_sha256":LABEL_SHA,
        "active_parameter_names":params.keys().collect::<Vec<_>>(),"initial_active_master_identities":identities(&params)?,
        "frozen_prototype_masters":frozen_prototype,"frozen_bridge_masters":frozen_bridge,"rates":rates,
        "continuation_initialization":"zero v2 field on accepted Source48/Generate64; failed U32/U64 fields not adopted",
        "phase_policy":loss_weight_policy(true,LossScope::All),"fresh_adam":true,"credit":"raw_identity",
        "read_state_pullback":"categorical","context_credit":"local full120 conditional utility through actual query+prefix ContextQ4; factual bank/selector remains live",
        "snapshot_policy":"fresh current Source/Potential, Generate and mutually bound U each batch; no fixed-position cache",
        "prototype_policy":"masters excluded from gradients accumulated for optimizer/global clipping; native prototype codes frozen; U prototype adjoint absent",
        "serving":"full legal vocabulary and physical Copy aliases, one common clip/marginal; actual own emitted prefix, no selected record supplied",
        "device_scope":"CUDA learning graphs and backward; native reference preparation, alias reduction, exports and evaluation on host; not fully device resident",
        "data_scope":"train and open development are the same exposed512; no held-out/chat qualification"}),
    )?;
    if reference.is_some() {
        let mut admission = read(&a.out.join("admission.json"))?;
        admission["reference_replay"] = reference_binding(a)?;
        write(a, "admission.json", &admission)?;
    }
    if a.native_code_proposals {
        let mut admission = read(&a.out.join("admission.json"))?;
        admission["native_code_proposals"] = proposal_policy(a);
        admission["fresh_adam"] = json!(false);
        admission["optimizer_updates"] = json!(0);
        admission["rates_role"] = json!("legacy parent/replay identity only; no rate applied");
        if reached.is_some() {
            admission["reached_frontier_objective"] = json!(frontier::OBJECTIVE);
            admission["batch"] = Value::Null;
            admission["phase_policy"] = json!("frontier equal episodes; successful trajectories equal episodes/nonempty phases/positions");
            write(
                a,
                "order.json",
                &json!({"seed":a.seed,"order":(0..eps.len()).collect::<Vec<_>>(),"batch":null,"updates":1,
                "policy":"all frozen reached-frontier and successful-trajectory terms; no B8 sampling"}),
            )?;
        }
        if a.constrained_context_learning {
            admission["constrained_context_learning"] = json!(true);
            admission["context_credit"] = json!("complete original-parent parameter gradient with all existing credit channels; one coordinated shared Context composite constrained by exact successful native decisions");
            admission["parameter_update_policy"] = json!("six shared Context bases only; one frozen coordinate order; all other master bits and categorical map preserved; no Adam or global clipping");
        }
        if a.categorical_action_learning {
            admission["categorical_action_learning"] =
                categorical_proposals::policy_for(a.categorical_action_only);
            admission["prototype_policy"] = json!("Generate prototypes frozen; independently learned onehot categorical action choices admitted at factual shared relative key");
            admission["context_credit"] = json!("existing full120 conditional Context utility retained exactly once; full120 factual-key action utility added; Context and action gradients recomputed between blocks");
        }
        if a.categorical_action_only {
            admission["categorical_action_only"] = json!(true);
            admission["context_credit"] = json!("existing full120 conditional Context utility remains in the graph; only original-parent shared action contrast is used for a proposal; Context masters fixed for attribution, not joint learning");
            admission["parameter_update_policy"] = json!("one shared categorical action row only; all non-action master bits preserved; no conditional Context step or second gradient pass");
        }
        write(a, "admission.json", &admission)?;
    }
    if a.constrained_emission_learning {
        let mut admission = read(&a.out.join("admission.json"))?;
        admission["immediate_input"] = constrained_emission::input_identity(a)?;
        admission["continuation_initialization"] = json!("authenticated retained Context checkpoint continuation master restore; numeric U frozen");
        admission["fresh_adam"] = json!(false);
        admission["parameter_update_policy"] = constrained_emission::policy();
        write(a, "admission.json", &admission)?;
    }
    let clock = Instant::now();
    let (initial_parent, initial_field, initial_receipt) = joint_checkpoint(a, 0, &l, &weights)?;
    let mut checkpoint_seconds = clock.elapsed().as_secs_f64();
    if initial_parent.binding != parent.binding
        || initial_parent.generate != parent.generate
        || initial_parent.bridge != parent.bridge
    {
        return Err(bad("joint zero-update parent replay differs"));
    }
    let all_indices = (0..eps.len()).collect::<Vec<_>>();
    let admission_indices = if reached.is_some() {
        all_indices.as_slice()
    } else {
        &schedule[..BATCH]
    };
    let parity = joint_native_parity(
        &l,
        &weights,
        &initial_parent,
        &initial_field,
        &eps,
        admission_indices,
        d,
        reached.as_ref(),
    )?;

    write(a, "zero-update-admission.json", &parity)?;
    let clock = Instant::now();
    let initial = continuation_evaluate(
        a,
        "development-0000",
        &initial_parent,
        &initial_field,
        &eps,
        start,
    )?;
    let initial_metrics = metrics(a, &initial, &eps)?;
    write(a, "metrics-0000.json", &initial_metrics)?;
    if let Some(plan) = &reference {
        let baseline = reference_outcomes(
            a,
            plan,
            &initial,
            &initial,
            &eps,
            if reached.is_some() {
                &[]
            } else {
                &schedule[..BATCH]
            },
        )?;
        write(a, "reference-initial-validation.json", &baseline)?;
    }
    if let Some(plan) = &reached {
        let measured = frontier::outcomes(
            a,
            plan,
            &initial,
            &initial,
            !a.constrained_emission_learning,
        )?;
        write(a, "frontier-initial-validation.json", &measured)?;
    }
    let mut evaluation_seconds = clock.elapsed().as_secs_f64();
    if reached.is_some() {
        evaluation_seconds += parity["elapsed_seconds"]
            .as_f64()
            .ok_or_else(|| bad("initial frontier parity timing absent"))?;
    }
    if initial["complete"] != 8 {
        return Err(bad("joint initial parent complete-answer replay differs"));
    }
    let coefficients = l
        .generate
        .parameters()
        .into_iter()
        .filter(|(name, _)| name != "generate.prototype_choices")
        .collect::<BTreeMap<_, _>>();
    let context = l.source.context_state_parameters();
    let potential = l.source.potential_parameters();
    let u = weights.parameters();
    let mut optimizers = if a.native_code_proposals {
        None
    } else {
        Some((
            optimizer(&coefficients, rates.generate_learning_rate)?,
            optimizer(&context, rates.context_learning_rate)?,
            optimizer(&potential, rates.potential_learning_rate)?,
            optimizer(&u, rates.continuation_learning_rate)?,
        ))
    };
    let mut updates = Vec::new();
    let fit_start = Instant::now();
    let mut fit_checkpoint_seconds = 0.;
    for update in 0..a.updates {
        deadline(a, start)?;
        disk_floor(a)?;
        let indices = if reached.is_some() {
            (0..eps.len()).collect::<Vec<_>>()
        } else {
            (0..BATCH)
                .map(|i| schedule[(update * BATCH + i) % schedule.len()])
                .collect::<Vec<_>>()
        };
        let clock = Instant::now();
        let master_snapshot = if reference.is_some() {
            Some(identities(&params)?)
        } else {
            None
        };
        let (mut grads, mut receipt) = if let Some((task, _)) = &reached_components {
            batch_live_weighted(
                a,
                &l,
                &eps,
                &indices,
                d,
                start,
                None,
                Some(&weights),
                Some(task),
            )?
        } else {
            batch_live(a, &l, &eps, &indices, d, start, None, Some(&weights))?
        };
        if reached.is_some() {
            receipt["reference_weighting"] = json!("one actual-prefix frontier per noncomplete episode, equal episode mass; no B8 sampling");
        }
        if let Some(canonical_plan) = &reference {
            let plan = reached_components
                .as_ref()
                .map(|(_, success)| success)
                .unwrap_or(canonical_plan);
            let indices = (0..eps.len()).collect::<Vec<_>>();
            let (reference_grads, reference_receipt) = batch_live_weighted(
                a,
                &l,
                &eps,
                &indices,
                d,
                start,
                None,
                Some(&weights),
                Some(plan),
            )?;
            replay_require(
                Some(identities(&params)?) == master_snapshot,
                "parameters changed between task/reference graphs",
            )?;
            for key in [
                "current_source_binding",
                "current_generate_sha256",
                "current_continuation_sha256",
                "current_categorical_sha256",
                "current_categorical_master_sha256",
            ] {
                replay_require(
                    receipt[key] == reference_receipt[key],
                    "task/reference native snapshots differ",
                )?;
            }
            let lambda = a
                .reference_replay
                .as_ref()
                .ok_or_else(|| bad("replay settings absent"))?
                .lambda;
            let baseline = if reached.is_some() {
                read(&a.out.join("frontier-initial-validation.json"))?
            } else {
                read(&a.out.join("reference-initial-validation.json"))?
            };
            let task_key = if reached.is_some() {
                "frontier_losses_before_after"
            } else {
                "current_batch_losses_before_after"
            };
            let reference_key = if reached.is_some() {
                "success_losses_before_after"
            } else {
                "reference_losses_before_after"
            };
            for (observed, expected) in [
                (&receipt["weighted_native_loss"], &baseline[task_key][0]),
                (
                    &reference_receipt["weighted_native_loss"],
                    &baseline[reference_key][0],
                ),
            ] {
                let observed = observed
                    .as_f64()
                    .ok_or_else(|| bad("live objective absent"))?;
                let expected = expected
                    .as_f64()
                    .ok_or_else(|| bad("baseline objective absent"))?;
                replay_require((observed-expected).abs()<=1e-10*(1.+expected.abs()),
                    "live task/reference objective differs from independently evaluated native baseline")?;
            }
            let combined = combine_reference_gradients(&grads, &reference_grads, lambda)?;
            let categorical_parameters = l
                .categorical
                .as_ref()
                .map(|c| c.parameters())
                .unwrap_or_default();
            let mut groups = vec![
                ("generate", &coefficients),
                ("context", &context),
                ("potential", &potential),
                ("continuation", &u),
            ];
            if l.categorical.is_some() {
                groups.push(("categorical_action", &categorical_parameters));
            }
            receipt["reference_replay"] = json!({"lambda":lambda,"reference_before_update":reference_receipt,
                "objective":"L_current_B8 + lambda*L_reference; no half-average or overlap deduplication",
                "combined_native_loss":receipt["weighted_native_loss"].as_f64().ok_or_else(||bad("task loss absent"))?+lambda*reference_receipt["weighted_native_loss"].as_f64().ok_or_else(||bad("reference loss absent"))?,
                "task_family_gradient_norms":reference_gradient_norms(&grads,&groups,d)?,
                "reference_family_gradient_norms":reference_gradient_norms(&reference_grads,&groups,d)?,
                "combined_family_gradient_norms":reference_gradient_norms(&combined,&groups,d)?,
                "same_master_snapshot":true,"same_native_snapshot":true,"clip_and_adam_policy":"sum first; one shared clip, one Adam step per active family"});
            if reached.is_some() {
                receipt["reference_replay"]["objective"] = json!(frontier::OBJECTIVE);
                receipt["reference_replay"]["reference_before_update"]["reference_weighting"] = json!("full accepted trajectories, equal successful episode/nonempty phase/position mass");
            }
            grads = combined;
        }
        if grads.contains_key("generate.prototype_choices") {
            return Err(bad("joint prototype entered active clip"));
        }
        let (denominator, norm) = clip_denominator(&grads, d)?;
        if a.native_code_proposals {
            receipt["reference_replay"]["clip_and_adam_policy"] =
                json!("none: gradients rank a frozen native-code candidate bank only");
            if a.categorical_action_only {
                receipt["reference_replay"]["clip_and_adam_policy"] = json!("none: one original-parent action gradient ranks at most one shared-key replacement; no Context update or conditional gradient recomputation");
                receipt["gradient_role"] = json!("categorical action contrasts choose the only permitted edit; other family gradients are diagnostic and never applied");
            }
            if a.constrained_emission_learning {
                receipt["reference_replay"]["clip_and_adam_policy"] = json!("none: one fresh retained-Context gradient ranks a single coordinated Generate unary/pair construction; no candidate bank, Adam or clipping");
                receipt["gradient_role"] = json!("shared Generate unary/pair fresh coefficient credit drives one once-only pass; all other master gradients remain diagnostic and are never applied");
            }
            if a.constrained_context_learning {
                receipt["reference_replay"]["clip_and_adam_policy"] = json!("none: one complete gradient drives a single coordinated constrained Context pass; no native candidate bank or Adam");
                receipt["gradient_role"] = json!("all existing credit channels aggregated; six shared Context basis families construct one native-constrained candidate; other masters frozen");
            }
            receipt["native_code_proposals"] = if a.constrained_emission_learning {
                constrained_emission::run(
                    a,
                    &l,
                    &weights,
                    &params,
                    &grads,
                    &initial_parent,
                    &initial_field,
                    &eps,
                    reference
                        .as_ref()
                        .ok_or_else(|| bad("emission reference absent"))?,
                    start,
                )?
            } else if a.constrained_context_learning {
                constrained_context::run(
                    a,
                    &l,
                    &weights,
                    &params,
                    &grads,
                    &initial_parent,
                    &initial_field,
                    &eps,
                    reference
                        .as_ref()
                        .ok_or_else(|| bad("constrained Context reference absent"))?,
                    start,
                )?
            } else if a.categorical_action_learning {
                categorical_proposals::run(
                    a,
                    &l,
                    &weights,
                    &params,
                    &grads,
                    &initial_parent,
                    &initial_field,
                    &eps,
                    &indices,
                    reference
                        .as_ref()
                        .ok_or_else(|| bad("categorical proposal reference absent"))?,
                    start,
                    d,
                )?
            } else {
                native_proposals::run(
                    a,
                    &l,
                    &weights,
                    &params,
                    &grads,
                    &initial_parent,
                    &initial_field,
                    &eps,
                    &indices,
                    reference
                        .as_ref()
                        .ok_or_else(|| bad("proposal reference absent"))?,
                    start,
                )?
            };
        } else {
            let (go, co, po, uo) = optimizers
                .as_mut()
                .ok_or_else(|| bad("joint optimizers absent"))?;
            apply(go, &coefficients, &grads, &denominator)?;
            apply(co, &context, &grads, &denominator)?;
            apply(po, &potential, &grads, &denominator)?;
            apply(uo, &u, &grads, &denominator)?;
            l.generate.project_shadow_range()?;
            for var in context.values() {
                var.set(&var.as_tensor().clamp(-1.75, 1.75)?)?;
            }
            l.source.project_potential_range()?;
            weights.project_shadow_range()?;
        }
        if identities(&prototype)? != frozen_prototype
            || identities(&l.original_bridge.parameters())? != frozen_bridge
            || identities(&l.marker.parameters())? != frozen_marker
        {
            return Err(bad("joint frozen prototype/bridge mutated"));
        }
        d.synchronize()?;
        updates.push(json!({"step":update+1,"before_update":receipt,
            "active_gradient_names":grads.keys().collect::<Vec<_>>(),"global_active_gradient_norm":norm,
            "elapsed_update_seconds":clock.elapsed().as_secs_f64()}));
        write(a, "updates.json", &json!(updates))?;
        if (update + 1) % 32 == 0 || update + 1 == a.updates {
            let clock = Instant::now();
            joint_checkpoint(a, update + 1, &l, &weights)?;
            fit_checkpoint_seconds += clock.elapsed().as_secs_f64();
        }
    }
    let mut fit_seconds = fit_start.elapsed().as_secs_f64() - fit_checkpoint_seconds;
    checkpoint_seconds += fit_checkpoint_seconds;
    if a.native_code_proposals {
        let proposal = read(&a.out.join("native-code-proposals.json"))?;
        let candidate_checkpoints = proposal["candidate_checkpoint_seconds"]
            .as_f64()
            .ok_or_else(|| bad("proposal checkpoint timing absent"))?;
        let native_objectives = proposal["native_objective_seconds"]
            .as_f64()
            .ok_or_else(|| bad("proposal native timing absent"))?;
        fit_seconds -= candidate_checkpoints + native_objectives;
        checkpoint_seconds += candidate_checkpoints;
        evaluation_seconds += native_objectives;
    }
    let (final_evaluation, final_metrics, final_receipt, final_parity) = if a.updates == 0 {
        (
            initial.clone(),
            initial_metrics.clone(),
            initial_receipt.clone(),
            parity,
        )
    } else {
        let root = a.out.join(format!("checkpoint-{:04}", a.updates));
        let current = ContinuationParent::from_checkpoint(&root)?;
        let bytes = fs::read(root.join("continuation-field.bin"))?;
        if current.receipt["continuation_sha256"] != sha256_bytes(&bytes) {
            return Err(bad("joint final U receipt differs"));
        }
        let field = NativeContinuationField::from_bytes(
            &bytes,
            &current.binding,
            current.generator()?.generate_model(),
        )?;
        if a.categorical_action_only {
            replay_require(
                current.binding == initial_parent.binding
                    && current.generate == initial_parent.generate
                    && bytes == initial_field.to_bytes()?,
                "parent action-only final Source/Generate/U differs from original parent",
            )?;
        }
        if a.native_code_proposals {
            let objective_clock = Instant::now();
            native_proposals::verify_final(
                a,
                &current,
                &field,
                &eps,
                reference
                    .as_ref()
                    .ok_or_else(|| bad("proposal reference absent"))?,
                start,
            )?;
            evaluation_seconds += objective_clock.elapsed().as_secs_f64();
        }
        let parity = joint_native_parity(
            &l,
            &weights,
            &current,
            &field,
            &eps,
            admission_indices,
            d,
            reached.as_ref(),
        )?;
        write(a, "final-native-parity.json", &parity)?;
        if reached.is_some() {
            evaluation_seconds += parity["elapsed_seconds"]
                .as_f64()
                .ok_or_else(|| bad("final frontier parity timing absent"))?;
        }
        let clock = Instant::now();
        if reference.is_some() {
            // Legacy uses its retained control list; the reached mode derives successes mechanically.
            let success_indices = reached
                .as_ref()
                .map(frontier::successful_indices)
                .unwrap_or_else(|| CONTROL_INDICES.to_vec());
            let selected = success_indices.iter().map(|&i| &eps[i]).collect::<Vec<_>>();
            let early = continuation_evaluate_rows(
                a,
                "pilot-original8",
                &current,
                &field,
                &selected,
                start,
            )?;
            write(
                a,
                "pilot-original8-receipt.json",
                &json!({"checkpoint_step":a.updates,
                "checkpoint_receipt_sha256":sha256_file(&root.join("receipt.json"))?,"indices":success_indices,
                "evaluation":early,"scope":"independently reloaded actual-artifact own-prefix original8, before full512; not a training gate"}),
            )?;
        }
        let result = continuation_evaluate(
            a,
            &format!("development-{:04}", a.updates),
            &current,
            &field,
            &eps,
            start,
        )?;
        let metrics = metrics(a, &result, &eps)?;
        write(a, &format!("metrics-{:04}.json", a.updates), &metrics)?;
        evaluation_seconds += clock.elapsed().as_secs_f64();
        (result, metrics, current.receipt, parity)
    };
    let rowwise = joint_row_comparison(&initial, &final_evaluation)?;
    write(a, "complete-row-comparison.json", &rowwise)?;
    let mut report = json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"joint_continuation",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"updates":a.updates,"batch":BATCH,"rates":rates,
        "credit":"raw_identity","read_state_pullback":"categorical","order_seed":a.seed,
        "initial_receipt":initial_receipt,"final_receipt":final_receipt,"final_native_parity":final_parity,
        "initial_evaluation":initial,"final_evaluation":final_evaluation,"initial_metrics":initial_metrics,"final_metrics":final_metrics,
        "rowwise":rowwise,"updates_receipt":updates,"fit_seconds":fit_seconds,"checkpoint_seconds":checkpoint_seconds,
        "native_evaluation_seconds":evaluation_seconds,"elapsed_seconds":start.elapsed().as_secs_f64(),
        "scope":"exposed512 joint Context/Potential/Generate-coefficient/U construction learning; frozen prototypes and categorical map; native independent reload and own-feedback outputs; no transfer/chat/energy qualification"});
    if a.native_code_proposals {
        report["native_code_proposals"] = read(&a.out.join("native-code-proposals.json"))?;
        report["optimizer_updates"] = json!(0);
    }
    if a.constrained_emission_learning {
        report["constrained_emission_learning"] = json!(true);
        report["immediate_input"] = constrained_emission::input_identity(a)?;
        report["scope"] = json!("exposed512 sequential shared Generate unary/pair learning at pinned retained Context endpoint; exact protected pooled outputs; upstream/bias/prototype/U numeric masters frozen; independent native reload and own-feedback outputs; no transfer/chat/energy qualification");
    }
    if a.constrained_context_learning {
        report["constrained_context_learning"] = json!(true);
        report["constrained_context_policy"] = constrained_context::policy();
        report["scope"] = json!("exposed512 one coordinated shared Context basis update constrained by exact original-success native decisions; Potential/Generate/U/prototypes/categorical map fixed; independently reloaded native own-prefix outputs; greedy construction, no transfer/chat/energy qualification");
    }
    if a.categorical_action_learning {
        report["categorical_action_learning"] =
            categorical_proposals::policy_for(a.categorical_action_only);
        report["scope"] = json!("exposed512 conditional Context plus shared geometric action map construction learning; original quarter provenance and Generate prototypes retained; independently reloaded native own-prefix outputs; no transfer/chat/energy qualification");
    }
    if a.categorical_action_only {
        report["categorical_action_only"] = json!(true);
        report["scope"] = json!("exposed512 original-parent shared categorical action-only attribution; Context/Potential/Generate/U/prototypes fixed; one original-parent gradient and at most one native action-row candidate; independently reloaded native own-prefix outputs; not joint Context adaptation or transfer/chat/energy qualification");
    }
    if let Some(plan) = &reference {
        let outcomes = reference_outcomes(
            a,
            plan,
            &initial,
            &final_evaluation,
            &eps,
            if reached.is_some() {
                &[]
            } else {
                &schedule[..BATCH]
            },
        )?;
        write(a, "reference-outcomes.json", &outcomes)?;
        report["reference_replay"] = reference_binding(a)?;
        report["reference_outcomes"] = outcomes;
        if let Some(plan) = &reached {
            let reached_outcomes = frontier::outcomes(a, plan, &initial, &final_evaluation, false)?;
            write(a, "frontier-outcomes.json", &reached_outcomes)?;
            report["reached_frontier_objective"] = json!(frontier::OBJECTIVE);
            report["frontier_outcomes"] = reached_outcomes;
            report["batch"] = Value::Null;
            report["order_policy"] =
                json!("all512 episodes, both frozen sparse components; no B8 sampling");
        }
        report["early_original8_receipt_sha256"] =
            json!(sha256_file(&a.out.join("pilot-original8-receipt.json"))?);
    }
    Ok(report)
}

fn run_continuation(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    cross_state_completion::settings(a)?;
    let cross_state = a.mode == Mode::CrossStateContinuation;
    let mode_name = if cross_state {
        "cross_state_continuation"
    } else {
        "continuation_only"
    };
    let c = continuation_settings(a)?.ok_or_else(|| bad("continuation settings absent"))?;
    let p = ContinuationParent::load(a)?;
    let public = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let train = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &p.integer,
        &p.tokenizer,
        &legal,
        512,
    )?;
    let dev = load_panel(
        &a.development_inputs,
        &a.development_labels,
        &p.integer,
        &p.tokenizer,
        &legal,
        512,
    )?;
    if train.len() != 512 || dev.len() != 512 {
        return Err(bad("complete512 continuation panel required"));
    }
    pairs(&train)?;
    pairs(&dev)?;
    let native = p.generator()?;
    if native.generate_model().lanes() != 8 || native.generate_model().vocab_size() != 4096 {
        return Err(bad("continuation parent8-lane/full4096 mismatch"));
    }
    let resume = cross_state_completion::load_resume(a, &p)?;
    let weights = if let Some(resume) = &resume {
        resume.restore_weights(&p, d)?
    } else if cross_state {
        ContinuationLearningWeights::zeroed_cross_state(
            p.integer.binding(),
            native.source_binding(),
            8,
            d,
        )?
    } else {
        ContinuationLearningWeights::zeroed_shared_action(
            p.integer.binding(),
            native.source_binding(),
            8,
            d,
        )?
    };
    // Cache only immutable upstream scores. Resumed trainable coefficients are
    // added exactly once by forward_coefficients_only, never baked into its base.
    let zero = if cross_state {
        NativeContinuationField::compile_cross_state(
            native.source_binding(),
            native.generate_model(),
            &vec![0; 8 << 13],
        )?
    } else {
        weights.export_native(native.source_binding(), native.generate_model())?
    };
    let zero_bytes = zero.to_bytes()?;
    let zero_sha = sha256_bytes(&zero_bytes);
    let native = native.with_continuation_field(BoundNativeBytes {
        bytes: &zero_bytes,
        sha256: &zero_sha,
    })?;
    let owner = PreparedFixedContinuationBank::new(native, &weights, &p.exp)?;
    let params = weights.parameters();
    let expected_coefficients = if cross_state { 115200 } else { 960 };
    if params.len() != 1
        || params.values().map(|v| v.elem_count()).sum::<usize>() != expected_coefficients
        || weights.shared_coefficients() != expected_coefficients
    {
        return Err(bad("continuation field parameter admission"));
    }
    let schedule = order(a.seed, train.len());
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,"batch":BATCH,"updates":a.updates,
        "policy":"existing full512 SplitMix64/Fisher-Yates cyclic batches; previous supervised tokens only; all answer positions including EOS"}),
    )?;
    let projected_tensor_bytes = train.iter().try_fold(0u64, |sum, e| -> Result<u64> {
        let copy = e
            .views
            .iter()
            .flatten()
            .map(|v| v.emitted_token_ids().len())
            .sum::<usize>() as u64;
        sum.checked_add(
            (4096 * 8 + copy * 4)
                .checked_mul(e.target.len() as u64)
                .ok_or_else(|| bad("cache projection overflow"))?,
        )
        .ok_or_else(|| bad("cache projection overflow"))
    })?;
    if projected_tensor_bytes > c.maximum_cache_tensor_bytes {
        return Err(bad("continuation device cache tensor cap"));
    }
    write(
        a,
        "admission.json",
        &json!({"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"mode":mode_name,
        "parent":p.receipt,"parent_report_sha256":CONTINUATION_PARENT_REPORT_SHA,"parent_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,
        "training_input_sha256":INPUT_SHA,"training_labels_sha256":LABEL_SHA,"construction_panel_overlap":"train=open-development512; no new held-out claim",
        "active_parameter_names":params.keys().collect::<Vec<_>>(),"shared_coefficients":expected_coefficients,"learning_rate":c.learning_rate,
        "continuation_policy":zero.metadata().policy,"continuation_score_shift":zero.metadata().score_shift,
        "continuation_action_support":"same token energy on Generate and every physical Copy before the sole common clip",
        "phase_policy":loss_weight_policy(true,LossScope::All),"loss_scope":"all","credit":a.credit.name(),"fresh_adam":true,
        "cross_state_resume":resume.as_ref().map(|r|&r.provenance),
        "local_updates":a.updates,"prior_updates":if resume.is_some(){64}else{0},
        "final_lineage_step":a.updates+if resume.is_some(){64}else{0},
        "initial_active_masters":identities(&params)?,
        "teacher_forcing":"native cache receives complete input packet and target[..t] only; current/future target used after common pool",
        "local_carrier":"ContextQ4 from identity over query || prior supervised prefix; independent of facts",
        "projected_device_cache_tensor_bytes":projected_tensor_bytes,"maximum_cache_tensor_bytes":c.maximum_cache_tensor_bytes,
        "cache_cap_scope":"I64 base Generate and F32 Copy tensors; host packet/provenance, model and scratch additionally charged",
        "common_pool_backend":"cpu-authenticated-native-alias-reducer; U gathers/credit/backward on CUDA; full score download and anchor/loss staging retained",
        "legacy_loader_fields":"categorical and parent_config are unused in this mode; native sidecars come only from exact sealed selected parent",
        "CUDA_VISIBLE_DEVICES":std::env::var("CUDA_VISIBLE_DEVICES").ok()}),
    )?;
    let cache_start = Instant::now();
    let mut cache = Vec::<Vec<FixedContinuationPosition>>::new();
    let mut cache_rows = Vec::new();
    let mut positions = 0usize;
    let mut copy_occurrences = 0usize;
    let mut encoding_reads = 0u64;
    for e in &train {
        deadline(a, start)?;
        disk_floor(a)?;
        let bank = owner.admit_bank(continuation_snapshot(&e.packet)?)?;
        let mut row = Vec::new();
        let mut provenance = Vec::new();
        for t in 0..e.target.len() {
            deadline(a, start)?;
            let position = owner.prepare_position(bank.clone(), &e.target[..t])?;
            let union = position
                .copy_token_ids()
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            let expected = e
                .views
                .iter()
                .flatten()
                .flat_map(|v| v.emitted_token_ids().iter().copied())
                .collect::<BTreeSet<_>>();
            if union != expected {
                return Err(bad("continuation fixed complete Copy union differs"));
            }
            positions += 1;
            copy_occurrences += position.copy_token_ids().len();
            encoding_reads = encoding_reads
                .checked_add(position.encoding_coefficient_reads())
                .ok_or_else(|| bad("cache read-count overflow"))?;
            provenance.push(json!({"prior_prefix_ids":position.actual_prefix_ids(),"local_state_codes":position.local_state().iter().map(|v|v.index()).collect::<Vec<_>>(),
                "post_state_codes":position.post_state().iter().map(|v|v.index()).collect::<Vec<_>>(),"copy_occurrences":position.copy_token_ids().len(),
                "native_generate_counts":position.native_generate_counts(),"encoding_coefficient_reads":position.encoding_coefficient_reads(),
                "factual_bank_trace_sha256":position.factual_bank_trace_sha256(),"factual_bank_binding_sha256":position.factual_bank_binding_sha256()}));
            row.push(position);
        }
        cache_rows.push(json!({"id":e.packet.id,"packet_sha256":sha256_bytes(&serde_json::to_vec(&e.packet)?),"positions":provenance}));
        cache.push(row);
    }
    if cross_state && positions != 6664 {
        return Err(bad(
            "cross-state frozen512 target-position coverage differs",
        ));
    }
    d.synchronize()?;
    let cache_seconds = cache_start.elapsed().as_secs_f64();
    write(
        a,
        "fixed-position-cache.json",
        &json!({"rows":cache_rows,"cases":train.len(),"positions":positions,
        "device_cache_tensor_bytes":projected_tensor_bytes,"copy_occurrences_across_positions":copy_occurrences,
        "local_encoding_coefficient_reads":encoding_reads,"preparation_seconds":cache_seconds,
        "preparation":"actual production native step, full bank and prior teacher prefix; upstream CPU integer work and uploads",
        "labels":"prefix/schedule are supervised; no current/future target or selected reference passed to cache API"}),
    )?;
    let mut checkpoint_seconds = 0.;
    let mut evaluation_seconds = 0.;
    let clock = Instant::now();
    let (initial_field, initial_receipt) =
        continuation_checkpoint(a, 0, &p, &weights, resume.as_ref())?;
    checkpoint_seconds += clock.elapsed().as_secs_f64();
    let clock = Instant::now();
    let initial = continuation_evaluate(a, "development-0000", &p, &initial_field, &dev, start)?;
    let initial_metrics = metrics(a, &initial, &dev)?;
    if cross_state {
        cross_state_completion::baseline(&initial, resume.as_ref().map(|r| &r.prior_final))?;
    }
    write(a, "metrics-0000.json", &initial_metrics)?;
    evaluation_seconds += clock.elapsed().as_secs_f64();
    let mut opt = optimizer(&params, c.learning_rate)?;
    let mut updates = Vec::new();
    let fit_start = Instant::now();
    let initial_checkpoint_seconds = checkpoint_seconds;
    let mut final_receipt = initial_receipt.clone();
    let mut target_position_draws = 0usize;
    for update in 0..a.updates {
        deadline(a, start)?;
        disk_floor(a)?;
        let prepared = owner.prepare_field(&weights)?;
        let indices = cross_state_completion::schedule_indices(&schedule, update);
        let mut sums = BTreeMap::<String, Tensor>::new();
        let mut loss_sum = 0.;
        let mut count = 0usize;
        let mut downloads = 0usize;
        let mut uploads = 0usize;
        let mut copy_index_uploads = 0usize;
        let mut loss_staging = 0usize;
        let mut field_costs = None;
        for &index in &indices {
            let e = &train[index];
            let fixed = &cache[index];
            let union = fixed
                .first()
                .ok_or_else(|| bad("empty continuation cache row"))?
                .copy_token_ids()
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            let mut plan = None;
            for (t, (&target, position)) in e.target.iter().zip(fixed).enumerate() {
                deadline(a, start)?;
                let out = position.forward_coefficients_only(&weights, &prepared)?;
                // The whole target-free pool is constructed before this label.
                if plan.is_none() {
                    plan = Some(episode_loss_weights(
                        &e.target,
                        &union,
                        indices.len(),
                        true,
                        LossScope::All,
                    )?);
                }
                let weight = plan
                    .as_ref()
                    .ok_or_else(|| bad("continuation loss phases absent"))?
                    .weights[t];
                let loss = (out.loss_with_credit(target, a.credit.policy())? * weight)?;
                if field_costs.is_none() {
                    field_costs = Some(serde_json::to_value(&out.continuation.costs)?);
                }
                let value = loss.to_scalar::<f32>()? as f64;
                if !value.is_finite() {
                    return Err(bad("nonfinite continuation loss"));
                }
                loss_sum += value;
                count += 1;
                downloads += out.common_pool_score_download_bytes;
                uploads += out.common_pool_anchor_upload_bytes;
                copy_index_uploads += out.common_pool_copy_index_upload_bytes;
                // Host alias loss uploads one F32 hard-score vector, one F32
                // target mask and one scalar probability per position.
                loss_staging += if d.is_cuda() {
                    8 * out.actions.actions.len() + 4
                } else {
                    0
                };
                let grads = loss.backward()?;
                for (name, var) in &params {
                    if let Some(g) = grads.get(var.as_tensor()) {
                        let next = if let Some(old) = sums.remove(name) {
                            (&old + g)?
                        } else {
                            g.clone()
                        };
                        sums.insert(name.clone(), next.detach());
                    }
                }
            }
        }
        let (denominator, norm) = clip_denominator(&sums, d)?;
        if cross_state && (sums.len() != params.len() || (update == 0 && norm <= 0.)) {
            return Err(bad("cross-state active gradient admission failed"));
        }
        apply(&mut opt, &params, &sums, &denominator)?;
        weights.project_shadow_range()?;
        d.synchronize()?;
        target_position_draws += count;
        updates.push(json!({"step":update+1,"lineage_step":update+1+if resume.is_some(){64}else{0},"indices":indices,"answer_positions_including_eos":count,"phase_balanced_loss":loss_sum,
            "global_active_gradient_norm":norm,"active_gradient_names":sums.keys().collect::<Vec<_>>(),
            "native_field_master_download_bytes":prepared.downloaded_master_bytes,
            "field_snapshot_and_per_position_costs":field_costs,
            "common_pool_score_download_bytes":downloads,"common_pool_anchor_upload_bytes":uploads,
            "common_pool_copy_index_upload_bytes":copy_index_uploads,
            "host_alias_loss_explicit_upload_bytes":loss_staging,
            "scalar_transfers":"per-position finite-score validation and reported loss; clip norm status scalars; CUDA synchronization",
            "common_pool_backend":"cpu-authenticated-native-alias-reducer","credit_scope":if cross_state {"115200 shared cross-state Q4 coefficient STE only; frozen factual/local states and prototypes"} else {"960 shared U coefficient STE only; frozen native states/prototypes"}}));
        write(a, "updates.json", &json!(updates))?;
        if (update + 1) % 32 == 0 {
            let clock = Instant::now();
            final_receipt =
                continuation_checkpoint(a, update + 1, &p, &weights, resume.as_ref())?.1;
            checkpoint_seconds += clock.elapsed().as_secs_f64();
        }
    }
    let fit_seconds =
        fit_start.elapsed().as_secs_f64() - (checkpoint_seconds - initial_checkpoint_seconds);
    let root = a.out.join(format!("checkpoint-{:04}", a.updates));
    let final_field_bytes = fs::read(root.join("continuation-field.bin"))?;
    if read(&root.join("receipt.json"))? != final_receipt
        || final_receipt["continuation_sha256"].as_str()
            != Some(sha256_bytes(&final_field_bytes).as_str())
    {
        return Err(bad("final continuation sidecar/receipt identity differs"));
    }
    let final_field = NativeContinuationField::from_bytes(
        &final_field_bytes,
        &p.binding,
        p.generator()?.generate_model(),
    )?;
    let clock = Instant::now();
    if cross_state {
        let early_indices = if resume.is_some() {
            initial["rows"]
                .as_array()
                .ok_or_else(|| bad("baseline rows absent"))?
                .iter()
                .enumerate()
                .filter(|(_, r)| r["complete"] == true)
                .map(|(i, _)| i)
                .collect::<Vec<_>>()
        } else {
            cross_state_completion::ORIGINAL8.to_vec()
        };
        let early = early_indices.iter().map(|&i| &dev[i]).collect::<Vec<_>>();
        continuation_evaluate_rows_impl(
            a,
            if resume.is_some() {
                "endpoint-prior22"
            } else {
                "endpoint-original8"
            },
            &p,
            &final_field,
            &early,
            start,
            false,
        )?;
    }
    let final_eval = continuation_evaluate(
        a,
        &format!("development-{:04}", a.updates),
        &p,
        &final_field,
        &dev,
        start,
    )?;
    let final_metrics = metrics(a, &final_eval, &dev)?;
    write(a, &format!("metrics-{:04}", a.updates), &final_metrics)?;
    evaluation_seconds += clock.elapsed().as_secs_f64();
    let cross_outcome = if cross_state {
        let outcome = cross_state_completion::outcomes(
            &initial,
            &final_eval,
            resume.as_ref().map(|r| &r.prior_final),
        )?;
        write(a, "cross-state-outcomes.json", &outcome)?;
        Some(outcome)
    } else {
        None
    };
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":mode_name,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"device":"cuda:0","updates":a.updates,"batch":BATCH,
        "prior_updates":if resume.is_some(){64}else{0},"lineage_step":a.updates+if resume.is_some(){64}else{0},
        "cross_state_resume":resume.as_ref().map(|r|&r.provenance),"fresh_adam":true,
        "training_row_draws":a.updates*BATCH,
        "target_position_draws":target_position_draws,
        "loss_scope":"all","phase_policy":loss_weight_policy(true,LossScope::All),"credit":a.credit.name(),"order_seed":a.seed,
        "initial_receipt":initial_receipt,"final_receipt":final_receipt,"initial_metrics":initial_metrics,"final_metrics":final_metrics,
        "initial":initial,"final":final_eval,"cross_state_outcome":cross_outcome,
        "final_active_masters":identities(&params)?,"shared_coefficients":expected_coefficients,"cache_positions":positions,
        "cache_preparation_seconds":cache_seconds,"fit_loop_seconds_excluding_checkpoints":fit_seconds,
        "checkpoint_seconds":checkpoint_seconds,"evaluation_seconds":evaluation_seconds,"elapsed_seconds":start.elapsed().as_secs_f64(),
        "control":if resume.is_some() {"checkpoint0000 is exact saved22 native field plus restored fractional masters; independently matched all512 outputs; cache separately uses zero field; no carrier ablation performed"} else if cross_state {"checkpoint0000 is exact accepted parent with zero joint factual/local field; no carrier ablation performed"} else {"checkpoint0000 is the same native parent with a null U field; constant-carrier control remains a prospective matched full-output evaluation"},
        "common_pool_backend":"cpu-authenticated-native-alias-reducer; full score/anchor/loss transfers retained; not fully resident CUDA alias reduction",
        "scope":if resume.is_some() {"saved22 cross-state parameter continuation with fresh Adam moments; frozen48/64 upstream;115200 Q4 coefficients, four exposed512 passes; fixed endpoint local256/lineage320; developmental KEEP netcomplete>22; no held-out transfer/chat/geometry/energy qualification"} else if cross_state {"frozen48/64 parent; only115200 joint factual/local signed-H4 Q4 coefficients learned on exposed512 all-answer positions; independently loaded own-prefix outputs; developmental KEEP netcomplete>8; no held-out transfer/chat/geometry/energy qualification"} else {"frozen48/64 parent; only960 continuation coefficients learned on exposed512 all-answer positions; independently loaded own-feedback outputs; no held-out transfer/chat/geometry/energy qualification"}}),
    )
}

fn run(a: &Args, start: Instant) -> Result<Value> {
    if a.coupled_episode_learning
        .as_ref()
        .is_some_and(|c| c.no_new_gradients())
    {
        return coupled_episode_learning::run(a, start, &Device::Cpu);
    }
    if a.generate_episode_completion.is_some() {
        return generate_episode_learning::run_completion(a, start);
    }
    if a.prefix_artifact_check.is_some() {
        return prefix_fragment_learning::run_artifact_check(a, start);
    }
    if a.prefix_fragment_learning
        .as_ref()
        .is_some_and(|c| c.trajectory.is_some())
    {
        return prefix_fragment_learning::run(a, start, &Device::Cpu);
    }
    if a.context_path_credit.is_some() {
        return context_path_credit::run(a, start);
    }
    let d = cuda()?;
    if reply_completion::is_mode(a.mode) {
        return reply_completion::run(a, start, &d);
    }
    if matches!(
        a.mode,
        Mode::ContinuationOnly | Mode::CrossStateContinuation
    ) {
        return run_continuation(a, start, &d);
    }
    if a.mode == Mode::JointContinuation {
        return run_joint_continuation(a, start, &d);
    }
    let l = load(a, &d)?;
    let public = NativeVocabularyActions::new(l.integer.binding().clone(), &l.exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let declared_panel = a.panel_inputs.is_some() || a.panel_labels.is_some();
    let (train_inputs, train_labels, dev_inputs, dev_labels) =
        match (&a.panel_inputs, &a.panel_labels) {
            (Some(inputs), Some(labels)) => (
                inputs.clone(),
                labels.clone(),
                inputs.clone(),
                labels.clone(),
            ),
            (None, None) => (
                a.training_inputs.clone(),
                a.training_labels.clone(),
                a.development_inputs.clone(),
                a.development_labels.clone(),
            ),
            _ => return Err(bad("a declared panel needs both inputs and labels")),
        };
    let panel_cap = if declared_panel { 4096 } else { 512 };
    let train = load_panel(
        &train_inputs,
        &train_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        panel_cap,
    )?;
    let dev = load_panel(
        &dev_inputs,
        &dev_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        panel_cap,
    )?;
    if declared_panel {
        if train.is_empty() || dev.is_empty() {
            return Err(bad("declared panel is empty"));
        }
    } else if train.len() != 512 || dev.len() != 512 {
        return Err(bad("full fixed panel required"));
    }
    // A declared panel is not the paired construction panel, so the swap-pair
    // audit and the retained-panel baseline evaluation do not apply to it.
    if !declared_panel {
        pairs(&train)?;
        pairs(&dev)?;
    }
    if a.mode == Mode::PredictionControl {
        return run_prediction_control(a, start, &d, &l, &train, dev);
    }
    if a.mode == Mode::EntryScorerFit {
        return run_entry_scorer_fit(a, start, &l, &dev);
    }
    let frozen_original = identities(&l.original_bridge.parameters())?;
    let frozen_marker = identities(&l.marker.parameters())?;
    let initial_masters = identities(&active(&l.source, &l.generate)?)?;
    let schedule = order(a.seed, train.len());
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,
        "policy":format!("SplitMix64/Fisher-Yates full512 once; fixed cyclic batches of8, {}updates; order only, no model reinitialization", a.updates)}),
    )?;
    write(
        a,
        "admission.json",
        &json!({"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "checkpoint_receipt_sha256":CP_RECEIPT_SHA,"parent_config_sha256":sha256_file(&a.parent_config)?,
        "training_input_sha256":INPUT_SHA,"training_labels_sha256":LABEL_SHA,
        "construction_panel_overlap":"training and open development are the identical retained512panel; not generalization",
        "original_bridge_masters":frozen_original,"marker_masters":frozen_marker,
        "categorical":l.categorical_receipt,"initial_active_masters":initial_masters,
        "active_parameter_names":active(&l.source,&l.generate)?.keys().collect::<Vec<_>>(),
        "fresh_adam":true,"rates":{"generate":0.003,"prototype":0.01,"context":0.002,"potential":0.003},
        "phase_policy":loss_weight_policy(true, a.loss_scope),"loss_scope":a.loss_scope.name(),"declared_updates":a.updates,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"CUDA_VISIBLE_DEVICES":std::env::var("CUDA_VISIBLE_DEVICES").ok()}),
    )?;
    let (initial_native, initial_generate, initial_bridge, initial_receipt) = checkpoint(a, 0, &l)?;
    if initial_native.binding().tokenizer_sha256() != l.integer.binding().tokenizer_sha256()
        || initial_generate.to_bytes()? != fs::read(a.checkpoint.join("generate.bin"))?
        || initial_receipt["parent"] != l.receipt["parent"]
    {
        return Err(bad("zero-update restored native identity differs"));
    }
    let indices = schedule[..BATCH].to_vec();
    let (_, admission) = batch(a, &l, &train, &indices, &d, start, Some(&initial_native))?;
    write(a, "zero-update-admission.json", &admission)?;
    let (initial_metrics, baseline_provenance) = if declared_panel {
        (
            json!(null),
            json!({"execution":"NOT_RUN: a declared diagnostic panel has no paired construction audit",
                "reused":false,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
        )
    } else {
        let (baseline, provenance) = if let Some(root) = &a.baseline {
            reuse_baseline(a, root, &dev, &initial_receipt)?
        } else {
            let evaluation = evaluate(
                a,
                "development-0000",
                &initial_native,
                &initial_generate,
                Some(&initial_bridge),
                &l.exp,
                &dev,
                &l.tokenizer,
                &l.cue,
                &l.prefix,
                start,
                None,
            )?;
            (
                evaluation,
                json!({"execution":"new native CPU integer evaluation on this pod",
            "reused":false,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
            )
        };
        let metrics = metrics(a, &baseline, &dev)?;
        write(a, "metrics-0000.json", &metrics)?;
        (metrics, provenance)
    };
    if a.mode == Mode::Admission {
        return Ok(
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"admission",
            "updates":0,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"order_seed":a.seed,"admission":admission,
            "initial_metrics":initial_metrics,"initial_evaluation_provenance":baseline_provenance,
            "elapsed_seconds":start.elapsed().as_secs_f64(),
            "scope":"zero-update authenticated categorical restart admission; no learning verdict"}),
        );
    }
    if a.mode == Mode::EntryCeiling {
        let ceiling = entry_ceiling_panel(
            a,
            &l,
            &initial_native,
            &initial_generate,
            &initial_bridge,
            &dev,
        )?;
        let scorer = if a.ceiling_scorer {
            Some(entry_scorer_read(
                a,
                &l,
                &initial_native,
                &initial_generate,
                &initial_bridge,
                &dev,
            )?)
        } else {
            None
        };
        return Ok(
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"entry_ceiling",
            "updates":0,"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),"order_seed":a.seed,
            "checkpoint_receipt_sha256":CP_RECEIPT_SHA,"ceiling":ceiling,"entry_scorer":scorer,"initial_metrics":initial_metrics,
            "declared_panel":declared_panel,"panel_rows":dev.len(),
            "query_conditioned_read":a.query_conditioned_read,
            "panel_inputs_sha256":sha256_file(&dev_inputs)?,"panel_labels_sha256":sha256_file(&dev_labels)?,
            "elapsed_seconds":start.elapsed().as_secs_f64(),
            "scope":"zero-update entry-position ceiling with the physical Copy channel removed; labels read only after the target-free forward; no fit and no serving change; a declared panel replaces the pinned one and its hashes are recorded here rather than asserted"}),
        );
    }
    let gp = l.generate.parameters();
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = gp
        .into_iter()
        .partition(|(name, _)| name == "generate.prototype_choices");
    let context = l.source.context_state_parameters();
    let potential = l.source.potential_parameters();
    let mut go = optimizer(&coefficients, 0.003)?;
    let mut po = optimizer(&prototype, 0.01)?;
    let mut co = optimizer(&context, 0.002)?;
    let mut vo = optimizer(&potential, 0.003)?;
    let mut updates = Vec::new();
    for update in 0..a.updates {
        disk_floor(a)?;
        deadline(a, start)?;
        let indices = (0..BATCH)
            .map(|i| schedule[(update * BATCH + i) % schedule.len()])
            .collect::<Vec<_>>();
        let (grads, receipt) = batch(a, &l, &train, &indices, &d, start, None)?;
        let (denominator, norm) = clip_denominator(&grads, &d)?;
        apply(&mut go, &coefficients, &grads, &denominator)?;
        apply(&mut po, &prototype, &grads, &denominator)?;
        apply(&mut co, &context, &grads, &denominator)?;
        apply(&mut vo, &potential, &grads, &denominator)?;
        l.generate.project_shadow_range()?;
        for var in context.values() {
            var.set(&var.as_tensor().clamp(-1.75, 1.75)?)?;
        }
        l.source.project_potential_range()?;
        if identities(&l.original_bridge.parameters())? != frozen_original
            || identities(&l.marker.parameters())? != frozen_marker
        {
            return Err(bad("frozen bridge master mutation"));
        }
        d.synchronize()?;
        updates.push(
            json!({"step":update+1,"read_state_pullback":a.read_state_pullback.name(),"before_update":receipt,"global_active_gradient_norm":norm}),
        );
        write(a, "updates.json", &json!(updates))?;
        // Recoverable exact checkpoints, not acceptance/reselection gates.
        if (update + 1) % 32 == 0 {
            checkpoint(a, update + 1, &l)?;
        }
    }
    let (native, generate, bridge, final_receipt) = {
        // Final checkpoint was already written at the declared dose; reload all
        // native paths.
        let cp = a.out.join(format!("checkpoint-{:04}", a.updates));
        let rec = read(&cp.join("receipt.json"))?;
        let binding: NativeArtifactBinding = serde_json::from_value(rec["parent"].clone())?;
        (
            IntegerRealizer::load_native(&cp.join("native"), &binding)?,
            NativeGeometricGenerate::from_bytes(
                &fs::read(cp.join("generate.bin"))?,
                l.generate.binding(),
            )?,
            NativeGeometricReadStateBridge::from_bytes(
                &fs::read(cp.join("read-state-bridge-categorical.bin"))?,
                l.generate.binding(),
            )?,
            rec,
        )
    };
    let final_eval = evaluate(
        a,
        &format!("development-{:04}", a.updates),
        &native,
        &generate,
        Some(&bridge),
        &l.exp,
        &dev,
        &l.tokenizer,
        &l.cue,
        &l.prefix,
        start,
        None,
    )?;
    let final_metrics = metrics(a, &final_eval, &dev)?;
    write(a, &format!("metrics-{:04}.json", a.updates), &final_metrics)?;
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"fit",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"device":"cuda:0","credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),
        "order_seed":a.seed,"updates":a.updates,"batch":BATCH,"token_backward_chunk":1,"loss_scope":a.loss_scope.name(),
        "initial_receipt":initial_receipt,"final_receipt":final_receipt,
        "initial_metrics":initial_metrics,"final_metrics":final_metrics,
        "initial_evaluation_provenance":baseline_provenance,
        "final_active_masters":identities(&active(&l.source,&l.generate)?)?,
        "frozen_original_masters":frozen_original,"frozen_marker_masters":frozen_marker,
        "elapsed_seconds":start.elapsed().as_secs_f64(),
        "scope":"matched fixed categorical-map checkpoint restart; context/potential/Generate learned from actual native alias objective; train=open-development512; first-source-word teacher metrics separate from own-prefix outputs; no held-out transfer, attention or chat qualification"}),
    )
}
fn main() -> Result<()> {
    let cli = std::env::args().collect::<Vec<_>>();
    if cli.len() == 3 && cli[1] == "prepare-reference-replay" {
        return prepare_reference_replay(Path::new(&cli[2]), false);
    }
    if cli.len() == 3 && cli[1] == "prepare-reached-frontier" {
        return prepare_reference_replay(Path::new(&cli[2]), true);
    }
    let (a, config_raw) = args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = (|| -> Result<Value> {
        if reply_completion::is_mode(a.mode)
            || a.context_path_credit.is_some()
            || a.prefix_artifact_check.is_some()
            || a.prefix_fragment_learning.is_some()
            || a.coupled_episode_learning.is_some()
            || a.generate_episode_learning.is_some()
            || a.generate_episode_completion.is_some()
            || a.context_cue_coadapt.is_some()
            || a.prefix_context_credit.is_some()
            || a.readout_coadaptation
                .as_ref()
                .and_then(|c| c.intermediate_candidate.as_ref())
                .is_some()
        {
            let path = cli
                .get(1)
                .ok_or_else(|| bad("external config path absent"))?;
            let raw = &config_raw;
            replay_require(
                raw.len() as u64 + size(&a.out)? < a.maximum_report_bytes,
                "external config report cap",
            )?;
            fs::write(a.out.join("config.json"), &raw)?;
            write(
                &a,
                "external-config-binding.json",
                &json!({"path":fs::canonicalize(path)?,"sha256":sha256_bytes(&raw),"bytes":raw.len(),"attempt_argv":cli}),
            )?;
        }
        run(&a, start)
    })();
    let report = match &result {
        Ok(value) => value.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"FAILED","error":e.to_string(),"credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),
            "elapsed_seconds":start.elapsed().as_secs_f64(),
            "model_verdict":"UNQUALIFIED; preserve partial checkpoints and completed rows; execution failure is not model failure"})
        }
    };
    // Failure receipts use a reserved margin even when report storage admission failed.
    fs::write(
        a.out.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn reference_row_fixture(phases: Vec<usize>, eligible: Vec<bool>) -> ReferenceRow {
        ReferenceRow {
            index: 0,
            id: "fixture".into(),
            packet_sha256: "a".repeat(64),
            saved_row_sha256: "b".repeat(64),
            targets: (0..phases.len() as u32).collect(),
            copy_ids: vec![],
            phases,
            eligible,
            weight_denominators: vec![],
            weights: vec![],
        }
    }
    #[test]
    fn reference_weights_balance_eligible_episodes_then_nonempty_phases() -> Result<()> {
        let mut rows = vec![
            reference_row_fixture(vec![0, 1, 1, 2], vec![true, true, false, true]),
            reference_row_fixture(vec![0, 1, 1, 1, 2], vec![false, true, true, true, false]),
            reference_row_fixture(vec![0, 1, 2], vec![false, false, false]),
        ];
        reference_weights(&mut rows)?;
        assert_eq!(rows[0].weight_denominators, vec![6, 6, 0, 6]);
        assert_eq!(rows[1].weight_denominators, vec![0, 6, 6, 6, 0]);
        assert_eq!(rows[2].weights, vec![0.; 3]);
        for row in &rows[..2] {
            assert!((row.weights.iter().sum::<f64>() - 0.5).abs() < 1e-14);
        }
        assert!((rows.iter().flat_map(|r| &r.weights).sum::<f64>() - 1.).abs() < 1e-14);
        // Eligibility stays frozen; weighting cannot turn an excluded parent error into a target.
        assert!(!rows[0].eligible[2]);
        assert_eq!(rows[0].weights[2], 0.);
        assert!(
            reference_weights(&mut [reference_row_fixture(vec![0, 1], vec![false, false])])
                .is_err()
        );
        assert!(
            reference_weights(&mut [reference_row_fixture(vec![0, 3], vec![true, true])]).is_err()
        );
        Ok(())
    }
    #[test]
    fn sparse_reference_preserves_ineligible_tokens_in_teacher_prefix() -> Result<()> {
        let targets = [11, 22, 33, 44];
        let eligible = [false, true, false, true];
        let selected = eligible
            .iter()
            .enumerate()
            .filter(|(_, yes)| **yes)
            .map(|(t, _)| supervised_prefix(&targets, t).map(|v| v.to_vec()))
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(selected, vec![vec![11], vec![11, 22, 33]]);
        assert!(supervised_prefix(&targets, 4).is_err());
        Ok(())
    }
    #[test]
    fn reference_gradients_add_overlap_before_one_global_clip() -> Result<()> {
        let d = Device::Cpu;
        let task = BTreeMap::from([("context".into(), Tensor::from_vec(vec![3f32, 0.], 2, &d)?)]);
        let reference =
            BTreeMap::from([("context".into(), Tensor::from_vec(vec![-3f32, 4.], 2, &d)?)]);
        let combined = combine_reference_gradients(&task, &reference, 1.)?;
        assert_eq!(combined["context"].to_vec1::<f32>()?, vec![0., 4.]);
        let (denominator, norm) = clip_denominator(&combined, &d)?;
        assert!((norm - 4.).abs() < 1e-6);
        assert_eq!(
            combined["context"]
                .broadcast_div(&denominator)?
                .to_vec1::<f32>()?,
            vec![0., 1.]
        );
        let overlap = combine_reference_gradients(&task, &task, 1.)?;
        assert_eq!(overlap["context"].to_vec1::<f32>()?, vec![6., 0.]);
        assert!(combine_reference_gradients(&task, &reference, f64::NAN).is_err());
        Ok(())
    }
    #[test]
    fn reference_pilot_requires_one_proposal_and_declared_weight_rates_seed() -> Result<()> {
        let base = json!({"mode":"joint_continuation","credit":"raw_identity","read_state_pullback":"categorical","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input","development_labels":"labels",
            "maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new","updates":1,
            "joint_continuation":{"generate_learning_rate":0.003,"context_learning_rate":0.002,"potential_learning_rate":0.003,"continuation_learning_rate":0.03},
            "reference_replay":{"plan_root":"plan","reference_root":"donor","expected_plan_report_sha256":"a".repeat(64),"expected_plan_manifest_sha256":"b".repeat(64),"lambda":1.0}});
        reference_replay_settings(&serde_json::from_value(base.clone())?)?;
        for (key, value) in [
            ("updates", json!(2)),
            ("seed", json!(1002)),
            ("mode", json!("fit")),
            ("query_conditioned_read", json!(true)),
        ] {
            let mut changed = base.clone();
            changed[key] = value;
            assert!(reference_replay_settings(&serde_json::from_value(changed)?).is_err());
        }
        for value in [0., 0.5, 2.] {
            let mut changed = base.clone();
            changed["reference_replay"]["lambda"] = json!(value);
            assert!(reference_replay_settings(&serde_json::from_value(changed)?).is_err());
        }
        let mut changed = base.clone();
        changed["reference_replay"]["expected_plan_report_sha256"] = json!("wrong");
        assert!(reference_replay_settings(&serde_json::from_value(changed)?).is_err());
        let mut changed = base.clone();
        changed["joint_continuation"]["context_learning_rate"] = json!(0.001);
        assert!(reference_replay_settings(&serde_json::from_value(changed)?).is_err());
        let mut enabled = base.clone();
        enabled["native_code_proposals"] = json!(true);
        reference_replay_settings(&serde_json::from_value(enabled.clone())?)?;
        let mut reached = enabled.clone();
        reached["reached_frontier_objective"] = json!(true);
        reference_replay_settings(&serde_json::from_value(reached.clone())?)?;
        let mut emission = reached.clone();
        emission["constrained_emission_learning"] = json!(true);
        emission["retained_context_root"] = json!("context-sealed");
        let admitted_emission: Args = serde_json::from_value(emission.clone())?;
        reference_replay_settings(&admitted_emission)?;
        assert_eq!(
            proposal_policy(&admitted_emission),
            constrained_emission::policy()
        );
        for (key, value) in [
            ("retained_context_root", Value::Null),
            ("constrained_emission_learning", json!(false)),
            ("constrained_context_learning", json!(true)),
            ("categorical_action_learning", json!(true)),
            ("reached_frontier_objective", json!(false)),
            ("native_code_proposals", json!(false)),
        ] {
            let mut invalid = emission.clone();
            invalid[key] = value;
            assert!(reference_replay_settings(&serde_json::from_value(invalid)?).is_err());
        }
        let mut constrained = reached.clone();
        constrained["constrained_context_learning"] = json!(true);
        let admitted: Args = serde_json::from_value(constrained.clone())?;
        reference_replay_settings(&admitted)?;
        assert_eq!(proposal_policy(&admitted), constrained_context::policy());
        for (key, value) in [
            ("categorical_action_learning", json!(true)),
            ("categorical_action_only", json!(true)),
            ("reached_frontier_objective", json!(false)),
            ("native_code_proposals", json!(false)),
            ("reference_replay", Value::Null),
            ("query_conditioned_read", json!(true)),
            ("updates", json!(2)),
        ] {
            let mut invalid = constrained.clone();
            invalid[key] = value;
            assert!(reference_replay_settings(&serde_json::from_value(invalid)?).is_err());
        }
        assert!(!serde_json::from_value::<Args>(base.clone())?.constrained_context_learning);
        let mut categorical = reached.clone();
        categorical["categorical_action_learning"] = json!(true);
        reference_replay_settings(&serde_json::from_value(categorical.clone())?)?;
        let old_categorical: Args = serde_json::from_value(categorical.clone())?;
        assert!(!old_categorical.categorical_action_only);
        assert_eq!(
            proposal_policy(&old_categorical),
            categorical_proposals::policy()
        );
        let mut action_only = categorical.clone();
        action_only["categorical_action_only"] = json!(true);
        let admitted: Args = serde_json::from_value(action_only.clone())?;
        reference_replay_settings(&admitted)?;
        assert_eq!(
            proposal_policy(&admitted),
            categorical_proposals::policy_for(true)
        );
        assert_ne!(
            proposal_policy(&admitted),
            proposal_policy(&old_categorical)
        );
        for (key, value) in [
            ("categorical_action_learning", json!(false)),
            ("reached_frontier_objective", json!(false)),
            ("native_code_proposals", json!(false)),
            ("reference_replay", Value::Null),
            ("query_conditioned_read", json!(true)),
            ("updates", json!(2)),
        ] {
            let mut invalid = action_only.clone();
            invalid[key] = value;
            assert!(reference_replay_settings(&serde_json::from_value(invalid)?).is_err());
        }
        assert!(!serde_json::from_value::<Args>(base.clone())?.categorical_action_only);
        categorical["reached_frontier_objective"] = json!(false);
        assert!(reference_replay_settings(&serde_json::from_value(categorical)?).is_err());
        assert!(!serde_json::from_value::<Args>(base.clone())?.categorical_action_learning);
        reached["native_code_proposals"] = json!(false);
        assert!(reference_replay_settings(&serde_json::from_value(reached)?).is_err());
        enabled
            .as_object_mut()
            .ok_or_else(|| bad("fixture object"))?
            .remove("reference_replay");
        assert!(reference_replay_settings(&serde_json::from_value(enabled)?).is_err());
        assert!(!serde_json::from_value::<Args>(base.clone())?.native_code_proposals);
        let mut legacy = base;
        legacy
            .as_object_mut()
            .ok_or_else(|| bad("fixture object"))?
            .remove("reference_replay");
        legacy["updates"] = json!(64);
        reference_replay_settings(&serde_json::from_value(legacy)?)?;
        Ok(())
    }

    #[test]
    fn legacy_saved_row_metrics_reproduce_exact_historical_schema() -> Result<()> {
        struct TemporaryRows(PathBuf);
        impl Drop for TemporaryRows {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let temp = TemporaryRows(std::env::temp_dir().join(format!(
            "uor-legacy-metrics-{}-{unique}",
            std::process::id()
        )));
        fs::create_dir(&temp.0)?;
        let a: Args = serde_json::from_value(
            json!({"mode":"prediction_control","credit":"raw_identity","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input","development_labels":"labels",
            "maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":temp.0}),
        )?;
        let mut episodes = Vec::new();
        let mut refs = Vec::new();
        let mut expected_arms = Vec::new();
        for (index, target, generated) in
            [(0usize, 20u32, vec![10, 20, 1]), (1, 30, vec![99, 30, 1])]
        {
            let id = format!("legacy-swap{index}-pair");
            let canonical = [10,target,1].into_iter().map(|chosen| json!({
                "native_target_mass":2,"native_denominator":4,
                "native":{"copy_token_ids":[target],"pool":{"summary":{"chosen_token_id":chosen}}}
            })).collect::<Vec<_>>();
            let filename = format!("row-{index}.json");
            fs::write(
                temp.0.join(&filename),
                serde_json::to_vec(&json!({"canonical":canonical,"generated_ids":generated}))?,
            )?;
            refs.push(json!({"id":id,"row_file":filename,"row_sha256":sha256_file(&temp.0.join(&filename))?,
                "complete":index==0,"eos":true}));
            expected_arms.push(json!({"index":index,"id":id,"position":1,"target":target,
                "teacher_prefix_correct":true,"own_position_correct":true,"own_prefix_reached_correctly":index==0,
                "target_mass":2,"denominator":4,"chosen_token":target}));
            episodes.push(Episode {
                packet: Packet {
                    id,
                    segments: vec![],
                    query_ids: vec![5],
                    actual_prefix_ids: vec![],
                },
                answers: serde_json::from_value(
                    json!({"intent":"current","accepted":["fixture"]}),
                )?,
                target: vec![10, target, 1],
                views: vec![],
            });
        }
        let actual = metrics(&a, &json!({"rows":refs}), &episodes)?;
        let historical = json!({"rows":2,"missing_rows":0,"entry_teacher_correct":2,"entry_own_correct":1,
            "first_divergent_teacher_correct":2,"first_divergent_own_position_correct":2,
            "first_divergent_own_prefix_reached":1,"paired_teacher_both_correct":1,"paired_own_position_both_correct":1,
            "pairs":[{"indices":[0,1],"first_divergent_position":1,"arms":expected_arms}],
            "later_copy_covered_positions":2,"later_copy_covered_teacher_correct":2,
            "scope":"complete native own-prefix outputs and separately canonical teacher-prefix metrics; own position matches without a correct preceding prefix are not successful source-dependent continuation"});
        assert_eq!(actual, historical);
        Ok(())
    }

    #[test]
    fn joint_continuation_admits_only_declared_families_and_credit() -> Result<()> {
        let base = json!({"mode":"joint_continuation","credit":"raw_identity","read_state_pullback":"categorical","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new","updates":0,
            "joint_continuation":{"generate_learning_rate":0.003,"context_learning_rate":0.002,
                "potential_learning_rate":0.003,"continuation_learning_rate":0.03}});
        let parsed: Args = serde_json::from_value(base.clone())?;
        control_settings(&parsed)?;
        continuation_settings(&parsed)?;
        assert!(joint_continuation_settings(&parsed)?.is_some());
        for (key, value) in [
            ("credit", json!("clipped")),
            ("read_state_pullback", json!("legacy")),
            ("loss_scope", json!("entry_only")),
            ("ceiling_float", json!(true)),
            ("mode", json!("fit")),
        ] {
            let mut changed = base.clone();
            changed[key] = value;
            assert!(
                joint_continuation_settings(&serde_json::from_value::<Args>(changed)?).is_err()
            );
        }
        let mut changed = base.clone();
        changed["joint_continuation"]["prototype_learning_rate"] = json!(0.01);
        assert!(serde_json::from_value::<Args>(changed).is_err());
        let mut changed = base;
        changed["joint_continuation"]["continuation_learning_rate"] = json!(0.);
        assert!(joint_continuation_settings(&serde_json::from_value::<Args>(changed)?).is_err());
        Ok(())
    }

    #[test]
    fn joint_rowwise_comparison_preserves_regressions_and_rejects_population_change() -> Result<()>
    {
        let before = json!({"rows":[{"id":"a","complete":true,"generated_ids":[1]},
            {"id":"b","complete":false,"generated_ids":[2]},
            {"id":"c","complete":true,"generated_ids":[3]}]});
        let after = json!({"rows":[{"id":"a","complete":true,"generated_ids":[1]},
            {"id":"b","complete":true,"generated_ids":[4]},
            {"id":"c","complete":false,"generated_ids":[5]}]});
        let result = joint_row_comparison(&before, &after)?;
        assert_eq!(result["retained_complete_ids"], json!(["a"]));
        assert_eq!(result["gained_complete_ids"], json!(["b"]));
        assert_eq!(result["lost_complete_ids"], json!(["c"]));
        assert_eq!(result["changed_output_ids"], json!(["b", "c"]));
        let mut wrong = after;
        wrong["rows"][1]["id"] = json!("unmatched");
        assert!(joint_row_comparison(&before, &wrong).is_err());
        Ok(())
    }

    #[test]
    fn continuation_mode_requires_exact_parent_all_answer_credit_and_no_entry_gate() -> Result<()> {
        let base = json!({"mode":"continuation_only","credit":"raw_identity","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new",
            "continuation":{"expected_model_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
                "expected_model_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,"learning_rate":0.003,
                "maximum_cache_tensor_bytes":536870912}});
        let parsed: Args = serde_json::from_value(base.clone())?;
        control_settings(&parsed)?;
        assert!(continuation_settings(&parsed)?.is_some());
        for (key, value) in [
            ("loss_scope", json!("entry_only")),
            ("ceiling_scorer", json!(true)),
            ("read_state_pullback", json!("categorical")),
            ("mode", json!("fit")),
        ] {
            let mut changed = base.clone();
            changed[key] = value;
            assert!(continuation_settings(&serde_json::from_value::<Args>(changed)?).is_err());
        }
        for (key, value) in [
            ("learning_rate", json!(0.)),
            ("maximum_cache_tensor_bytes", json!(0)),
            ("expected_model_report_sha256", json!("wrong")),
        ] {
            let mut changed = base.clone();
            changed["continuation"][key] = value;
            assert!(continuation_settings(&serde_json::from_value::<Args>(changed)?).is_err());
        }
        let mut changed = base.clone();
        changed["prediction_control_updates"] = json!(32);
        assert!(control_settings(&serde_json::from_value::<Args>(changed)?).is_err());
        let mut changed = base;
        changed["continuation"]["selected_record"] = json!(1);
        assert!(serde_json::from_value::<Args>(changed).is_err());
        let plan =
            episode_loss_weights(&[7, 11, 1], &BTreeSet::from([11]), 8, true, LossScope::All)?;
        assert_eq!(plan.weights.len(), 3);
        assert!(plan.weights.iter().all(|w| *w > 0.));
        Ok(())
    }

    #[test]
    fn continuation_snapshot_retains_both_sources_and_rejects_supplied_prefix() -> Result<()> {
        let mut p = Packet {
            id: "query-job".into(),
            query_ids: vec![7],
            actual_prefix_ids: Vec::new(),
            segments: vec![
                Segment::Context {
                    event: 1,
                    role: 1,
                    token_ids: vec![8],
                },
                Segment::Source {
                    event: 1,
                    record: 101,
                    commit: 1,
                    scope: "scope".into(),
                    entity: vec![1],
                    relation: 1,
                    view: 0,
                    original_source_ids: vec![9],
                },
                Segment::Context {
                    event: 2,
                    role: 1,
                    token_ids: vec![10],
                },
                Segment::Source {
                    event: 2,
                    record: 102,
                    commit: 2,
                    scope: "scope".into(),
                    entity: vec![1],
                    relation: 2,
                    view: 0,
                    original_source_ids: vec![11],
                },
            ],
        };
        let snapshot = continuation_snapshot(&p)?;
        assert_eq!(snapshot.segments.len(), 4);
        assert_eq!(snapshot.query_ids, p.query_ids);
        let sources = snapshot
            .segments
            .iter()
            .filter_map(|s| match s {
                OwnedBankSegment::Source(s) => Some((s.record, s.original_token_ids.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(sources, vec![(101, vec![9]), (102, vec![11])]);
        assert_eq!(snapshot.pin.commit, 2);
        p.actual_prefix_ids.push(7);
        assert!(continuation_snapshot(&p).is_err());
        Ok(())
    }
    #[test]
    fn read_state_pullback_config_defaults_serializes_and_rejects_unknown() -> Result<()> {
        let original = json!({"mode":"fit","credit":"raw_identity","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let legacy: Args = serde_json::from_value(original.clone())?;
        assert_eq!(legacy.read_state_pullback, ReadStatePullback::Legacy);
        assert_eq!(
            serde_json::to_value(legacy.read_state_pullback)?,
            json!("legacy")
        );
        let mut categorical = original.clone();
        categorical["read_state_pullback"] = json!("categorical");
        let parsed: Args = serde_json::from_value(categorical.clone())?;
        assert_eq!(parsed.read_state_pullback, ReadStatePullback::Categorical);
        assert_eq!(
            serde_json::to_value(parsed.read_state_pullback)?,
            json!("categorical")
        );
        assert!(parsed.credit == Credit::RawIdentity);
        categorical["read_state_pullback"] = json!("query_keep");
        assert!(serde_json::from_value::<Args>(categorical).is_err());
        Ok(())
    }
    #[test]
    fn prediction_control_config_is_explicit_and_cannot_change_fixed_fit() -> Result<()> {
        let base = json!({"mode":"fit","credit":"raw_identity","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let fixed: Args = serde_json::from_value(base.clone())?;
        let (_, defaults) = control_settings(&fixed)?;
        assert_eq!(defaults.generate, 0.003);
        let mut config = base.clone();
        config["prediction_control_updates"] = json!(16);
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["mode"] = json!("prediction_control");
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            16
        );
        config["prediction_control_updates"] = json!(0);
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            0
        );
        config
            .as_object_mut()
            .ok_or_else(|| bad("test config object"))?
            .remove("prediction_control_updates");
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            32
        );
        config["prediction_control_updates"] = json!(64);
        assert_eq!(
            control_settings(&serde_json::from_value::<Args>(config.clone())?)?.0,
            64
        );
        config["prediction_control_updates"] = json!(65);
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_updates"] = json!(8);
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_updates"] = json!(32);
        config["prediction_control_rates"] =
            json!({"generate":0.04,"prototype":0.01,"context":0.002,"potential":0.003});
        let mut parsed: Args = serde_json::from_value(config.clone())?;
        assert_eq!(control_settings(&parsed)?.1.generate, 0.04);
        parsed
            .prediction_control_rates
            .as_mut()
            .ok_or_else(|| bad("test rates absent"))?
            .generate = f64::NAN;
        assert!(control_settings(&parsed).is_err());
        config["mode"] = json!("admission");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["mode"] = json!("prediction_control");
        config["prediction_control_trainable"] = json!("generate_field");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_resume"] = json!("sealed-control32");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_ok());
        config["prediction_control_trainable"] = json!("potential_generate_field");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_ok());
        config["prediction_control_updates"] = json!(0);
        config["prediction_control_resume_source_step"] = json!(48);
        config["prediction_control_resume_generate_step"] = json!(64);
        let mixed: Args = serde_json::from_value(config.clone())?;
        assert_eq!(mixed.prediction_control_resume_source_step, Some(48));
        assert_eq!(mixed.prediction_control_resume_generate_step, Some(64));
        assert_eq!(control_settings(&mixed)?.0, 0);
        config
            .as_object_mut()
            .ok_or_else(|| bad("test object"))?
            .remove("prediction_control_resume");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_resume"] = json!("sealed-control32");
        config["prediction_control_trainable"] = json!("joint");
        assert!(control_settings(&serde_json::from_value::<Args>(config.clone())?).is_err());
        config["prediction_control_trainable"] = json!("unknown");
        assert!(serde_json::from_value::<Args>(config.clone()).is_err());
        config["mode"] = json!("unknown_control");
        assert!(serde_json::from_value::<Args>(config).is_err());
        Ok(())
    }
    #[test]
    fn phase_weights_conserve_equal_nonempty_phase_mass() -> Result<()> {
        let target = [10, 20, 20, 30, 31];
        let copy = BTreeSet::from([20]);
        let p = episode_loss_weights(&target, &copy, 8, true, LossScope::All)?;
        assert_eq!(p.counts, [1, 2, 2]);
        for phase in 0..3 {
            let sum = p
                .weights
                .iter()
                .zip(&p.phases)
                .filter(|(_, n)| **n == phase)
                .map(|(w, _)| w)
                .sum::<f64>();
            assert!((sum - 1. / 24.).abs() < 1e-14);
        }
        assert!((p.weights.iter().sum::<f64>() - 1. / 8.).abs() < 1e-14);
        Ok(())
    }
    /// The isolation arm removes exactly one thing: later-position credit. The
    /// entry position keeps its own balanced weight, so any difference between
    /// the arms is the later-phase gradient and not an entry reweighting.
    #[test]
    fn entry_only_scope_zeroes_later_credit_and_preserves_entry_weight() -> Result<()> {
        let target = [10, 20, 20, 30, 31];
        let copy = BTreeSet::from([20]);
        let all = episode_loss_weights(&target, &copy, 8, true, LossScope::All)?;
        let only = episode_loss_weights(&target, &copy, 8, true, LossScope::EntryOnly)?;
        assert_eq!(only.phases, all.phases);
        assert_eq!(only.counts, all.counts);
        assert_eq!(only.weights[0], all.weights[0]);
        assert!(only.weights[1..].iter().all(|w| *w == 0.));
        assert!((only.weights.iter().sum::<f64>() - 1. / 24.).abs() < 1e-14);
        // The legacy position-mean policy is untouched by the scope.
        let legacy = episode_loss_weights(&target, &copy, 8, false, LossScope::EntryOnly)?;
        assert!(legacy.weights.iter().all(|w| *w > 0.));
        Ok(())
    }
    /// The scorer is exported data, not floats, and its choice is the strongest
    /// training count; an unseen feature must fall back to the pool.
    #[test]
    fn entry_scorer_is_data_and_chooses_the_strongest_count() -> Result<()> {
        let scorer = EntryScorer {
            schema: ENTRY_SCORER_SCHEMA.into(),
            feature_law: "test".into(),
            rows: vec![EntryScorerRow {
                feature: "seen".into(),
                scores: vec![(5, 2), (7, 3)],
            }],
        };
        assert_eq!(scorer.choose("seen"), Some(7));
        assert_eq!(scorer.choose("unseen"), None);
        let bytes = serde_json::to_vec(&scorer)?;
        let back: EntryScorer = serde_json::from_slice(&bytes)?;
        assert_eq!(back.choose("seen"), Some(7));
        assert_eq!(sha256_bytes(&bytes), back.sha256()?);
        // Ties break to the lower token id so the choice is deterministic.
        let tie = EntryScorer {
            schema: ENTRY_SCORER_SCHEMA.into(),
            feature_law: "test".into(),
            rows: vec![EntryScorerRow {
                feature: "f".into(),
                scores: vec![(9, 4), (3, 4)],
            }],
        };
        assert_eq!(tie.choose("f"), Some(3));
        Ok(())
    }
    #[test]
    fn entry_ceiling_mode_is_a_distinct_zero_update_configuration() -> Result<()> {
        let config = json!({"mode":"entry_ceiling","credit":"clipped","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let a: Args = serde_json::from_value(config.clone())?;
        assert!(a.mode == Mode::EntryCeiling);
        // The ceiling is a zero-update bound, so it must not be reachable through
        // the fit or control modes, and it must not imply a declared dose.
        assert!(a.mode != Mode::Fit && a.mode != Mode::Admission);
        assert!(serde_json::from_value::<Args>(config).is_ok());
        Ok(())
    }
    #[test]
    fn loss_scope_and_dose_default_to_the_retained_campaign() -> Result<()> {
        let config = json!({"mode":"fit","credit":"clipped","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent",
            "training_inputs":"input","training_labels":"labels","development_inputs":"input",
            "development_labels":"labels","maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":"new"});
        let a: Args = serde_json::from_value(config.clone())?;
        assert_eq!(a.updates, UPDATES);
        assert_eq!(a.loss_scope, LossScope::All);
        assert_eq!(a.loss_scope.name(), "all");
        assert_eq!(
            serde_json::to_value(a.loss_scope)?,
            json!("all"),
            "the default scope must serialize as the declared phase-balanced policy"
        );
        let mut isolated = config;
        isolated["loss_scope"] = json!("entry_only");
        isolated["updates"] = json!(64);
        let a: Args = serde_json::from_value(isolated)?;
        assert_eq!(a.loss_scope, LossScope::EntryOnly);
        assert_eq!(a.updates, 64);
        Ok(())
    }
    #[test]
    fn order_is_seeded_complete_and_changes_only_schedule() {
        let a = order(1001, 512);
        assert_eq!(a, order(1001, 512));
        assert_eq!(
            a.iter().copied().collect::<BTreeSet<_>>(),
            (0..512).collect()
        );
        assert_ne!(a, order(1002, 512));
        assert_ne!(order(1002, 512), order(1003, 512));
    }
    #[test]
    fn clip_preserves_tiny_gradient_norm_without_bridge_groups() -> Result<()> {
        let d = Device::Cpu;
        let gradients = BTreeMap::from([(
            "consumer.potential.context_unary".into(),
            Tensor::from_vec(vec![1e-25f32, 2e-25], 2, &d)?,
        )]);
        let (denominator, norm) = clip_denominator(&gradients, &d)?;
        assert!(norm > 2e-25 && norm < 3e-25);
        assert_eq!(denominator.to_scalar::<f32>()?, 1.);
        Ok(())
    }
    fn baseline_fixture() -> (Value, Value) {
        let mut base =
            json!({"source_commit":BASELINE_PRODUCER,"credit":"clipped","order_seed":1001});
        for field in [
            "checkpoint_receipt_sha256",
            "parent_config_sha256",
            "training_input_sha256",
            "training_labels_sha256",
            "original_bridge_masters",
            "marker_masters",
            "categorical",
            "initial_active_masters",
            "active_parameter_names",
            "rates",
            "phase_policy",
        ] {
            base[field] = json!({"identity":field});
        }
        base["fresh_adam"] = json!(true);
        let mut checkpoint = json!({"step":0,"native_independently_reloaded":true,"masters_independently_reloaded":true});
        for field in [
            "parent",
            "original_parent",
            "source_metadata_rebound",
            "generate_sha256",
            "original_quarter_sha256",
            "categorical_sha256",
            "categorical_receipt",
        ] {
            checkpoint[field] = json!({"identity":field});
        }
        (base, checkpoint)
    }
    #[test]
    fn baseline_accepts_only_same_initialized_model_independent_of_order_and_credit() -> Result<()>
    {
        let (base, checkpoint) = baseline_fixture();
        let mut current = base.clone();
        current["credit"] = json!("raw_identity");
        current["order_seed"] = json!(1003);
        current["read_state_pullback"] = json!("categorical");
        baseline_identity(&base, &current, &checkpoint, &checkpoint)?;
        current["initial_active_masters"] = json!({"different_context_master":true});
        assert!(baseline_identity(&base, &current, &checkpoint, &checkpoint).is_err());
        let mut changed = checkpoint.clone();
        changed["generate_sha256"] = json!("different");
        assert!(baseline_identity(&base, &base, &checkpoint, &changed).is_err());
        Ok(())
    }
    #[test]
    fn baseline_rejects_unreviewed_producer_or_incomplete_initial_identity() {
        let (mut base, checkpoint) = baseline_fixture();
        base["source_commit"] = json!("unreviewed-producer");
        assert!(baseline_identity(&base, &base, &checkpoint, &checkpoint).is_err());
        base["source_commit"] = json!(BASELINE_PRODUCER);
        base.as_object_mut().map(|m| m.remove("marker_masters"));
        assert!(baseline_identity(&base, &base, &checkpoint, &checkpoint).is_err());
        let (base, mut checkpoint) = baseline_fixture();
        checkpoint["step"] = json!(1);
        assert!(baseline_identity(&base, &base, &checkpoint, &checkpoint).is_err());
    }
}
