//! Parent-free learned Copy/Period/Stop realization of an admitted source view.
//!
//! This module is a child of `geometric_occurrence_consumer` so its prepared
//! context and coefficient sources are reused without widening their APIs.
//! Existing geometric occurrence scores become Copy logits, existing NoRead
//! becomes Stop, and an independent q4 NoRead-shaped operator supplies Period.
//! There is no parent distribution, authored output schedule or target input to
//! native reading. The supported output alphabet is source-view tokens plus
//! the tokenizer-bound period and protocol EOS; it is not general prose.
//!
//! Native execution reuses the occurrence reader (including its currently
//! unused per-head reductions), replays integer context once more for Period,
//! and calls the joint action reducer. These extra work/allocation boundaries
//! are explicit, not an allocation-free or compiled-instruction claim.
//! Training retains exact native masses in the loss forward and substitutes
//! the declared softmax of summed-head scores only for its first-order adjoint.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use candle_core::{Device, IndexOp, Tensor, Var};
use serde::{Deserialize, Serialize};
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_no_read::{NativeGeometricNoRead, NoReadConfig},
    geometric_occurrence_read::SelectedRecordFrame,
    geometric_potential::AddressLane,
    h4_tables::H4Code,
};

use super::{
    ConsumerIdentity, ConsumerWeights, NativeConsumerArtifact, PreparedConsumerStep,
    SourceEmissionTrace, SOURCE_VIEW_POLICY, SOURCE_VIEW_SCHEMA,
};
use crate::{
    geometric_context::NativeContextTrace,
    geometric_context_credit::{frozen_no_read_forward, frozen_potential_forward},
    geometric_no_read::{NoReadBatch, NoReadWeights},
    geometric_source_actions::{
        ActionHeadScores, ActionTrace, NativeSourceActions, SourceActionBinding,
        POLICY as ACTION_POLICY,
    },
    geometric_source_emission_view::SourceEmissionView,
    invalid, sha256_bytes, Result,
};

pub const SCHEMA: &str = "uor-r4.geometric-source-realizer/1";
pub const SOURCE_SCHEMA: &str = "uor-r4.geometric-source-realizer-source/1";
pub const POLICY: &str = "parent-free-source-view-Copy-Period-Stop;current-packed-H4-context+potential+Stop+independent-Period;sum-head-logits-before-one-native-normalization;token-alias-mass-sum;no-authored-phase-or-length;128-sequence/1";
pub const SURROGATE: &str = "native-joint-Q31-token-probability-forward;softmax-summed-head-Q24-score-adjoint;quarter-grid-coefficient-STE+frozen-coefficient-context-credit;subtraction-first-zero-forward;target-loss-only;prepared-source-immutable-until-drop/1";
const SOURCE_FILES: [&str; 9] = [
    "tokenizer.json",
    "consumer/context-config.json",
    "consumer/context.safetensors",
    "consumer/potential/metadata.json",
    "consumer/potential/potential-q4-parameters.safetensors",
    "consumer/no-read/metadata.json",
    "consumer/no-read/no-read-parameters.safetensors",
    "period/metadata.json",
    "period/no-read-parameters.safetensors",
];
const NATIVE_FILES: [&str; 7] = [
    "tokenizer.json",
    "period-q4.bin",
    "consumer/metadata.json",
    "consumer/context-q4.bin",
    "consumer/potential-q4.bin",
    "consumer/no-read-q4.bin",
    "consumer/exp-q31.bin",
];

