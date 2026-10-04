//! Checked packed-artifact source realization without offline training weights.
//! Shared integer execution preserves the allocating occurrence/action traces.
//! Trusted receipt loading is separate from offline float-source equivalence.
use crate::{
    geometric_context_q4::{ContextQ4Config, NativeContextQ4},
    geometric_no_read::{NativeGeometricNoRead, NoReadConfig},
    geometric_occurrence_read::{
        NativeOccurrenceReader, OccurrenceComponents, OccurrenceRead, PreparedOccurrenceContext,
        QuerySnapshot, SelectedRecordFrame,
    },
    geometric_potential::{AddressLane, NativePotentialTables},
    geometric_potential_q4::{NativePotentialQ4, PotentialQ4Config},
    geometric_read_feedback::{
        FeedbackInputMode, FeedbackTrace, NativeReadFeedback, QuerySnapshotReport,
    },
    geometric_source_actions::{
        ActionHeadScores, ActionTrace, NativeSourceActions, SourceActionBinding,
    },
    geometric_source_emission_view::{SourceEmissionCompiler, SourceEmissionView},
    h4_tables::HistoricalH4Tables,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::Read,
    path::Path,
};
pub const SCHEMA: &str = "uor-r4.geometric-source-realizer/1";
pub const POLICY:&str="parent-free-source-view-Copy-Period-Stop;current-packed-H4-context+potential+Stop+independent-Period;sum-head-logits-before-one-native-normalization;token-alias-mass-sum;no-authored-phase-or-length;128-sequence/1";
const REALIZER_SURROGATE:&str="native-joint-Q31-token-probability-forward;softmax-summed-head-Q24-score-adjoint;quarter-grid-coefficient-STE+frozen-coefficient-context-credit;subtraction-first-zero-forward;target-loss-only;prepared-source-immutable-until-drop/1";
const CONSUMER_SURROGATE:&str="native-Q31-normalized-forward;softmax-score-adjoint;hard-quarter-source-STE;coefficient-and-frozen-context-input-credit;whole-parent-NoRead-fallback;target-loss-only/1";
const ALGEBRA: &[u8] = include_bytes!("../fixtures/historical-h4-tables-v1.bin");
pub const CANONICAL_EXP_SHA256: &str =
    "79485d6e63cc28f5e01d98c5d73abe021db33fa7fef368142ae6592d06b4817f";
