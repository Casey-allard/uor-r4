//! Bound compiler/store/E8-route/native-emitter graph. No optimizer or run loop.
//! The offline CPU reference records actual sampled native decisions; it is
//! not a qualified CUDA producer or an unbiased gradient of packed argmax.

use candle_core::{DType, Device, IndexOp, Tensor, Var};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uor_r4_core::native_geometric::learner::native_bank_generate::{
    BankChoiceControl, BoundNativeBytes, GenerationLimits, NativeBankArtifacts,
    NativeBankGenerateStep, NativeBankGenerator,
};
use uor_r4_integer::geometric_context::{
    ContextChoiceControl, ContextChoiceGuard, ContextDecisionFamily, ContextError, ContextResult,
    NativeContextState,
};
use uor_r4_integer::geometric_cue_carrier::{CueJointConfig, CueJointQ4};
use uor_r4_integer::geometric_potential::AddressLane;
use uor_r4_integer::geometric_potential_q4::{pack_coefficients, unpack_coefficients};
use uor_r4_integer::geometric_source_realizer::{
    NativeArtifactBinding, NativeSourceRealizer as IntegerSource, PrefixBankRealizerTrace,
};
use uor_r4_integer::geometric_turn_compiler::TurnHeadConfig;
use uor_r4_integer::h4_tables::H4Code;
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::geometric_continuation_learning::{
    ContinuationLearningWeights, PreparedContinuationLearning,
};
use crate::geometric_generate_learning::{GenerateLearningWeights, PreparedGenerateLearning};
use crate::geometric_occurrence_consumer::source_realizer::{
    CueAngularWeights, NativeSourceRealizer, PrefixAngularWeights, SourceRealizerWeights,
};
use crate::geometric_potential_q4::{PotentialQ4Pair, PotentialQ4Weights};
use crate::geometric_read_state_bridge::BridgeLearningWeights;
use crate::geometric_turn_compiler::{FitConfig, NativeTurnCompiler, TurnChoiceControl};
use crate::native_grounded_session::{
    NativeGroundedSession, NativePreparedTurn, NativeRecordSelection, NativeSessionLimits,
    NativeTurnOutcome,
};
use crate::stack_grounded_session::{CompiledAction, MemoryEffect, SessionScope, SourceSpan};
use crate::stack_store::{HistoryView, StackStore, StoreRead};
use crate::{invalid, sha256_bytes, Result};

#[path = "whole_path_device.rs"]
pub mod device;

pub const SCHEMA: &str = "uor-r4.whole-path/1";
const Q24: f64 = 16777216.;