pub struct SourceRealizerWeights {
    consumer: ConsumerWeights,
    period: NoReadWeights,
    binding: SourceActionBinding,
    tokenizer_bytes: Vec<u8>,
    period_seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterIdentity {
    shape: Vec<usize>,
    f32_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    source_view_policy: String,
    action_policy: String,
    tokenizer_sha256: String,
    period: NoReadConfig,
    period_seed: u64,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    source_view_policy: String,
    action_policy: String,
    identity: ConsumerIdentity,
    period: NoReadConfig,
    source_parameters: BTreeMap<String, ParameterIdentity>,
    files: BTreeMap<String, String>,
}

impl SourceRealizerWeights {
    /// Consume the admitted consumer source and initialize only the additional
    /// Period coefficients. The explicit seed has no answer/data dependence.
    pub fn new(
        consumer: ConsumerWeights,
        tokenizer_bytes: &[u8],
        period_seed: u64,
    ) -> Result<Self> {
        let c = consumer.config();
        let period = NoReadWeights::new(c.vocab_size, c.heads, c.lanes_per_head)?;
        let mut state = if period_seed == 0 {
            0x9e3779b97f4a7c15
        } else {
            period_seed
        };
        super::set_seeded(period.parameters(), &mut state)?;
        let result = Self {
            consumer,
            period,
            binding: SourceActionBinding::new(tokenizer_bytes)?,
            tokenizer_bytes: tokenizer_bytes.to_vec(),
            period_seed,
        };
        result.validate()?;
        Ok(result)
    }

    fn validate(&self) -> Result<()> {
        let c = self.consumer.config();
        let expected = NoReadConfig {
            vocabulary: c.vocab_size,
            heads: c.heads,
            latent_lanes_per_head: c.lanes_per_head,
        };
        if *self.period.config() != expected
            || *self.consumer.no_read.config() != expected
            || self.binding.vocab_size() != c.vocab_size
            || self.consumer.potential.parent().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || sha256_bytes(&self.tokenizer_bytes) != self.binding.tokenizer_sha256()
        {
            return Err(invalid(
                "realizer tokenizer/source/Period dimensions differ",
            ));
        }
        self.period.packed_coefficients()?;
        self.consumer.context.packed_coefficients()?;
        self.consumer.potential.packed_coefficients()?;
        self.consumer.no_read.packed_coefficients()?;
        Ok(())
    }

    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }

    pub fn parameters(&self) -> BTreeMap<String, Var> {
        self.consumer
            .parameters()
            .into_iter()
            .map(|(name, var)| (format!("consumer.{name}"), var))
            .chain(
                self.period
                    .parameters()
                    .iter()
                    .map(|(name, var)| (format!("period.{name}"), var.clone())),
            )
            .collect()
    }

    pub fn project_quarter_range(&self) -> Result<()> {
        self.consumer.project_quarter_range()?;
        self.period.project_shadow_range()
    }

    pub fn compile(&self, identity: ConsumerIdentity) -> Result<NativeSourceRealizer> {
        self.validate()?;
        if identity.tokenizer_sha256 != self.binding.tokenizer_sha256() {
            return Err(invalid("realizer tokenizer differs from source identity"));
        }
        let consumer = self.consumer.compile_source_view(identity.clone())?;
        let period = self.period.native()?;
        let metadata = NativeMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            surrogate: SURROGATE.into(),
            source_view_policy: SOURCE_VIEW_POLICY.into(),
            action_policy: ACTION_POLICY.into(),
            identity,
            period: *self.period.config(),
            source_parameters: parameter_identities(self)?,
            files: BTreeMap::new(),
        };
        Ok(NativeSourceRealizer {
            consumer,
            period,
            binding: self.binding.clone(),
            tokenizer_bytes: self.tokenizer_bytes.clone(),
            metadata,
        })
    }

    /// Prepare once per optimizer update. All source Vars must remain unchanged
    /// until this object and its loss graphs are dropped; this includes updates
    /// through previously cloned Vars. Recompile after each completed update.
    pub fn prepare<'a>(
        &'a self,
        native: &'a NativeSourceRealizer,
    ) -> Result<PreparedSourceRealizer<'a>> {
        native.validate_source(self)?;
        Ok(PreparedSourceRealizer {
            source: self,
            native,
            consumer: self.consumer.prepare(&native.consumer)?,
        })
    }

    pub fn save_source(&self, path: &Path) -> Result<()> {
        self.validate()?;
        fs::create_dir(path)?;
        self.consumer.save_source(&path.join("consumer"))?;
        self.period.save(&path.join("period"))?;
        fs::write(path.join("tokenizer.json"), &self.tokenizer_bytes)?;
        let metadata = SourceMetadata {
            schema: SOURCE_SCHEMA.into(),
            policy: POLICY.into(),
            surrogate: SURROGATE.into(),
            source_view_policy: SOURCE_VIEW_POLICY.into(),
            action_policy: ACTION_POLICY.into(),
            tokenizer_sha256: self.binding.tokenizer_sha256().into(),
            period: *self.period.config(),
            period_seed: self.period_seed,
            files: snapshot(path, &SOURCE_FILES)?,
        };
        fs::write(
            path.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        Ok(())
    }

    pub fn load_source(path: &Path, tokenizer_bytes: &[u8]) -> Result<Self> {
        let metadata: SourceMetadata =
            serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
        if metadata.schema != SOURCE_SCHEMA
            || metadata.policy != POLICY
            || metadata.surrogate != SURROGATE
            || metadata.source_view_policy != SOURCE_VIEW_POLICY
            || metadata.action_policy != ACTION_POLICY
            || metadata.tokenizer_sha256 != sha256_bytes(tokenizer_bytes)
            || fs::read(path.join("tokenizer.json"))? != tokenizer_bytes
            || snapshot(path, &SOURCE_FILES)? != metadata.files
        {
            return Err(invalid(
                "realizer source policy/tokenizer/file binding differs",
            ));
        }
        let result = Self {
            consumer: ConsumerWeights::load_source(&path.join("consumer"))?,
            period: NoReadWeights::load(&path.join("period"))?,
            binding: SourceActionBinding::new(tokenizer_bytes)?,
            tokenizer_bytes: tokenizer_bytes.to_vec(),
            period_seed: metadata.period_seed,
        };
        result.validate()?;
        if *result.period.config() != metadata.period {
            return Err(invalid("realizer saved Period config differs"));
        }
        Ok(result)
    }
}

