//! Offline answer-credit bridge for a native selected-record occurrence reader.
//!
//! The integer reader supplies exact Q31 occurrence/NoRead weights. NoRead
//! carries the complete frozen parent distribution (including its pointer).
//! Targets enter only the loss. The hard forward uses native weights; backward
//! uses the declared soft-normalization and existing q4 context/potential STEs.
//! Mixing with a floating parent is a research boundary, not native chat.

use std::{collections::BTreeMap, fs, path::Path};

use candle_core::{Device, IndexOp, Tensor, Var};
use serde::{Deserialize, Serialize};
use uor_r4_integer::{
    geometric_context_q4::{ContextQ4Config, NativeContextQ4},
    geometric_no_read::NativeGeometricNoRead,
    geometric_occurrence_read::{
        NativeOccurrenceReader, OccurrenceComponents, SelectedRecordFrame,
    },
    geometric_potential::AddressLane,
    geometric_potential_q4::NativePotentialQ4,
    geometric_read::{EXP_TABLE_LEN, WEIGHT_ONE},
    h4_tables::{H4Code, HistoricalH4Tables},
};

use crate::{
    geometric_address::GeometricAddressConfig,
    geometric_context::{ContextWeights, GeometricContextConfig, PreparedContextQ4},
    geometric_context_credit::{frozen_no_read_forward, frozen_potential_forward},
    geometric_no_read::{NoReadBatch, NoReadWeights},
    geometric_potential_q4::PotentialQ4Weights,
    geometric_source_emission_view::{SourceEmissionView, POLICY as SOURCE_VIEW_POLICY},
    geometric_stack::{ReadIdentityLatch, ReadScore, StackArch, StackConfig, StackModel},
    invalid, sha256_bytes, Result,
};

#[path = "geometric_source_realizer.rs"]
pub mod source_realizer;

pub const SCHEMA: &str = "uor-r4.geometric-occurrence-consumer/1";
pub const SOURCE_VIEW_SCHEMA: &str = "uor-r4.geometric-occurrence-consumer/2";
pub const SURROGATE: &str = "native-Q31-normalized-forward;softmax-score-adjoint;hard-quarter-source-STE;coefficient-and-frozen-context-input-credit;whole-parent-NoRead-fallback;target-loss-only/1";
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
const NATIVE_FILES: [&str; 4] = [
    "context-q4.bin",
    "potential-q4.bin",
    "no-read-q4.bin",
    "exp-q31.bin",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumerIdentity {
    pub tokenizer_sha256: String,
    pub parent_checkpoint_manifest_sha256: String,
    pub parent_model_sha256: String,
    pub parent_config_sha256: String,
}
impl ConsumerIdentity {
    pub fn new(tokenizer: &[u8], checkpoint: &Path) -> Result<Self> {
        if tokenizer.is_empty() {
            return Err(invalid("consumer tokenizer is empty"));
        }
        let loaded = crate::stack_checkpoint::load_checkpoint(checkpoint, &Device::Cpu)
            .map_err(|e| invalid(e.to_string()))?;
        loaded
            .identity()
            .check_tokenizer(tokenizer)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            tokenizer_sha256: sha256_bytes(tokenizer),
            parent_checkpoint_manifest_sha256: crate::stack_checkpoint::sealed_manifest_sha256(
                checkpoint,
            )
            .map_err(|e| invalid(e.to_string()))?,
            parent_model_sha256: sha256_bytes(&fs::read(checkpoint.join("model.safetensors"))?),
            parent_config_sha256: sha256_bytes(&fs::read(checkpoint.join("config.json"))?),
        })
    }
    fn validate(&self) -> Result<()> {
        for value in [
            &self.tokenizer_sha256,
            &self.parent_checkpoint_manifest_sha256,
            &self.parent_model_sha256,
            &self.parent_config_sha256,
        ] {
            if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(invalid("consumer identity digest differs"));
            }
        }
        Ok(())
    }
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