fn route_geometry_sha256() -> Result<String> {
    let tables = crate::e8_memory::e8_tables()?;
    let group = uor_r4_core::spiralcore_operator::cl06_finite_composition_table()
        .map_err(|e| invalid(e.to_string()))?;
    if group.identity_index() != 0 {
        return Err(invalid("route frame identity differs from schema1"));
    }
    let roots = (0..240)
        .map(|r| tables.exact_root_key(r))
        .collect::<Result<Vec<_>>>()?;
    let mut images = Vec::with_capacity(64 * 240);
    let mut products = Vec::with_capacity(64 * 64);
    for a in 0..64 {
        for root in 0..240 {
            images.push(tables.transport_root(root, a)?);
        }
        for b in 0..64 {
            products.push(
                group
                    .compose_index(a, b)
                    .ok_or_else(|| invalid("frame composition absent"))?,
            );
        }
    }
    Ok(sha256_bytes(&serde_json::to_vec(&(
        "exact-2x-E8-coordinates;canonical-key;Cl06-left-frame-action/1",
        roots,
        &tables.orbit_of,
        &tables.members,
        images,
        products,
    ))?))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteLimits {
    pub per_root: usize,
    pub families: usize,
    pub records: usize,
}
impl RouteLimits {
    fn validate(&self) -> Result<()> {
        if self.per_root == 0
            || self.per_root > 32
            || self.families == 0
            || self.families > 15
            || self.records == 0
            || self.records > 32
        {
            return Err(invalid("whole-path route limits invalid"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sampling {
    pub seed: u64,
    pub temperature: f32,
    pub exploration: f32,
    pub maximum_decisions: usize,
}
impl Sampling {
    fn validate(&self) -> Result<()> {
        if !self.temperature.is_finite()
            || self.temperature <= 0.
            || !self.temperature.recip().is_finite()
            || !self.exploration.is_finite()
            || !(0. ..1.).contains(&self.exploration)
            || self.exploration == 0.
            || self.exploration / 4096. == 0.
            || self.maximum_decisions == 0
            || self.maximum_decisions > 100_000
        {
            return Err(invalid("whole-path sampling support/limits invalid"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Greedy,
    Sampled(Sampling),
}

fn values(v: &Var) -> Result<Vec<f32>> {
    Ok(v.flatten_all()?.to_vec1::<f32>()?)
}
fn parameter_digest(p: &BTreeMap<String, Var>) -> Result<String> {
    let mut receipts = BTreeMap::new();
    for (name, v) in p {
        let vals = values(v)?;
        if vals.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite whole-path master"));
        }
        let bytes = vals
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        receipts.insert(name.clone(), (v.dims().to_vec(), sha256_bytes(&bytes)));
    }
    Ok(sha256_bytes(&serde_json::to_vec(&receipts)?))
}
fn packed_var(v: &Var) -> Result<Vec<u8>> {
    let x = values(v)?;
    if x.iter().any(|x| x.abs() > 1.75) {
        return Err(invalid("whole-path quarter master outside range"));
    }
    pack_coefficients(&x.iter().map(|x| (x * 4.).round() as i8).collect::<Vec<_>>())
        .map_err(|e| invalid(e.to_string()))
}
fn anchor(hard: &[f32], graph: Tensor) -> Result<Tensor> {
    if graph.dims() != [hard.len()] {
        return Err(invalid("whole-path hard anchor shape differs"));
    }
    Ok(
        (Tensor::from_vec(hard.to_vec(), hard.len(), graph.device())?
            + (&graph - graph.detach())?)?,
    )
}
fn log_sum_exp(values: &Tensor) -> Result<Tensor> {
    let shift = values.max(0)?.detach();
    Ok((values.broadcast_sub(&shift)?.exp()?.sum_all()?.log()? + shift)?)
}
fn quarter_scale(coefficient: i8, value: i32) -> Result<i64> {
    if !(-7..=7).contains(&coefficient) {
        return Err(invalid("profile coefficient outside signed-q4"));
    }
    let magnitude = coefficient.unsigned_abs();
    let value = i64::from(value);
    let mut out = 0;
    if magnitude & 1 != 0 {
        out += value;
    }
    if magnitude & 2 != 0 {
        out += value << 1;
    }
    if magnitude & 4 != 0 {
        out += value << 2;
    }
    Ok(if coefficient < 0 { -out } else { out })
}
fn head_count(c: TurnHeadConfig) -> Result<usize> {
    if c.classes == 1 {
        return Ok(TurnHeadConfig {
            classes: 2,
            slots: c.slots,
        }
        .coefficient_count()
        .map_err(|e| invalid(e.to_string()))?
            / 2);
    }
    if c.classes == 240 {
        return Ok(4 * TurnHeadConfig {
            classes: 60,
            slots: c.slots,
        }
        .coefficient_count()
        .map_err(|e| invalid(e.to_string()))?);
    }
    c.coefficient_count().map_err(|e| invalid(e.to_string()))
}
fn table_scores(v: &Var, rows: &[Vec<Option<u8>>], c: TurnHeadConfig) -> Result<Tensor> {
    if v.dims() != [head_count(c)?]
        || rows
            .iter()
            .any(|r| r.len() != c.slots || r.iter().flatten().any(|&r| r >= 120))
    {
        return Err(invalid("whole-path additive head shape/code differs"));
    }
    let stride = 1 + c.slots * 120;
    let mut index = Vec::new();
    let mut mask = Vec::new();
    for class in 0..c.classes {
        index.push((class * stride) as u32);
        mask.push(1.);
        for row in rows {
            for (slot, code) in row.iter().enumerate() {
                index.push(
                    (class * stride + 1 + slot * 120 + usize::from(code.unwrap_or(0))) as u32,
                );
                mask.push(f32::from(u8::from(code.is_some())));
            }
        }
    }
    let device = v.device();
    let g = crate::geometric_context::q4_shadow_ste(v.as_tensor())?
        .index_select(&Tensor::from_vec(index, mask.len(), device)?, 0)?;
    Ok((g * Tensor::from_vec(mask.clone(), mask.len(), device)?)?
        .reshape((c.classes, 1 + rows.len() * c.slots))?
        .sum(1)?)
}
fn head_scores(packed: &[u8], rows: &[Vec<Option<u8>>], c: TurnHeadConfig) -> Result<Vec<i64>> {
    if c.classes == 1 {
        let mut quarters =
            unpack_coefficients(head_count(c)?, packed).map_err(|e| invalid(e.to_string()))?;
        quarters.resize(quarters.len() * 2, 0);
        let bytes = pack_coefficients(&quarters).map_err(|e| invalid(e.to_string()))?;
        let mut scores = uor_r4_integer::geometric_turn_compiler::NativeTurnHead::new(
            TurnHeadConfig {
                classes: 2,
                slots: c.slots,
            },
            &bytes,
        )
        .and_then(|h| h.scores_rows(rows))
        .map_err(|e| invalid(e.to_string()))?;
        scores.truncate(1);
        return Ok(scores);
    }
    if c.classes == 240 {
        // Preserve the compiler kernel's 64-class contract. These four heads
        // are one ordered 240-way key law, not four independent decisions.
        let quarters =
            unpack_coefficients(head_count(c)?, packed).map_err(|e| invalid(e.to_string()))?;
        let chunk = TurnHeadConfig {
            classes: 60,
            slots: c.slots,
        };
        let mut scores = Vec::with_capacity(240);
        for q in quarters.chunks_exact(head_count(chunk)?) {
            let bytes = pack_coefficients(q).map_err(|e| invalid(e.to_string()))?;
            scores.extend(
                uor_r4_integer::geometric_turn_compiler::NativeTurnHead::new(chunk, &bytes)
                    .and_then(|h| h.scores_rows(rows))
                    .map_err(|e| invalid(e.to_string()))?,
            );
        }
        return Ok(scores);
    }
    uor_r4_integer::geometric_turn_compiler::NativeTurnHead::new(c, packed)
        .and_then(|h| h.scores_rows(rows))
        .map_err(|e| invalid(e.to_string()))
}

/// References, not duplicated coefficient masters. Every exported component
/// and every gradient adapter below shares these exact Vars.
pub struct WholePathWeights<'a> {
    source: &'a SourceRealizerWeights,
    generate: &'a GenerateLearningWeights,
    cue: &'a CueAngularWeights,
    prefix: &'a PrefixAngularWeights,
    bridge: &'a BridgeLearningWeights,
    continuation: &'a ContinuationLearningWeights,
    compiler: [Var; 4],
    key: Var,
    family: Var,
    rank: Var,
    signal: Var,
    joint: Option<(CueJointConfig, Var)>,
    relations: Vec<String>,
    config: FitConfig,
    limits: RouteLimits,
    lanes: usize,
}
impl<'a> WholePathWeights<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source: &'a SourceRealizerWeights,
        generate: &'a GenerateLearningWeights,
        cue: &'a CueAngularWeights,
        prefix: &'a PrefixAngularWeights,
        bridge: &'a BridgeLearningWeights,
        continuation: &'a ContinuationLearningWeights,
        relations: Vec<String>,
        config: FitConfig,
        limits: RouteLimits,
    ) -> Result<Self> {
        limits.validate()?;
        if relations.is_empty()
            || relations.len() > 32
            || relations.iter().any(|r| r.is_empty())
            || relations.iter().collect::<BTreeSet<_>>().len() != relations.len()
            || source.binding().tokenizer_sha256() != generate.binding().tokenizer_sha256()
            || bridge.binding().tokenizer_sha256() != generate.binding().tokenizer_sha256()
            || continuation.binding().tokenizer_sha256() != generate.binding().tokenizer_sha256()
            || bridge.lanes() != generate.lanes()
            || continuation.lanes() != generate.lanes()
            || source.context_shape().0 * source.context_shape().1 != generate.lanes()
            || cue.config().heads * cue.config().lanes_per_head != generate.lanes()
            || prefix.config().heads * prefix.config().lanes_per_head != generate.lanes()
            || config.max_tokens == 0
            || config.max_tokens > 128
            || config.max_words == 0
            || config.max_words > 64
            || config.max_value_words == 0
            || config.max_value_words > 8
        {
            return Err(invalid("whole-path component/schema/limits differ"));
        }
        let lanes = generate.lanes();
        let device = generate.device();
        // Packing/rebinding is an explicit artifact boundary. Episode work on
        // CUDA uses the device producer, never the host reference session.
        if (!device.is_cpu() && !device.is_cuda())
            || source
                .parameters()
                .values()
                .any(|p| !p.device().same_device(device))
        {
            return Err(invalid(
                "whole-path masters must share an admitted CPU or CUDA device",
            ));
        }
        let slots = config.feature_mode.turn_slots(lanes);
        let span_slots = config.feature_mode.span_slots(lanes);
        let make = |classes, slots| -> Result<Var> {
            Ok(Var::zeros(
                head_count(TurnHeadConfig { classes, slots })?,
                DType::F32,
                device,
            )?)
        };
        let native_cue = cue.native()?;
        let joint = native_cue
            .joint()
            .map(|j| {
                let q = unpack_coefficients(16, j.packed_coefficients())
                    .map_err(|e| invalid(e.to_string()))?;
                Ok::<_, crate::TrainingError>((
                    j.config(),
                    Var::from_vec(q.iter().map(|&x| f32::from(x) * 0.25).collect(), 16, device)?,
                ))
            })
            .transpose()?;
        Ok(Self {
            source,
            generate,
            cue,
            prefix,
            bridge,
            continuation,
            compiler: [
                make(4, slots)?,
                make(relations.len() + 1, slots)?,
                make(2, span_slots)?,
                make(4, slots)?,
            ],
            key: make(240, lanes)?,
            family: Var::zeros((15, lanes, 120, 9), DType::F32, device)?,
            rank: make(1, lanes * 3)?,
            signal: Var::zeros((33, generate.vocab_size()), DType::F32, device)?,
            joint,
            relations,
            config,
            limits,
            lanes,
        })
    }
    pub fn parameters(&self) -> Result<BTreeMap<String, Var>> {
        // Period/NoRead are preserved legacy source components; this bank
        // emitter never uses them, so they are not falsely called trainable.
        let mut p = self
            .source
            .parameters()
            .into_iter()
            .filter(|(n, _)| {
                n.starts_with("consumer.context.") || n.starts_with("consumer.potential.")
            })
            .collect::<BTreeMap<_, _>>();
        for part in [
            self.generate.parameters(),
            self.cue.parameters(),
            self.prefix.parameters(),
            self.bridge.parameters(),
            self.continuation.parameters(),
        ] {
            for (name, v) in part {
                if p.insert(name, v).is_some() {
                    return Err(invalid("whole-path duplicate parameter owner"));
                }
            }
        }
        for (name, var) in ["act", "relation", "span", "view"]
            .iter()
            .zip(&self.compiler)
        {
            p.insert(format!("compiler.{name}"), var.clone());
        }
        p.extend([
            ("route.key".into(), self.key.clone()),
            ("route.family".into(), self.family.clone()),
            ("route.rank".into(), self.rank.clone()),
            ("memory.signal".into(), self.signal.clone()),
        ]);
        if let Some((_, v)) = &self.joint {
            p.insert("cue.joint".into(), v.clone());
        }
        let mut ids = BTreeSet::new();
        if p.values().any(|v| !ids.insert(format!("{:?}", v.id()))) {
            return Err(invalid("one whole-path Var has multiple optimizer names"));
        }
        Ok(p)
    }
    pub fn parameter_sha256(&self) -> Result<String> {
        parameter_digest(&self.parameters()?)
    }
    /// Restore fractional masters into the same declared owners. This is not
    /// optimizer/RNG/trajectory recovery; callers must start a fresh revision.
    pub fn restore_masters(&self, revision: &WholePathRevision) -> Result<()> {
        let compiler: serde_json::Value = serde_json::from_slice(&revision.compiler)?;
        if revision.manifest.lanes != self.lanes
            || revision.manifest.limits != self.limits
            || compiler["relations"] != serde_json::to_value(&self.relations)?
            || compiler["feature_mode"] != serde_json::to_value(self.config.feature_mode)?
            || compiler["max_tokens"] != serde_json::json!(self.config.max_tokens)
            || compiler["max_words"] != serde_json::json!(self.config.max_words)
            || compiler["max_value_words"] != serde_json::json!(self.config.max_value_words)
        {
            return Err(invalid(
                "fractional master compiler/route configuration differs",
            ));
        }
        let owners = self.parameters()?;
        let saved = candle_core::safetensors::load(
            revision.root.join("masters.safetensors"),
            self.generate.device(),
        )?;
        if saved.len() != owners.len() {
            return Err(invalid("master owner inventory differs"));
        }
        let mut check = BTreeMap::new();
        for (name, owner) in &owners {
            let tensor = saved
                .get(name)
                .ok_or_else(|| invalid("saved owner missing"))?;
            if tensor.dtype() != DType::F32 || tensor.dims() != owner.dims() {
                return Err(invalid("saved master shape/dtype differs"));
            }
            let value = Var::from_tensor(tensor)?;
            if name != "generate.prototype_choices" {
                packed_var(&value)?;
            }
            check.insert(name.clone(), value);
        }
        if parameter_digest(&check)? != revision.manifest.parameters_sha256 {
            return Err(invalid("saved fractional master identity differs"));
        }
        for (name, owner) in owners {
            owner.set(&saved[&name])?;
        }
        Ok(())
    }

    /// Reconstruct the learning adapters after independent revision/master
    /// reload. Rebuild graphs, never deserialize autodiff pointers or RNG guesses.
    pub fn bind_recovery(
        &self,
        parent: &NativeSourceRealizer,
        revision: &WholePathRevision,
    ) -> Result<()> {
        if self.parameter_sha256()? != revision.parameter_sha256()
            || self.limits != revision.manifest.limits
            || self.lanes != revision.manifest.lanes
        {
            return Err(invalid("recovery masters/revision differ"));
        }
        let native_path = revision.root.join("native");
        let native =
            NativeSourceRealizer::load(&native_path, self.source, parent.consumer_identity())?;
        let joint = self
            .joint
            .as_ref()
            .map(|(c, v)| CueJointQ4::new(*c, packed_var(v)?).map_err(|e| invalid(e.to_string())))
            .transpose()?;
        let cue = self.cue.rebind_joint_with(&native, &native_path, joint)?;
        let nc = native.compile_cue_carrier(cue.native()?)?;
        let prefix = self.prefix.rebind_joint(&native, &native_path, &nc)?;
        let mut tape = CreditTape::new(
            self.parameters()?,
            revision.manifest.mode.clone(),
            self.lanes,
        )?;
        let prototypes = self
            .generate
            .prototype_choices
            .reshape((self.generate.vocab_size() * self.lanes, 120))?;
        let mut codes = Vec::new();
        for row in 0..self.generate.vocab_size() * self.lanes {
            let logits = prototypes.i(row)?;
            codes.push(tape.choose(
                &format!("generate.prototype/{row}"),
                logits.clone(),
                logits.argmax(0)?.to_scalar::<u32>()? as usize,
            )? as u8);
        }
        let gs = self.generate.prepare_with_prototypes(&codes)?;
        if sha256_bytes(&gs.native.to_bytes().map_err(|e| invalid(e.to_string()))?)
            != revision.manifest.files["generate.bin"]
        {
            return Err(invalid("recovered prototype draw/Generate differs"));
        }
        let us = self
            .continuation
            .prepare_native(&revision.manifest.source, &gs.native)?;
        if sha256_bytes(&us.native.to_bytes().map_err(|e| invalid(e.to_string()))?)
            != revision.manifest.files["continuation.bin"]
        {
            return Err(invalid("recovered continuation differs"));
        }
        tape.copy = Some(Arc::new(CopyCredit {
            potential: self.source.shared_potential(),
            cue,
            prefix,
            joint: self.joint.clone(),
        }));
        tape.generate = Some(Arc::new(self.generate.shared()));
        tape.gs = Some(Arc::new(gs));
        tape.continuation = Some(Arc::new(self.continuation.shared()));
        tape.us = Some(Arc::new(us));
        tape.native_generate = Some(revision.generator(None)?.generate_model().clone());
        let mut slot = revision
            .tape_slot
            .lock()
            .map_err(|_| invalid("tape lock poisoned"))?;
        if slot.is_some() {
            return Err(invalid("recovery cannot replace a live tape"));
        }
        *slot = Some(Arc::new(tape));
        Ok(())
    }

    /// Exclusive saved/reloaded revision. No optimizer, data fit or checkpoint
    /// selection is hidden in construction. Retains fractional masters too.
    pub fn rebind(
        &self,
        parent: &NativeSourceRealizer,
        root: &Path,
        mode: Mode,
    ) -> Result<WholePathRevision> {
        if let Mode::Sampled(s) = &mode {
            s.validate()?;
        }
        let parameters = self.parameters()?;
        let digest = parameter_digest(&parameters)?;
        uor_r4_core::report_output::claim(root)?;
        let native_path = root.join("native");
        self.source
            .compile_joint_rebound(parent)?
            .save(&native_path)?;
        let native =
            NativeSourceRealizer::load(&native_path, self.source, parent.consumer_identity())?;
        let binding = native.artifact_binding()?;
        let joint = self
            .joint
            .as_ref()
            .map(|(c, v)| CueJointQ4::new(*c, packed_var(v)?).map_err(|e| invalid(e.to_string())))
            .transpose()?;
        let cue = self.cue.rebind_joint_with(&native, &native_path, joint)?;
        let nc = native.compile_cue_carrier(cue.native()?)?;
        let prefix = self.prefix.rebind_joint(&native, &native_path, &nc)?;
        let np = native.compile_prefix_transport(&nc, prefix.native()?)?;
        let compiler = crate::geometric_turn_compiler::joint_artifact(
            &IntegerSource::load_native(&native_path, &binding)
                .map_err(|e| invalid(e.to_string()))?,
            &self.relations,
            &self.config,
            &[
                values(&self.compiler[0])?,
                values(&self.compiler[1])?,
                values(&self.compiler[2])?,
                values(&self.compiler[3])?,
            ],
        )?;
        let mut tape = CreditTape::new(parameters, mode.clone(), self.lanes)?;
        let mut codes = Vec::new();
        let prototypes = self
            .generate
            .prototype_choices
            .reshape((self.generate.vocab_size() * self.lanes, 120))?;
        for row in 0..self.generate.vocab_size() * self.lanes {
            let logits = prototypes.i(row)?;
            codes.push(tape.choose(
                &format!("generate.prototype/{row}"),
                logits.clone(),
                logits.argmax(0)?.to_scalar::<u32>()? as usize,
            )? as u8);
        }
        let gs = self.generate.prepare_with_prototypes(&codes)?;
        let us = self.continuation.prepare_native(&binding, &gs.native)?;
        let generate = gs.native.to_bytes().map_err(|e| invalid(e.to_string()))?;
        let bridge = self
            .bridge
            .export_native()?
            .to_bytes()
            .map_err(|e| invalid(e.to_string()))?;
        let continuation = us.native.to_bytes().map_err(|e| invalid(e.to_string()))?;
        let signal = packed_var(&self.signal)?;
        let mut files = BTreeMap::from([
            ("compiler.bin", compiler.clone()),
            ("generate.bin", generate),
            ("bridge.bin", bridge),
            ("continuation.bin", continuation),
            ("signal.bin", signal),
            ("cue.bin", cue.packed_coefficients()?),
            ("prefix.bin", prefix.packed_coefficients()?),
            ("cue.json", serde_json::to_vec(nc.metadata())?),
            ("prefix.json", serde_json::to_vec(np.metadata())?),
            ("key.bin", packed_var(&self.key)?),
            ("family.bin", packed_var(&self.family)?),
            ("rank.bin", packed_var(&self.rank)?),
        ]);
        if let Some(j) = nc.joint() {
            files.insert("cue-joint.bin", j.packed_coefficients().to_vec());
        }
        for (name, bytes) in &files {
            std::fs::write(root.join(name), bytes)?;
        }
        let tensors = self
            .parameters()?
            .into_iter()
            .map(|(n, v)| (n, v.as_tensor().clone()))
            .collect::<std::collections::HashMap<_, _>>();
        candle_core::safetensors::save(&tensors, root.join("masters.safetensors"))?;
        let manifest = Manifest {
            schema: SCHEMA.into(),
            source: binding.clone(),
            parameters_sha256: digest,
            geometry_sha256: route_geometry_sha256()?,
            mode,
            limits: self.limits.clone(),
            lanes: self.lanes,
            files: files
                .iter()
                .map(|(n, b)| (n.to_string(), sha256_bytes(b)))
                .collect(),
            master_sha256: crate::sha256_file(&root.join("masters.safetensors"))?,
        };
        std::fs::write(root.join("revision.json"), serde_json::to_vec(&manifest)?)?;
        uor_r4_core::report_output::seal(root)?;
        let revision =
            WholePathRevision::load(root, &crate::sha256_file(&root.join("revision.json"))?)?;
        tape.copy = Some(Arc::new(CopyCredit {
            potential: self.source.shared_potential(),
            cue,
            prefix,
            joint: self.joint.clone(),
        }));
        tape.generate = Some(Arc::new(self.generate.shared()));
        tape.gs = Some(Arc::new(gs));
        tape.continuation = Some(Arc::new(self.continuation.shared()));
        tape.us = Some(Arc::new(us));
        tape.native_generate = Some(revision.generator(None)?.generate_model().clone());
        revision
            .tape_slot
            .lock()
            .map_err(|_| invalid("whole-path tape lock poisoned"))?
            .replace(Arc::new(tape));
        if self.parameter_sha256()? != revision.manifest.parameters_sha256 {
            return Err(invalid("whole-path masters changed during revision export"));
        }
        Ok(revision)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    source: NativeArtifactBinding,
    parameters_sha256: String,
    geometry_sha256: String,
    mode: Mode,
    limits: RouteLimits,
    lanes: usize,
    files: BTreeMap<String, String>,
    master_sha256: String,
}
pub struct WholePathRevision {
    root: PathBuf,
    manifest: Manifest,
    sha256: String,
    encoder: IntegerSource,
    compiler: Vec<u8>,
    key: Vec<u8>,
    family: Vec<i8>,
    rank: Vec<u8>,
    tape_slot: Mutex<Option<Arc<CreditTape>>>,
}
impl WholePathRevision {
    /// Independent encoders/controllers and RNG, same original coefficient Vars
    /// and immutable native revision. Prototype choices are NOT drawn again.
    pub fn fork_episode(&self, seed: u64) -> Result<Self> {
        let slot = self
            .tape_slot
            .lock()
            .map_err(|_| invalid("tape lock poisoned"))?;
        let tape = slot
            .as_ref()
            .ok_or_else(|| invalid("fork requires bound learning masters"))?;
        let fork = WholePathRevision::load(&self.root, &self.sha256)?;
        *fork
            .tape_slot
            .lock()
            .map_err(|_| invalid("fork lock poisoned"))? = Some(Arc::new(tape.fork_episode(seed)?));
        Ok(fork)
    }

    /// Empty-store episodes with distinct RNG seeds. Their clocks and masters
    /// are not copied; only per-episode decisions/state are isolated.
    pub fn batch(&self, seeds: &[u64]) -> Result<WholePathBatch<'_>> {
        if seeds.len() < 2
            || seeds.len() > 64
            || seeds.iter().collect::<BTreeSet<_>>().len() != seeds.len()
        {
            return Err(invalid("batch requires2..64 distinct episode seeds"));
        }
        let prototype = self
            .tape_slot
            .lock()
            .map_err(|_| invalid("tape lock poisoned"))?
            .clone()
            .ok_or_else(|| invalid("batch requires bound tape"))?;
        if prototype
            .receipts()?
            .iter()
            .any(|r| !r.owner.starts_with("generate.prototype/"))
        {
            return Err(invalid(
                "batch prototype tape already contains episode decisions",
            ));
        }
        Ok(WholePathBatch {
            revision: self,
            episodes: seeds
                .iter()
                .map(|&s| self.fork_episode(s))
                .collect::<Result<_>>()?,
            prototype,
        })
    }

    /// Recovery is performed in a fresh episode. A mismatch returns no session
    /// and never overwrites another episode's exact store or tape.
    pub fn recover_sampled<'a>(
        &'a self,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        bytes: &[u8],
        trusted_sha: &str,
    ) -> Result<WholePathSession<'a>> {
        if bytes.len() > 16 * 1024 * 1024 || sha256_bytes(bytes) != trusted_sha {
            return Err(invalid("sampled checkpoint custody/admission differs"));
        }
        let saved: SampledCheckpoint = serde_json::from_slice(bytes)?;
        let tape = self
            .tape_slot
            .lock()
            .map_err(|_| invalid("tape lock poisoned"))?
            .clone()
            .ok_or_else(|| invalid("recovery requires rebound masters"))?;
        if saved.schema != "uor-r4.whole-path-sampled/1"
            || saved.revision != self.sha256
            || saved.parameters != self.parameter_sha256()
            || saved.geometry != route_geometry_sha256()?
            || saved.mode != tape.mode
            || saved.episode.limits.max_turns
                < saved
                    .events
                    .iter()
                    .filter(|e| !matches!(e, ReplayEvent::Transport { .. }))
                    .count()
            || saved.events.len() > 4096
        {
            return Err(invalid("sampled checkpoint revision/mode/episode differs"));
        }
        let mut session = self.session(
            tokenizer,
            tokenizer_bytes,
            saved.episode.scope.clone(),
            saved.episode.limits.clone(),
            saved.episode.lineage,
            saved.episode.capacity,
        )?;
        for event in &saved.events {
            match event {
                ReplayEvent::Transport { action } => session.transport_routes(*action)?,
                ReplayEvent::Turn {
                    source,
                    maximum_tokens,
                    retained_steps,
                    emitted,
                    outcome_sha256,
                } => {
                    let out = session.turn(
                        source,
                        GenerationLimits {
                            maximum_tokens: *maximum_tokens,
                            maximum_retained_steps: *retained_steps,
                        },
                    )?;
                    if out.generation.generated_ids != *emitted
                        || outcome_digest(&out)? != *outcome_sha256
                    {
                        return Err(invalid("sampled replay outcome differs"));
                    }
                }
                ReplayEvent::Teacher {
                    source,
                    answer_with_eos,
                    baseline,
                    value,
                } => {
                    let pending = session.prepare_turn(source)?;
                    let loss =
                        session.teacher_ranking_objective(&pending, answer_with_eos, *baseline)?;
                    if loss.to_scalar::<f32>()?.to_bits() != value.to_bits() {
                        return Err(invalid("teacher replay value differs"));
                    }
                }
            }
        }
        if session.sampled_checkpoint()? != bytes {
            return Err(invalid("sampled replay RNG/decisions/store/routes differ"));
        }
        Ok(session)
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn parameter_sha256(&self) -> &str {
        &self.manifest.parameters_sha256
    }
    pub fn load(root: &Path, trusted_sha: &str) -> Result<Self> {
        uor_r4_core::report_output::verify(root)?;
        let bytes = std::fs::read(root.join("revision.json"))?;
        if bytes.len() > 128 * 1024 || sha256_bytes(&bytes) != trusted_sha {
            return Err(invalid("whole-path manifest custody differs"));
        }
        let m: Manifest = serde_json::from_slice(&bytes)?;
        m.limits.validate()?;
        if m.schema != SCHEMA
            || !(1..=8).contains(&m.lanes)
            || m.geometry_sha256 != route_geometry_sha256()?
        {
            return Err(invalid("whole-path revision schema/lanes invalid"));
        }
        for (name, sha) in &m.files {
            if !matches!(
                name.as_str(),
                "compiler.bin"
                    | "generate.bin"
                    | "bridge.bin"
                    | "continuation.bin"
                    | "signal.bin"
                    | "cue.bin"
                    | "prefix.bin"
                    | "cue.json"
                    | "prefix.json"
                    | "key.bin"
                    | "family.bin"
                    | "rank.bin"
                    | "cue-joint.bin"
            ) || crate::sha256_file(&root.join(name))? != *sha
            {
                return Err(invalid("whole-path exact file identity differs"));
            }
        }
        if m.files.len() != 12 + usize::from(m.files.contains_key("cue-joint.bin"))
            || crate::sha256_file(&root.join("masters.safetensors"))? != m.master_sha256
        {
            return Err(invalid("whole-path parameter custody differs"));
        }
        let cue_meta: uor_r4_integer::geometric_cue_carrier::CueCarrierMetadata =
            serde_json::from_slice(&std::fs::read(root.join("cue.json"))?)?;
        if cue_meta.joint.is_some() != m.files.contains_key("cue-joint.bin") {
            return Err(invalid("joint cue inventory differs"));
        }
        if let Mode::Sampled(s) = &m.mode {
            s.validate()?;
        }
        let encoder = IntegerSource::load_native(&root.join("native"), &m.source)
            .map_err(|e| invalid(e.to_string()))?;
        let compiler = std::fs::read(root.join("compiler.bin"))?;
        let key = std::fs::read(root.join("key.bin"))?;
        let rank = std::fs::read(root.join("rank.bin"))?;
        let family = unpack_coefficients(
            15 * m.lanes * 120 * 9,
            &std::fs::read(root.join("family.bin"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            root: root.into(),
            manifest: m,
            sha256: trusted_sha.into(),
            encoder,
            compiler,
            key,
            family,
            rank,
            tape_slot: Mutex::new(None),
        })
    }
    fn generator(&self, tape: Option<Arc<CreditTape>>) -> Result<NativeBankGenerator> {
        let read = |name: &str| -> Result<Vec<u8>> {
            let bytes = std::fs::read(self.root.join(name))?;
            if self.manifest.files.get(name) != Some(&sha256_bytes(&bytes)) {
                return Err(invalid("revision file changed after load"));
            }
            Ok(bytes)
        };
        let generate = read("generate.bin")?;
        let bridge = read("bridge.bin")?;
        let cue = read("cue.bin")?;
        let prefix = read("prefix.bin")?;
        let cue_meta = serde_json::from_slice::<
            uor_r4_integer::geometric_cue_carrier::CueCarrierMetadata,
        >(&read("cue.json")?)?;
        let prefix_meta = serde_json::from_slice(&read("prefix.json")?)?;
        let joint = if cue_meta.joint.is_some() {
            Some(read("cue-joint.bin")?)
        } else {
            None
        };
        let exp = std::fs::read(self.root.join("native/consumer/exp-q31.bin"))?;
        let mut g = NativeBankGenerator::load(NativeBankArtifacts {
            native_directory: &self.root.join("native"),
            source_binding: &self.manifest.source,
            generate: BoundNativeBytes {
                bytes: &generate,
                sha256: &self.manifest.files["generate.bin"],
            },
            bridge: Some(BoundNativeBytes {
                bytes: &bridge,
                sha256: &self.manifest.files["bridge.bin"],
            }),
            cue_packed: &cue,
            cue_joint_packed: joint.as_deref(),
            cue_metadata: &cue_meta,
            prefix_packed: &prefix,
            prefix_metadata: &prefix_meta,
            exp: BoundNativeBytes {
                bytes: &exp,
                sha256: &sha256_bytes(&exp),
            },
        })
        .map_err(|e| invalid(e.to_string()))?;
        let u = read("continuation.bin")?;
        let s = read("signal.bin")?;
        g = g
            .with_continuation_field(BoundNativeBytes {
                bytes: &u,
                sha256: &self.manifest.files["continuation.bin"],
            })
            .and_then(|g| {
                g.with_status_field(BoundNativeBytes {
                    bytes: &s,
                    sha256: &self.manifest.files["signal.bin"],
                })
            })
            .map_err(|e| invalid(e.to_string()))?;
        if let Some(t) = tape {
            if matches!(self.manifest.mode, Mode::Sampled(_)) {
                g = g
                    .with_choice_control(t)
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        Ok(g)
    }
    pub fn session<'a>(
        &'a self,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        scope: SessionScope,
        limits: NativeSessionLimits,
        lineage: u64,
        capacity: usize,
    ) -> Result<WholePathSession<'a>> {
        let tape = self
            .tape_slot
            .lock()
            .map_err(|_| invalid("whole-path tape lock poisoned"))?
            .clone();
        if matches!(self.manifest.mode, Mode::Sampled(_)) && tape.is_none() {
            return Err(invalid(
                "sampled revision requires its live bound learning tape",
            ));
        }
        if tape
            .as_ref()
            .is_some_and(|t| t.params.values().any(|v| !v.device().is_cpu()))
        {
            return Err(invalid(
                "CUDA masters require CudaWholePathSession; no host fallback",
            ));
        }
        let mut compiler = NativeTurnCompiler::load(
            &self.compiler,
            &self.manifest.files["compiler.bin"],
            &self.encoder,
            tokenizer,
            tokenizer_bytes,
        )?;
        let mut guards = Vec::new();
        let g = self.generator(tape.clone())?;
        if let Some(t) = &tape {
            t.validate()?;
            if t.session_claimed
                .swap(true, std::sync::atomic::Ordering::AcqRel)
            {
                return Err(invalid(
                    "learning revision already owns an episode; rebind for another trajectory",
                ));
            }
            if matches!(self.manifest.mode, Mode::Sampled(_)) {
                compiler = compiler.with_choice_control(t.clone());
                guards.push(
                    self.encoder
                        .context_encoder_parts()
                        .0
                        .control_choices(t.clone())
                        .map_err(|e| invalid(e.to_string()))?,
                );
                guards.push(
                    g.control_context_choices(t.clone())
                        .map_err(|e| invalid(e.to_string()))?,
                );
            }
        }
        let native = NativeGroundedSession::new(
            compiler,
            g,
            tokenizer,
            tokenizer_bytes,
            scope.clone(),
            limits.clone(),
            lineage,
            capacity,
        )?;
        Ok(WholePathSession {
            owner: Arc::new(()),
            revision: self,
            native,
            scope: scope.clone(),
            tokenizer,
            index: RouteIndex::new(self.manifest.limits.clone()),
            tape,
            _guards: guards,
            failed: false,
            pending: false,
            episode: EpisodeSpec {
                scope,
                limits,
                lineage,
                capacity,
            },
            journal: vec![],
            completed_teacher: None,
        })
    }
    fn encode(&self, tokenizer: &ByteBpeTokenizer, source: &str) -> Result<Vec<u8>> {
        let tokens = tokenizer.encode(source);
        if tokens.is_empty() || tokens.len() > 128 {
            return Err(invalid("whole-path route source token budget invalid"));
        }
        let (tables, algebra) = self.encoder.context_encoder_parts();
        let mut state = NativeContextState::new(tables.heads(), tables.lanes_per_head())
            .map_err(|e| invalid(e.to_string()))?;
        for id in tokens {
            state
                .step(id as usize, tables, algebra)
                .map_err(|e| invalid(e.to_string()))?;
        }
        Ok(state.states().iter().map(|h| h.index()).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteRecord {
    record: u64,
    relation: u32,
    commit: u64,
    root: u16,
    states: Vec<u8>,
    payload_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteIndex {
    limits: RouteLimits,
    frame: u8,
    records: BTreeMap<u64, RouteRecord>,
    rings: Vec<VecDeque<u64>>,
}
impl RouteIndex {
    fn new(limits: RouteLimits) -> Self {
        Self {
            limits,
            frame: 0,
            records: BTreeMap::new(),
            rings: vec![VecDeque::new(); 240],
        }
    }
    fn transport(&mut self, action: u8) -> Result<()> {
        let group = uor_r4_core::spiralcore_operator::cl06_finite_composition_table()
            .map_err(|e| invalid(e.to_string()))?;
        let frame = group
            .compose_index(action, self.frame)
            .ok_or_else(|| invalid("route frame action invalid"))?;
        let tables = crate::e8_memory::e8_tables()?;
        let mut rings = vec![VecDeque::new(); 240];
        for (root, ring) in self.rings.iter().enumerate() {
            rings[tables.transport_root(root as u16, action)? as usize] = ring.clone();
        }
        for r in self.records.values_mut() {
            r.root = tables.transport_root(r.root, action)?;
        }
        self.rings = rings;
        self.frame = frame;
        Ok(())
    }
    fn remove(&mut self, id: u64) {
        if let Some(r) = self.records.remove(&id) {
            self.rings[r.root as usize].retain(|&i| i != id);
        }
    }
    fn insert(&mut self, r: RouteRecord) -> Result<()> {
        if r.root >= 240 || r.states.is_empty() || r.states.iter().any(|&s| s >= 120) {
            return Err(invalid("route record root/state invalid"));
        }
        self.remove(r.record);
        let old = self
            .records
            .values()
            .filter(|old| old.relation == r.relation)
            .map(|old| old.record)
            .collect::<Vec<_>>();
        for id in old {
            self.remove(id);
        }
        let ring = &mut self.rings[r.root as usize];
        if ring.len() == self.limits.per_root {
            if let Some(id) = ring.pop_front() {
                self.records.remove(&id);
            }
        }
        ring.push_back(r.record);
        self.records.insert(r.record, r);
        Ok(())
    }
    fn prune(&mut self, store: &StackStore, scope: &SessionScope) -> Result<()> {
        let mut remove = Vec::new();
        for r in self.records.values() {
            let read = store
                .read(
                    &scope.scope,
                    &scope.entity,
                    r.relation,
                    HistoryView::Current,
                )
                .map_err(|e| invalid(e.to_string()))?;
            if !read
                .value()
                .is_some_and(|v| v.record == r.record && !v.conflict)
            {
                remove.push(r.record);
            }
        }
        for id in remove {
            self.remove(id);
        }
        Ok(())
    }
    fn profiles(&self) -> Result<[[i32; 9]; 15]> {
        self.profiles_after(0)
    }
    fn canonical_profiles(&self) -> Result<[[i32; 9]; 15]> {
        let group = uor_r4_core::spiralcore_operator::cl06_finite_composition_table()
            .map_err(|e| invalid(e.to_string()))?;
        self.profiles_after(
            group
                .inverse_index(self.frame)
                .ok_or_else(|| invalid("profile frame inverse absent"))?,
        )
    }
    fn profiles_after(&self, action: u8) -> Result<[[i32; 9]; 15]> {
        let tables = crate::e8_memory::e8_tables()?;
        let mut out = [[0i32; 9]; 15];
        for r in self.records.values() {
            let orbit = usize::from(tables.orbit_of[r.root as usize]);
            out[orbit][0] += 1;
            let root = tables.transport_root(r.root, action)?;
            for (i, x) in tables.exact_root_key(root)?.iter().enumerate() {
                out[orbit][i + 1] += i32::from(*x);
            }
        }
        Ok(out)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeSpec {
    pub scope: SessionScope,
    pub limits: NativeSessionLimits,
    pub lineage: u64,
    pub capacity: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum ReplayEvent {
    Transport {
        action: u8,
    },
    Turn {
        source: String,
        maximum_tokens: usize,
        retained_steps: usize,
        emitted: Vec<u32>,
        outcome_sha256: String,
    },
    Teacher {
        source: String,
        answer_with_eos: Vec<u32>,
        baseline: f32,
        value: f32,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SampledCheckpoint {
    schema: String,
    revision: String,
    parameters: String,
    geometry: String,
    mode: Mode,
    episode: EpisodeSpec,
    events: Vec<ReplayEvent>,
    decisions: Vec<DecisionReceipt>,
    rng: u64,
    native_sha256: String,
    index: RouteIndex,
}
fn outcome_digest(out: &NativeTurnOutcome) -> Result<String> {
    Ok(sha256_bytes(&serde_json::to_vec(&(
        &out.action,
        &out.effect,
        &out.reads,
        &out.generation.generated_ids,
        format!("{:?}", out.generation.stop),
        out.generation.executed_steps,
    ))?))
}

/// One shared prototype decision graph plus isolated complete episodes. No
/// optimizer, teacher serving or parameter update is performed by this API.
pub struct WholePathBatch<'a> {
    revision: &'a WholePathRevision,
    episodes: Vec<WholePathRevision>,
    prototype: Arc<CreditTape>,
}
#[derive(Clone)]
pub struct EpisodeCredit {
    tape: Arc<CreditTape>,
    loss: Tensor,
    cost: f32,
    baseline: f32,
    decisions: usize,
}
impl WholePathBatch<'_> {
    pub fn episode(&self, index: usize) -> Result<&WholePathRevision> {
        self.episodes
            .get(index)
            .ok_or_else(|| invalid("batch episode absent"))
    }
    pub fn len(&self) -> usize {
        self.episodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.episodes.is_empty()
    }
    /// Equal episode weight, independent of answer length. Local objectives
    /// already include their own ancestry. Shared codebook credit is added
    /// exactly once with the mean episode advantage, not once per token/episode.
    pub fn objective(&self, episodes: &[EpisodeCredit]) -> Result<Tensor> {
        if episodes.len() != self.len() {
            return Err(invalid("batch episode credit count differs"));
        }
        for (credit, revision) in episodes.iter().zip(&self.episodes) {
            let slot = revision
                .tape_slot
                .lock()
                .map_err(|_| invalid("episode lock poisoned"))?;
            if !slot.as_ref().is_some_and(|t| Arc::ptr_eq(t, &credit.tape)) {
                return Err(invalid("batch credit belongs to another episode/order"));
            }
            if credit.tape.receipts()?.len() != credit.decisions {
                return Err(invalid("episode changed after credit capture"));
            }
        }
        let local = episodes.iter().map(|e| e.loss.clone()).collect::<Vec<_>>();
        let costs = episodes.iter().map(|e| e.cost).collect::<Vec<_>>();
        let baselines = episodes.iter().map(|e| e.baseline).collect::<Vec<_>>();
        if local.len() != self.len()
            || costs.len() != self.len()
            || baselines.len() != self.len()
            || costs.iter().chain(&baselines).any(|v| !v.is_finite())
            || local
                .iter()
                .any(|t| t.elem_count() != 1 || !t.device().is_cpu())
        {
            return Err(invalid("batch objective inventory/value differs"));
        }
        self.prototype.validate()?;
        if self.revision.parameter_sha256() != self.prototype.parameter_identity {
            return Err(invalid("batch master identity differs"));
        }
        let mean = Tensor::stack(&local, 0)?.mean_all()?;
        let cost = costs.iter().sum::<f32>() / self.len() as f32;
        let baseline = baselines.iter().sum::<f32>() / self.len() as f32;
        let global = self.prototype.objective(cost, baseline)?;
        Ok((&mean + (&global - global.detach())?)?)
    }
}

pub struct WholePreparedTurn {
    owner: Arc<()>,
    native: NativePreparedTurn,
    index: RouteIndex,
    revision_sha256: String,
    source: String,
}
pub struct WholePathSession<'a> {
    owner: Arc<()>,
    revision: &'a WholePathRevision,
    native: NativeGroundedSession<'a, NativeTurnCompiler<'a>>,
    scope: SessionScope,
    tokenizer: &'a ByteBpeTokenizer,
    index: RouteIndex,
    tape: Option<Arc<CreditTape>>,
    _guards: Vec<ContextChoiceGuard>,
    failed: bool,
    pending: bool,
    episode: EpisodeSpec,
    journal: Vec<ReplayEvent>,
    completed_teacher: Option<EpisodeCredit>,
}
impl WholePathSession<'_> {
    pub fn completed_teacher_credit(&self) -> Result<EpisodeCredit> {
        let credit = self
            .completed_teacher
            .as_ref()
            .ok_or_else(|| invalid("no completed teacher objective"))?;
        credit.tape.validate()?;
        Ok(credit.clone())
    }
    pub fn terminal_credit(&self, cost: f32, baseline: f32) -> Result<EpisodeCredit> {
        if self.pending
            || !self
                .journal
                .iter()
                .any(|e| matches!(e, ReplayEvent::Turn { .. }))
        {
            return Err(invalid("terminal credit needs a complete emitted episode"));
        }
        let loss = self.episode_objective(cost, baseline)?;
        Ok(EpisodeCredit {
            tape: self
                .tape
                .as_ref()
                .ok_or_else(|| invalid("no tape"))?
                .clone(),
            loss,
            cost,
            baseline,
            decisions: self
                .tape
                .as_ref()
                .ok_or_else(|| invalid("no tape"))?
                .receipts()?
                .len(),
        })
    }
    pub fn teacher_credit(
        &mut self,
        prepared: &WholePreparedTurn,
        answer: &[u32],
        baseline: f32,
    ) -> Result<EpisodeCredit> {
        self.teacher_ranking_objective(prepared, answer, baseline)?;
        self.completed_teacher_credit()
    }
    /// Save committed raw inputs and the actual stochastic decisions. Autodiff
    /// graphs are reconstructed by replay; fractional masters live in revision
    /// custody. Checkpoints do not imply optimizer state or a completed run.
    pub fn sampled_checkpoint(&self) -> Result<Vec<u8>> {
        if self.pending
            || (self.failed && !matches!(self.journal.last(), Some(ReplayEvent::Teacher { .. })))
        {
            return Err(invalid(
                "checkpoint requires a committed or complete teacher boundary",
            ));
        }
        let tape = self
            .tape
            .as_ref()
            .ok_or_else(|| invalid("checkpoint has no tape"))?;
        tape.validate()?;
        if !matches!(tape.mode, Mode::Sampled(_)) {
            return Err(invalid("sampled checkpoint requires sampled mode"));
        }
        let state = tape
            .state
            .lock()
            .map_err(|_| invalid("tape lock poisoned"))?;
        let saved = SampledCheckpoint {
            schema: "uor-r4.whole-path-sampled/1".into(),
            revision: self.revision.sha256.clone(),
            parameters: tape.parameter_identity.clone(),
            geometry: route_geometry_sha256()?,
            mode: tape.mode.clone(),
            episode: self.episode.clone(),
            events: self.journal.clone(),
            decisions: state.receipts.clone(),
            rng: state.rng,
            native_sha256: sha256_bytes(&self.native.snapshot()?),
            index: self.index.clone(),
        };
        let bytes = serde_json::to_vec(&saved)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid("sampled checkpoint exceeds admission"));
        }
        Ok(bytes)
    }
    pub fn store(&self) -> &StackStore {
        self.native.store()
    }
    pub fn route_profiles(&self) -> Result<[[i32; 9]; 15]> {
        self.index.profiles()
    }
    fn valid(&self) -> Result<()> {
        if self.failed {
            return Err(invalid(
                "failed sampled trajectory must be discarded/replayed",
            ));
        }
        if let Some(t) = &self.tape {
            t.validate()?;
        }
        Ok(())
    }
    pub fn prepare_turn(&mut self, source: &str) -> Result<WholePreparedTurn> {
        self.valid()?;
        if self.pending {
            return Err(invalid("whole-path prepared turn already pending"));
        }
        let result = self.prepare_inner(source);
        if result.is_ok() {
            self.pending = true;
        } else if matches!(self.revision.manifest.mode, Mode::Sampled(_)) {
            self.failed = true;
        }
        result
    }
    fn prepare_inner(&self, source: &str) -> Result<WholePreparedTurn> {
        Self::prepare_policy(
            &self.owner,
            self.revision,
            &self.native,
            self.tokenizer,
            &self.scope,
            &self.index,
            source,
            &mut CpuRoutePolicy {
                revision: self.revision,
                tape: &self.tape,
            },
        )
    }
    fn prepare_policy<C: crate::stack_grounded_session::TurnCompiler>(
        owner: &Arc<()>,
        revision: &WholePathRevision,
        native: &NativeGroundedSession<'_, C>,
        tokenizer: &ByteBpeTokenizer,
        scope: &SessionScope,
        index: &RouteIndex,
        source: &str,
        policy: &mut impl RouteDecisions,
    ) -> Result<WholePreparedTurn> {
        let mut index = index.clone();
        // Unresolved compiler output is not a fabricated FoundCurrent read.
        let mut signal = 32;
        let pending = native
            .prepare_selected_turn(source, |store, action, effect| {
                index.prune(store, scope)?;
                let relation = match action {
                    CompiledAction::Assert { relation, .. }
                    | CompiledAction::Correct { relation, .. }
                    | CompiledAction::Query { relation, .. }
                    | CompiledAction::QueryCurrent { relation } => Some(*relation),
                    _ => None,
                };
                let view = match action {
                    CompiledAction::Query { view, .. } => *view,
                    _ => HistoryView::Current,
                };
                if let (
                    Some(relation),
                    MemoryEffect::Write {
                        written,
                        value_tokens: _,
                    },
                ) = (relation, effect)
                {
                    let read = store
                        .read(&scope.scope, &scope.entity, relation, HistoryView::Current)
                        .map_err(|e| invalid(e.to_string()))?;
                    if read
                        .value()
                        .is_some_and(|v| v.record == written.id && !v.conflict)
                    {
                        let h = policy.encode(tokenizer, source)?;
                        let chosen = policy.key(&h)?;
                        let record = store
                            .memory()
                            .record_ref(written.id)
                            .ok_or_else(|| invalid("new exact record missing"))?;
                        let root = crate::e8_memory::e8_tables()?
                            .transport_root(chosen as u16, index.frame)?;
                        index.insert(RouteRecord {
                            record: written.id,
                            relation,
                            commit: record.commit,
                            root,
                            states: h,
                            payload_sha256: record.payload_sha256.clone(),
                        })?;
                    }
                }
                let direct = relation
                    .map(|r| {
                        store
                            .read(&scope.scope, &scope.entity, r, view)
                            .map_err(|e| invalid(e.to_string()))
                    })
                    .transpose()?;
                signal = direct.as_ref().map_or(32, |read| {
                    ((read.signal(view).code() as u8) << 1)
                        | u8::from(read.value().is_some_and(|v| v.conflict))
                });
                let query = policy.encode(tokenizer, source)?;
                // Q is learned in the revision's canonical coordinates.
                // Transported roots are pulled back, not compared against
                // untransported coefficients in a different frame.
                let profiles = index.canonical_profiles()?;
                let families = policy.families(&query, &profiles, index.limits.families)?;
                let tables = crate::e8_memory::e8_tables()?;
                let mut candidates = Vec::new();
                for orbit in families {
                    for &root in &tables.members[orbit] {
                        for id in &index.rings[root as usize] {
                            if let Some(r) = index.records.get(id) {
                                // A historical request changes this relation's view.
                                if relation != Some(r.relation) || view == HistoryView::Current {
                                    candidates.push(r.clone());
                                }
                            }
                        }
                    }
                }
                if let (Some(relation), Some(StoreRead::Found(v))) = (relation, &direct) {
                    if !v.conflict && !candidates.iter().any(|r| r.record == v.record) {
                        let record = store
                            .memory()
                            .record_ref(v.record)
                            .ok_or_else(|| invalid("direct exact record missing"))?;
                        let original = if let MemoryEffect::Write { written, .. } = effect {
                            if written.id == v.record {
                                source
                            } else {
                                native.record_source(v.record)?
                            }
                        } else {
                            native.record_source(v.record)?
                        };
                        candidates.push(RouteRecord {
                            record: v.record,
                            relation,
                            commit: v.commit,
                            root: 0,
                            states: policy.encode(tokenizer, original)?,
                            payload_sha256: record.payload_sha256.clone(),
                        });
                    }
                }
                candidates.sort_by_key(|r| (r.commit, r.record));
                candidates.dedup_by_key(|r| r.record);
                let mut used = tokenizer.encode(source).len();
                let mut costs = BTreeMap::new();
                for r in &candidates {
                    let original = if let MemoryEffect::Write { written, .. } = effect {
                        if written.id == r.record {
                            source
                        } else {
                            native.record_source(r.record)?
                        }
                    } else {
                        native.record_source(r.record)?
                    };
                    let exact = store
                        .memory()
                        .record_ref(r.record)
                        .ok_or_else(|| invalid("candidate record absent"))?;
                    let view = revision
                        .encoder
                        .compile_view(&exact.payload)
                        .map_err(|e| invalid(e.to_string()))?;
                    costs.insert(
                        r.record,
                        tokenizer.encode(original).len() + view.emitted_token_ids().len(),
                    );
                }
                let mut selected = Vec::new();
                while !candidates.is_empty() && selected.len() < index.limits.records {
                    candidates.retain(|r| costs[&r.record] + used <= 120);
                    if candidates.is_empty() {
                        break;
                    }
                    let chosen = policy.rank(&query, &candidates)?;
                    let r = candidates.remove(chosen);
                    used += costs[&r.record];
                    selected.push(NativeRecordSelection {
                        relation: r.relation,
                        view: if relation == Some(r.relation) {
                            view
                        } else {
                            HistoryView::Current
                        },
                        record: r.record,
                    });
                }
                Ok(selected)
            })?
            .with_status(signal)?;
        Ok(WholePreparedTurn {
            owner: owner.clone(),
            native: pending,
            index,
            revision_sha256: revision.sha256.clone(),
            source: source.into(),
        })
    }
    pub fn emit_prepared(
        &mut self,
        prepared: WholePreparedTurn,
        limits: GenerationLimits,
    ) -> Result<NativeTurnOutcome> {
        self.valid()?;
        if !self.pending
            || !Arc::ptr_eq(&self.owner, &prepared.owner)
            || prepared.revision_sha256 != self.revision.sha256
        {
            return Err(invalid("whole-path prepared revision differs"));
        }
        let tape = self.tape.clone();
        let out = self.native.emit_prepared_with(prepared.native, limits, |generator, bank, limits| {
            let generated = generator.generate_with_choice(bank, limits, |step| match &tape {
                Some(t) => t.token(step, bank.status_code()).map_err(|e| uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGenerateError::Execution(e.to_string())),
                None => Ok(step.actions.summary.chosen_token_id),
            }).map_err(|e| invalid(e.to_string()))?;
            if let Some(t) = &tape { t.validate()?; }
            Ok(generated)
        });
        self.pending = false;
        match out {
            Ok(out) => {
                self.index = prepared.index;
                self.journal.push(ReplayEvent::Turn {
                    source: prepared.source,
                    maximum_tokens: limits.maximum_tokens,
                    retained_steps: limits.maximum_retained_steps,
                    emitted: out.generation.generated_ids.clone(),
                    outcome_sha256: outcome_digest(&out)?,
                });
                Ok(out)
            }
            Err(e) => {
                if matches!(self.revision.manifest.mode, Mode::Sampled(_)) {
                    self.failed = true;
                }
                Err(e)
            }
        }
    }
    pub fn turn(&mut self, source: &str, limits: GenerationLimits) -> Result<NativeTurnOutcome> {
        let p = self.prepare_turn(source)?;
        self.emit_prepared(p, limits)
    }
    /// One objective/pullback, no optimizer. Baseline must be fixed from before
    /// this episode's decisions, never fitted to the current terminal outcome.
    pub fn episode_objective(&self, terminal_cost: f32, baseline: f32) -> Result<Tensor> {
        self.valid()?;
        self.tape
            .as_ref()
            .ok_or_else(|| invalid("no live learning tape"))?
            .objective(terminal_cost, baseline)
    }
    /// Canonical teacher-prefix objective, not an own-prefix next-token label.
    /// Log-mean-exp of pooled winner/rival softplus losses across the whole
    /// answer including EOS. Coefficients and ancestors share one backward.
    /// A teacher trajectory is separate from a free-rollout trajectory.
    pub fn teacher_ranking_objective(
        &mut self,
        prepared: &WholePreparedTurn,
        answer_with_eos: &[u32],
        baseline: f32,
    ) -> Result<Tensor> {
        self.valid()?;
        if !self.pending
            || !Arc::ptr_eq(&self.owner, &prepared.owner)
            || prepared.revision_sha256 != self.revision.sha256
            || answer_with_eos.is_empty()
            || answer_with_eos.len() > 128
            || answer_with_eos.last() != Some(&self.revision.encoder.binding().eos_token_id())
            || !baseline.is_finite()
            || (!answer_with_eos.is_empty()
                && answer_with_eos[..answer_with_eos.len() - 1]
                    .contains(&self.revision.encoder.binding().eos_token_id()))
            || answer_with_eos
                .iter()
                .any(|&id| !self.revision.encoder.binding().admits_token(id))
        {
            return Err(invalid("teacher complete-answer/EOS binding differs"));
        }
        let result = self.teacher_ranking_inner(prepared, answer_with_eos, baseline);
        // Once replay starts, either outcome closes this trajectory. A failed
        // teacher replay cannot quietly append a second RNG/credit history.
        self.failed = true;
        if let Ok(objective) = &result {
            self.journal.push(ReplayEvent::Teacher {
                source: prepared.source.clone(),
                answer_with_eos: answer_with_eos.into(),
                baseline,
                value: objective.to_scalar::<f32>()?,
            });
            self.pending = false;
            self.completed_teacher = Some(EpisodeCredit {
                decisions: self
                    .tape
                    .as_ref()
                    .ok_or_else(|| invalid("no teacher tape"))?
                    .receipts()?
                    .len(),
                tape: self
                    .tape
                    .as_ref()
                    .ok_or_else(|| invalid("no teacher tape"))?
                    .clone(),
                loss: objective.clone(),
                cost: objective.to_scalar::<f32>()?,
                baseline,
            });
        }
        result
    }
    fn teacher_ranking_inner(
        &mut self,
        prepared: &WholePreparedTurn,
        answer_with_eos: &[u32],
        baseline: f32,
    ) -> Result<Tensor> {
        let tape = self
            .tape
            .as_ref()
            .ok_or_else(|| invalid("teacher objective needs live shared masters"))?;
        let mut margins = Vec::new();
        for (position, &target) in answer_with_eos.iter().enumerate() {
            let step = self
                .native
                .teacher_step(&prepared.native, &answer_with_eos[..position])?;
            let (generated, copied) =
                tape.coefficient_scores(&step, prepared.native.bank().status_code())?;
            let rival = step
                .actions
                .token_masses
                .iter()
                .filter(|m| m.token_id != target)
                .max_by(|a, b| {
                    a.weight_q31
                        .cmp(&b.weight_q31)
                        .then_with(|| b.token_id.cmp(&a.token_id))
                })
                .ok_or_else(|| invalid("teacher ranking has no wrong rival"))?
                .token_id;
            margins.push(
                crate::geometric_generate_learning::vocabulary_log_mass_margin(
                    &step.actions,
                    &generated,
                    if copied.elem_count() == 0 {
                        None
                    } else {
                        Some(&copied)
                    },
                    target,
                    rival,
                )?,
            );
        }
        let losses = margins
            .iter()
            .map(|margin| {
                let m = f64::from(margin.to_scalar::<f32>()?);
                let value = ((-m).max(0.) + (-m.abs()).exp().ln_1p()) as f32;
                let slope = if m >= 0. {
                    -(-m).exp() / (1. + (-m).exp())
                } else {
                    -1. / (1. + m.exp())
                };
                let partial = margin.affine(slope, 0.)?;
                Ok((Tensor::new(value, margin.device())? + (&partial - partial.detach())?)?)
            })
            .collect::<Result<Vec<_>>>()?;
        let all = Tensor::stack(&losses, 0)?;
        let shift = all.max(0)?.detach();
        let rank = (all.broadcast_sub(&shift)?.exp()?.mean_all()?.log()? + &shift)?;
        let value = rank.to_scalar::<f32>()?;
        // No token-sampling term in this teacher trajectory. All recorded hard
        // ancestors, including shared prototypes, receive this episode cost.
        let objective = if matches!(tape.mode, Mode::Sampled(_)) {
            let score = tape.objective(value, baseline)?;
            (&rank + (&score - score.detach())?)?
        } else {
            rank
        };
        Ok(objective)
    }
    pub fn transport_routes(&mut self, action: u8) -> Result<()> {
        self.valid()?;
        if self.pending || self.journal.len() >= 4096 {
            return Err(invalid("frame transport requires an idle bounded session"));
        }
        self.index.transport(action)?;
        self.journal.push(ReplayEvent::Transport { action });
        Ok(())
    }
    pub fn credit_inventory(&self) -> Result<BTreeMap<String, usize>> {
        self.tape
            .as_ref()
            .ok_or_else(|| invalid("no learning tape"))?
            .inventory()
    }
    pub fn snapshot(&self) -> Result<Vec<u8>> {
        self.valid()?;
        if matches!(self.revision.manifest.mode, Mode::Sampled(_)) {
            return Err(invalid("sampled sessions require sampled_checkpoint; serving snapshots require a greedy revision"));
        }
        let bytes = self.native.snapshot()?;
        Ok(serde_json::to_vec(&(
            SCHEMA,
            &self.revision.sha256,
            &self.index,
            sha256_bytes(&bytes),
            bytes,
        ))?)
    }
    pub fn restore(&mut self, bytes: &[u8], trusted_sha: &str) -> Result<()> {
        self.valid()?;
        if matches!(self.revision.manifest.mode, Mode::Sampled(_))
            || bytes.len() > 16 * 1024 * 1024
            || sha256_bytes(bytes) != trusted_sha
        {
            return Err(invalid("whole-path snapshot mode/custody differs"));
        }
        let (schema, revision, index, inner_sha, inner): (
            String,
            String,
            RouteIndex,
            String,
            Vec<u8>,
        ) = serde_json::from_slice(bytes)?;
        if schema != SCHEMA
            || revision != self.revision.sha256
            || index.limits != self.index.limits
            || index.rings.len() != 240
            || index.records.len() > 240 * index.limits.per_root
        {
            return Err(invalid("whole-path snapshot index/revision differs"));
        }
        let mut seen = BTreeSet::new();
        for (root, ring) in index.rings.iter().enumerate() {
            if ring.len() > index.limits.per_root {
                return Err(invalid("whole-path snapshot ring too large"));
            }
            for id in ring {
                let r = index
                    .records
                    .get(id)
                    .ok_or_else(|| invalid("index ring lacks record"))?;
                if !seen.insert(*id)
                    || r.root as usize != root
                    || r.states.len() != self.revision.manifest.lanes
                    || r.states.iter().any(|&r| r >= 120)
                {
                    return Err(invalid("index root/state/identity differs"));
                }
            }
        }
        if seen.len() != index.records.len() {
            return Err(invalid("unindexed route records"));
        }
        let group = uor_r4_core::spiralcore_operator::cl06_finite_composition_table()
            .map_err(|e| invalid(e.to_string()))?;
        group
            .inverse_index(index.frame)
            .ok_or_else(|| invalid("snapshot frame invalid"))?;
        self.native
            .restore_checked(&inner, &inner_sha, |store, witnesses| {
                let mut checked = index.clone();
                checked.prune(store, &self.scope)?;
                if checked != index {
                    return Err(invalid("snapshot routes not eligible in exact store"));
                }
                for r in index.records.values() {
                    let exact = store
                        .memory()
                        .record_ref(r.record)
                        .ok_or_else(|| invalid("snapshot exact record missing"))?;
                    let source = witnesses
                        .iter()
                        .find(|(id, _)| *id == r.record)
                        .ok_or_else(|| invalid("snapshot route source absent"))?;
                    let states = self.revision.encode(self.tokenizer, &source.1)?;
                    let rows = vec![states.iter().copied().map(Some).collect()];
                    let key = choose_table(
                        &None,
                        "route.key",
                        &rows,
                        TurnHeadConfig {
                            classes: 240,
                            slots: self.revision.manifest.lanes,
                        },
                        &head_scores(
                            &self.revision.key,
                            &rows,
                            TurnHeadConfig {
                                classes: 240,
                                slots: self.revision.manifest.lanes,
                            },
                        )?,
                    )?;
                    let root =
                        crate::e8_memory::e8_tables()?.transport_root(key as u16, index.frame)?;
                    if exact.payload_sha256 != r.payload_sha256
                        || exact.commit != r.commit
                        || states != r.states
                        || root != r.root
                    {
                        return Err(invalid("snapshot exact route/encoder content differs"));
                    }
                }
                Ok(())
            })?;
        self.index = index;
        Ok(())
    }
}

trait RouteDecisions {
    fn encode(&mut self, tokenizer: &ByteBpeTokenizer, source: &str) -> Result<Vec<u8>>;
    fn key(&mut self, states: &[u8]) -> Result<usize>;
    fn families(
        &mut self,
        states: &[u8],
        profiles: &[[i32; 9]; 15],
        count: usize,
    ) -> Result<Vec<usize>>;
    fn rank(&mut self, query: &[u8], candidates: &[RouteRecord]) -> Result<usize>;
}
struct CpuRoutePolicy<'a> {
    revision: &'a WholePathRevision,
    tape: &'a Option<Arc<CreditTape>>,
}
impl RouteDecisions for CpuRoutePolicy<'_> {
    fn encode(&mut self, tokenizer: &ByteBpeTokenizer, source: &str) -> Result<Vec<u8>> {
        self.revision.encode(tokenizer, source)
    }
    fn key(&mut self, states: &[u8]) -> Result<usize> {
        let rows = vec![states.iter().copied().map(Some).collect()];
        let c = TurnHeadConfig {
            classes: 240,
            slots: states.len(),
        };
        choose_table(
            self.tape,
            "route.key",
            &rows,
            c,
            &head_scores(&self.revision.key, &rows, c)?,
        )
    }
    fn families(
        &mut self,
        states: &[u8],
        profiles: &[[i32; 9]; 15],
        count: usize,
    ) -> Result<Vec<usize>> {
        let mut hard = Vec::new();
        for (o, profile) in profiles.iter().enumerate() {
            let mut sum = 0i64;
            for (lane, &h) in states.iter().enumerate() {
                let at = ((o * states.len() + lane) * 120 + h as usize) * 9;
                for (d, &x) in profile.iter().enumerate() {
                    sum += quarter_scale(self.revision.family[at + d], x)?;
                }
            }
            hard.push(sum);
        }
        choose_families(self.tape, states, profiles, &hard, count)
    }
    fn rank(&mut self, query: &[u8], candidates: &[RouteRecord]) -> Result<usize> {
        let mut rows = Vec::new();
        let mut hard = Vec::new();
        let (_, algebra) = self.revision.encoder.context_encoder_parts();
        for r in candidates {
            let relative = query
                .iter()
                .zip(&r.states)
                .map(|(&q, &k)| {
                    Ok(Some(
                        algebra
                            .compose(
                                algebra.inverse(
                                    H4Code::try_from(q).map_err(|e| invalid(e.to_string()))?,
                                ),
                                H4Code::try_from(k).map_err(|e| invalid(e.to_string()))?,
                            )
                            .index(),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            let row = query
                .iter()
                .chain(&r.states)
                .copied()
                .map(Some)
                .chain(relative)
                .collect::<Vec<_>>();
            hard.push(
                head_scores(
                    &self.revision.rank,
                    &[row.clone()],
                    TurnHeadConfig {
                        classes: 1,
                        slots: query.len() * 3,
                    },
                )?[0],
            );
            rows.push(row);
        }
        choose_records(self.tape, &rows, &hard)
    }
}

struct CopyCredit {
    potential: PotentialQ4Weights,
    cue: CueAngularWeights,
    prefix: PrefixAngularWeights,
    joint: Option<(CueJointConfig, Var)>,
}
impl CopyCredit {
    fn partial(&self, trace: &PrefixBankRealizerTrace) -> Result<Tensor> {
        let b = &trace.cue_bank.bank;
        let n = b.candidates.len();
        let h = b.context.heads;
        let l = b.context.lanes_per_head;
        let t = b.context.tokens.len();
        if n == 0 || t == 0 || b.context.codes.len() != t * h * l {
            return Err(invalid("copy coefficient trace shape differs"));
        }
        let codes = b
            .context
            .codes
            .iter()
            .map(|c| {
                AddressLane::new(c.root, c.radius_bin, c.present)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        let absent = vec![
            AddressLane::new(H4Code::IDENTITY.index(), 0, false)
                .map_err(|e| invalid(e.to_string()))?;
            l
        ];
        let mut pairs = Vec::new();
        for head in 0..h {
            let q = ((t - 1) * h + head) * l;
            for c in &b.candidates {
                let k = (c.context_position * h + head) * l;
                pairs.push(PotentialQ4Pair {
                    head,
                    query_content: &absent,
                    candidate_content: &absent,
                    query_context: &codes[q..q + l],
                    candidate_context: &codes[k..k + l],
                });
            }
        }
        let p = self
            .potential
            .forward_selected_codes(&pairs)?
            .scores
            .reshape((h, n))?
            .sum(0)?;
        let device = p.device();
        let cue = self
            .cue
            .coefficient_credit(&trace.cue_bank.carrier, device)?;
        let prefix = self.prefix.coefficient_credit(&trace.prefix, device)?;
        let mut out = (&p + &cue + &prefix)?;
        if let Some((config, coefficients)) = &self.joint {
            let roots = &trace.cue_bank.carrier.relative_roots;
            let left = config.head * l + config.left_lane;
            let right = config.head * l + config.right_lane;
            let basis = uor_r4_integer::geometric_context_q4::canonical_basis_q25();
            let mut features = vec![0f32; n * 16];
            for i in 0..n {
                if let (Some(a), Some(b)) = (roots[left][i], roots[right][i]) {
                    for x in 0..4 {
                        for y in 0..4 {
                            features[i * 16 + x * 4 + y] = ((f64::from(basis[a as usize][x])
                                / 33554432.)
                                * (f64::from(basis[b as usize][y]) / 33554432.))
                                as f32;
                        }
                    }
                }
            }
            let joint = Tensor::from_vec(features, (n, 16), device)?
                .matmul(
                    &crate::geometric_context::q4_shadow_ste(coefficients.as_tensor())?
                        .reshape((16, 1))?,
                )?
                .flatten_all()?;
            out = (&out + joint)?;
        }
        Ok(out)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionReceipt {
    pub owner: String,
    pub chosen: usize,
    pub alternatives: usize,
    pub probability: f32,
    pub logits_sha256: String,
    pub rng_before: u64,
    pub rng_after: u64,
}
#[derive(Debug)]
struct TapeState {
    rng: u64,
    logs: Vec<Tensor>,
    receipts: Vec<DecisionReceipt>,
}
pub struct CreditTape {
    identity: String,
    parameter_identity: String,
    params: BTreeMap<String, Var>,
    mode: Mode,
    lanes: usize,
    state: Mutex<TapeState>,
    session_claimed: std::sync::atomic::AtomicBool,
    copy: Option<Arc<CopyCredit>>,
    generate: Option<Arc<GenerateLearningWeights>>,
    gs: Option<Arc<PreparedGenerateLearning>>,
    continuation: Option<Arc<ContinuationLearningWeights>>,
    us: Option<Arc<PreparedContinuationLearning>>,
    native_generate:
        Option<uor_r4_core::native_geometric::learner::geometric_generate::NativeGeometricGenerate>,
}
impl std::fmt::Debug for CreditTape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreditTape")
            .field("identity", &self.identity)
            .finish()
    }
}
impl CreditTape {
    fn fork_episode(&self, seed: u64) -> Result<Self> {
        self.validate()?;
        let Mode::Sampled(mut sampling) = self.mode.clone() else {
            return Err(invalid("episode fork requires sampled revision"));
        };
        sampling.seed = seed;
        let mut tape = Self::new(self.params.clone(), Mode::Sampled(sampling), self.lanes)?;
        tape.copy = self.copy.clone();
        tape.generate = self.generate.clone();
        tape.gs = self.gs.clone();
        tape.continuation = self.continuation.clone();
        tape.us = self.us.clone();
        tape.native_generate = self.native_generate.clone();
        Ok(tape)
    }

    fn new(params: BTreeMap<String, Var>, mode: Mode, lanes: usize) -> Result<Self> {
        let seed = match &mode {
            Mode::Greedy => 1,
            Mode::Sampled(s) => s.seed,
        };
        let parameter_identity = parameter_digest(&params)?;
        let identity = sha256_bytes(&serde_json::to_vec(&(
            SCHEMA,
            &parameter_identity,
            &mode,
            lanes,
        ))?);
        Ok(Self {
            identity,
            parameter_identity,
            params,
            mode,
            lanes,
            session_claimed: std::sync::atomic::AtomicBool::new(false),
            state: Mutex::new(TapeState {
                rng: seed,
                logs: vec![],
                receipts: vec![],
            }),
            copy: None,
            generate: None,
            gs: None,
            continuation: None,
            us: None,
            native_generate: None,
        })
    }
    fn var(&self, name: &str) -> Result<&Var> {
        self.params
            .get(name)
            .ok_or_else(|| invalid(format!("whole-path parameter absent: {name}")))
    }
    fn validate(&self) -> Result<()> {
        if parameter_digest(&self.params)? != self.parameter_identity {
            return Err(invalid("whole-path live masters changed; rebind required"));
        }
        Ok(())
    }
    fn choose(&self, owner: &str, logits: Tensor, greedy: usize) -> Result<usize> {
        self.choose_scaled(owner, logits, greedy, true)
    }
    fn choose_scaled(
        &self,
        owner: &str,
        logits: Tensor,
        greedy: usize,
        temperature: bool,
    ) -> Result<usize> {
        if (!logits.device().is_cpu()
            && !(logits.device().is_cuda() && owner.starts_with("generate.prototype/")))
            || logits.dtype() != DType::F32
            || logits.dims().len() != 1
            || logits.elem_count() == 0
            || greedy >= logits.elem_count()
        {
            return Err(invalid("whole-path CPU choice/logit shape invalid"));
        }
        if matches!(self.mode, Mode::Greedy) {
            return Ok(greedy);
        }
        let Mode::Sampled(config) = &self.mode else {
            return Err(invalid("no sampling mode"));
        };
        let scale = if temperature {
            1. / f64::from(config.temperature)
        } else {
            1.
        };
        let centered = logits.broadcast_sub(&logits.max(0)?.detach())?;
        let probability = candle_nn::ops::softmax(&centered.affine(scale, 0.)?, 0)?.affine(
            1. - f64::from(config.exploration),
            f64::from(config.exploration) / logits.elem_count() as f64,
        )?;
        let p = probability.to_vec1::<f32>()?;
        if p.iter().any(|p| !p.is_finite() || *p <= 0.) {
            return Err(invalid("whole-path choice has no finite full support"));
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| invalid("whole-path tape lock poisoned"))?;
        if state.logs.len() >= config.maximum_decisions {
            return Err(invalid("whole-path decision admission exceeded"));
        }
        state.rng = state.rng.wrapping_add(0x9e3779b97f4a7c15);
        let rng_before = state.rng.wrapping_sub(0x9e3779b97f4a7c15);
        let mut z = state.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        let u = ((z >> 11) as f64) / ((1u64 << 53) as f64);
        let mut sum = 0.;
        let mut chosen = p.len() - 1;
        for (i, &v) in p.iter().enumerate() {
            sum += f64::from(v);
            if u < sum {
                chosen = i;
                break;
            }
        }
        if logits.device().is_cuda() {
            chosen = device::prototype_choice(&probability, u)?;
        }
        state.logs.push(probability.i(chosen)?.log()?);
        let rng_after = state.rng;
        state.receipts.push(DecisionReceipt {
            owner: owner.into(),
            chosen,
            alternatives: p.len(),
            probability: p[chosen],
            logits_sha256: sha256_bytes(
                &logits
                    .to_vec1::<f32>()?
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>(),
            ),
            rng_before,
            rng_after,
        });
        Ok(chosen)
    }
    pub fn receipts(&self) -> Result<Vec<DecisionReceipt>> {
        Ok(self
            .state
            .lock()
            .map_err(|_| invalid("tape lock poisoned"))?
            .receipts
            .clone())
    }
    fn inventory(&self) -> Result<BTreeMap<String, usize>> {
        let mut out = BTreeMap::new();
        for r in self.receipts()? {
            *out.entry(r.owner.split('/').next().unwrap_or(&r.owner).to_string())
                .or_default() += 1;
        }
        Ok(out)
    }
    fn objective(&self, terminal_cost: f32, baseline: f32) -> Result<Tensor> {
        self.validate()?;
        if !terminal_cost.is_finite() || !baseline.is_finite() || matches!(self.mode, Mode::Greedy)
        {
            return Err(invalid(
                "episode objective requires sampled trajectory and fixed finite cost/baseline",
            ));
        }
        let state = self
            .state
            .lock()
            .map_err(|_| invalid("whole-path tape lock poisoned"))?;
        if state.logs.is_empty() {
            return Err(invalid("empty whole-path credit tape"));
        }
        let term = Tensor::stack(&state.logs, 0)?
            .sum_all()?
            .affine(f64::from(terminal_cost - baseline), 0.)?;
        Ok((Tensor::new(terminal_cost, term.device())? + (&term - term.detach())?)?)
    }
    fn context_logits(
        &self,
        token: usize,
        lane: usize,
        family: ContextDecisionFamily,
        own: H4Code,
        neighbor: H4Code,
        scores: &[i64],
    ) -> Result<Tensor> {
        let names = match family {
            ContextDecisionFamily::Transition => {
                ["token_transition", "self_transition", "neighbor_transition"]
            }
            ContextDecisionFamily::Root => ["token_root", "self_root", "neighbor_root"],
            ContextDecisionFamily::Category => {
                ["token_category", "self_category", "neighbor_category"]
            }
        };
        let n = scores.len();
        let token_var = self.var(&format!("consumer.context.{}", names[0]))?;
        let logits = crate::geometric_context::q4_shadow_ste(token_var.as_tensor())?
            .reshape((token_var.elem_count() / n, n))?
            .i(token * self.lanes + lane)?;
        let basis = uor_r4_integer::geometric_context_q4::canonical_basis_q25();
        let mut out = logits;
        for (name, code) in [(names[1], own), (names[2], neighbor)] {
            if let Some(var) = self.params.get(&format!("consumer.context.{name}")) {
                let coefficients = crate::geometric_context::q4_shadow_ste(var.as_tensor())?
                    .reshape((self.lanes, n, 4))?
                    .i(lane)?;
                let v = Tensor::from_vec(
                    basis[code.index() as usize]
                        .iter()
                        .map(|&x| x as f32 / 33554432.)
                        .collect::<Vec<_>>(),
                    (4, 1),
                    &Device::Cpu,
                )?;
                out = (&out + coefficients.matmul(&v)?.flatten_all()?)?;
            }
        }
        anchor(
            &scores
                .iter()
                .map(|&s| (s as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            out,
        )
    }
    fn coefficient_scores(
        &self,
        step: &NativeBankGenerateStep,
        status: Option<u8>,
    ) -> Result<(Tensor, Tensor)> {
        let gs = self
            .gs
            .as_ref()
            .ok_or_else(|| invalid("no shared Generate snapshot"))?;
        let g = self
            .generate
            .as_ref()
            .ok_or_else(|| invalid("no shared Generate coefficients"))?;
        let mut generated = g
            .forward_prepared_coefficients_only(gs, &step.post_state)?
            .raw_scores;
        let mut copied = match &step.bank_trace {
            Some(trace) => self
                .copy
                .as_ref()
                .ok_or_else(|| invalid("copy coefficient adapter absent"))?
                .partial(trace)?,
            None => Tensor::zeros(0, DType::F32, &Device::Cpu)?,
        };
        if let Some(local) = &step.continuation {
            let weights = self
                .continuation
                .as_ref()
                .ok_or_else(|| invalid("continuation masters absent"))?;
            let u = weights
                .forward_prepared_coefficients_with_factual_on_device(
                    self.us
                        .as_ref()
                        .ok_or_else(|| invalid("continuation snapshot absent"))?,
                    &step.post_state,
                    &local.state_codes,
                )?
                .delta_raw_scores;
            generated = (&generated + &u)?;
            if weights.applies_to_copy() && !step.copy_token_ids.is_empty() {
                copied = (&copied
                    + u.index_select(
                        &Tensor::from_vec(
                            step.copy_token_ids.clone(),
                            step.copy_token_ids.len(),
                            &Device::Cpu,
                        )?,
                        0,
                    )?)?;
            }
        }
        let status = usize::from(status.ok_or_else(|| invalid("numerical store status missing"))?);
        let signal =
            crate::geometric_context::q4_shadow_ste(self.var("memory.signal")?.as_tensor())?
                .i(status)?;
        generated = (&generated + &signal)?;
        if !step.copy_token_ids.is_empty() {
            copied = (&copied
                + signal.index_select(
                    &Tensor::from_vec(
                        step.copy_token_ids.clone(),
                        step.copy_token_ids.len(),
                        &Device::Cpu,
                    )?,
                    0,
                )?)?;
        }
        let generated = anchor(
            &step
                .generate_raw_scores_q24
                .iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            generated,
        )?;
        let copied = anchor(
            &step
                .copy_raw_scores_q24
                .iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            copied,
        )?;
        Ok((generated, copied))
    }
    fn token(&self, step: &NativeBankGenerateStep, status: Option<u8>) -> Result<u32> {
        if matches!(self.mode, Mode::Greedy) {
            return Ok(step.actions.summary.chosen_token_id);
        }
        let (generated, copied) = self.coefficient_scores(step, status)?;
        let mut raw = Vec::new();
        let mut hard = Vec::new();
        for a in &step.actions.actions {
            // The actual trace order/aliases is the authority, not reauthored
            // rows or a selected winning occurrence.
            let value = match a.action {
                uor_r4_integer::geometric_vocabulary_actions::VocabularyAction::Generate {
                    token_id,
                } => generated.i(token_id as usize)?,
                uor_r4_integer::geometric_vocabulary_actions::VocabularyAction::Copy {
                    source_offset,
                } => copied.i(source_offset)?,
            };
            raw.push(value);
            hard.push((a.score_q24 as f64 / Q24) as f32);
        }
        let graph = Tensor::stack(&raw, 0)?.clamp(-8f32, 8f32)?;
        let logits = anchor(&hard, graph)?;
        let temperature = match &self.mode {
            Mode::Sampled(s) => s.temperature,
            Mode::Greedy => 1.,
        };
        let scaled = logits.affine(1. / f64::from(temperature), 0.)?;
        let masses = &step.actions.token_masses;
        let mut token_log = Vec::new();
        for mass in masses {
            let index = step
                .actions
                .actions
                .iter()
                .enumerate()
                .filter(|(_, a)| a.token_id == mass.token_id)
                .map(|(i, _)| i as u32)
                .collect::<Vec<_>>();
            token_log.push(log_sum_exp(&scaled.index_select(
                &Tensor::from_vec(index.clone(), index.len(), &Device::Cpu)?,
                0,
            )?)?);
        }
        let greedy = masses
            .iter()
            .position(|m| m.token_id == step.actions.summary.chosen_token_id)
            .ok_or_else(|| invalid("native winner has no token mass"))?;
        let chosen = self.choose_scaled(
            "emitter.token",
            Tensor::stack(&token_log, 0)?,
            greedy,
            false,
        )?;
        Ok(masses[chosen].token_id)
    }
}
impl ContextChoiceControl for CreditTape {
    fn identity(&self) -> &str {
        &self.identity
    }
    fn select(
        &self,
        token: usize,
        lane: usize,
        family: ContextDecisionFamily,
        own: H4Code,
        neighbor: H4Code,
        scores_q24: &[i64],
    ) -> ContextResult<u8> {
        let run = || -> Result<u8> {
            let logits = self.context_logits(token, lane, family, own, neighbor, scores_q24)?;
            Ok(self.choose(&format!("context.{family:?}"), logits, 0)? as u8)
        };
        run().map_err(|_| ContextError::ChoiceControl)
    }
}
impl TurnChoiceControl for CreditTape {
    fn select(
        &self,
        family: &str,
        rows: &[Vec<Option<u8>>],
        c: TurnHeadConfig,
        scores: &[i64],
    ) -> Result<usize> {
        let graph = table_scores(self.var(&format!("compiler.{family}"))?, rows, c)?;
        let hard = scores.iter().map(|&x| x as f32 * 0.25).collect::<Vec<_>>();
        let greedy = scores
            .iter()
            .enumerate()
            .max_by(|(a, x), (b, y)| x.cmp(y).then(b.cmp(a)))
            .map(|(i, _)| i)
            .ok_or_else(|| invalid("empty compiler head"))?;
        self.choose(&format!("compiler.{family}"), anchor(&hard, graph)?, greedy)
    }
    fn span(
        &self,
        rows: &[Vec<Option<u8>>],
        c: TurnHeadConfig,
        scores: &[Vec<i64>],
        spans: &[SourceSpan],
        max_words: usize,
    ) -> Result<Option<SourceSpan>> {
        let mut soft_words = Vec::new();
        for row in rows {
            let out = table_scores(self.var("compiler.span")?, &[row.clone()], c)?;
            soft_words.push((&out.i(1)? - &out.i(0)?)?);
        }
        let mut alternatives = vec![None];
        let mut hard = vec![0f32];
        let mut soft = vec![Tensor::new(0f32, &Device::Cpu)?];
        for start in 0..scores.len() {
            let mut score = 0i64;
            let mut graph = Tensor::new(0f32, &Device::Cpu)?;
            for end in start..scores.len().min(start + max_words) {
                score += scores[end][1] - scores[end][0];
                graph = (&graph + &soft_words[end])?;
                alternatives.push(Some(SourceSpan {
                    start: spans[start].start,
                    end: spans[end].end,
                }));
                hard.push(score as f32 * 0.25);
                soft.push(graph.clone());
            }
        }
        let greedy = hard
            .iter()
            .enumerate()
            .max_by(|(a, x), (b, y)| x.total_cmp(y).then(b.cmp(a)))
            .map(|(i, _)| i)
            .ok_or_else(|| invalid("no span alternatives"))?;
        let chosen = self.choose(
            "compiler.span",
            anchor(&hard, Tensor::stack(&soft, 0)?)?,
            greedy,
        )?;
        Ok(alternatives[chosen])
    }
}
impl BankChoiceControl for CreditTape {
    fn identity(&self) -> &str {
        &self.identity
    }
    fn donor(
        &self,
        trace: &PrefixBankRealizerTrace,
        scores: &[i64],
        greedy: usize,
    ) -> uor_r4_core::native_geometric::learner::native_bank_generate::Result<usize> {
        let run = || -> Result<usize> {
            if matches!(self.mode, Mode::Greedy) {
                return Ok(greedy);
            }
            let graph = self
                .copy
                .as_ref()
                .ok_or_else(|| invalid("donor Copy adapter absent"))?
                .partial(trace)?;
            self.choose(
                "reader.donor",
                anchor(
                    &scores
                        .iter()
                        .map(|&x| (x as f64 / Q24) as f32)
                        .collect::<Vec<_>>(),
                    graph,
                )?,
                greedy,
            )
        };
        run().map_err(|e| uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGenerateError::Execution(e.to_string()))
    }
    fn action(
        &self,
        lane: usize,
        query: H4Code,
        source: H4Code,
        scores: &[i64],
        greedy: u8,
    ) -> uor_r4_core::native_geometric::learner::native_bank_generate::Result<u8> {
        let run = || -> Result<u8> {
            let native = self
                .native_generate
                .as_ref()
                .ok_or_else(|| invalid("bridge algebra snapshot absent"))?;
            let d = native
                .algebra()
                .relative(query.index(), native.algebra().identity(), source.index())
                .map_err(|e| invalid(e.to_string()))? as usize;
            let graph = (&crate::geometric_context::q4_shadow_ste(
                self.var("read_state_bridge.bias")?.as_tensor(),
            )?
            .i(lane)?
                + crate::geometric_context::q4_shadow_ste(
                    self.var("read_state_bridge.relative")?.as_tensor(),
                )?
                .i((lane, .., d))?)?
            .affine(0.25, 0.)?;
            Ok(self.choose(
                "bridge.action",
                anchor(
                    &scores
                        .iter()
                        .map(|&x| (x as f64 / Q24) as f32)
                        .collect::<Vec<_>>(),
                    graph,
                )?,
                greedy as usize,
            )? as u8)
        };
        run().map_err(|e| uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGenerateError::Execution(e.to_string()))
    }
}

fn choose_table(
    tape: &Option<Arc<CreditTape>>,
    name: &str,
    rows: &[Vec<Option<u8>>],
    c: TurnHeadConfig,
    hard: &[i64],
) -> Result<usize> {
    let greedy = hard
        .iter()
        .enumerate()
        .max_by(|(a, x), (b, y)| x.cmp(y).then(b.cmp(a)))
        .map(|(i, _)| i)
        .ok_or_else(|| invalid("empty route head"))?;
    match tape {
        Some(t) => t.choose(
            name,
            anchor(
                &hard.iter().map(|&x| x as f32 * 0.25).collect::<Vec<_>>(),
                table_scores(t.var(name)?, rows, c)?,
            )?,
            greedy,
        ),
        None => Ok(greedy),
    }
}
fn choose_families(
    tape: &Option<Arc<CreditTape>>,
    states: &[u8],
    profiles: &[[i32; 9]; 15],
    hard: &[i64],
    count: usize,
) -> Result<Vec<usize>> {
    let mut alive = (0..15).filter(|&o| profiles[o][0] > 0).collect::<Vec<_>>();
    let mut out = Vec::new();
    while !alive.is_empty() && out.len() < count {
        let greedy = alive
            .iter()
            .enumerate()
            .max_by(|(a, x), (b, y)| hard[**x].cmp(&hard[**y]).then(b.cmp(a)))
            .map(|(i, _)| i)
            .ok_or_else(|| invalid("no live family"))?;
        let chosen = match tape {
            Some(t) => {
                let mut rows = Vec::new();
                for &orbit in &alive {
                    let mut sum = Tensor::new(0f32, &Device::Cpu)?;
                    for (lane, &h) in states.iter().enumerate() {
                        let c = crate::geometric_context::q4_shadow_ste(
                            t.var("route.family")?.as_tensor(),
                        )?
                        .i((orbit, lane, h as usize))?;
                        let features = Tensor::from_vec(
                            profiles[orbit]
                                .iter()
                                .map(|&x| x as f32)
                                .collect::<Vec<_>>(),
                            9,
                            &Device::Cpu,
                        )?;
                        sum = (&sum + (&c * features)?.sum_all()?)?;
                    }
                    rows.push(sum);
                }
                t.choose(
                    "route.family",
                    anchor(
                        &alive
                            .iter()
                            .map(|&i| hard[i] as f32 * 0.25)
                            .collect::<Vec<_>>(),
                        Tensor::stack(&rows, 0)?,
                    )?,
                    greedy,
                )?
            }
            None => greedy,
        };
        out.push(alive.remove(chosen));
    }
    Ok(out)
}
fn choose_records(
    tape: &Option<Arc<CreditTape>>,
    rows: &[Vec<Option<u8>>],
    hard: &[i64],
) -> Result<usize> {
    let greedy = hard
        .iter()
        .enumerate()
        .max_by(|(a, x), (b, y)| x.cmp(y).then(b.cmp(a)))
        .map(|(i, _)| i)
        .ok_or_else(|| invalid("no route records"))?;
    match tape {
        Some(t) => {
            let c = TurnHeadConfig {
                classes: 1,
                slots: t.lanes * 3,
            };
            let g = rows
                .iter()
                .map(|r| {
                    table_scores(t.var("route.rank")?, &[r.clone()], c)?
                        .i(0)
                        .map_err(Into::into)
                })
                .collect::<Result<Vec<_>>>()?;
            t.choose(
                "route.rank",
                anchor(
                    &hard.iter().map(|&x| x as f32 * 0.25).collect::<Vec<_>>(),
                    Tensor::stack(&g, 0)?,
                )?,
                greedy,
            )
        }
        None => Ok(greedy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_occurrence_consumer::source_realizer::tests::{Fixture, TOK};
    use crate::geometric_turn_compiler::FeatureMode;
    use uor_r4_integer::geometric_cue_carrier::CueScoreMode;
    use uor_r4_integer::geometric_prefix_transport::PrefixScoreMode;

    #[test]
    fn whole_path_profile_quarters_use_exact_add_shift_scale() -> Result<()> {
        for q in -7..=7 {
            for x in [-2147483648, -31, 0, 31, 2147483647] {
                assert_eq!(quarter_scale(q, x)?, i64::from(q) * i64::from(x));
            }
        }
        assert!(quarter_scale(-8, 1).is_err());
        assert!(quarter_scale(8, 1).is_err());
        Ok(())
    }

    struct Parts {
        fixture: Fixture,
        parent: NativeSourceRealizer,
        generate: GenerateLearningWeights,
        cue: CueAngularWeights,
        prefix: PrefixAngularWeights,
        bridge: BridgeLearningWeights,
        continuation: ContinuationLearningWeights,
        a: u8,
        b: u8,
    }
    fn set(var: &Var, data: Vec<f32>) -> Result<()> {
        Ok(var.set(&Tensor::from_vec(data, var.shape(), var.device())?)?)
    }
    fn parts() -> Result<Parts> {
        let fixture = Fixture::new()?;
        let algebra = uor_r4_integer::h4_tables::HistoricalH4Tables::from_bytes(include_bytes!(
            "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let (a, b) = (0..120u8)
            .flat_map(|a| (0..120u8).map(move |b| (a, b)))
            .find(|&(a, b)| {
                let a = H4Code::try_from(a).ok();
                let b = H4Code::try_from(b).ok();
                a.zip(b).is_some_and(|(a, b)| {
                    algebra.compose(a, b) != algebra.compose(b, a)
                        && algebra.compose(a, a) != algebra.compose(a, b)
                        && algebra.compose(a, a) != algebra.compose(b, a)
                })
            })
            .ok_or_else(|| invalid("noncommuting H4 fixture pair missing"))?;
        let p = fixture.weights.parameters();
        let token = &p["consumer.context.token_transition"];
        let mut values = vec![0f32; token.elem_count()];
        for (id, row) in values.chunks_exact_mut(120).enumerate() {
            let code = if id == 4 || id == 7 {
                a
            } else if id == 5 {
                b
            } else {
                H4Code::IDENTITY.index()
            };
            row[code as usize] = 1.75;
        }
        set(token, values)?;
        let category = &p["consumer.context.token_category"];
        let mut values = vec![0f32; category.elem_count()];
        for row in values.chunks_exact_mut(33) {
            row[1] = 1.75;
        }
        set(category, values)?;
        let parent_path = fixture.path.join("whole-parent");
        fixture
            .weights
            .compile(fixture.identity.clone())?
            .save(&parent_path)?;
        let parent = NativeSourceRealizer::load(&parent_path, &fixture.weights, &fixture.identity)?;
        let generate =
            GenerateLearningWeights::seeded(parent.binding().clone(), 1, 41, &Device::Cpu)?;
        for (name, v) in generate.parameters() {
            set(
                &v,
                vec![if name == "generate.bias" { -1.25 } else { 0. }; v.elem_count()],
            )?;
        }
        let cue = CueAngularWeights::zero(&parent, &parent_path, CueScoreMode::DirectedRelative)?;
        let cue_path = fixture.path.join("whole-cue");
        cue.save(&cue_path)?;
        let nc = parent.compile_cue_carrier(cue.native()?)?;
        let prefix = PrefixAngularWeights::zero(
            &parent,
            &parent_path,
            &cue_path,
            &nc,
            PrefixScoreMode::DirectedRelative,
        )?;
        let bridge = BridgeLearningWeights::zeroed(parent.binding(), 1, &Device::Cpu)?;
        let continuation = ContinuationLearningWeights::zeroed_cross_state(
            parent.binding(),
            &parent.artifact_binding()?,
            1,
            &Device::Cpu,
        )?;
        Ok(Parts {
            fixture,
            parent,
            generate,
            cue,
            prefix,
            bridge,
            continuation,
            a,
            b,
        })
    }
    fn weights(parts: &Parts) -> Result<WholePathWeights<'_>> {
        let w = WholePathWeights::new(
            &parts.fixture.weights,
            &parts.generate,
            &parts.cue,
            &parts.prefix,
            &parts.bridge,
            &parts.continuation,
            vec!["job".into(), "home".into()],
            FitConfig {
                feature_mode: FeatureMode::LocalProduct,
                max_tokens: 32,
                max_words: 4,
                max_value_words: 1,
                ..FitConfig::default()
            },
            RouteLimits {
                per_root: 2,
                families: 1,
                records: 1,
            },
        )?;
        let algebra = uor_r4_integer::h4_tables::HistoricalH4Tables::from_bytes(include_bytes!(
            "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let a = H4Code::try_from(parts.a).map_err(|e| invalid(e.to_string()))?;
        let b = H4Code::try_from(parts.b).map_err(|e| invalid(e.to_string()))?;
        let ab = algebra.compose(a, b).index();
        let ba = algebra.compose(b, a).index();
        let aa = algebra.compose(a, a).index();
        let head = |var: &Var,
                    slots: usize,
                    biases: &[(usize, f32)],
                    cells: &[(usize, usize, u8, f32)]|
         -> Result<()> {
            let mut data = vec![0f32; var.elem_count()];
            let stride = 1 + slots * 120;
            for &(class, value) in biases {
                data[class * stride] = value;
            }
            for &(class, slot, code, value) in cells {
                data[class * stride + 1 + slot * 120 + code as usize] = value;
            }
            set(var, data)
        };
        head(
            &w.compiler[0],
            3,
            &[(2, 0.5)],
            &[(0, 2, ab, 1.75), (0, 2, aa, 1.75), (1, 2, ba, 1.75)],
        )?;
        head(&w.compiler[1], 3, &[(0, 0.5)], &[(1, 2, aa, 1.75)])?;
        head(
            &w.compiler[2],
            4,
            &[],
            &[(1, 1, parts.a, 1.5), (1, 1, parts.b, 1.5)],
        )?;
        head(&w.compiler[3], 3, &[], &[(3, 0, parts.b, 1.75)])?;
        let second = crate::e8_memory::e8_tables()?.members[1][0];
        head(&w.key, 1, &[(0, 0.5)], &[(second as usize, 0, aa, 1.75)])?;
        head(&w.rank, 3, &[], &[(0, 1, ab, 1.75), (0, 1, ba, 1.75)])?;
        Ok(w)
    }
    fn tokenizer() -> Result<ByteBpeTokenizer> {
        ByteBpeTokenizer::from_tokenizer_json_bytes(TOK.as_bytes())
            .ok_or_else(|| invalid("fixture tokenizer invalid"))
    }
    fn scope() -> SessionScope {
        SessionScope {
            scope: b"whole-fixture".to_vec(),
            entity: vec![4],
        }
    }
    fn limits() -> NativeSessionLimits {
        NativeSessionLimits {
            max_turns: 8,
            max_source_bytes: 64,
            max_store_records: 16,
            max_snapshot_bytes: 2 * 1024 * 1024,
        }
    }
    fn one() -> GenerationLimits {
        GenerationLimits {
            maximum_tokens: 1,
            maximum_retained_steps: 1,
        }
    }
    fn sampled() -> Mode {
        Mode::Sampled(Sampling {
            seed: 1001,
            temperature: 0.02,
            exploration: 0.00001,
            maximum_decisions: 4096,
        })
    }

    #[test]
    fn whole_path_real_compiler_memory_routing_emitter_and_greedy_reload() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("revision"), Mode::Greedy)?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 731, 16)?;
        let first = session.turn("a b", one())?;
        assert_eq!(
            first.action,
            CompiledAction::Assert {
                relation: 1,
                span: SourceSpan { start: 2, end: 3 }
            }
        );
        assert_eq!(first.generation.generated_ids, vec![5]);
        session.turn("a a", one())?;
        let changed = session.turn("b a", one())?;
        assert_eq!(
            changed.action,
            CompiledAction::Correct {
                relation: 1,
                span: SourceSpan { start: 2, end: 3 }
            }
        );
        // Existing native lexicalization emits the leading-space " a" token;
        // the exact store still owns the original independent slice [4].
        assert_eq!(changed.generation.generated_ids, vec![7]);
        assert_eq!(session.index.records.len(), 2);
        assert_eq!(
            session
                .route_profiles()?
                .iter()
                .filter(|p| p[0] > 0)
                .count(),
            2
        );
        let current = session.turn("a", one())?;
        assert_eq!(current.generation.generated_ids, vec![7]);
        let initial = session.turn("b", one())?;
        assert_eq!(
            initial.action,
            CompiledAction::Query {
                relation: 1,
                view: HistoryView::Initial
            }
        );
        assert_eq!(initial.generation.generated_ids, vec![5]);
        let snapshot = session.snapshot()?;
        let loaded = WholePathRevision::load(&revision.root, revision.sha256())?;
        let mut restored = loaded.session(&tok, TOK.as_bytes(), scope(), limits(), 731, 16)?;
        restored.restore(&snapshot, &sha256_bytes(&snapshot))?;
        assert_eq!(session.index, restored.index);
        assert_eq!(restored.turn("a", one())?.generation.generated_ids, vec![7]);
        Ok(())
    }

    #[test]
    fn whole_path_device_graph_reference_matches_native_compiler_store_and_q24() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(
            &p.parent,
            &p.fixture.path.join("device-reference"),
            Mode::Greedy,
        )?;
        let Mode::Sampled(sampling) = sampled() else {
            return Err(invalid("sampled fixture absent"));
        };
        assert!(
            device::WholePathCudaProducer::new(&revision, w.parameters()?, sampling.clone())
                .is_err()
        );
        let mut producer =
            device::WholePathCudaProducer::admit(&revision, w.parameters()?, sampling.clone())?;
        producer.greedy = true;
        let mut native = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 791, 16)?;
        let mut device_session = device::CudaWholePathSession::from_producer(
            &revision,
            &tok,
            TOK.as_bytes(),
            producer,
            EpisodeSpec {
                scope: scope(),
                limits: limits(),
                lineage: 791,
                capacity: 16,
            },
        )?;
        for source in ["a b", "a a", "b a", "b"] {
            let out = native.turn(source, one())?;
            let got = device_session.turn(source, one())?;
            assert_eq!(outcome_digest(&out)?, outcome_digest(&got)?);
            assert_eq!(native.store().commit(), device_session.store().commit());
        }
        let pending = native.prepare_turn("b")?;
        let expected = native.native.teacher_step(&pending.native, &[])?;
        let mut producer =
            device::WholePathCudaProducer::admit(&revision, w.parameters()?, sampling)?;
        producer.greedy = true;
        let step = producer.bank_step(pending.native.bank(), &[])?;
        assert_eq!(step.copy_ids, expected.copy_token_ids);
        assert_eq!(
            step.generated_native_q24.to_vec1::<f64>()?,
            expected
                .generate_raw_scores_q24
                .iter()
                .map(|&v| v as f64)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            step.copied_native_q24.to_vec1::<f64>()?,
            expected
                .copy_raw_scores_q24
                .iter()
                .map(|&v| v as f64)
                .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn whole_path_device_nonzero_shared_factors_match_native_q24() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        for (name, var) in w.parameters()? {
            if [
                "consumer.potential.",
                "cue.",
                "prefix.",
                "read_state_bridge.",
                "generate.",
                "continuation.",
                "memory.signal",
            ]
            .iter()
            .any(|prefix| name.starts_with(prefix))
            {
                set(
                    &var,
                    (0..var.elem_count())
                        .map(|i| (i % 15) as f32 * 0.25 - 1.75)
                        .collect(),
                )?;
            }
        }
        let tok = tokenizer()?;
        let revision = w.rebind(
            &p.parent,
            &p.fixture.path.join("device-nonzero"),
            Mode::Greedy,
        )?;
        let mut native = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 790, 16)?;
        native.turn("a b", one())?;
        let pending = native.prepare_turn("b")?;
        let Mode::Sampled(sampling) = sampled() else {
            return Err(invalid("sampled fixture absent"));
        };
        let mut producer =
            device::WholePathCudaProducer::admit(&revision, w.parameters()?, sampling)?;
        producer.greedy = true;
        for prefix in [vec![], vec![5], vec![5, 4]] {
            let expected = native.native.teacher_step(&pending.native, &prefix)?;
            let got = producer.bank_step(pending.native.bank(), &prefix)?;
            assert_eq!(got.copy_ids, expected.copy_token_ids);
            assert_eq!(
                got.generated_native_q24.to_vec1::<f64>()?,
                expected
                    .generate_raw_scores_q24
                    .iter()
                    .map(|&v| v as f64)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                got.copied_native_q24.to_vec1::<f64>()?,
                expected
                    .copy_raw_scores_q24
                    .iter()
                    .map(|&v| v as f64)
                    .collect::<Vec<_>>()
            );
        }
        Ok(())
    }

    #[test]
    fn whole_path_device_reference_credit_reaches_shared_owners() -> Result<()> {
        let p = parts()?;
        let mut w = weights(&p)?;
        // Admit both occupied families so rank credit is exercised regardless
        // of which family the tiny stochastic fixture selects first.
        w.limits.families = 15;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("device-credit"), sampled())?;
        let Mode::Sampled(sampling) = sampled() else {
            return Err(invalid("sampled fixture absent"));
        };
        let mut producer =
            device::WholePathCudaProducer::admit(&revision, w.parameters()?, sampling)?;
        producer.restore_prototype_credit(&revision)?;
        let mut session = device::CudaWholePathSession::from_producer(
            &revision,
            &tok,
            TOK.as_bytes(),
            producer,
            EpisodeSpec {
                scope: scope(),
                limits: limits(),
                lineage: 796,
                capacity: 16,
            },
        )?;
        for source in ["a b", "a a", "b a", "a"] {
            session.turn(source, one())?;
        }
        let gradients = session.terminal_objective(1., 0.)?.backward()?;
        for name in [
            "compiler.act",
            "compiler.relation",
            "compiler.span",
            "route.key",
            "route.family",
            "route.rank",
            "read_state_bridge.bias",
            "generate.bias",
            "generate.prototype_choices",
            "memory.signal",
            "consumer.context.token_transition",
            "consumer.potential.context_unary",
            "cue.coefficients",
            "prefix.coefficients",
            "continuation.cross_state",
        ] {
            let parameters = w.parameters()?;
            let var = &parameters[name];
            let gradient = gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid(format!("device credit absent:{name}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(gradient.iter().all(|v| v.is_finite()), "{name}");
            assert!(
                gradient.iter().any(|v| *v != 0.),
                "zero device credit:{name}"
            );
        }
        assert_eq!(w.parameter_sha256()?, revision.parameter_sha256());
        Ok(())
    }

    #[test]
    fn whole_path_device_teacher_rejects_foreign_prepared_episode() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("device-foreign"), sampled())?;
        let Mode::Sampled(sampling) = sampled() else {
            return Err(invalid("sampled fixture absent"));
        };
        let mut sessions = Vec::new();
        // Even identical durable snapshots must have separate live credit owners.
        for lineage in [797, 797] {
            let producer =
                device::WholePathCudaProducer::admit(&revision, w.parameters()?, sampling.clone())?;
            sessions.push(device::CudaWholePathSession::from_producer(
                &revision,
                &tok,
                TOK.as_bytes(),
                producer,
                EpisodeSpec {
                    scope: scope(),
                    limits: limits(),
                    lineage,
                    capacity: 16,
                },
            )?);
        }
        let own = sessions[0].prepare_turn("a b")?;
        let foreign = sessions[1].prepare_turn("b a")?;
        assert!(sessions[0]
            .teacher_ranking_objective(&foreign, &[5, 1], 0.)
            .is_err());
        // Refusal must not consume the pending packet or poison its graph.
        sessions[0].teacher_ranking_objective(&own, &[5, 1], 0.)?;
        assert_eq!(sessions[0].store().commit(), 0);
        Ok(())
    }

    #[test]
    fn whole_path_cpu_teacher_rejects_foreign_identical_snapshot() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("cpu-foreign"), sampled())?;
        let left = revision.fork_episode(1001)?;
        let right = revision.fork_episode(2001)?;
        let mut a = left.session(&tok, TOK.as_bytes(), scope(), limits(), 799, 16)?;
        let mut b = right.session(&tok, TOK.as_bytes(), scope(), limits(), 799, 16)?;
        let own = a.prepare_turn("a b")?;
        let foreign = b.prepare_turn("b a")?;
        assert!(a.teacher_ranking_objective(&foreign, &[5, 1], 0.).is_err());
        a.teacher_ranking_objective(&own, &[5, 1], 0.)?;
        assert_eq!(a.store().commit(), 0);
        Ok(())
    }

    #[test]
    fn whole_path_device_reference_recovery_rebuilds_shared_credit_without_updates() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("device-replay"), sampled())?;
        let Mode::Sampled(sampling) = sampled() else {
            return Err(invalid("sampled fixture absent"));
        };
        let build = || -> Result<device::WholePathCudaProducer> {
            let mut p =
                device::WholePathCudaProducer::admit(&revision, w.parameters()?, sampling.clone())?;
            p.restore_prototype_credit(&revision)?;
            Ok(p)
        };
        let mut original = device::CudaWholePathSession::from_producer(
            &revision,
            &tok,
            TOK.as_bytes(),
            build()?,
            EpisodeSpec {
                scope: scope(),
                limits: limits(),
                lineage: 792,
                capacity: 16,
            },
        )?;
        original.turn("a b", one())?;
        let pending = original.prepare_turn("b")?;
        let loss = original.teacher_ranking_objective(&pending, &[5, 1], 0.)?;
        let saved = original.sampled_checkpoint()?;
        let parsed = device::CudaWholePathSession::read_checkpoint(
            &revision,
            &saved,
            &sha256_bytes(&saved),
        )?;
        let recovered = device::CudaWholePathSession::replay_checkpoint(
            &revision,
            &tok,
            TOK.as_bytes(),
            build()?,
            parsed,
            &saved,
        )?;
        assert_eq!(saved, recovered.sampled_checkpoint()?);
        let left = loss.backward()?;
        let right = recovered
            .completed_teacher_credit()?
            .objective()
            .backward()?;
        for (name, var) in w.parameters()? {
            match (left.get(var.as_tensor()), right.get(var.as_tensor())) {
                (Some(a), Some(b)) => assert_eq!(
                    a.flatten_all()?.to_vec1::<f32>()?,
                    b.flatten_all()?.to_vec1::<f32>()?,
                    "{name}"
                ),
                (None, None) => {}
                _ => return Err(invalid(format!("recovered gradient differs:{name}"))),
            }
        }
        assert_eq!(w.parameter_sha256()?, revision.parameter_sha256());
        Ok(())
    }

    #[test]
    fn whole_path_device_reference_batch_recovery_and_all_spiral_frames() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("device-batch"), sampled())?;
        let Mode::Sampled(sampling) = sampled() else {
            return Err(invalid("sampled fixture absent"));
        };
        let build = || -> Result<device::WholePathCudaProducer> {
            let mut p =
                device::WholePathCudaProducer::admit(&revision, w.parameters()?, sampling.clone())?;
            p.restore_prototype_credit(&revision)?;
            Ok(p)
        };
        let global = build()?;
        let tables = crate::e8_memory::e8_tables()?;
        for frame in 0..64 {
            let ids = (0..240)
                .map(|r| tables.transport_root(r, frame).map(u32::from))
                .collect::<Result<Vec<_>>>()?;
            let physical = Tensor::from_vec(ids, 240, &Device::Cpu)?;
            assert_eq!(
                global.canonical_roots(&physical, frame)?.to_vec1::<u32>()?,
                (0..240u32).collect::<Vec<_>>()
            );
        }
        let expected = global.objective(2., 0.)?;
        let spec = EpisodeSpec {
            scope: scope(),
            limits: limits(),
            lineage: 795,
            capacity: 16,
        };
        let episodes = [(1001, spec.clone()), (2001, spec)];
        let mut batch = device::CudaWholePathBatch::from_global(
            &revision,
            &tok,
            TOK.as_bytes(),
            global,
            &episodes,
        )?;
        let mut credits = Vec::new();
        for (i, cost) in [1., 3.].into_iter().enumerate() {
            let episode = batch.episode_mut(i)?;
            episode.turn("a b", one())?;
            episode.transport_routes(7)?;
            credits.push(episode.terminal_credit(cost, 0.)?);
        }
        let saved = batch.episode_mut(1)?.sampled_checkpoint()?;
        let loss = batch.objective(&credits)?;
        assert_eq!(loss.to_scalar::<f32>()?, 2.);
        let left = loss.backward()?;
        let right = expected.backward()?;
        let v = w.generate.prototype_choices.as_tensor();
        assert_eq!(
            left.get(v)
                .ok_or_else(|| invalid("batch prototype gradient absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?,
            right
                .get(v)
                .ok_or_else(|| invalid("reference prototype gradient absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?
        );
        credits.swap(0, 1);
        assert!(batch.objective(&credits).is_err());
        credits.swap(0, 1);
        let mut fresh = device::CudaWholePathBatch::from_global(
            &revision,
            &tok,
            TOK.as_bytes(),
            build()?,
            &episodes,
        )?;
        let mut corrupted: serde_json::Value = serde_json::from_slice(&saved)?;
        let rng = corrupted["rng"]
            .as_u64()
            .ok_or_else(|| invalid("missing saved RNG"))?;
        corrupted["rng"] = serde_json::json!(rng ^ 1);
        let corrupted = serde_json::to_vec(&corrupted)?;
        assert!(fresh
            .recover_episode(1, TOK.as_bytes(), &corrupted, &sha256_bytes(&corrupted))
            .is_err());
        assert_eq!(fresh.episode_mut(1)?.store().commit(), 0);
        fresh.recover_episode(1, TOK.as_bytes(), &saved, &sha256_bytes(&saved))?;
        assert_eq!(saved, fresh.episode_mut(1)?.sampled_checkpoint()?);
        let a = batch.episode_mut(1)?.turn("b a", one())?;
        let b = fresh.episode_mut(1)?.turn("b a", one())?;
        assert_eq!(outcome_digest(&a)?, outcome_digest(&b)?);
        assert_eq!(
            batch.episode_mut(1)?.sampled_checkpoint()?,
            fresh.episode_mut(1)?.sampled_checkpoint()?
        );
        batch.episode_mut(0)?.turn("a a", one())?;
        assert!(batch.objective(&credits).is_err());
        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "requires separately admitted CUDA qualification; no automatic pod or training"]
    fn whole_path_cuda_actual_device_preserves_native_q24_and_recovery() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(
            &p.parent,
            &p.fixture.path.join("cuda-qualification"),
            Mode::Greedy,
        )?;
        let cuda = Device::new_cuda(0)?;
        let params = w
            .parameters()?
            .into_iter()
            .map(|(name, v)| Ok((name, Var::from_tensor(&v.to_device(&cuda)?)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let Mode::Sampled(sampling) = sampled() else {
            return Err(invalid("sampled fixture absent"));
        };
        let mut host = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 793, 16)?;
        let pending = host.prepare_turn("a b")?;
        let expected = host.native.teacher_step(&pending.native, &[])?;
        let mut producer =
            device::WholePathCudaProducer::new(&revision, params.clone(), sampling.clone())?;
        producer.greedy = true;
        let step = producer.bank_step(pending.native.bank(), &[])?;
        assert!(step.generated.device().is_cuda());
        assert_eq!(
            step.generated_native_q24.to_vec1::<f64>()?,
            expected
                .generate_raw_scores_q24
                .iter()
                .map(|&v| v as f64)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            step.copied_native_q24.to_vec1::<f64>()?,
            expected
                .copy_raw_scores_q24
                .iter()
                .map(|&v| v as f64)
                .collect::<Vec<_>>()
        );
        let mut session = device::CudaWholePathSession::new(
            &revision,
            &tok,
            TOK.as_bytes(),
            params.clone(),
            sampling,
            EpisodeSpec {
                scope: scope(),
                limits: limits(),
                lineage: 794,
                capacity: 16,
            },
        )?;
        session.turn("a b", one())?;
        let checkpoint = session.sampled_checkpoint()?;
        let restored = device::CudaWholePathSession::recover_sampled(
            &revision,
            &tok,
            TOK.as_bytes(),
            params,
            &checkpoint,
            &sha256_bytes(&checkpoint),
        )?;
        assert_eq!(checkpoint, restored.sampled_checkpoint()?);
        Ok(())
    }

    #[test]
    fn whole_path_rebinding_rejects_stale_shared_masters_packets_and_snapshots() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("before"), Mode::Greedy)?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 732, 16)?;
        session.turn("a b", one())?;
        let snapshot = session.snapshot()?;
        let old = session.prepare_turn("a")?;
        set(&w.compiler[1], vec![0f32; w.compiler[1].elem_count()])?;
        assert!(session.emit_prepared(old, one()).is_err());
        let next = w.rebind(&p.parent, &p.fixture.path.join("after"), Mode::Greedy)?;
        assert_ne!(next.sha256(), revision.sha256());
        let mut fresh = next.session(&tok, TOK.as_bytes(), scope(), limits(), 732, 16)?;
        assert!(fresh.restore(&snapshot, &sha256_bytes(&snapshot)).is_err());
        assert_eq!(fresh.store().commit(), 0);
        Ok(())
    }

    #[test]
    fn whole_path_failed_emit_and_bad_snapshot_are_atomic() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("atomic"), Mode::Greedy)?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 733, 16)?;
        let pending = session.prepare_turn("a b")?;
        assert!(session
            .emit_prepared(
                pending,
                GenerationLimits {
                    maximum_tokens: 0,
                    maximum_retained_steps: 0
                }
            )
            .is_err());
        assert_eq!(session.store().commit(), 0);
        assert!(session.index.records.is_empty());
        session.turn("a b", one())?;
        let snapshot = session.snapshot()?;
        let mut tuple: (String, String, RouteIndex, String, Vec<u8>) =
            serde_json::from_slice(&snapshot)?;
        tuple
            .2
            .records
            .values_mut()
            .next()
            .ok_or_else(|| invalid("route absent"))?
            .payload_sha256 = "f".repeat(64);
        let corrupt = serde_json::to_vec(&tuple)?;
        let loaded = WholePathRevision::load(&revision.root, revision.sha256())?;
        let mut fresh = loaded.session(&tok, TOK.as_bytes(), scope(), limits(), 733, 16)?;
        assert!(fresh.restore(&corrupt, &sha256_bytes(&corrupt)).is_err());
        assert_eq!(fresh.store().commit(), 0);
        assert!(fresh.index.records.is_empty());
        Ok(())
    }

    #[test]
    fn whole_path_spiral_frame_roundtrip_preserves_exact_handles_and_reload() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("frame"), Mode::Greedy)?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 734, 16)?;
        session.turn("a b", one())?;
        session.turn("a a", one())?;
        let before = session.index.clone();
        let bytes = session
            .store()
            .to_bytes()
            .map_err(|e| invalid(e.to_string()))?;
        let group = uor_r4_core::spiralcore_operator::cl06_finite_composition_table()
            .map_err(|e| invalid(e.to_string()))?;
        session.transport_routes(7)?;
        let snapshot = session.snapshot()?;
        let loaded = WholePathRevision::load(&revision.root, revision.sha256())?;
        let mut fresh = loaded.session(&tok, TOK.as_bytes(), scope(), limits(), 734, 16)?;
        fresh.restore(&snapshot, &sha256_bytes(&snapshot))?;
        fresh.transport_routes(
            group
                .inverse_index(7)
                .ok_or_else(|| invalid("no inverse"))?,
        )?;
        assert_eq!(fresh.index, before);
        assert_eq!(
            fresh
                .store()
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?,
            bytes
        );
        Ok(())
    }

    #[test]
    fn whole_path_sampled_trajectory_credit_reaches_actual_connected_owners() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("sampled"), sampled())?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 735, 16)?;
        session.turn("a b", one())?;
        session.turn("a a", one())?;
        session.turn("b a", one())?;
        session.turn("a", one())?;
        let inventory = session.credit_inventory()?;
        for owner in [
            "compiler.act",
            "compiler.relation",
            "compiler.span",
            "compiler.view",
            "route.key",
            "route.family",
            "route.rank",
            "reader.donor",
            "bridge.action",
            "emitter.token",
            "context.Transition",
            "context.Root",
            "context.Category",
            "generate.prototype",
        ] {
            assert!(
                inventory.get(owner).is_some_and(|&n| n > 0),
                "missing executed owner {owner}: {inventory:?}"
            );
        }
        let objective = session.episode_objective(1., 0.)?;
        let gradients = objective.backward()?;
        let parameters = w.parameters()?;
        for owner in [
            "compiler.act",
            "compiler.relation",
            "compiler.span",
            "route.key",
            "route.family",
            "route.rank",
            "read_state_bridge.bias",
            "generate.bias",
            "generate.prototype_choices",
            "memory.signal",
            "consumer.context.token_transition",
            "consumer.potential.context_unary",
            "cue.coefficients",
            "prefix.coefficients",
            "continuation.cross_state",
        ] {
            let var = &parameters[owner];
            let gradient = gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid(format!("credit absent {owner}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(gradient.iter().all(|x| x.is_finite()));
            assert!(
                gradient.iter().any(|x| x.abs() > 0.),
                "zero connected credit {owner}"
            );
        }
        assert!(session.snapshot().is_err());
        assert!(revision
            .session(&tok, TOK.as_bytes(), scope(), limits(), 735, 16)
            .is_err());
        Ok(())
    }

    #[test]
    fn whole_path_teacher_pooled_complete_answer_credit_is_not_first_token_ce() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("teacher"), sampled())?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 736, 16)?;
        let pending = session.prepare_turn("a b")?;
        assert!(session
            .teacher_ranking_objective(&pending, &[5], 0.)
            .is_err());
        assert!(session
            .teacher_ranking_objective(&pending, &[1, 5, 1], 0.)
            .is_err());
        let loss = session.teacher_ranking_objective(&pending, &[5, 1], 0.)?;
        assert!(loss.to_scalar::<f32>()?.is_finite());
        let gradients = loss.backward()?;
        for name in [
            "generate.bias",
            "memory.signal",
            "compiler.act",
            "consumer.context.token_transition",
        ] {
            let parameters = w.parameters()?;
            let g = gradients
                .get(parameters[name].as_tensor())
                .ok_or_else(|| invalid(format!("teacher credit absent {name}")))?;
            assert!(g
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|x| x.abs() > 0.));
        }
        assert_eq!(session.store().commit(), 0);
        assert!(session.emit_prepared(pending, one()).is_err());
        Ok(())
    }

    #[test]
    fn whole_path_sampling_score_function_matches_exact_tiny_expectation() -> Result<()> {
        let variable = Var::from_vec(vec![0.2f32, -0.1], 2, &Device::Cpu)?;
        let p = candle_nn::ops::softmax(variable.as_tensor(), 0)?.affine(0.9, 0.05)?;
        let probabilities = p.to_vec1::<f32>()?;
        let costs = [2f32, -1.];
        let mut expected = vec![0f32; 2];
        for i in 0..2 {
            let term = p.i(i)?.log()?.affine(f64::from(costs[i]), 0.)?;
            let gradient = term
                .backward()?
                .get(variable.as_tensor())
                .ok_or_else(|| invalid("tiny policy credit missing"))?
                .to_vec1::<f32>()?;
            for j in 0..2 {
                expected[j] += probabilities[i] * gradient[j];
            }
        }
        let exact = (&p * Tensor::from_vec(costs.to_vec(), 2, &Device::Cpu)?)?.sum_all()?;
        let gradient = exact
            .backward()?
            .get(variable.as_tensor())
            .ok_or_else(|| invalid("tiny exact credit missing"))?
            .to_vec1::<f32>()?;
        for (a, b) in expected.iter().zip(gradient) {
            assert!((a - b).abs() < 1e-6);
        }
        Ok(())
    }

    #[test]
    fn whole_path_sampled_recovery_rebuilds_graph_rng_store_and_next_output() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("recover"), sampled())?;
        let mut original = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 801, 16)?;
        original.turn("a b", one())?;
        original.turn("b a", one())?;
        let checkpoint = original.sampled_checkpoint()?;
        let loaded = WholePathRevision::load(&revision.root, revision.sha256())?;
        w.bind_recovery(&p.parent, &loaded)?;
        let mut recovered = loaded.recover_sampled(
            &tok,
            TOK.as_bytes(),
            &checkpoint,
            &sha256_bytes(&checkpoint),
        )?;
        assert_eq!(recovered.sampled_checkpoint()?, checkpoint);
        let a = original.turn("a", one())?;
        let b = recovered.turn("a", one())?;
        assert_eq!(outcome_digest(&a)?, outcome_digest(&b)?);
        assert_eq!(
            original.sampled_checkpoint()?,
            recovered.sampled_checkpoint()?
        );
        let ga = original.episode_objective(2., 0.5)?.backward()?;
        let gb = recovered.episode_objective(2., 0.5)?.backward()?;
        for (_, var) in w.parameters()? {
            match (ga.get(var.as_tensor()), gb.get(var.as_tensor())) {
                (Some(a), Some(b)) => assert_eq!(
                    a.flatten_all()?.to_vec1::<f32>()?,
                    b.flatten_all()?.to_vec1::<f32>()?
                ),
                (None, None) => {}
                _ => return Err(invalid("recovery owner inventory differs")),
            }
        }
        Ok(())
    }

    #[test]
    fn whole_path_sampled_recovery_rejects_reauthored_rng_and_decisions() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("bad-recover"), sampled())?;
        let fork = revision.fork_episode(9001)?;
        let mut session = fork.session(&tok, TOK.as_bytes(), scope(), limits(), 802, 16)?;
        session.turn("a b", one())?;
        let mut saved: SampledCheckpoint = serde_json::from_slice(&session.sampled_checkpoint()?)?;
        saved.rng ^= 1;
        let bad = serde_json::to_vec(&saved)?;
        let fresh = revision.fork_episode(9001)?;
        assert!(fresh
            .recover_sampled(&tok, TOK.as_bytes(), &bad, &sha256_bytes(&bad))
            .is_err());
        let mismatch = revision.fork_episode(9002)?;
        assert!(mismatch
            .recover_sampled(&tok, TOK.as_bytes(), &bad, &sha256_bytes(&bad))
            .is_err());
        assert_eq!(session.store().commit(), 1);
        Ok(())
    }

    #[test]
    fn whole_path_batch_has_shared_vars_and_single_global_prototype_credit() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("batch"), sampled())?;
        let batch = revision.batch(&[1001, 2001])?;
        let mut sessions = Vec::new();
        for i in 0..2 {
            let mut session = batch.episode(i)?.session(
                &tok,
                TOK.as_bytes(),
                scope(),
                limits(),
                803 + i as u64,
                16,
            )?;
            assert_eq!(session.store().commit(), 0);
            session.turn("a b", one())?;
            session.turn("a", one())?;
            assert!(!session
                .credit_inventory()?
                .contains_key("generate.prototype"));
            sessions.push(session);
        }
        let credits = vec![
            sessions[0].terminal_credit(2., 0.)?,
            sessions[1].terminal_credit(4., 0.)?,
        ];
        let objective = batch.objective(&credits)?;
        assert_eq!(objective.to_scalar::<f32>()?, 3.);
        let gradient = objective.backward()?;
        let expected = batch.prototype.objective(3., 0.)?.backward()?;
        let var = &w.generate.prototype_choices;
        assert_eq!(
            gradient
                .get(var.as_tensor())
                .ok_or_else(|| invalid("global prototype credit absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?,
            expected
                .get(var.as_tensor())
                .ok_or_else(|| invalid("expected global credit absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?
        );
        assert!(batch
            .objective(&[
                sessions[1].terminal_credit(4., 0.)?,
                sessions[0].terminal_credit(2., 0.)?
            ])
            .is_err());
        assert!(revision.batch(&[1, 1]).is_err());
        let checkpoint = sessions[1].sampled_checkpoint()?;
        let fresh = revision.fork_episode(2001)?;
        let recovered = fresh.recover_sampled(
            &tok,
            TOK.as_bytes(),
            &checkpoint,
            &sha256_bytes(&checkpoint),
        )?;
        assert_eq!(recovered.sampled_checkpoint()?, checkpoint);
        Ok(())
    }

    #[test]
    fn whole_path_teacher_checkpoint_recovers_complete_loss_and_ancestry() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let revision = w.rebind(
            &p.parent,
            &p.fixture.path.join("teacher-recover"),
            sampled(),
        )?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 806, 16)?;
        let pending = session.prepare_turn("a b")?;
        session.teacher_credit(&pending, &[5, 1], 0.)?;
        let checkpoint = session.sampled_checkpoint()?;
        let loaded = WholePathRevision::load(&revision.root, revision.sha256())?;
        w.bind_recovery(&p.parent, &loaded)?;
        let recovered = loaded.recover_sampled(
            &tok,
            TOK.as_bytes(),
            &checkpoint,
            &sha256_bytes(&checkpoint),
        )?;
        assert_eq!(recovered.sampled_checkpoint()?, checkpoint);
        assert!(recovered.terminal_credit(1., 0.).is_err());
        Ok(())
    }

    #[test]
    fn whole_path_typed_store_signal_enters_the_native_pool() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let absent_initial = crate::stack_store::StoreSignal {
            status: crate::stack_store::StoreStatus::Absent,
            view: HistoryView::Initial,
        }
        .code() as usize
            * 2;
        let mut signal = vec![0f32; w.signal.elem_count()];
        signal[absent_initial * 8 + 1] = 1.75;
        set(&w.signal, signal)?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("signal"), Mode::Greedy)?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 737, 16)?;
        let pending = session.prepare_turn("b")?;
        assert_eq!(
            pending.native.bank().status_code(),
            Some(absent_initial as u8)
        );
        let result = session.emit_prepared(pending, one())?;
        assert_eq!(result.generation.generated_ids, [1]);
        assert!(result.generation.steps[0].bank_trace.is_none());
        assert_eq!(session.store().commit(), 0);
        Ok(())
    }

    #[test]
    fn whole_path_joint_rebind_refreshes_all_component_parents_without_cloning_masters(
    ) -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let before = w.rebind(&p.parent, &p.fixture.path.join("all-before"), Mode::Greedy)?;
        let parameters = w.parameters()?;
        for name in [
            "consumer.context.token_root",
            "consumer.potential.context_unary",
            "generate.bias",
            "read_state_bridge.bias",
            "cue.coefficients",
            "prefix.coefficients",
            "continuation.cross_state",
        ] {
            let v = &parameters[name];
            let mut data = values(v)?;
            data[0] += 0.25;
            set(v, data)?;
        }
        let mut prototypes = values(&p.generate.prototype_choices)?;
        prototypes[1] = 1.75;
        set(&p.generate.prototype_choices, prototypes)?;
        let after = w.rebind(&p.parent, &p.fixture.path.join("all-after"), Mode::Greedy)?;
        assert_ne!(
            before.manifest.source.metadata_sha256,
            after.manifest.source.metadata_sha256
        );
        for name in [
            "compiler.bin",
            "generate.bin",
            "bridge.bin",
            "cue.bin",
            "prefix.bin",
            "continuation.bin",
        ] {
            assert_ne!(
                before.manifest.files[name], after.manifest.files[name],
                "stale payload {name}"
            );
        }
        let live = after
            .tape_slot
            .lock()
            .map_err(|_| invalid("tape lock"))?
            .clone()
            .ok_or_else(|| invalid("no bound tape"))?;
        for (name, v) in parameters {
            assert_eq!(v.id(), live.params[&name].id(), "duplicated master {name}");
        }
        let cue: uor_r4_integer::geometric_cue_carrier::CueCarrierMetadata =
            serde_json::from_slice(&std::fs::read(after.root.join("cue.json"))?)?;
        assert_eq!(cue.parent_artifact, after.manifest.source);
        after.generator(None)?;
        Ok(())
    }

    #[test]
    fn whole_path_pooled_sampler_uses_one_temperature_and_all_aliases() -> Result<()> {
        let var = Var::from_vec(vec![0.2f32, -0.3, 0.7], 3, &Device::Cpu)?;
        let tape = CreditTape::new(
            BTreeMap::from([("fixture.actions".into(), var.clone())]),
            Mode::Sampled(Sampling {
                seed: 9,
                temperature: 0.5,
                exploration: 0.1,
                maximum_decisions: 8,
            }),
            1,
        )?;
        let physical = candle_nn::ops::softmax(&var.affine(2., 0.)?, 0)?;
        let aliases = (physical.i(0)? + physical.i(1)?)?;
        let token_logits = Tensor::stack(&[aliases.log()?, physical.i(2)?.log()?], 0)?;
        let chosen = tape.choose_scaled("emitter.token", token_logits, 0, false)?;
        let expected = if chosen == 0 {
            aliases.to_scalar::<f32>()?
        } else {
            physical.i(2)?.to_scalar::<f32>()?
        } * 0.9
            + 0.05;
        let receipt = tape.receipts()?;
        assert_eq!(receipt.len(), 1);
        assert!((receipt[0].probability - expected).abs() < 1e-6);
        let gradient = tape
            .objective(1., 0.)?
            .backward()?
            .get(var.as_tensor())
            .ok_or_else(|| invalid("alias credit missing"))?
            .to_vec1::<f32>()?;
        assert!(gradient.iter().all(|x| x.is_finite() && x.abs() > 1e-8));
        Ok(())
    }

    #[test]
    fn whole_path_fractional_masters_restore_into_same_owners_and_new_revision() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let mut data = values(&w.key)?;
        data[0] = 0.501;
        set(&w.key, data)?;
        let before = w.rebind(&p.parent, &p.fixture.path.join("fractional"), Mode::Greedy)?;
        let old_id = w.key.id();
        set(&w.key, vec![0f32; w.key.elem_count()])?;
        assert_ne!(w.parameter_sha256()?, before.parameter_sha256());
        w.restore_masters(&before)?;
        assert_eq!(w.key.id(), old_id);
        assert_eq!(values(&w.key)?[0].to_bits(), 0.501f32.to_bits());
        assert_eq!(w.parameter_sha256()?, before.parameter_sha256());
        let rebound = w.rebind(
            &p.parent,
            &p.fixture.path.join("fractional-rebound"),
            Mode::Greedy,
        )?;
        assert_eq!(before.manifest.files, rebound.manifest.files);
        Ok(())
    }

    #[test]
    fn whole_path_sampled_failed_preparation_does_not_commit_or_continue_rng() -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        let mode = Mode::Sampled(Sampling {
            seed: 11,
            temperature: 0.02,
            exploration: 0.01,
            maximum_decisions: 12,
        });
        let revision = w.rebind(&p.parent, &p.fixture.path.join("failed-sample"), mode)?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 739, 16)?;
        assert!(session.prepare_turn("a b").is_err());
        assert_eq!(session.store().commit(), 0);
        assert!(session.index.records.is_empty());
        let receipts = session.credit_inventory()?;
        assert!(session.prepare_turn("a").is_err());
        assert_eq!(receipts, session.credit_inventory()?);
        Ok(())
    }

    #[test]
    fn whole_path_alias_logsumexp_keeps_underflowed_alternatives_finite() -> Result<()> {
        let var = Var::from_vec(vec![8f32, -8., -8.], 3, &Device::Cpu)?;
        let tape = CreditTape::new(
            BTreeMap::from([("fixture.extreme".into(), var.clone())]),
            Mode::Sampled(Sampling {
                seed: 13,
                temperature: 0.02,
                exploration: 0.1,
                maximum_decisions: 4,
            }),
            1,
        )?;
        let scaled = var.affine(50., 0.)?;
        let law = Tensor::stack(
            &[
                log_sum_exp(&scaled.narrow(0, 0, 1)?)?,
                log_sum_exp(&scaled.narrow(0, 1, 2)?)?,
            ],
            0,
        )?;
        assert!(law.to_vec1::<f32>()?.iter().all(|x| x.is_finite()));
        let chosen = tape.choose_scaled("emitter.token", law, 0, false)?;
        let p = tape.receipts()?[0].probability;
        assert!((p - if chosen == 0 { 0.95 } else { 0.05 }).abs() < 1e-6);
        let gradients = tape.objective(1., 0.)?.backward()?;
        assert!(gradients
            .get(var.as_tensor())
            .ok_or_else(|| invalid("underflow gradient absent"))?
            .to_vec1::<f32>()?
            .iter()
            .all(|x| x.is_finite()));
        Ok(())
    }

    #[test]
    fn whole_path_family_consumption_pulls_transported_profiles_back_to_the_same_frame(
    ) -> Result<()> {
        let p = parts()?;
        let w = weights(&p)?;
        let tok = tokenizer()?;
        set(
            &w.family,
            (0..w.family.elem_count())
                .map(|i| (i % 7) as f32 * 0.25 - 0.75)
                .collect(),
        )?;
        let revision = w.rebind(&p.parent, &p.fixture.path.join("covariant"), Mode::Greedy)?;
        let mut session = revision.session(&tok, TOK.as_bytes(), scope(), limits(), 740, 16)?;
        session.turn("a b", one())?;
        session.turn("a a", one())?;
        let snapshot = session.snapshot()?;
        let canonical = session.index.canonical_profiles()?;
        for action in 0..64 {
            let mut transformed = session.index.clone();
            transformed.transport(action)?;
            assert_eq!(canonical, transformed.canonical_profiles()?);
        }
        let loaded = WholePathRevision::load(&revision.root, revision.sha256())?;
        let a = session.turn("a", one())?;
        for action in 0..64 {
            let mut framed = loaded.session(&tok, TOK.as_bytes(), scope(), limits(), 740, 16)?;
            framed.restore(&snapshot, &sha256_bytes(&snapshot))?;
            framed.transport_routes(action)?;
            let b = framed.turn("a", one())?;
            assert_eq!(outcome_digest(&a)?, outcome_digest(&b)?, "frame {action}");
            assert_eq!(
                a.generation.steps[0].copy_token_ids,
                b.generation.steps[0].copy_token_ids
            );
            assert_eq!(
                a.generation.steps[0].generate_raw_scores_q24,
                b.generation.steps[0].generate_raw_scores_q24
            );
            assert_eq!(
                a.generation.steps[0].copy_raw_scores_q24,
                b.generation.steps[0].copy_raw_scores_q24
            );
        }
        Ok(())
    }
}