fn parameter_identities(
    weights: &SourceRealizerWeights,
) -> Result<BTreeMap<String, ParameterIdentity>> {
    weights
        .parameters()
        .into_iter()
        .map(|(name, value)| {
            let values = value.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|x| !x.is_finite()) {
                return Err(invalid("realizer nonfinite source parameter"));
            }
            let bytes = values
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>();
            Ok((
                name,
                ParameterIdentity {
                    shape: value.dims().to_vec(),
                    f32_sha256: sha256_bytes(&bytes),
                },
            ))
        })
        .collect()
}

/// Verify the exact recursive file set, excluding only this root's own metadata.
/// Never traverse symbolic links or trust file paths supplied by metadata.
fn snapshot(root: &Path, expected: &[&str]) -> Result<BTreeMap<String, String>> {
    fn visit(
        root: &Path,
        here: &Path,
        files: &mut BTreeMap<String, String>,
        dirs: &mut BTreeSet<String>,
    ) -> Result<()> {
        for entry in fs::read_dir(here)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .map_err(|_| invalid("realizer file escaped root"))?
                .to_str()
                .ok_or_else(|| invalid("realizer path is not UTF8"))?
                .replace('\\', "/");
            if ty.is_dir() {
                dirs.insert(rel);
                visit(root, &path, files, dirs)?;
            } else if ty.is_file() {
                if rel != "metadata.json" {
                    files.insert(rel, sha256_bytes(&fs::read(path)?));
                }
            } else {
                return Err(invalid("realizer source contains a non-file/non-directory"));
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    let mut dirs = BTreeSet::new();
    visit(root, root, &mut files, &mut dirs)?;
    let expected_files = expected
        .iter()
        .map(|x| x.to_string())
        .collect::<BTreeSet<_>>();
    let mut expected_dirs = BTreeSet::new();
    for name in expected {
        let mut parent = Path::new(name).parent();
        while let Some(p) = parent.filter(|p| !p.as_os_str().is_empty()) {
            expected_dirs.insert(p.to_string_lossy().replace('\\', "/"));
            parent = p.parent();
        }
    }
    if files.keys().cloned().collect::<BTreeSet<_>>() != expected_files || dirs != expected_dirs {
        return Err(invalid("realizer exact file/directory inventory differs"));
    }
    Ok(files)
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
impl SerializableContextReplay {
    fn matches(&self, trace: &NativeContextTrace) -> bool {
        trace.batch == 1
            && trace.time == self.tokens.len()
            && trace.heads == self.heads
            && trace.lanes_per_head == self.lanes_per_head
            && trace.states == self.states
            && trace.actions == self.actions
            && trace.emitted_roots == self.raw_roots
            && trace.categories == self.categories
            && trace.coefficient_reads == self.coefficient_reads
            && trace
                .codes
                .iter()
                .copied()
                .map(ObservedCode::from)
                .eq(self.codes.iter().cloned())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RealizerTrace {
    pub policy: &'static str,
    pub source: SourceEmissionTrace,
    /// The additional full integer replay needed by this wrapper for Period.
    pub period_context: SerializableContextReplay,
    pub period_q24: Vec<i64>,
    pub actions: ActionTrace,
}

pub struct NativeSourceRealizer {
    consumer: NativeConsumerArtifact,
    period: NativeGeometricNoRead,
    binding: SourceActionBinding,
    tokenizer_bytes: Vec<u8>,
    metadata: NativeMetadata,
}

impl NativeSourceRealizer {
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    fn validate_source(&self, source: &SourceRealizerWeights) -> Result<()> {
        source.validate()?;
        self.consumer.validate_source(&source.consumer)?;
        if self.consumer.metadata.schema != SOURCE_VIEW_SCHEMA
            || self.consumer.metadata.source_view_policy.as_deref() != Some(SOURCE_VIEW_POLICY)
            || self.period.config() != *source.period.config()
            || self.period.packed_coefficients() != source.period.packed_coefficients()?
            || self.metadata.source_parameters != parameter_identities(source)?
            || self.tokenizer_bytes != source.tokenizer_bytes
        {
            return Err(invalid(
                "realizer current source differs from compiled source",
            ));
        }
        Ok(())
    }

    pub fn read(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<RealizerTrace> {
        // This admission rejects unsupported frame status, empty query, wrong
        // tokenizer/view/original IDs, OOV IDs and total sequence >128 first.
        let source = self.consumer.read_source_view(frame, view, query, prefix)?;
        let ids = view
            .emitted_token_ids()
            .iter()
            .chain(query)
            .chain(prefix)
            .copied()
            .collect::<Vec<_>>();
        let c = self.consumer.metadata.context;
        let lanes = c.heads * c.lanes_per_head;
        let mut context = NativeContextState::new(c.heads, c.lanes_per_head)
            .map_err(|e| invalid(e.to_string()))?;
        let mut replay = SerializableContextReplay {
            tokens: ids.clone(),
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            states: Vec::with_capacity(ids.len()),
            actions: Vec::with_capacity(ids.len()),
            raw_roots: Vec::with_capacity(ids.len() * lanes),
            categories: Vec::with_capacity(ids.len() * lanes),
            codes: Vec::with_capacity(ids.len() * lanes),
            coefficient_reads: 0,
        };
        let mut last = None;
        for &token in &ids {
            let step = context
                .step(
                    token as usize,
                    self.consumer.context.native(),
                    &self.consumer.geometry,
                )
                .map_err(|e| invalid(e.to_string()))?;
            replay
                .states
                .push(step.states[..lanes].iter().map(|x| x.index()).collect());
            replay
                .actions
                .push(step.actions[..lanes].iter().map(|x| x.index()).collect());
            replay
                .raw_roots
                .extend(step.readout_roots[..lanes].iter().map(|x| x.index()));
            replay
                .categories
                .extend_from_slice(&step.categories[..lanes]);
            replay
                .codes
                .extend(step.output[..lanes].iter().copied().map(ObservedCode::from));
            replay.coefficient_reads += step.coefficient_reads;
            last = Some((token, step));
        }
        let (token, last) = last.ok_or_else(|| invalid("realizer context sequence is empty"))?;
        let period = self
            .period
            .score(
                token as usize,
                &last.states[..lanes],
                &last.output[..lanes],
                None,
            )
            .map_err(|e| invalid(e.to_string()))?;
        let stop = self
            .consumer
            .no_read
            .score(
                token as usize,
                &last.states[..lanes],
                &last.output[..lanes],
                None,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if source
            .view_kernel_trace
            .heads
            .iter()
            .zip(stop)
            .any(|(h, s)| h.no_read_q24 != s)
        {
            return Err(invalid(
                "realizer additional context replay Stop score differs",
            ));
        }
        let mut reducer =
            NativeSourceActions::new(self.binding.clone(), c.heads, &self.consumer.exp)?;
        let head_scores = source
            .view_kernel_trace
            .heads
            .iter()
            .enumerate()
            .map(|(head, scores)| ActionHeadScores {
                copy_q24: &scores.scores_q24,
                period_q24: period[head],
                stop_q24: scores.no_read_q24,
            })
            .collect::<Vec<_>>();
        let actions = reducer.reduce(view.emitted_token_ids(), &head_scores)?;
        Ok(RealizerTrace {
            policy: POLICY,
            source,
            period_context: replay,
            period_q24: period[..c.heads].to_vec(),
            actions,
        })
    }

    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({"consumer":self.consumer.stats(),"period":self.period.stats(),
            "extra_context_replays_per_read":1,"final_joint_reductions_per_read":1,
            "inherited_per_head_reductions_discarded":self.consumer.metadata.context.heads,
            "scope":"native integer components inside allocating wrapper; no parent-model inference, no complete-path opcode/allocation qualification"})
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        self.consumer.save(&path.join("consumer"))?;
        fs::write(path.join("tokenizer.json"), &self.tokenizer_bytes)?;
        fs::write(
            path.join("period-q4.bin"),
            self.period.packed_coefficients(),
        )?;
        let mut metadata = self.metadata.clone();
        metadata.files = snapshot(path, &NATIVE_FILES)?;
        fs::write(
            path.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        Ok(())
    }

    pub fn load(
        path: &Path,
        source: &SourceRealizerWeights,
        identity: &ConsumerIdentity,
    ) -> Result<Self> {
        identity.validate()?;
        source.validate()?;
        let metadata: NativeMetadata =
            serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
        let tokenizer_bytes = fs::read(path.join("tokenizer.json"))?;
        if metadata.schema != SCHEMA
            || metadata.policy != POLICY
            || metadata.surrogate != SURROGATE
            || metadata.source_view_policy != SOURCE_VIEW_POLICY
            || metadata.action_policy != ACTION_POLICY
            || metadata.identity != *identity
            || metadata.period != *source.period.config()
            || tokenizer_bytes != source.tokenizer_bytes
            || sha256_bytes(&tokenizer_bytes) != identity.tokenizer_sha256
            || metadata.source_parameters != parameter_identities(source)?
            || snapshot(path, &NATIVE_FILES)? != metadata.files
        {
            return Err(invalid(
                "realizer native/source/tokenizer policy or file identity differs",
            ));
        }
        let packed = fs::read(path.join("period-q4.bin"))?;
        if packed != source.period.packed_coefficients()? {
            return Err(invalid("realizer exported Period differs from source"));
        }
        // Execute admitted saved bytes, not a freshly compiled replacement.
        let result = Self {
            consumer: NativeConsumerArtifact::load(
                &path.join("consumer"),
                &source.consumer,
                identity,
            )?,
            period: NativeGeometricNoRead::new(metadata.period, &packed)
                .map_err(|e| invalid(e.to_string()))?,
            binding: SourceActionBinding::new(&tokenizer_bytes)?,
            tokenizer_bytes,
            metadata,
        };
        result.validate_source(source)?;
        Ok(result)
    }
}

pub struct RealizerLoss {
    pub loss: Tensor,
    pub trace: RealizerTrace,
    pub target_probability: f64,
}
pub struct PreparedSourceRealizer<'a> {
    source: &'a SourceRealizerWeights,
    native: &'a NativeSourceRealizer,
    consumer: PreparedConsumerStep<'a>,
}

impl PreparedSourceRealizer<'_> {
    pub fn loss(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        target: u32,
    ) -> Result<RealizerLoss> {
        if target as usize >= self.source.binding.vocab_size() {
            return Err(invalid("realizer target is out of vocabulary"));
        }
        let trace = self.native.read(frame, view, query, prefix)?;
        let mass = trace
            .actions
            .token_masses
            .iter()
            .find(|m| m.token_id == target)
            .map_or(0, |m| m.weight_q31);
        if mass == 0 {
            return Err(invalid(
                "realizer target has zero native action mass; no parent or probability floor",
            ));
        }
        let target_probability = mass as f64 / trace.actions.total_weight_q31 as f64;
        let ids = &trace.period_context.tokens;
        let time = ids.len();
        let n = view.emitted_token_ids().len();
        let c = self.source.consumer.config();
        let width = c.heads * c.lanes_per_head;
        let context = self.consumer.context.forward(ids, 1, time, false)?;
        if !trace.period_context.matches(&context.trace) {
            return Err(invalid("realizer native/training context replay differs"));
        }
        let absent = AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?;
        let content = vec![absent; time * width];
        let copy_coeff = self.source.consumer.potential.forward_codes(
            1,
            time,
            &content,
            &context.trace.codes,
        )?;
        let copy_context =
            frozen_potential_forward(&self.source.consumer.potential, &content, &context)?;
        if copy_coeff.scores_q24 != copy_context.scores_q24 {
            return Err(invalid("realizer Copy credit hard scores differ"));
        }
        let copy = (&copy_coeff.scores + (&copy_context.scores - copy_context.scores.detach())?)?;
        let latent = context
            .trace
            .states
            .iter()
            .flatten()
            .map(|&x| H4Code::try_from(x).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let held = vec![H4Code::IDENTITY; latent.len()];
        let valid = vec![false; time];
        let batch = NoReadBatch {
            ids,
            batch: 1,
            time,
            latent: &latent,
            observed: &context.trace.codes,
            held: &held,
            span_valid: &valid,
        };
        let period_coeff = self.source.period.forward(batch)?;
        let stop_coeff = self.source.consumer.no_read.forward(batch)?;
        let period_context =
            frozen_no_read_forward(&self.source.period, ids, &context, &held, &valid)?;
        let stop_context =
            frozen_no_read_forward(&self.source.consumer.no_read, ids, &context, &held, &valid)?;
        let mut heads = Vec::with_capacity(c.heads);
        for h in 0..c.heads {
            let hard = &trace.source.view_kernel_trace.heads[h];
            let at = h * time + time - 1;
            if hard
                .scores_q24
                .iter()
                .enumerate()
                .any(|(k, &s)| s != copy_coeff.scores_q24[at * time + k])
                || hard.no_read_q24 != stop_context.scores_q24[at]
                || trace.period_q24[h] != period_context.scores_q24[at]
            {
                return Err(invalid(
                    "realizer native/training Copy/Period/Stop Q24 differs",
                ));
            }
            let period = scalar_credit(
                trace.period_q24[h],
                &period_coeff.i((0, h, time - 1))?,
                &period_context.scores.i((0, h, time - 1))?,
            )?;
            let stop = scalar_credit(
                hard.no_read_q24,
                &stop_coeff.i((0, h, time - 1))?,
                &stop_context.scores.i((0, h, time - 1))?,
            )?;
            heads.push(Tensor::cat(
                &[
                    copy.i((0, h, time - 1))?.narrow(0, 0, n)?,
                    period.reshape(1)?,
                    stop.reshape(1)?,
                ],
                0,
            )?);
        }
        let summed = Tensor::stack(&heads, 0)?.sum(0)?;
        if summed.to_vec1::<f32>()?.iter().any(|x| !x.is_finite()) {
            return Err(invalid("realizer summed-score surrogate is nonfinite"));
        }
        let hard = Tensor::from_vec(
            trace
                .actions
                .actions
                .iter()
                .map(|a| (a.score_q24 as f64 / 16_777_216.) as f32)
                .collect::<Vec<_>>(),
            n + 2,
            &Device::Cpu,
        )?;
        let joint = (&hard + (&summed - summed.detach())?)?;
        let soft = candle_nn::ops::softmax(&joint, 0)?;
        let mask = Tensor::from_vec(
            trace
                .actions
                .actions
                .iter()
                .map(|a| if a.token_id == target { 1f32 } else { 0f32 })
                .collect::<Vec<_>>(),
            n + 2,
            &Device::Cpu,
        )?;
        let surrogate_probability = (soft * mask)?.sum_all()?;
        // Round the *aggregated integer token mass* once at this floating loss
        // boundary, rather than separately rounding each alias probability.
        let hard_probability = Tensor::new(target_probability as f32, &Device::Cpu)?;
        let probability =
            (&hard_probability + (&surrogate_probability - surrogate_probability.detach())?)?;
        if !probability.to_scalar::<f32>()?.is_finite() || probability.to_scalar::<f32>()? <= 0. {
            return Err(invalid(
                "realizer native probability cannot be represented by loss",
            ));
        }
        Ok(RealizerLoss {
            loss: probability.log()?.neg()?,
            trace,
            target_probability,
        })
    }
}

fn scalar_credit(hard_q24: i64, coefficient: &Tensor, context: &Tensor) -> Result<Tensor> {
    if !coefficient.to_scalar::<f32>()?.is_finite() || !context.to_scalar::<f32>()?.is_finite() {
        return Err(invalid("realizer action score surrogate is nonfinite"));
    }
    let hard = Tensor::new((hard_q24 as f64 / 16_777_216.) as f32, &Device::Cpu)?;
    Ok((&hard + ((coefficient - coefficient.detach())? + (context - context.detach())?)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_source_emission_view::SourceEmissionCompiler;
    use std::sync::atomic::{AtomicU64, Ordering};
    use uor_r4_integer::geometric_occurrence_read::{FrameMetadata, FrameStatus, SourceIdentity};
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"Ġ":6,"Ġa":7},"merges":["Ġ a"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        path: std::path::PathBuf,
        weights: SourceRealizerWeights,
        identity: ConsumerIdentity,
    }
    impl Fixture {
        fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!(
                "uor-source-realizer-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            let consumer =
                ConsumerWeights::initialize(&path.join("base"), TOK.as_bytes(), 8, 1, 1, 41)?;
            let weights = SourceRealizerWeights::new(consumer, TOK.as_bytes(), 43)?;
            // Uniform hard scores give exact, independently known alias loss
            // and coefficient adjoint signs, without a model training dose.
            for var in weights.parameters().values() {
                var.set(&Tensor::zeros_like(var.as_tensor())?)?;
            }
            let identity = ConsumerIdentity {
                tokenizer_sha256: sha256_bytes(TOK.as_bytes()),
                parent_checkpoint_manifest_sha256: "a".repeat(64),
                parent_model_sha256: "b".repeat(64),
                parent_config_sha256: "c".repeat(64),
            };
            Ok(Self {
                path,
                weights,
                identity,
            })
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
    fn frame(ids: &[u32]) -> SelectedRecordFrame<'_> {
        SelectedRecordFrame {
            identity: SourceIdentity {
                record: 7,
                commit: 9,
            },
            metadata: FrameMetadata {
                scope: b"fixture",
                entity: &[],
                relation: 3,
                view: 0,
                status: FrameStatus::Found,
            },
            token_ids: ids,
        }
    }

    #[test]
    fn source_realizer_copy_period_stop_loss_aliases_and_repeated_backward() -> Result<()> {
        let fixture = Fixture::new()?;
        let w = &fixture.weights;
        let native = w.compile(fixture.identity.clone())?;
        let original = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&original)?;
        assert_eq!(view.emitted_token_ids(), &[7, 4, 4]);
        let expected = native.read(frame(&original), &view, &[5], &[])?;
        let prepared = w.prepare(&native)?;
        for (target, p) in [(4, 0.4f64), (3, 0.2), (1, 0.2)] {
            let output = prepared.loss(frame(&original), &view, &[5], &[], target)?;
            assert_eq!(output.trace, expected);
            assert_eq!(output.target_probability, p);
            assert!((f64::from(output.loss.to_scalar::<f32>()?) + p.ln()).abs() < 1e-6);
            let grad = output.loss.backward()?;
            for (name, selected) in [
                ("period.coefficients", target == 3),
                ("consumer.no_read.coefficients", target == 1),
            ] {
                let variable = w
                    .parameters()
                    .remove(name)
                    .ok_or_else(|| invalid("fixture parameter absent"))?;
                let bias = grad
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid("realizer action gradient disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?[0];
                let expected = if selected { -0.8 } else { 0.2 };
                assert!(
                    (bias - expected).abs() < 1e-5,
                    "{name} target{target}: {bias}"
                );
            }
        }
        assert!(prepared
            .loss(frame(&original), &view, &[5], &[], 5)
            .is_err());
        // A changed output prefix is consumed by context and never added as a
        // copy candidate. No forced end/period phase is supplied to this API.
        let later = native.read(frame(&original), &view, &[5], &[3, 7])?;
        assert_eq!(later.period_context.tokens, [7, 4, 4, 5, 3, 7]);
        assert_eq!(later.actions.actions.len(), expected.actions.actions.len());
        drop(prepared);
        // Isolate Period-to-context credit: Copy and Stop coefficients stay
        // zero, and only one retained-root coordinate enters Period. Its
        // coefficient is fixed before taking ordinary Period-token CE.
        let period = w
            .period
            .parameters()
            .get("coefficients")
            .ok_or_else(|| invalid("fixture Period absent"))?;
        let mut values = period.flatten_all()?.to_vec1::<f32>()?;
        values[1 + w.consumer.config().vocab_size + 1] = 0.25;
        period.set(&Tensor::from_vec(values, period.shape(), &Device::Cpu)?)?;
        let current = w.compile(fixture.identity.clone())?;
        let output = w
            .prepare(&current)?
            .loss(frame(&original), &view, &[5], &[], 3)?;
        let grad = output.loss.backward()?;
        let transition = w
            .parameters()
            .remove("consumer.context.token_transition")
            .ok_or_else(|| invalid("fixture transition absent"))?;
        let credit = grad
            .get(transition.as_tensor())
            .ok_or_else(|| invalid("Period-to-context credit disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(credit[7 * 120..8 * 120].iter().any(|x| x.abs() > 1e-9));
        Ok(())
    }

    #[test]
    fn source_realizer_source_and_native_reload_reject_tamper_and_stale_shadows() -> Result<()> {
        let fixture = Fixture::new()?;
        let source_path = fixture.path.join("source");
        let native_path = fixture.path.join("native");
        fixture.weights.save_source(&source_path)?;
        let w = SourceRealizerWeights::load_source(&source_path, TOK.as_bytes())?;
        let native = w.compile(fixture.identity.clone())?;
        native.save(&native_path)?;
        let loaded = NativeSourceRealizer::load(&native_path, &w, &fixture.identity)?;
        let original = [4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&original)?;
        assert_eq!(
            loaded.read(frame(&original), &view, &[5], &[])?,
            native.read(frame(&original), &view, &[5], &[])?
        );
        let file = native_path.join("period-q4.bin");
        let bytes = fs::read(&file)?;
        let mut changed = bytes.clone();
        changed[0] ^= 1;
        fs::write(&file, &changed)?;
        // Even a resealed file digest cannot authorize coefficients absent
        // from the independently loaded source.
        let meta_path = native_path.join("metadata.json");
        let meta_bytes = fs::read(&meta_path)?;
        let mut meta: NativeMetadata = serde_json::from_slice(&meta_bytes)?;
        meta.files
            .insert("period-q4.bin".into(), sha256_bytes(&changed));
        fs::write(&meta_path, serde_json::to_vec(&meta)?)?;
        assert!(NativeSourceRealizer::load(&native_path, &w, &fixture.identity).is_err());
        fs::write(&file, bytes)?;
        fs::write(&meta_path, &meta_bytes)?;
        fs::write(native_path.join("unexpected.bin"), b"extra")?;
        assert!(NativeSourceRealizer::load(&native_path, &w, &fixture.identity).is_err());
        fs::remove_file(native_path.join("unexpected.bin"))?;
        let period = w
            .period
            .parameters()
            .get("coefficients")
            .ok_or_else(|| invalid("fixture Period absent"))?;
        let mut values = period.flatten_all()?.to_vec1::<f32>()?;
        values[0] = 0.01;
        period.set(&Tensor::from_vec(values, period.shape(), &Device::Cpu)?)?;
        assert!(w.prepare(&native).is_err()); // Same q4 bin still has changed source bits.
        let fresh = w.compile(fixture.identity.clone())?;
        w.prepare(&fresh)?;
        let source_meta = source_path.join("metadata.json");
        let mut metadata: SourceMetadata = serde_json::from_slice(&fs::read(&source_meta)?)?;
        metadata.policy = "unknown".into();
        fs::write(source_meta, serde_json::to_vec(&metadata)?)?;
        assert!(SourceRealizerWeights::load_source(&source_path, TOK.as_bytes()).is_err());
        Ok(())
    }
}