pub struct ConsumerWeights {
    context: ContextWeights,
    potential: PotentialQ4Weights,
    no_read: NoReadWeights,
}
pub struct ConsumerLoss {
    pub loss: Tensor,
    pub trace: OccurrenceTrace,
    /// Normalized floating reference log probabilities, not native serving.
    pub mixed_scores: Vec<f32>,
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
pub struct SourceEmissionLoss {
    pub loss: Tensor,
    pub trace: SourceEmissionTrace,
    pub mixed_scores: Vec<f32>,
}
fn source_view_trace(
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

fn set_seeded(parameters: &BTreeMap<String, Var>, state: &mut u64) -> Result<()> {
    // Construction-only legal q4 coefficients, independent of data/answers.
    // These are not transferred synthetic language weights or a fitted model.
    for variable in parameters.values() {
        let mut values = Vec::with_capacity(variable.elem_count());
        for _ in 0..variable.elem_count() {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            values.push((((*state >> 32) % 3) as i32 - 1) as f32 * 0.25);
        }
        variable.set(&Tensor::from_vec(values, variable.shape(), &Device::Cpu)?)?;
    }
    Ok(())
}

impl ConsumerWeights {
    /// Fresh BPE-sized construction initializer. The small float Stack is only
    /// an existing potential-source constructor; it never authors responses.
    pub fn initialize(
        base: &Path,
        tokenizer: &[u8],
        vocabulary: usize,
        heads: usize,
        lanes: usize,
        seed: u64,
    ) -> Result<Self> {
        let width = heads
            .checked_mul(lanes)
            .and_then(|x| x.checked_mul(4))
            .ok_or_else(|| invalid("consumer dimensions overflow"))?;
        let context = ContextWeights::new(vocabulary, width, heads, seed)?.into_q4()?;
        fs::create_dir(base)?;
        let mut config = StackConfig::transformer_control(seed);
        config.arch = StackArch::Geometric;
        config.vocab_size = vocabulary;
        config.width = width;
        config.heads = heads;
        config.mlp_hidden = 32;
        config.context = 128;
        config.pattern = "rra".into();
        config.read = ReadScore::Lorentz;
        config.rotation = true;
        let mut initializer = StackModel::new(config, &Device::Cpu)?;
        // Required only by the existing potential-source constructor's admission.
        // This Stack is never evaluated; the consumer has no held-span channel.
        initializer.set_read_identity_latch(ReadIdentityLatch::Held)?;
        initializer.set_geometric_address(GeometricAddressConfig::new(width, heads)?)?;
        initializer.save(base)?;
        let potential = PotentialQ4Weights::from_base(base, tokenizer)?;
        let no_read = NoReadWeights::new(vocabulary, heads, lanes)?;
        let mut state = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
        set_seeded(context.parameters(), &mut state)?;
        set_seeded(potential.parameters(), &mut state)?;
        set_seeded(no_read.parameters(), &mut state)?;
        Ok(Self {
            context,
            potential,
            no_read,
        })
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        let mut result = BTreeMap::new();
        for (prefix, map) in [
            ("context", self.context.parameters()),
            ("potential", self.potential.parameters()),
            ("no_read", self.no_read.parameters()),
        ] {
            for (name, variable) in map {
                result.insert(format!("{prefix}.{name}"), variable.clone());
            }
        }
        result
    }
    fn config(&self) -> ContextQ4Config {
        let c = self.context.config();
        ContextQ4Config {
            vocab_size: c.vocab_size,
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
        }
    }
    pub fn compile(&self, identity: ConsumerIdentity) -> Result<NativeConsumerArtifact> {
        identity.validate()?;
        if self.potential.parent().tokenizer_sha256 != identity.tokenizer_sha256 {
            return Err(invalid(
                "consumer source tokenizer differs from admitted parent identity",
            ));
        }
        NativeConsumerArtifact::from_parts(
            self.config(),
            *self.potential.config(),
            *self.no_read.config(),
            self.context.packed_coefficients()?,
            self.potential.packed_coefficients()?,
            self.no_read.packed_coefficients()?,
            canonical_exp(),
            identity,
        )
    }
    /// Same learned numeric operators, with a distinct admitted lexical view.
    pub fn compile_source_view(
        &self,
        identity: ConsumerIdentity,
    ) -> Result<NativeConsumerArtifact> {
        let mut native = self.compile(identity)?;
        native.metadata.schema = SOURCE_VIEW_SCHEMA.into();
        native.metadata.source_view_policy = Some(SOURCE_VIEW_POLICY.into());
        Ok(native)
    }
    pub fn save_source(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        fs::write(
            path.join("context-config.json"),
            serde_json::to_vec_pretty(self.context.config())?,
        )?;
        let tensors = self
            .context
            .parameters()
            .iter()
            .map(|(n, v)| (n.clone(), v.as_tensor().clone()))
            .collect::<std::collections::HashMap<_, _>>();
        candle_core::safetensors::save(&tensors, path.join("context.safetensors"))?;
        self.potential.save(&path.join("potential"))?;
        self.no_read.save(&path.join("no-read"))?;
        Ok(())
    }
    pub fn load_source(path: &Path) -> Result<Self> {
        let config: GeometricContextConfig =
            serde_json::from_slice(&fs::read(path.join("context-config.json"))?)?;
        config.validate()?;
        let context =
            ContextWeights::new(config.vocab_size, config.width, config.heads, config.seed)?
                .into_q4()?;
        if context.config() != &config {
            return Err(invalid("consumer context source policy differs"));
        }
        let mut values =
            candle_core::safetensors::load(path.join("context.safetensors"), &Device::Cpu)?;
        if values.len() != context.parameters().len() {
            return Err(invalid("consumer context source inventory differs"));
        }
        for (name, var) in context.parameters() {
            let value = values
                .remove(name)
                .ok_or_else(|| invalid("consumer context source family absent"))?;
            if value.dims() != var.dims() {
                return Err(invalid("consumer context source shape differs"));
            }
            var.set(&value)?;
        }
        context.packed_coefficients()?;
        Ok(Self {
            context,
            potential: PotentialQ4Weights::load(&path.join("potential"))?,
            no_read: NoReadWeights::load(&path.join("no-read"))?,
        })
    }

    /// Admit source/native identity and prepare immutable graphs once per update.
    /// Drop this value before modifying any parameter returned by `parameters`.
    pub fn prepare<'a>(
        &'a self,
        native: &'a NativeConsumerArtifact,
    ) -> Result<PreparedConsumerStep<'a>> {
        native.validate_source(self)?;
        Ok(PreparedConsumerStep {
            source: self,
            native,
            context: self.context.prepare_q4(&native.context)?,
        })
    }