pub const CANONICAL_EXP_BYTES: usize = 32776;
#[derive(Debug)]
pub enum SourceRuntimeError {
    Invalid(String),
    Io(std::io::Error),
    Json(serde_json::Error),
}
impl fmt::Display for SourceRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native source runtime: {self:?}")
    }
}
impl std::error::Error for SourceRuntimeError {}
impl From<std::io::Error> for SourceRuntimeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for SourceRuntimeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
pub type SourceRuntimeResult<T> = std::result::Result<T, SourceRuntimeError>;
use SourceRuntimeResult as Result;
pub(crate) fn invalid(message: impl Into<String>) -> SourceRuntimeError {
    SourceRuntimeError::Invalid(message.into())
}
pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceIdentity {
    pub record: u64,
    pub commit: u64,
    pub token_offset: u32,
    pub token_id: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadTrace {
    pub scores_q24: Vec<i64>,
    pub weights_q31: Vec<u64>,
    pub no_read_q24: i64,
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceTrace {
    pub occurrences: Vec<OccurrenceIdentity>,
    pub heads: Vec<HeadTrace>,
    pub sequence_tokens: usize,
    pub context_coefficient_reads: usize,
    pub potential_table_reads: usize,
    pub no_read_table_reads: usize,
    pub geometry_relative_reads: usize,
    pub logical_owned_bytes: usize,
}

/// The inner token offsets belong to the explicitly derived emission view.
/// Original source bytes, token identities and byte-span provenance remain in
/// `emission_view`; they must not be confused with those view offsets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceEmissionTrace {
    pub record: u64,
    pub commit: u64,
    pub scope: Vec<u8>,
    pub entity: Vec<u32>,
    pub relation: u32,
    pub source_store_view: u32,
    pub emission_view: SourceEmissionView,
    pub view_kernel_trace: OccurrenceTrace,
}
pub fn source_view_trace(
    frame: SelectedRecordFrame<'_>,
    view: &SourceEmissionView,
    kernel: OccurrenceTrace,
) -> SourceEmissionTrace {
    SourceEmissionTrace {
        record: frame.identity.record,
        commit: frame.identity.commit,
        scope: frame.metadata.scope.to_vec(),
        entity: frame.metadata.entity.to_vec(),
        relation: frame.metadata.relation,
        source_store_view: frame.metadata.view,
        emission_view: view.clone(),
        view_kernel_trace: kernel,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObservedCode {
    pub root: u8,
    pub radius_bin: u8,
    pub present: bool,
}
impl From<AddressLane> for ObservedCode {
    fn from(code: AddressLane) -> Self {
        Self {
            root: code.root(),
            radius_bin: code.radius_bin(),
            present: code.present(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SerializableContextReplay {
    pub tokens: Vec<u32>,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub states: Vec<Vec<u8>>,
    pub actions: Vec<Vec<u8>>,
    pub raw_roots: Vec<u8>,
    pub categories: Vec<u8>,
    pub codes: Vec<ObservedCode>,
    pub coefficient_reads: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RealizerTrace {
    pub policy: &'static str,
    pub source: SourceEmissionTrace,
    /// Original causal replay shared by source scoring and controller observation.
    /// A dependent update is recorded separately, never disguised as token replay.
    pub period_context: SerializableContextReplay,
    pub period_q24: Vec<i64>,
    pub actions: ActionTrace,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DependentReadTrace {
    pub policy: &'static str,
    pub stage1: RealizerTrace,
    pub stage2: RealizerTrace,
    pub feedback: FeedbackTrace,
    /// Authoritative input for ALL stage2 Copy/Stop/Period scoring.
    pub stage2_controller_snapshot: QuerySnapshotReport,
    pub stage2_context_replay_policy: &'static str,
    pub logical_prepared_bytes: usize,
    pub original_context_replays: usize,
    pub scoring_stages: usize,
}

pub fn read_occurrence(
    components: OccurrenceComponents<'_>,
    frame: SelectedRecordFrame<'_>,
    query: &[u32],
    prefix: &[u32],
) -> Result<OccurrenceTrace> {
    let mut reader = NativeOccurrenceReader::new(components).map_err(|e| invalid(e.to_string()))?;
    let output = reader
        .read(frame, query, prefix)
        .map_err(|e| invalid(e.to_string()))?;
    trace_occurrence(output)
}
fn trace_occurrence(output: OccurrenceRead<'_>) -> Result<OccurrenceTrace> {
    let mut heads = Vec::with_capacity(output.heads.len());
    for h in 0..output.heads.len() {
        let head = output
            .head(h)
            .ok_or_else(|| invalid("consumer head absent"))?;
        heads.push(HeadTrace {
            scores_q24: head.potential_q24.to_vec(),
            weights_q31: head.occurrence_weights_q31.to_vec(),
            no_read_q24: head.no_read_q24,
            no_read_weight_q31: head.no_read_weight_q31,
            total_weight_q31: head.total_weight_q31,
        });
    }
    Ok(OccurrenceTrace {
        occurrences: output
            .occurrences
            .iter()
            .map(|x| OccurrenceIdentity {
                record: x.source.record,
                commit: x.source.commit,
                token_offset: x.token_offset,
                token_id: x.token_id,
            })
            .collect(),
        heads,
        sequence_tokens: output.stats.sequence_tokens,
        context_coefficient_reads: output.stats.context_coefficient_reads,
        potential_table_reads: output.stats.potential_table_reads,
        no_read_table_reads: output.stats.no_read_table_reads,
        geometry_relative_reads: output.stats.geometry_relative_reads,
        logical_owned_bytes: output.stats.logical_owned_bytes,
    })
}
pub struct RealizerExecution<'a> {
    pub context: &'a NativeContextQ4,
    pub potential_tables: &'a NativePotentialTables,
    pub no_read: &'a NativeGeometricNoRead,
    pub geometry: &'a HistoricalH4Tables,
    pub exp: &'a [u32],
    pub period: &'a NativeGeometricNoRead,
    pub binding: &'a SourceActionBinding,
}
impl<'a> RealizerExecution<'a> {
    pub fn read_source_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<SourceEmissionTrace> {
        if view.policy() != crate::geometric_source_emission_view::POLICY
            || view.tokenizer_sha256() != self.binding.tokenizer_sha256()
        {
            return Err(invalid(
                "source emission view policy/tokenizer differs from artifact",
            ));
        }
        self.binding.validate_tokens(query)?;
        self.binding.validate_tokens(prefix)?;
        let kernel = read_occurrence(
            OccurrenceComponents {
                context: self.context.native(),
                potential: self.potential_tables,
                no_read: self.no_read,
                geometry: self.geometry,
                exp_q31: self.exp,
            },
            view.derived_frame(frame)?,
            query,
            prefix,
        )?;
        Ok(source_view_trace(frame, view, kernel))
    }
    fn prepare_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<(
        NativeOccurrenceReader<'a>,
        PreparedOccurrenceContext<'a>,
        SerializableContextReplay,
    )> {
        if self.period.config() != self.no_read.config()
            || self.binding.vocab_size() != self.context.config().vocab_size
            || view.policy() != crate::geometric_source_emission_view::POLICY
            || view.tokenizer_sha256() != self.binding.tokenizer_sha256()
        {
            return Err(invalid(
                "realizer context/controller/view/tokenizer differs",
            ));
        }
        self.binding.validate_tokens(query)?;
        self.binding.validate_tokens(prefix)?;
        let reader = NativeOccurrenceReader::new(OccurrenceComponents {
            context: self.context.native(),
            potential: self.potential_tables,
            no_read: self.no_read,
            geometry: self.geometry,
            exp_q31: self.exp,
        })
        .map_err(|e| invalid(e.to_string()))?;
        let prepared = reader
            .prepare(view.derived_frame(frame)?, query, prefix)
            .map_err(|e| invalid(e.to_string()))?;
        let c = self.context.config();
        let lanes = c.heads * c.lanes_per_head;
        let mut replay = SerializableContextReplay {
            tokens: prepared.tokens().to_vec(),
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            states: Vec::new(),
            actions: Vec::new(),
            raw_roots: Vec::new(),
            categories: Vec::new(),
            codes: Vec::new(),
            coefficient_reads: prepared.context_coefficient_reads(),
        };
        for step in prepared.steps() {
            replay
                .states
                .push(step.states[..lanes].iter().map(|r| r.index()).collect());
            replay
                .actions
                .push(step.actions[..lanes].iter().map(|r| r.index()).collect());
            replay
                .raw_roots
                .extend(step.readout_roots[..lanes].iter().map(|r| r.index()));
            replay
                .categories
                .extend_from_slice(&step.categories[..lanes]);
            replay
                .codes
                .extend(step.output[..lanes].iter().copied().map(ObservedCode::from));
        }
        Ok((reader, prepared, replay))
    }
    fn score_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        reader: &mut NativeOccurrenceReader<'a>,
        prepared: &PreparedOccurrenceContext<'a>,
        snapshot: &QuerySnapshot<'_, 'a>,
        replay: &SerializableContextReplay,
    ) -> Result<RealizerTrace> {
        let source = source_view_trace(
            frame,
            view,
            trace_occurrence(
                reader
                    .score(prepared, snapshot)
                    .map_err(|e| invalid(e.to_string()))?,
            )?,
        );
        let period = self
            .period
            .score(
                snapshot.last_token() as usize,
                snapshot.states(),
                snapshot.codes(),
                None,
            )
            .map_err(|e| invalid(e.to_string()))?;
        // Stop in source is scored from exactly this same checked snapshot.
        let heads = self.context.config().heads;
        let head_scores = source
            .view_kernel_trace
            .heads
            .iter()
            .enumerate()
            .map(|(head, s)| ActionHeadScores {
                copy_q24: &s.scores_q24,
                period_q24: period[head],
                stop_q24: s.no_read_q24,
            })
            .collect::<Vec<_>>();
        let actions = NativeSourceActions::new(self.binding.clone(), heads, self.exp)?
            .reduce(view.emitted_token_ids(), &head_scores)?;
        Ok(RealizerTrace {
            policy: POLICY,
            source,
            period_context: replay.clone(),
            period_q24: period[..heads].to_vec(),
            actions,
        })
    }
    pub fn read(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<RealizerTrace> {
        let (mut reader, prepared, replay) = self.prepare_view(frame, view, query, prefix)?;
        let snapshot = prepared
            .original_snapshot()
            .map_err(|e| invalid(e.to_string()))?;
        self.score_view(frame, view, &mut reader, &prepared, &snapshot, &replay)
    }
    pub fn read_dependent(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        parent: &NativeArtifactBinding,
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
    ) -> Result<DependentReadTrace> {
        let (mut reader, prepared, replay) = self.prepare_view(frame, view, query, prefix)?;
        let snapshot = prepared
            .original_snapshot()
            .map_err(|e| invalid(e.to_string()))?;
        let stage1 = self.score_view(frame, view, &mut reader, &prepared, &snapshot, &replay)?;
        let (updated, feedback_trace) =
            feedback.apply(parent, &prepared, &snapshot, &stage1.actions, mode)?;
        let stage2 = self.score_view(frame, view, &mut reader, &prepared, &updated, &replay)?;
        Ok(DependentReadTrace {
            policy: crate::geometric_read_feedback::POLICY,
            stage1,
            stage2,
            stage2_controller_snapshot: feedback_trace.after.clone(),
            stage2_context_replay_policy: "stage2.period_context is original token replay only; stage2_controller_snapshot is authoritative updated Copy/Stop/Period input",
            feedback: feedback_trace,
            logical_prepared_bytes: prepared.logical_prepared_bytes(),
            original_context_replays: 1,
            scoring_stages: 2,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub tokenizer_sha256: String,
    pub parent_checkpoint_manifest_sha256: String,
    pub parent_model_sha256: String,
    pub parent_config_sha256: String,
}
impl ArtifactIdentity {
    fn validate(&self) -> Result<()> {
        for digest in [
            &self.tokenizer_sha256,
            &self.parent_checkpoint_manifest_sha256,
            &self.parent_model_sha256,
            &self.parent_config_sha256,
        ] {
            require_digest(digest)?;
        }
        Ok(())
    }
}
/// The caller obtains this receipt from trusted deployment/selection evidence,
/// never from the untrusted directory that this loader is about to admit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeArtifactBinding {
    pub metadata_sha256: String,
    pub identity: ArtifactIdentity,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterIdentity {
    shape: Vec<usize>,
    f32_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RealizerMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    source_view_policy: String,
    action_policy: String,
    identity: ArtifactIdentity,
    period: NoReadConfig,
    source_parameters: BTreeMap<String, ParameterIdentity>,
    files: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConsumerMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    identity: ArtifactIdentity,
    context: ContextQ4Config,
    potential: PotentialQ4Config,
    no_read: NoReadConfig,
    algebra_sha256: String,
    files: BTreeMap<String, String>,
    source_view_policy: Option<String>,
}
const NATIVE_FILES: [&str; 7] = [
    "tokenizer.json",
    "period-q4.bin",
    "consumer/metadata.json",
    "consumer/context-q4.bin",
    "consumer/potential-q4.bin",
    "consumer/no-read-q4.bin",
    "consumer/exp-q31.bin",
];
const CONSUMER_FILES: [&str; 4] = [
    "context-q4.bin",
    "potential-q4.bin",
    "no-read-q4.bin",
    "exp-q31.bin",
];
const MAX_METADATA_BYTES: usize = 1 << 20;
const MAX_TOKENIZER_BYTES: usize = 8 << 20;
fn require_digest(digest: &str) -> Result<()> {
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("artifact digest format differs"));
    }
    Ok(())
}
fn read_bounded(path: &Path, maximum: usize) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > maximum as u64 {
        return Err(invalid("artifact file type or bounded byte length differs"));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid("artifact grew beyond admitted byte bound"));
    }
    Ok(bytes)
}
fn read_exact(path: &Path, length: usize) -> Result<Vec<u8>> {
    let bytes = read_bounded(path, length)?;
    if bytes.len() != length {
        return Err(invalid("artifact exact byte length differs"));
    }
    Ok(bytes)
}
fn require_inventory(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(invalid("native artifact root must be a real directory"));
    }
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| invalid("artifact path is not UTF8"))?;
        let ty = entry.file_type()?;
        if name == "consumer" && ty.is_dir() {
            for child in fs::read_dir(entry.path())? {
                let child = child?;
                let name = child
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("artifact path is not UTF8"))?;
                if !child.file_type()?.is_file() {
                    return Err(invalid("consumer artifact contains directory or symlink"));
                }
                actual.insert(format!("consumer/{name}"));
            }
        } else if ty.is_file() {
            actual.insert(name);
        } else {
            return Err(invalid("artifact contains unexpected directory or symlink"));
        }
    }
    let expected = NATIVE_FILES
        .into_iter()
        .chain(std::iter::once("metadata.json"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(invalid("native artifact exact inventory differs"));
    }
    Ok(())
}
fn require_hashes(files: &BTreeMap<String, String>, names: &[&str]) -> Result<()> {
    if files.keys().map(String::as_str).collect::<BTreeSet<_>>() != names.iter().copied().collect()
    {
        return Err(invalid("declared artifact inventory differs"));
    }
    for digest in files.values() {
        require_digest(digest)?;
    }
    Ok(())
}
fn check_file(files: &BTreeMap<String, String>, name: &str, bytes: &[u8]) -> Result<()> {
    if files.get(name) != Some(&sha256_bytes(bytes)) {
        return Err(invalid(format!("native artifact digest differs: {name}")));
    }
    Ok(())
}
fn canonical_exp(bytes: &[u8]) -> Result<Vec<u32>> {
    if bytes.len() != CANONICAL_EXP_BYTES || sha256_bytes(bytes) != CANONICAL_EXP_SHA256 {
        return Err(invalid(
            "native artifact canonical exponential bytes differ",
        ));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}
fn source_parameter_provenance(metadata: &RealizerMetadata, c: &ConsumerMetadata) -> Result<()> {
    let mut shapes = BTreeMap::new();
    for (name, shape) in c
        .context
        .coefficient_shapes()
        .map_err(|e| invalid(e.to_string()))?
    {
        shapes.insert(format!("consumer.context.{name}"), shape);
    }
    for (name, shape) in c
        .potential
        .coefficient_shapes()
        .map_err(|e| invalid(e.to_string()))?
    {
        shapes.insert(format!("consumer.potential.{name}"), shape);
    }
    shapes.insert(
        "consumer.no_read.coefficients".into(),
        vec![c.no_read.heads, c.no_read.coefficients_per_head()],
    );
    shapes.insert(
        "period.coefficients".into(),
        vec![
            metadata.period.heads,
            metadata.period.coefficients_per_head(),
        ],
    );
    if shapes.keys().ne(metadata.source_parameters.keys()) {
        return Err(invalid(
            "native artifact source-provenance parameter inventory differs",
        ));
    }
    for (name, shape) in shapes {
        let saved = &metadata.source_parameters[&name];
        require_digest(&saved.f32_sha256)?;
        if saved.shape != shape {
            return Err(invalid("native artifact source-provenance shape differs"));
        }
    }
    Ok(())
}
pub struct NativeSourceRealizer {
    context: NativeContextQ4,
    potential: NativePotentialQ4,
    potential_tables: NativePotentialTables,
    no_read: NativeGeometricNoRead,
    period: NativeGeometricNoRead,
    geometry: HistoricalH4Tables,
    exp: Vec<u32>,
    binding: SourceActionBinding,
    compiler: SourceEmissionCompiler,
    artifact_binding: NativeArtifactBinding,
}
impl NativeSourceRealizer {
    pub fn load_native(path: &Path, expected: &NativeArtifactBinding) -> Result<Self> {
        expected.identity.validate()?;
        require_digest(&expected.metadata_sha256)?;
        require_inventory(path)?;
        let outer = read_bounded(&path.join("metadata.json"), MAX_METADATA_BYTES)?;
        if sha256_bytes(&outer) != expected.metadata_sha256 {
            return Err(invalid(
                "native artifact trusted outer metadata digest differs",
            ));
        }
        let metadata: RealizerMetadata = serde_json::from_slice(&outer)?;
        require_hashes(&metadata.files, &NATIVE_FILES)?;
        let tokenizer = read_bounded(&path.join("tokenizer.json"), MAX_TOKENIZER_BYTES)?;
        let nested = read_bounded(&path.join("consumer/metadata.json"), MAX_METADATA_BYTES)?;
        check_file(&metadata.files, "tokenizer.json", &tokenizer)?;
        check_file(&metadata.files, "consumer/metadata.json", &nested)?;
        if metadata.schema != SCHEMA
            || metadata.policy != POLICY
            || metadata.surrogate != REALIZER_SURROGATE
            || metadata.action_policy != crate::geometric_source_actions::POLICY
            || metadata.source_view_policy != crate::geometric_source_emission_view::POLICY
            || metadata.identity != expected.identity
            || sha256_bytes(&tokenizer) != expected.identity.tokenizer_sha256
        {
            return Err(invalid(
                "native artifact realizer policy or trusted identity differs",
            ));
        }
        let consumer: ConsumerMetadata = serde_json::from_slice(&nested)?;
        require_hashes(&consumer.files, &CONSUMER_FILES)?;
        consumer
            .context
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        consumer
            .potential
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        consumer
            .no_read
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        metadata
            .period
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        let c = consumer.context;
        if consumer.schema != "uor-r4.geometric-occurrence-consumer/2"
            || consumer.policy != crate::geometric_occurrence_read::POLICY
            || consumer.surrogate != CONSUMER_SURROGATE
            || consumer.source_view_policy.as_deref()
                != Some(crate::geometric_source_emission_view::POLICY)
            || consumer.identity != expected.identity
            || consumer.algebra_sha256 != sha256_bytes(ALGEBRA)
            || c.heads != consumer.potential.heads
            || c.lanes_per_head != consumer.potential.lanes_per_head
            || c.heads != consumer.no_read.heads
            || c.lanes_per_head != consumer.no_read.latent_lanes_per_head
            || c.vocab_size != consumer.no_read.vocabulary
            || metadata.period != consumer.no_read
        {
            return Err(invalid(
                "native artifact nested policy, geometry, identity or component shapes differ",
            ));
        }
        let binding = SourceActionBinding::new(&tokenizer)?;
        if binding.vocab_size() != c.vocab_size {
            return Err(invalid(
                "native artifact tokenizer/component vocabulary differs",
            ));
        }
        source_parameter_provenance(&metadata, &consumer)?;
        let lengths = [
            c.coefficient_count()
                .map_err(|e| invalid(e.to_string()))?
                .div_ceil(2),
            consumer
                .potential
                .coefficient_count()
                .map_err(|e| invalid(e.to_string()))?
                .div_ceil(2),
            consumer.no_read.coefficient_count().div_ceil(2),
            CANONICAL_EXP_BYTES,
        ];
        let mut bytes = Vec::new();
        for (name, length) in CONSUMER_FILES.into_iter().zip(lengths) {
            let data = read_exact(&path.join("consumer").join(name), length)?;
            check_file(&consumer.files, name, &data)?;
            check_file(&metadata.files, &format!("consumer/{name}"), &data)?;
            bytes.push(data);
        }
        let period_bytes = read_exact(
            &path.join("period-q4.bin"),
            metadata.period.coefficient_count().div_ceil(2),
        )?;
        check_file(&metadata.files, "period-q4.bin", &period_bytes)?;
        let context = NativeContextQ4::new(c, &bytes[0]).map_err(|e| invalid(e.to_string()))?;
        let potential = NativePotentialQ4::new(consumer.potential, &bytes[1])
            .map_err(|e| invalid(e.to_string()))?;
        let potential_tables = NativePotentialQ4::new(consumer.potential, &bytes[1])
            .map_err(|e| invalid(e.to_string()))?
            .into_native()
            .map_err(|e| invalid(e.to_string()))?;
        let no_read = NativeGeometricNoRead::new(consumer.no_read, &bytes[2])
            .map_err(|e| invalid(e.to_string()))?;
        let period = NativeGeometricNoRead::new(metadata.period, &period_bytes)
            .map_err(|e| invalid(e.to_string()))?;
        let exp = canonical_exp(&bytes[3])?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        NativeOccurrenceReader::new(OccurrenceComponents {
            context: context.native(),
            potential: &potential_tables,
            no_read: &no_read,
            geometry: &geometry,
            exp_q31: &exp,
        })
        .map_err(|e| invalid(e.to_string()))?;
        let compiler = SourceEmissionCompiler::new(&tokenizer)?;
        Ok(Self {
            context,
            potential,
            potential_tables,
            no_read,
            period,
            geometry,
            exp,
            binding,
            compiler,
            artifact_binding: expected.clone(),
        })
    }
    pub fn artifact_binding(&self) -> &NativeArtifactBinding {
        &self.artifact_binding
    }
    pub fn context_config(&self) -> ContextQ4Config {
        self.context.config()
    }
    pub fn read_dependent(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
    ) -> Result<DependentReadTrace> {
        if feedback.metadata().context != self.context.config()
            || feedback.metadata().parent_artifact != self.artifact_binding
        {
            return Err(invalid(
                "dependent feedback does not bind this native artifact",
            ));
        }
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_dependent(
            frame,
            view,
            query,
            prefix,
            &self.artifact_binding,
            feedback,
            mode,
        )
    }
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn compile_view(&self, original_ids: &[u32]) -> Result<SourceEmissionView> {
        self.compiler.compile(original_ids)
    }
    pub fn read(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        own_prefix: &[u32],
    ) -> Result<RealizerTrace> {
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read(frame, view, query, own_prefix)
    }
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({"context":self.context.stats(),"potential":self.potential.stats(),"Stop":self.no_read.stats(),"Period":self.period.stats(),
            "exp_bytes":self.exp.len()*4,"algebra_bytes":ALGEBRA.len(),"additional_potential_table_copy_bytes":self.potential.table_bytes().len(),
            "extra_context_replays_per_read":0,"original_context_replays_per_read":1,"prepared_context_scratch_bytes":std::mem::size_of::<PreparedOccurrenceContext>(),"final_joint_reductions_per_read":1,
            "inherited_per_head_reductions_discarded":self.context.config().heads,
            "scope":"source-free packed integer components; loading/tokenization/trace wrapper allocate; no general chat or allocation-free qualification"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_occurrence_read::{FrameMetadata, FrameStatus, SourceIdentity};
    use std::sync::atomic::{AtomicU64, Ordering};
    const TOKENIZER:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"\u0120":6},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"},{"id":9,"content":"<gap>"}]}"#;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: std::path::PathBuf,
        binding: NativeArtifactBinding,
    }
    impl Fixture {
        fn new() -> Result<Self> {
            let root = std::env::temp_dir().join(format!(
                "native-source-loader-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root)?;
            fs::create_dir(root.join("consumer"))?;
            let identity = ArtifactIdentity {
                tokenizer_sha256: sha256_bytes(TOKENIZER),
                parent_checkpoint_manifest_sha256: "a".repeat(64),
                parent_model_sha256: "b".repeat(64),
                parent_config_sha256: "c".repeat(64),
            };
            let c = ContextQ4Config {
                vocab_size: 10,
                heads: 1,
                lanes_per_head: 1,
            };
            let p = PotentialQ4Config {
                heads: 1,
                lanes_per_head: 1,
            };
            let n = NoReadConfig {
                vocabulary: 10,
                heads: 1,
                latent_lanes_per_head: 1,
            };
            let data = [
                vec![
                    0;
                    c.coefficient_count()
                        .map_err(|e| invalid(e.to_string()))?
                        .div_ceil(2)
                ],
                vec![
                    0;
                    p.coefficient_count()
                        .map_err(|e| invalid(e.to_string()))?
                        .div_ceil(2)
                ],
                vec![0; n.coefficient_count().div_ceil(2)],
                vec![0; CANONICAL_EXP_BYTES],
            ];
            let mut inner = BTreeMap::new();
            for (name, bytes) in CONSUMER_FILES.into_iter().zip(&data) {
                fs::write(root.join("consumer").join(name), bytes)?;
                inner.insert(name, sha256_bytes(bytes));
            }
            let nested = serde_json::json!({"schema":"uor-r4.geometric-occurrence-consumer/2","policy":crate::geometric_occurrence_read::POLICY,
                "surrogate":CONSUMER_SURROGATE,"identity":identity,"context":c,"potential":p,"no_read":n,
                "algebra_sha256":sha256_bytes(ALGEBRA),"files":inner,"source_view_policy":crate::geometric_source_emission_view::POLICY});
            fs::write(
                root.join("consumer/metadata.json"),
                serde_json::to_vec(&nested)?,
            )?;
            fs::write(root.join("tokenizer.json"), TOKENIZER)?;
            fs::write(root.join("period-q4.bin"), &data[2])?;
            let mut shapes = BTreeMap::new();
            for (name, shape) in c.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
                shapes.insert(format!("consumer.context.{name}"), shape);
            }
            for (name, shape) in p.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
                shapes.insert(format!("consumer.potential.{name}"), shape);
            }
            shapes.insert(
                "consumer.no_read.coefficients".into(),
                vec![1, n.coefficients_per_head()],
            );
            shapes.insert(
                "period.coefficients".into(),
                vec![1, n.coefficients_per_head()],
            );
            let provenance = shapes
                .into_iter()
                .map(|(name, shape)| {
                    (
                        name,
                        serde_json::json!({"shape":shape,"f32_sha256":"d".repeat(64)}),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let files = NATIVE_FILES
                .into_iter()
                .map(|name| Ok((name, sha256_bytes(&fs::read(root.join(name))?))))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let outer = serde_json::to_vec(
                &serde_json::json!({"schema":SCHEMA,"policy":POLICY,"surrogate":REALIZER_SURROGATE,
                "source_view_policy":crate::geometric_source_emission_view::POLICY,"action_policy":crate::geometric_source_actions::POLICY,
                "identity":identity,"period":n,"source_parameters":provenance,"files":files}),
            )?;
            fs::write(root.join("metadata.json"), &outer)?;
            Ok(Self {
                root,
                binding: NativeArtifactBinding {
                    metadata_sha256: sha256_bytes(&outer),
                    identity,
                },
            })
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    fn error(f: &Fixture) -> String {
        match NativeSourceRealizer::load_native(&f.root, &f.binding) {
            Ok(_) => "accepted".into(),
            Err(e) => e.to_string(),
        }
    }
    #[test]
    fn source_free_loader_checks_trusted_receipt_inventory_and_payloads() -> Result<()> {
        let mut f = Fixture::new()?;
        // This deliberately noncanonical table reaches the exact exp trust
        // boundary; tests never generate floating exponentials as a fixture.
        assert!(error(&f).contains("canonical exponential bytes"));
        f.binding.metadata_sha256 = "e".repeat(64);
        assert!(error(&f).contains("trusted outer metadata"));
        let f = Fixture::new()?;
        fs::write(f.root.join("unexpected"), b"extra")?;
        assert!(error(&f).contains("exact inventory"));
        let f = Fixture::new()?;
        fs::write(f.root.join("consumer/no-read-q4.bin"), [1u8])?;
        assert!(error(&f).contains("exact byte length"));
        let f = Fixture::new()?;
        let mut bytes = fs::read(f.root.join("consumer/potential-q4.bin"))?;
        bytes[0] = 1;
        fs::write(f.root.join("consumer/potential-q4.bin"), bytes)?;
        assert!(error(&f).contains("digest differs"));
        let f = Fixture::new()?;
        fs::write(f.root.join("consumer/metadata.json"), b"{}")?;
        assert!(error(&f).contains("digest differs"));
        Ok(())
    }
    #[test]
    fn source_free_loader_rejects_consistently_rehashed_config_and_provenance_tampering(
    ) -> Result<()> {
        fn replace_outer(f: &mut Fixture, outer: &serde_json::Value) -> Result<()> {
            let bytes = serde_json::to_vec(outer)?;
            fs::write(f.root.join("metadata.json"), &bytes)?;
            f.binding.metadata_sha256 = sha256_bytes(&bytes);
            Ok(())
        }
        let mut f = Fixture::new()?;
        let mut nested: serde_json::Value =
            serde_json::from_slice(&fs::read(f.root.join("consumer/metadata.json"))?)?;
        nested["potential"]["heads"] = serde_json::json!(2);
        let bytes = serde_json::to_vec(&nested)?;
        fs::write(f.root.join("consumer/metadata.json"), &bytes)?;
        let mut outer: serde_json::Value =
            serde_json::from_slice(&fs::read(f.root.join("metadata.json"))?)?;
        outer["files"]["consumer/metadata.json"] = serde_json::json!(sha256_bytes(&bytes));
        replace_outer(&mut f, &outer)?;
        assert!(error(&f).contains("component shapes"));
        let mut f = Fixture::new()?;
        let mut outer: serde_json::Value =
            serde_json::from_slice(&fs::read(f.root.join("metadata.json"))?)?;
        outer["source_parameters"]["period.coefficients"]["shape"] = serde_json::json!([1, 999]);
        replace_outer(&mut f, &outer)?;
        assert!(error(&f).contains("provenance shape"));
        let mut f = Fixture::new()?;
        f.binding.identity.tokenizer_sha256 = "e".repeat(64);
        assert!(error(&f).contains("trusted identity"));
        Ok(())
    }
    #[cfg(unix)]
    #[test]
    fn source_free_loader_rejects_payload_symlinks_and_bounded_reads() -> Result<()> {
        let f = Fixture::new()?;
        fs::remove_file(f.root.join("consumer/context-q4.bin"))?;
        std::os::unix::fs::symlink("potential-q4.bin", f.root.join("consumer/context-q4.bin"))?;
        assert!(error(&f).contains("symlink"));
        let f = Fixture::new()?;
        assert!(read_bounded(&f.root.join("tokenizer.json"), 1).is_err());
        Ok(())
    }
    #[test]
    fn canonical_exp_rejects_partial_words_and_numerically_valid_substitution() -> Result<()> {
        let mut bytes = vec![0; CANONICAL_EXP_BYTES];
        bytes[..4].copy_from_slice(&(crate::geometric_read::WEIGHT_ONE as u32).to_le_bytes());
        let values = bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        assert!(crate::geometric_read::NativeGeometricRead::new(1, 1, &values).is_ok());
        assert!(canonical_exp(&bytes).is_err());
        assert!(canonical_exp(&bytes[..bytes.len() - 1]).is_err());
        Ok(())
    }
    #[test]
    fn checked_shared_runtime_rejects_sparse_query_wrong_frame_and_window() -> Result<()> {
        let binding = SourceActionBinding::new(TOKENIZER)?;
        let compiler = SourceEmissionCompiler::new(TOKENIZER)?;
        let view = compiler.compile(&[4])?;
        let c = ContextQ4Config {
            vocab_size: 10,
            heads: 1,
            lanes_per_head: 1,
        };
        let p = PotentialQ4Config {
            heads: 1,
            lanes_per_head: 1,
        };
        let n = NoReadConfig {
            vocabulary: 10,
            heads: 1,
            latent_lanes_per_head: 1,
        };
        let context = NativeContextQ4::new(
            c,
            &vec![
                0;
                c.coefficient_count()
                    .map_err(|e| invalid(e.to_string()))?
                    .div_ceil(2)
            ],
        )
        .map_err(|e| invalid(e.to_string()))?;
        let potential = NativePotentialQ4::new(
            p,
            &vec![
                0;
                p.coefficient_count()
                    .map_err(|e| invalid(e.to_string()))?
                    .div_ceil(2)
            ],
        )
        .map_err(|e| invalid(e.to_string()))?
        .into_native()
        .map_err(|e| invalid(e.to_string()))?;
        let no_read = NativeGeometricNoRead::new(n, &vec![0; n.coefficient_count().div_ceil(2)])
            .map_err(|e| invalid(e.to_string()))?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let mut exp = vec![0; crate::geometric_read::EXP_TABLE_LEN];
        exp[0] = crate::geometric_read::WEIGHT_ONE as u32;
        let executor = RealizerExecution {
            context: &context,
            potential_tables: &potential,
            no_read: &no_read,
            geometry: &geometry,
            exp: &exp,
            period: &no_read,
            binding: &binding,
        };
        fn frame(tokens: &[u32]) -> SelectedRecordFrame<'_> {
            SelectedRecordFrame {
                identity: SourceIdentity {
                    record: 7,
                    commit: 9,
                },
                metadata: FrameMetadata {
                    scope: b"unit",
                    entity: &[],
                    relation: 3,
                    view: 0,
                    status: FrameStatus::Found,
                },
                token_ids: tokens,
            }
        }

        assert!(executor.read(frame(&[4]), &view, &[7], &[]).is_err());
        assert!(executor.read(frame(&[5]), &view, &[4], &[]).is_err());
        assert!(executor.read(frame(&[4]), &view, &[], &[]).is_err());
        assert!(executor
            .read(frame(&[4]), &view, &[4], &vec![4; 128])
            .is_err());
        let valid = executor.read(frame(&[4]), &view, &[4], &[])?;
        assert_eq!(
            valid.period_context.tokens,
            view.emitted_token_ids()
                .iter()
                .copied()
                .chain([4])
                .collect::<Vec<_>>()
        );
        assert!(valid
            .actions
            .token_masses
            .iter()
            .any(|token| token.token_id == 4));
        Ok(())
    }
}