    pub fn project_quarter_range(&self) -> Result<()> {
        self.context.project_shadow_range()?;
        self.potential.project_shadow_range()?;
        self.no_read.project_shadow_range()
    }

    /// One ordinary answer-token CE. Target enters only the loss.
    pub fn loss(
        &self,
        frame: SelectedRecordFrame<'_>,
        query: &[u32],
        prefix: &[u32],
        parent: &[f32],
        target: u32,
        native: &NativeConsumerArtifact,
    ) -> Result<ConsumerLoss> {
        self.prepare(native)?
            .loss(frame, query, prefix, parent, target)
    }
}

/// Update-local graph reuse; source variables must remain unchanged until dropped.
pub struct PreparedConsumerStep<'a> {
    source: &'a ConsumerWeights,
    native: &'a NativeConsumerArtifact,
    context: PreparedContextQ4<'a>,
}
impl PreparedConsumerStep<'_> {
    pub fn loss(
        &self,
        frame: SelectedRecordFrame<'_>,
        query: &[u32],
        prefix: &[u32],
        parent: &[f32],
        target: u32,
    ) -> Result<ConsumerLoss> {
        self.native.require_raw_source()?;
        self.loss_kernel(frame, query, prefix, parent, target)
    }
    pub fn loss_source_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        parent: &[f32],
        target: u32,
    ) -> Result<SourceEmissionLoss> {
        self.native.require_source_view(view)?;
        let derived = view.derived_frame(frame)?;
        let out = self.loss_kernel(derived, query, prefix, parent, target)?;
        Ok(SourceEmissionLoss {
            loss: out.loss,
            trace: source_view_trace(frame, view, out.trace),
            mixed_scores: out.mixed_scores,
        })
    }
    fn loss_kernel(
        &self,
        frame: SelectedRecordFrame<'_>,
        query: &[u32],
        prefix: &[u32],
        parent: &[f32],
        target: u32,
    ) -> Result<ConsumerLoss> {
        if parent.len() != self.source.config().vocab_size || target as usize >= parent.len() {
            return Err(invalid("consumer parent/target vocabulary differs"));
        }
        let trace = self.native.read_kernel(frame, query, prefix)?;
        let mixed_scores = mix_scores(parent, &trace, true)?;
        let probabilities = normalized_parent(parent)?;
        let ids = frame
            .token_ids
            .iter()
            .chain(query)
            .chain(prefix)
            .copied()
            .collect::<Vec<_>>();
        let time = ids.len();
        let heads = self.source.config().heads;
        let lanes = self.source.config().lanes_per_head;
        let context = self.context.forward(&ids, 1, time, false)?;
        let absent = AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?;
        let content = vec![absent; time * heads * lanes];
        let coefficient =
            self.source
                .potential
                .forward_codes(1, time, &content, &context.trace.codes)?;
        let input_credit = frozen_potential_forward(&self.source.potential, &content, &context)?;
        if coefficient.scores_q24 != input_credit.scores_q24 {
            return Err(invalid("consumer potential credit hard traces differ"));
        }
        let scores =
            (&coefficient.scores + (&input_credit.scores - input_credit.scores.detach())?)?;
        let latent = context
            .trace
            .states
            .iter()
            .flatten()
            .map(|&x| H4Code::try_from(x).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let held = vec![H4Code::IDENTITY; latent.len()];
        let valid = vec![false; time];
        let null_coeff = self.source.no_read.forward(NoReadBatch {
            ids: &ids,
            batch: 1,
            time,
            latent: &latent,
            observed: &context.trace.codes,
            held: &held,
            span_valid: &valid,
        })?;
        let null_context =
            frozen_no_read_forward(&self.source.no_read, &ids, &context, &held, &valid)?;
        let mut target_probabilities = Vec::with_capacity(heads);
        let n = frame.token_ids.len();
        for head in 0..heads {
            let hard = &trace.heads[head];
            if hard
                .scores_q24
                .iter()
                .enumerate()
                .any(|(k, s)| *s != coefficient.scores_q24[(head * time + time - 1) * time + k])
                || hard.no_read_q24 != null_context.scores_q24[head * time + time - 1]
            {
                return Err(invalid("consumer native/training hard score differs"));
            }
            let row = scores.i((0, head, time - 1))?.narrow(0, 0, n)?;
            let hard_null = Tensor::new(hard.no_read_q24 as f32 / 16_777_216., &Device::Cpu)?;
            let coefficient_null = null_coeff.i((0, head, time - 1))?;
            let context_null = null_context.scores.i((0, head, time - 1))?;
            let null = (&hard_null + (&coefficient_null - coefficient_null.detach())?)?;
            let null = (&null + (&context_null - context_null.detach())?)?.reshape(1)?;
            let all_scores = Tensor::cat(&[row, null], 0)?;
            let soft = candle_nn::ops::softmax(&all_scores, 0)?;
            let alpha = hard
                .weights_q31
                .iter()
                .chain(std::iter::once(&hard.no_read_weight_q31))
                .map(|&x| (x as f64 / hard.total_weight_q31 as f64) as f32)
                .collect::<Vec<_>>();
            let hard_alpha = Tensor::from_vec(alpha, n + 1, &Device::Cpu)?;
            let alpha = (&hard_alpha + (&soft - soft.detach())?)?;
            let mut target_mass = frame
                .token_ids
                .iter()
                .map(|&id| if id == target { 1. } else { 0. })
                .collect::<Vec<f32>>();
            target_mass.push(probabilities[target as usize] as f32);
            let mass = Tensor::from_vec(target_mass, n + 1, &Device::Cpu)?;
            target_probabilities.push((&alpha * mass)?.sum_all()?);
        }
        let probability = (Tensor::stack(&target_probabilities, 0)?.sum_all()? / heads as f64)?;
        let p = probability.to_scalar::<f32>()?;
        if !p.is_finite() || p <= 0. {
            return Err(invalid(
                "consumer F32 answer probability underflow/nonfinite",
            ));
        }
        Ok(ConsumerLoss {
            loss: probability.log()?.neg()?,
            trace,
            mixed_scores,
        })
    }
}

fn canonical_exp() -> Vec<u32> {
    (0..EXP_TABLE_LEN)
        .map(|i| ((-(i as f64) / 256.).exp() * WEIGHT_ONE as f64).round() as u32)
        .collect()
}
fn normalized_parent(scores: &[f32]) -> Result<Vec<f64>> {
    if scores.is_empty() || scores.iter().any(|x| !x.is_finite()) {
        return Err(invalid("consumer parent scores must be finite/nonempty"));
    }
    let maximum = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let mut p = scores
        .iter()
        .map(|&x| (f64::from(x) - maximum).exp())
        .collect::<Vec<_>>();
    let total: f64 = p.iter().sum();
    if !total.is_finite() || total <= 0. {
        return Err(invalid("consumer parent normalization differs"));
    }
    for x in &mut p {
        *x /= total;
    }
    Ok(p)
}
/// Floating reference boundary. Disabled returns the exact original scores.
pub fn mix_scores(parent: &[f32], trace: &OccurrenceTrace, enabled: bool) -> Result<Vec<f32>> {
    if !enabled {
        return Ok(parent.to_vec());
    }
    let p = normalized_parent(parent)?;
    if trace.heads.is_empty() {
        return Err(invalid("consumer head trace empty"));
    }
    let mut mixture = vec![0.; parent.len()];
    for head in &trace.heads {
        let sum = head
            .weights_q31
            .iter()
            .try_fold(head.no_read_weight_q31, |a, &b| a.checked_add(b))
            .ok_or_else(|| invalid("consumer trace denominator overflow"))?;
        if head.weights_q31.len() != trace.occurrences.len()
            || head.total_weight_q31 == 0
            || sum != head.total_weight_q31
        {
            return Err(invalid(
                "consumer trace denominator/occurrence shape differs",
            ));
        }
        let scale = 1. / head.total_weight_q31 as f64 / trace.heads.len() as f64;
        for (out, &value) in mixture.iter_mut().zip(&p) {
            *out += head.no_read_weight_q31 as f64 * scale * value;
        }
        for (occurrence, &weight) in trace.occurrences.iter().zip(&head.weights_q31) {
            let out = mixture
                .get_mut(occurrence.token_id as usize)
                .ok_or_else(|| invalid("consumer occurrence token out of vocabulary"))?;
            *out += weight as f64 * scale;
        }
    }
    Ok(mixture.into_iter().map(|p| p.ln() as f32).collect())
}
pub fn mix_source_view_scores(
    parent: &[f32],
    trace: &SourceEmissionTrace,
    enabled: bool,
) -> Result<Vec<f32>> {
    mix_scores(parent, &trace.view_kernel_trace, enabled)
}
/// Same-information uniform diagnostic; not a matched learned-capacity control.
pub fn uniform_scores(parent: &[f32], trace: &OccurrenceTrace) -> Result<Vec<f32>> {
    let mut uniform = trace.clone();
    for head in &mut uniform.heads {
        head.weights_q31.fill(WEIGHT_ONE);
        head.no_read_weight_q31 = WEIGHT_ONE;
        head.total_weight_q31 = (trace.occurrences.len() as u64 + 1) * WEIGHT_ONE;
    }
    mix_scores(parent, &uniform, true)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    identity: ConsumerIdentity,
    context: ContextQ4Config,
    potential: uor_r4_integer::geometric_potential_q4::PotentialQ4Config,
    no_read: uor_r4_integer::geometric_no_read::NoReadConfig,
    algebra_sha256: String,
    files: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_view_policy: Option<String>,
}
pub struct NativeConsumerArtifact {
    metadata: NativeMetadata,
    context: NativeContextQ4,
    potential: NativePotentialQ4,
    potential_tables: uor_r4_integer::geometric_potential::NativePotentialTables,
    no_read: NativeGeometricNoRead,
    geometry: HistoricalH4Tables,
    exp: Vec<u32>,
}
impl NativeConsumerArtifact {
    #[allow(clippy::too_many_arguments)]
    fn from_parts(
        c: ContextQ4Config,
        p: uor_r4_integer::geometric_potential_q4::PotentialQ4Config,
        n: uor_r4_integer::geometric_no_read::NoReadConfig,
        context: Vec<u8>,
        potential: Vec<u8>,
        no_read: Vec<u8>,
        exp: Vec<u32>,
        identity: ConsumerIdentity,
    ) -> Result<Self> {
        if c.heads != p.heads
            || c.lanes_per_head != p.lanes_per_head
            || c.heads != n.heads
            || c.lanes_per_head != n.latent_lanes_per_head
            || c.vocab_size != n.vocabulary
        {
            return Err(invalid("consumer source component shapes differ"));
        }
        let context = NativeContextQ4::new(c, &context).map_err(|e| invalid(e.to_string()))?;
        let potential =
            NativePotentialQ4::new(p, &potential).map_err(|e| invalid(e.to_string()))?;
        let potential_tables = NativePotentialQ4::new(p, potential.packed_coefficients())
            .map_err(|e| invalid(e.to_string()))?
            .into_native()
            .map_err(|e| invalid(e.to_string()))?;
        let no_read =
            NativeGeometricNoRead::new(n, &no_read).map_err(|e| invalid(e.to_string()))?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let metadata = NativeMetadata {
            schema: SCHEMA.into(),
            policy: uor_r4_integer::geometric_occurrence_read::POLICY.into(),
            surrogate: SURROGATE.into(),
            identity,
            context: c,
            potential: p,
            no_read: n,
            algebra_sha256: sha256_bytes(ALGEBRA),
            files: BTreeMap::new(),
            source_view_policy: None,
        };
        let result = Self {
            metadata,
            context,
            potential,
            potential_tables,
            no_read,
            geometry,
            exp,
        };
        result.reader()?; // Admission validates exp table/shape, not just metadata.
        Ok(result)
    }
    fn reader(&self) -> Result<NativeOccurrenceReader<'_>> {
        NativeOccurrenceReader::new(OccurrenceComponents {
            context: self.context.native(),
            potential: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp_q31: &self.exp,
        })
        .map_err(|e| invalid(e.to_string()))
    }
    fn validate_source(&self, source: &ConsumerWeights) -> Result<()> {
        if self.metadata.context != source.config()
            || self.metadata.potential != *source.potential.config()
            || self.metadata.no_read != *source.no_read.config()
            || self.context.packed_coefficients() != source.context.packed_coefficients()?
            || self.potential.packed_coefficients() != source.potential.packed_coefficients()?
            || self.no_read.packed_coefficients() != source.no_read.packed_coefficients()?
        {
            return Err(invalid(
                "consumer current source differs from native artifact",
            ));
        }
        Ok(())
    }
    pub fn read(
        &self,
        frame: SelectedRecordFrame<'_>,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<OccurrenceTrace> {
        self.require_raw_source()?;
        self.read_kernel(frame, query, prefix)
    }
    fn require_raw_source(&self) -> Result<()> {
        if self.metadata.schema != SCHEMA || self.metadata.source_view_policy.is_some() {
            return Err(invalid(
                "emission-view artifact cannot consume a raw source frame",
            ));
        }
        Ok(())
    }
    fn require_source_view(&self, view: &SourceEmissionView) -> Result<()> {
        if self.metadata.schema != SOURCE_VIEW_SCHEMA
            || self.metadata.source_view_policy.as_deref() != Some(SOURCE_VIEW_POLICY)
            || view.policy() != SOURCE_VIEW_POLICY
            || view.tokenizer_sha256() != self.metadata.identity.tokenizer_sha256
        {
            return Err(invalid(
                "source emission view policy/tokenizer differs from artifact",
            ));
        }
        Ok(())
    }
    pub fn read_source_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<SourceEmissionTrace> {
        self.require_source_view(view)?;
        let kernel = self.read_kernel(view.derived_frame(frame)?, query, prefix)?;
        Ok(source_view_trace(frame, view, kernel))
    }
    fn read_kernel(
        &self,
        frame: SelectedRecordFrame<'_>,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<OccurrenceTrace> {
        let mut reader = self.reader()?;
        let output = reader
            .read(frame, query, prefix)
            .map_err(|e| invalid(e.to_string()))?;
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
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({"context":self.context.stats(),"potential":self.potential.stats(),"no_read":self.no_read.stats(),
            "exp_bytes":self.exp.len()*4,"algebra_bytes":ALGEBRA.len(),
            "additional_potential_table_copy_bytes":self.potential.table_bytes().len(),
            "scope":"logical packed/expanded component payload; excludes allocator/RSS/source shadows/parent and wrapper per-call construction"})
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        let exp = self
            .exp
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>();
        let files = [
            self.context.packed_coefficients(),
            self.potential.packed_coefficients(),
            self.no_read.packed_coefficients(),
            &exp,
        ];
        let mut metadata = self.metadata.clone();
        for (name, bytes) in NATIVE_FILES.into_iter().zip(files) {
            fs::write(path.join(name), bytes)?;
            metadata.files.insert(name.into(), sha256_bytes(bytes));
        }
        fs::write(
            path.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        Ok(())
    }
    pub fn load(
        path: &Path,
        source: &ConsumerWeights,
        identity: &ConsumerIdentity,
    ) -> Result<Self> {
        identity.validate()?;
        let metadata: NativeMetadata =
            serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
        let expected = source.compile(identity.clone())?;
        let representation_valid = (metadata.schema == SCHEMA
            && metadata.source_view_policy.is_none())
            || (metadata.schema == SOURCE_VIEW_SCHEMA
                && metadata.source_view_policy.as_deref() == Some(SOURCE_VIEW_POLICY));
        if !representation_valid
            || metadata.policy != expected.metadata.policy
            || metadata.surrogate != SURROGATE
            || metadata.identity != *identity
            || metadata.context != expected.metadata.context
            || metadata.potential != expected.metadata.potential
            || metadata.no_read != expected.metadata.no_read
            || metadata.algebra_sha256 != sha256_bytes(ALGEBRA)
            || metadata.files.len() != NATIVE_FILES.len()
        {
            return Err(invalid("consumer native/source/parent identity differs"));
        }
        let mut files = Vec::new();
        for name in NATIVE_FILES {
            let bytes = fs::read(path.join(name))?;
            if metadata.files.get(name) != Some(&sha256_bytes(&bytes)) {
                return Err(invalid("consumer native payload digest differs"));
            }
            files.push(bytes);
        }
        if files[0] != source.context.packed_coefficients()?
            || files[1] != source.potential.packed_coefficients()?
            || files[2] != source.no_read.packed_coefficients()?
            || files[3]
                != canonical_exp()
                    .iter()
                    .flat_map(|x| x.to_le_bytes())
                    .collect::<Vec<_>>()
        {
            return Err(invalid(
                "consumer native payload differs from actual source/canonical table",
            ));
        }
        // Execute the exported payload, not the source-recompiled comparator.
        let [context, potential, no_read, exp_bytes]: [Vec<u8>; 4] = files
            .try_into()
            .map_err(|_| invalid("consumer native payload inventory differs"))?;
        let exp = exp_bytes
            .chunks_exact(4)
            .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
            .collect();
        let mut loaded = Self::from_parts(
            metadata.context,
            metadata.potential,
            metadata.no_read,
            context,
            potential,
            no_read,
            exp,
            identity.clone(),
        )?;
        loaded.metadata = metadata;
        Ok(loaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace() -> OccurrenceTrace {
        OccurrenceTrace {
            occurrences: [1, 1, 2]
                .into_iter()
                .enumerate()
                .map(|(offset, token_id)| OccurrenceIdentity {
                    record: 7,
                    commit: 9,
                    token_offset: offset as u32,
                    token_id,
                })
                .collect(),
            heads: vec![HeadTrace {
                scores_q24: vec![0; 3],
                weights_q31: vec![1, 2, 3],
                no_read_q24: 0,
                no_read_weight_q31: 4,
                total_weight_q31: 10,
            }],
            sequence_tokens: 5,
            context_coefficient_reads: 0,
            potential_table_reads: 0,
            no_read_table_reads: 0,
            geometry_relative_reads: 0,
            logical_owned_bytes: 0,
        }
    }

    #[test]
    fn occurrence_mixture_preserves_repeats_and_entire_parent_mass() -> Result<()> {
        let parent = [0.1f32.ln(), 0.2f32.ln(), 0.3f32.ln(), 0.4f32.ln()];
        let mixed = mix_scores(&parent, &trace(), true)?;
        for (actual, expected) in mixed.iter().zip([0.04, 0.38, 0.42, 0.16]) {
            assert!((f64::from(*actual).exp() - expected).abs() < 1e-7);
        }
        assert!((mixed.iter().map(|x| f64::from(*x).exp()).sum::<f64>() - 1.).abs() < 1e-7);
        let mut null_only = trace();
        null_only.occurrences.clear();
        null_only.heads[0].weights_q31.clear();
        null_only.heads[0].scores_q24.clear();
        null_only.heads[0].total_weight_q31 = 4;
        for (a, b) in mix_scores(&parent, &null_only, true)?.iter().zip(parent) {
            assert!((*a - b).abs() < 1e-6);
        }
        Ok(())
    }

    #[test]
    fn occurrence_disabled_is_bit_exact_and_bad_denominators_reject() -> Result<()> {
        let parent = [-0.0, 0.0, -17.25, 3.5];
        let disabled = mix_scores(&parent, &trace(), false)?;
        assert_eq!(
            disabled.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            parent.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
        let mut bad = trace();
        bad.heads[0].total_weight_q31 = 11;
        assert!(mix_scores(&parent, &bad, true).is_err());
        bad.heads[0].weights_q31[0] = u64::MAX;
        assert!(mix_scores(&parent, &bad, true).is_err());
        assert!(mix_scores(&[f32::NAN], &trace(), true).is_err());
        let mut exact_copy = trace();
        exact_copy.heads[0].no_read_weight_q31 = 0;
        exact_copy.heads[0].total_weight_q31 = 6;
        assert_eq!(
            mix_scores(&parent, &exact_copy, true)?[0],
            f32::NEG_INFINITY
        );
        Ok(())
    }
}
