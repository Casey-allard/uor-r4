//! Offline local finite-choice credit for native selected-source state transport.
//! Actual occurrence selection is stopped/caller-owned. Incoming state_choices
//! are hard full120 carriers, NOT probabilities to normalize again. Factual
//! action carry preserves query temporal credit at zero/identity initialization.
//! Refresh prepare_native after every update. CUDA graph operations stay device
//! resident; native hard reference/export and geometric index staging are host.
use crate::{
    geometric_context::{q4_project_tensor, q4_shadow_ste},
    invalid, sha256_bytes, Result,
};
use candle_core::{DType, Device, Tensor, Var};
use std::collections::BTreeMap;
use uor_r4_core::native_geometric::learner::geometric_read_state_bridge::{
    BridgeReadCounts, NativeGeometricReadStateBridge, BIAS_BYTES_PER_LANE, RELATIVE_BYTES_PER_LANE,
    ROOTS,
};
use uor_r4_integer::{geometric_source_actions::SourceActionBinding, h4_tables::H4Code};
pub const CREDIT_SCOPE:&str="factual-coefficient-quarter-STE-once;full120-action-choice-pullback;factual-query-times-action-permutation;hard-factual-action-query-carry;detached-table-local120-query/key-score-extensions;no-renormalized-input-carrier;no-hard-occurrence-selection-credit;not-global-posterior/1";
pub struct BridgeLearningWeights {
    binding: SourceActionBinding,
    pub bias: Var,
    pub relative: Var,
    lanes: usize,
}
pub struct PreparedBridgeLearning {
    pub native: NativeGeometricReadStateBridge,
    pub downloaded_master_bytes: usize,
}
pub struct BridgeLearningOutput {
    pub post_state_codes: Vec<H4Code>,
    pub action_codes: Vec<H4Code>,
    pub state_choices: Tensor,
    pub action_scores_q24: Vec<i64>,
    pub counts: BridgeReadCounts,
    pub validation_scalar_reads: usize,
    pub credit_scope: &'static str,
}
fn native_error(e: impl std::fmt::Display) -> crate::TrainingError {
    invalid(e.to_string())
}
fn device_admit(d: &Device) -> Result<()> {
    if !d.is_cpu() && !d.is_cuda() {
        return Err(invalid("bridge learning supports CPU/CUDA only"));
    }
    Ok(())
}
fn pack(values: &[f32], padded: bool) -> Result<Vec<u8>> {
    if values.iter().any(|x| !x.is_finite() || x.abs() > 1.75) {
        return Err(invalid("bridge master outside strict finite quarter range"));
    }
    let mut out = vec![
        0;
        if padded {
            values.len() / 120 * 64
        } else {
            values.len().div_ceil(2)
        }
    ];
    for (i, &v) in values.iter().enumerate() {
        let at = if padded { (i / 120) * 128 + i % 120 } else { i };
        let n = ((v * 4.).round() as i8 as u8) & 15;
        out[at >> 1] |= n << if at & 1 == 0 { 0 } else { 4 };
    }
    Ok(out)
}
fn hard_choices(codes: &[H4Code], device: &Device) -> Result<Tensor> {
    let mut hard = vec![0f32; codes.len() * 120];
    for (lane, code) in codes.iter().enumerate() {
        hard[lane * 120 + usize::from(code.index())] = 1.;
    }
    Ok(Tensor::from_vec(hard, (codes.len(), 120), device)?)
}
fn carrier_admit(t: &Tensor, codes: &[H4Code], device: &Device) -> Result<()> {
    if t.dims() != [codes.len(), ROOTS]
        || t.dtype() != DType::F32
        || !t.device().same_device(device)
    {
        return Err(invalid("bridge carrier shape/dtype/device mismatch"));
    }
    let error = (t - hard_choices(codes, device)?)?
        .flatten_all()?
        .abs()?
        .max(0)?
        .to_scalar::<f32>()?;
    if error != 0. || !error.is_finite() {
        return Err(invalid(
            "bridge carrier differs from actual native hard state",
        ));
    }
    Ok(())
}
impl BridgeLearningWeights {
    pub fn zeroed(binding: &SourceActionBinding, lanes: usize, device: &Device) -> Result<Self> {
        let native =
            NativeGeometricReadStateBridge::zeroed(binding, lanes).map_err(native_error)?;
        Self::from_native(&native, binding, device)
    }
    pub fn from_native(
        native: &NativeGeometricReadStateBridge,
        binding: &SourceActionBinding,
        device: &Device,
    ) -> Result<Self> {
        device_admit(device)?;
        let admitted = NativeGeometricReadStateBridge::from_bytes(
            &native.to_bytes().map_err(native_error)?,
            binding,
        )
        .map_err(native_error)?;
        let lanes = admitted.lanes();
        let mut b = Vec::new();
        let mut t = Vec::new();
        for lane in 0..lanes {
            for a in 0..120 {
                b.push(f32::from(admitted.coefficient_bias(lane, a).map_err(native_error)?) * 0.25);
                for d in 0..120 {
                    t.push(
                        f32::from(
                            admitted
                                .coefficient_relative(lane, a, d)
                                .map_err(native_error)?,
                        ) * 0.25,
                    );
                }
            }
        }
        Ok(Self {
            binding: binding.clone(),
            bias: Var::from_vec(b, (lanes, 120), device)?,
            relative: Var::from_vec(t, (lanes, 120, 120), device)?,
            lanes,
        })
    }
    pub fn device(&self) -> &Device {
        self.bias.device()
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        BTreeMap::from([
            ("read_state_bridge.bias".into(), self.bias.clone()),
            ("read_state_bridge.relative".into(), self.relative.clone()),
        ])
    }
    pub fn project_coefficients(&self) -> Result<()> {
        self.bias.set(&q4_project_tensor(self.bias.as_tensor())?)?;
        self.relative
            .set(&q4_project_tensor(self.relative.as_tensor())?)?;
        Ok(())
    }
    pub fn prepare_native(&self) -> Result<PreparedBridgeLearning> {
        let b = self.bias.flatten_all()?.to_vec1::<f32>()?;
        let t = self.relative.flatten_all()?.to_vec1::<f32>()?;
        let native = NativeGeometricReadStateBridge::compile(
            &self.binding,
            self.lanes,
            &pack(&b, false)?,
            &pack(&t, true)?,
        )
        .map_err(native_error)?;
        Ok(PreparedBridgeLearning {
            native,
            downloaded_master_bytes: 4 * (b.len() + t.len()),
        })
    }
    pub fn export_native(&self) -> Result<NativeGeometricReadStateBridge> {
        Ok(self.prepare_native()?.native)
    }
    pub fn forward_prepared(
        &self,
        prepared: &PreparedBridgeLearning,
        query: &[H4Code],
        selected_source: &[H4Code],
        query_choices: &Tensor,
        source_choices: &Tensor,
    ) -> Result<BridgeLearningOutput> {
        device_admit(self.device())?;
        if prepared.native.lanes() != self.lanes
            || query.len() != self.lanes
            || selected_source.len() != self.lanes
            || prepared.native.metadata().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || !self.relative.device().same_device(self.device())
        {
            return Err(invalid("bridge snapshot/query/source binding differs"));
        }
        carrier_admit(query_choices, query, self.device())?;
        carrier_admit(source_choices, selected_source, self.device())?;
        let mut post = vec![H4Code::IDENTITY; self.lanes];
        let mut actions = post.clone();
        let mut scores = vec![0i64; self.lanes * 120];
        let mut counts = BridgeReadCounts::default();
        prepared
            .native
            .apply_into(
                query,
                selected_source,
                &mut post,
                &mut actions,
                &mut scores,
                &mut counts,
            )
            .map_err(native_error)?;
        let b = q4_shadow_ste(self.bias.as_tensor())?;
        let table = q4_shadow_ste(self.relative.as_tensor())?;
        let detached = table.detach();
        let algebra = prepared.native.algebra().map_err(native_error)?;
        let mut outputs = Vec::new();
        for lane in 0..self.lanes {
            let q = query[lane].index();
            let k = selected_source[lane].index();
            let d = algebra
                .compose(algebra.inverse(q).map_err(native_error)?, k)
                .map_err(native_error)?;
            let index = (0..120)
                .map(|a| (a * 120 + usize::from(d)) as u32)
                .collect::<Vec<_>>();
            let index = Tensor::from_vec(index, 120, self.device())?;
            let factual = table
                .narrow(0, lane, 1)?
                .flatten_all()?
                .index_select(&index, 0)?;
            let coefficient = (&b.narrow(0, lane, 1)?.reshape(120)? + factual)?.affine(0.25, 0.)?;
            let hard_scores = Tensor::from_vec(
                scores[lane * 120..(lane + 1) * 120]
                    .iter()
                    .map(|&s| (s as f64 / 16_777_216.) as f32)
                    .collect::<Vec<_>>(),
                120,
                self.device(),
            )?;
            if (&coefficient - &hard_scores)?
                .abs()?
                .max(0)?
                .to_scalar::<f32>()?
                != 0.
            {
                return Err(invalid("bridge factual coefficient graph differs from prepared native scores; refresh snapshot"));
            }
            let mut qi = Vec::with_capacity(14400);
            let mut ki = Vec::with_capacity(14400);
            for c in 0..120u8 {
                let qd = algebra
                    .compose(algebra.inverse(c).map_err(native_error)?, k)
                    .map_err(native_error)?;
                let kd = algebra
                    .compose(algebra.inverse(q).map_err(native_error)?, c)
                    .map_err(native_error)?;
                for a in 0..120 {
                    qi.push((a * 120 + usize::from(qd)) as u32);
                    ki.push((a * 120 + usize::from(kd)) as u32);
                }
            }
            let constants = detached.narrow(0, lane, 1)?.flatten_all()?;
            let qtable = constants
                .index_select(&Tensor::from_vec(qi, 14400, self.device())?, 0)?
                .reshape((120, 120))?;
            let ktable = constants
                .index_select(&Tensor::from_vec(ki, 14400, self.device())?, 0)?
                .reshape((120, 120))?;
            let qdependency = qtable
                .broadcast_mul(&query_choices.narrow(0, lane, 1)?.reshape((120, 1))?)?
                .sum(0)?
                .affine(0.25, 0.)?;
            let kdependency = ktable
                .broadcast_mul(&source_choices.narrow(0, lane, 1)?.reshape((120, 1))?)?
                .sum(0)?
                .affine(0.25, 0.)?;
            let logits = ((&hard_scores + (&coefficient - coefficient.detach())?)?
                + (&qdependency - qdependency.detach())?)?;
            let logits = (&logits + (&kdependency - kdependency.detach())?)?;
            let probability = candle_nn::ops::softmax(&logits, 0)?;
            // Poststate permutation is exact bijection. Factual-action query carry is a
            // separate first-order path; do not softmax query/source carriers here.
            let invq = algebra.inverse(q).map_err(native_error)?;
            let inva = algebra
                .inverse(actions[lane].index())
                .map_err(native_error)?;
            let mut pi = Vec::new();
            let mut carry = Vec::new();
            for r in 0..120u8 {
                pi.push(u32::from(algebra.compose(invq, r).map_err(native_error)?));
                carry.push(u32::from(algebra.compose(r, inva).map_err(native_error)?));
            }
            let expected =
                probability.index_select(&Tensor::from_vec(pi, 120, self.device())?, 0)?;
            let carried = query_choices
                .narrow(0, lane, 1)?
                .reshape(120)?
                .index_select(&Tensor::from_vec(carry, 120, self.device())?, 0)?;
            let hard = hard_choices(&post[lane..lane + 1], self.device())?.reshape(120)?;
            outputs.push(
                ((&hard + (&expected - expected.detach())?)? + (&carried - carried.detach())?)?,
            );
        }
        Ok(BridgeLearningOutput {
            post_state_codes: post,
            action_codes: actions,
            state_choices: Tensor::stack(&outputs, 0)?,
            action_scores_q24: scores,
            counts,
            validation_scalar_reads: 2 + self.lanes,
            credit_scope: CREDIT_SCOPE,
        })
    }
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn lanes(&self) -> usize {
        self.lanes
    }
    pub fn payload_sha256(&self) -> Result<String> {
        Ok(sha256_bytes(
            &self.export_native()?.to_bytes().map_err(native_error)?,
        ))
    }
    pub fn padded_native_coefficient_bytes(&self) -> usize {
        self.lanes * (BIAS_BYTES_PER_LANE + RELATIVE_BYTES_PER_LANE)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn fixture(device: &Device) -> Result<(SourceActionBinding, BridgeLearningWeights)> {
        let b = SourceActionBinding::new(TOK.as_bytes()).map_err(native_error)?;
        let w = BridgeLearningWeights::zeroed(&b, 1, device)?;
        Ok((b, w))
    }
    #[test]
    fn zero_bridge_preserves_native_output_and_query_credit() -> Result<()> {
        let (b, w) = fixture(&Device::Cpu)?;
        let q = [H4Code::try_from(119).map_err(native_error)?];
        let k = [H4Code::IDENTITY];
        let qc = Var::from_tensor(&hard_choices(&q, &Device::Cpu)?)?;
        let kc = Var::from_tensor(&hard_choices(&k, &Device::Cpu)?)?;
        let snapshot = w.prepare_native()?;
        let restored = NativeGeometricReadStateBridge::from_bytes(
            &snapshot.native.to_bytes().map_err(native_error)?,
            &b,
        )
        .map_err(native_error)?;
        assert_eq!(
            restored.to_bytes().map_err(native_error)?,
            snapshot.native.to_bytes().map_err(native_error)?
        );
        let out = w.forward_prepared(&snapshot, &q, &k, qc.as_tensor(), kc.as_tensor())?;
        assert_eq!(out.post_state_codes, q);
        assert_eq!(out.state_choices.to_vec2::<f32>()?, qc.to_vec2::<f32>()?);
        let utility = Tensor::from_vec(
            (0..120)
                .map(|i| if i == 3 { 1f32 } else { 0. })
                .collect::<Vec<_>>(),
            (1, 120),
            &Device::Cpu,
        )?;
        let loss = (&out.state_choices * &utility)?.sum_all()?;
        let g = loss.backward()?;
        for parameter in [&w.bias, &w.relative] {
            let gradient = g
                .get(parameter.as_tensor())
                .ok_or_else(|| invalid("zero-init bridge parameter credit absent"))?;
            let values = gradient.flatten_all()?.to_vec1::<f32>()?;
            assert!(values.iter().all(|x| x.is_finite()));
            assert!(values.iter().any(|x| *x != 0.));
        }
        assert_eq!(
            g.get(qc.as_tensor())
                .ok_or_else(|| invalid("query credit absent"))?
                .to_vec2::<f32>()?,
            utility.to_vec2::<f32>()?
        );
        assert!(g
            .get(kc.as_tensor())
            .ok_or_else(|| invalid("key credit absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|&x| x == 0.));
        assert_eq!(w.padded_native_coefficient_bytes(), 7740);
        Ok(())
    }
    #[test]
    fn nonconstant_relation_credits_source_and_native_scores() -> Result<()> {
        let (_, w) = fixture(&Device::Cpu)?;
        let mut t = vec![0f32; 14400];
        t[2 * 120 + 2] = 1.75;
        w.relative
            .set(&Tensor::from_vec(t, (1, 120, 120), &Device::Cpu)?)?;
        let q = [H4Code::IDENTITY];
        let k = [H4Code::try_from(2).map_err(native_error)?];
        let qc = Var::from_tensor(&hard_choices(&q, &Device::Cpu)?)?;
        let kc = Var::from_tensor(&hard_choices(&k, &Device::Cpu)?)?;
        let s = w.prepare_native()?;
        let o = w.forward_prepared(&s, &q, &k, qc.as_tensor(), kc.as_tensor())?;
        assert_eq!(o.action_codes[0].index(), 2);
        let loss = o.state_choices.narrow(1, 2, 1)?.sum_all()?;
        let g = loss.backward()?;
        assert!(
            g.get(kc.as_tensor())
                .ok_or_else(|| invalid("source gradient absent"))?
                .abs()?
                .sum_all()?
                .to_scalar::<f32>()?
                > 0.
        );
        assert!(
            g.get(w.relative.as_tensor())
                .ok_or_else(|| invalid("coefficient gradient absent"))?
                .abs()?
                .sum_all()?
                .to_scalar::<f32>()?
                > 0.
        );
        assert_eq!(o.action_scores_q24[2], 7 << 20);
        assert!(w
            .forward_prepared(
                &s,
                &q,
                &k,
                &Tensor::zeros((1, 120), DType::F32, &Device::Cpu)?,
                kc.as_tensor()
            )
            .is_err());
        Ok(())
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn cuda_bridge_native_carrier_and_all_adjoints_match_cpu() -> Result<()> {
        // Requested CUDA coverage is mandatory: unavailable device is an error,
        // never a skipped test counted as parity PASS.
        let device = Device::new_cuda(0)?;
        let (binding, cpu) = fixture(&Device::Cpu)?;
        let values = (0..14400)
            .map(|i| ((i % 15) as f32 - 7.) * 0.25)
            .collect::<Vec<_>>();
        cpu.relative
            .set(&Tensor::from_vec(values, (1, 120, 120), &Device::Cpu)?)?;
        let native = cpu.export_native()?;
        let gpu = BridgeLearningWeights::from_native(&native, &binding, &device)?;
        let q = [H4Code::try_from(119).map_err(native_error)?];
        let k = [H4Code::try_from(0).map_err(native_error)?];
        let qc = Var::from_tensor(&hard_choices(&q, &Device::Cpu)?)?;
        let kc = Var::from_tensor(&hard_choices(&k, &Device::Cpu)?)?;
        let qg = Var::from_tensor(&hard_choices(&q, &device)?)?;
        let kg = Var::from_tensor(&hard_choices(&k, &device)?)?;
        let co = cpu.forward_prepared(
            &cpu.prepare_native()?,
            &q,
            &k,
            qc.as_tensor(),
            kc.as_tensor(),
        )?;
        let go = gpu.forward_prepared(
            &gpu.prepare_native()?,
            &q,
            &k,
            qg.as_tensor(),
            kg.as_tensor(),
        )?;
        assert!(go.state_choices.device().is_cuda());
        assert_eq!(co.post_state_codes, go.post_state_codes);
        assert_eq!(co.action_codes, go.action_codes);
        assert_eq!(co.action_scores_q24, go.action_scores_q24);
        assert_eq!(
            co.state_choices.to_vec2::<f32>()?,
            go.state_choices.to_vec2::<f32>()?
        );
        let utility = Tensor::from_vec(
            (0..120).map(|i| (i % 7) as f32 - 3.).collect::<Vec<_>>(),
            (1, 120),
            &Device::Cpu,
        )?;
        let cg = (&co.state_choices * &utility)?.sum_all()?.backward()?;
        let gg = (&go.state_choices * utility.to_device(&device)?)?
            .sum_all()?
            .backward()?;
        for (c, g) in [
            (qc.as_tensor(), qg.as_tensor()),
            (kc.as_tensor(), kg.as_tensor()),
            (cpu.bias.as_tensor(), gpu.bias.as_tensor()),
            (cpu.relative.as_tensor(), gpu.relative.as_tensor()),
        ] {
            let cc = cg
                .get(c)
                .ok_or_else(|| invalid("CPU bridge gradient missing"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            let gv = gg
                .get(g)
                .ok_or_else(|| invalid("CUDA bridge gradient missing"))?;
            assert!(gv.device().is_cuda());
            let gc = gv.flatten_all()?.to_vec1::<f32>()?;
            for (a, b) in cc.iter().zip(gc) {
                assert!((*a - b).abs() < 2e-5);
            }
        }
        Ok(())
    }
    #[test]
    fn categorical_master_winners_all_signed_frames_and_original_export_unchanged() -> Result<()> {
        use uor_r4_integer::h4_tables::HistoricalH4Tables;
        let (binding, _) = fixture(&Device::Cpu)?;
        let w = BridgeLearningWeights::zeroed(&binding, 8, &Device::Cpu)?;
        let mut relative = vec![0f32; 8 * ROOTS * ROOTS];
        for l in 0..8 {
            for d in 0..ROOTS {
                if d % 7 != 0 {
                    relative[(l * ROOTS + (d + l + 2) % ROOTS) * ROOTS + d] = 0.01;
                }
            }
        }
        w.relative
            .set(Var::from_vec(relative.clone(), (8, ROOTS, ROOTS), &Device::Cpu)?.as_tensor())?;
        let original = w.export_native()?.to_bytes().map_err(native_error)?;
        let artifact = w.export_categorical_actions()?;
        assert_eq!(artifact.receipt.categorical_keys, 960);
        assert_eq!(w.relative.flatten_all()?.to_vec1::<f32>()?, relative);
        assert_eq!(
            w.export_native()?.to_bytes().map_err(native_error)?,
            original
        );
        let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
            "../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
        ))
        .map_err(native_error)?;
        let mut checked = 0;
        for q in 0..ROOTS {
            let query = H4Code::try_from(q as u8).map_err(native_error)?;
            for d in 0..ROOTS {
                let key = geometry.compose(query, H4Code::try_from(d as u8).map_err(native_error)?);
                let mut post = [query; 8];
                let mut actions = [query; 8];
                let mut scores = [0i64; 8 * ROOTS];
                artifact
                    .native
                    .apply_into(
                        &[query; 8],
                        &[key; 8],
                        &mut post,
                        &mut actions,
                        &mut scores,
                        &mut BridgeReadCounts::default(),
                    )
                    .map_err(native_error)?;
                for l in 0..8 {
                    let a = if d % 7 == 0 { 1 } else { (d + l + 2) % ROOTS };
                    assert_eq!(
                        artifact
                            .native
                            .coefficient_relative(l, a, d)
                            .map_err(native_error)?,
                        1
                    );
                    let expected = H4Code::try_from(a as u8).map_err(native_error)?;
                    assert_eq!(actions[l], expected);
                    assert_eq!(post[l], geometry.compose(query, expected));
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 115200);
        Ok(())
    }
    #[test]
    fn categorical_nonzero_bias_dominance_and_identity_tie() -> Result<()> {
        let (binding, _) = fixture(&Device::Cpu)?;
        let w = BridgeLearningWeights::zeroed(&binding, 1, &Device::Cpu)?;
        let mut bias = vec![0f32; ROOTS];
        bias[1] = 0.25;
        bias[2] = 0.5;
        bias[3] = 0.25;
        let mut relative = vec![0f32; ROOTS * ROOTS];
        relative[2 * ROOTS] = -0.5;
        relative[3 * ROOTS + 1] = 0.5;
        relative[2 * ROOTS + 2] = 0.5;
        w.bias
            .set(Var::from_vec(bias.clone(), (1, ROOTS), &Device::Cpu)?.as_tensor())?;
        w.relative
            .set(Var::from_vec(relative.clone(), (1, ROOTS, ROOTS), &Device::Cpu)?.as_tensor())?;
        let artifact = w.export_categorical_actions()?;
        // d=0: identity ties action 3 at .25; strict tie must retain identity.
        // d=1: relative action 3 beats the otherwise bias-dominant action 2.
        // d>=2: bias-dominant action 2, including a reinforcing d=2 term.
        for d in 0..ROOTS {
            let winner = if d == 0 {
                1
            } else if d == 1 {
                3
            } else {
                2
            };
            for a in 0..ROOTS {
                assert_eq!(
                    artifact
                        .native
                        .coefficient_bias(0, a)
                        .map_err(native_error)?,
                    0
                );
                assert_eq!(
                    artifact
                        .native
                        .coefficient_relative(0, a, d)
                        .map_err(native_error)?,
                    if a == winner { 1 } else { 0 }
                );
            }
        }
        assert_eq!(artifact.receipt.tied_keys, 1);
        assert_eq!(w.bias.flatten_all()?.to_vec1::<f32>()?, bias);
        assert_eq!(w.relative.flatten_all()?.to_vec1::<f32>()?, relative);
        Ok(())
    }
}

/// Receipt-bound categorical action export. Scores are markers, not trained energies.
/// The serving format is unchanged; retain this receipt beside the artifact.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CategoricalBridgeReceipt {
    pub policy: &'static str,
    pub original_bias_f32_sha256: String,
    pub original_relative_f32_sha256: String,
    pub native_sha256: String,
    pub tied_keys: usize,
    pub categorical_keys: usize,
    pub downloaded_master_bytes: usize,
}
pub struct CategoricalBridgeExport {
    pub native: NativeGeometricReadStateBridge,
    pub receipt: CategoricalBridgeReceipt,
}
impl BridgeLearningWeights {
    /// Preserve the original master-space hard action map, discarding energies.
    /// Explicit export only: never use this as an unchanged-energy quarter-STE
    /// training snapshot. No labels, selection oracle or scale search.
    pub fn export_categorical_actions(&self) -> Result<CategoricalBridgeExport> {
        let b = self.bias.flatten_all()?.to_vec1::<f32>()?;
        let t = self.relative.flatten_all()?.to_vec1::<f32>()?;
        if self.bias.dims() != [self.lanes, ROOTS]
            || self.relative.dims() != [self.lanes, ROOTS, ROOTS]
            || !self.relative.device().same_device(self.device())
        {
            return Err(invalid("categorical bridge master shapes/devices differ"));
        }
        // Validate the retained masters with the original exporter. No clipping.
        pack(&b, false)?;
        pack(&t, true)?;
        let markers_bias = vec![0f32; b.len()];
        let mut markers = vec![0f32; t.len()];
        let mut tied_keys = 0usize;
        for l in 0..self.lanes {
            for d in 0..ROOTS {
                let score = |a: usize| {
                    f64::from(b[l * ROOTS + a]) + f64::from(t[(l * ROOTS + a) * ROOTS + d])
                };
                let mut best = 1;
                for a in 0..ROOTS {
                    if score(a) > score(best) {
                        best = a;
                    }
                }
                tied_keys +=
                    usize::from((0..ROOTS).filter(|&a| score(a) == score(best)).count() > 1);
                // Existing quarter pack emits coefficient1, all other scores0.
                markers[(l * ROOTS + best) * ROOTS + d] = 0.25;
            }
        }
        let native = NativeGeometricReadStateBridge::compile(
            &self.binding,
            self.lanes,
            &pack(&markers_bias, false)?,
            &pack(&markers, true)?,
        )
        .map_err(native_error)?;
        let bytes = native.to_bytes().map_err(native_error)?;
        let native = NativeGeometricReadStateBridge::from_bytes(&bytes, &self.binding)
            .map_err(native_error)?;
        let receipt = CategoricalBridgeReceipt {
            policy: "retained-F32-individually-F64-B+T-argmax;identity-first-strict-ties;zero-bias-single-.25-marker-per-key;0/1-nibbles;original-energies-discarded;no-labels-or-fit/1",
            original_bias_f32_sha256: sha256_bytes(
                &b.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>(),
            ),
            original_relative_f32_sha256: sha256_bytes(
                &t.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>(),
            ),
            native_sha256: sha256_bytes(&bytes),
            tied_keys,
            categorical_keys: self.lanes * ROOTS,
            downloaded_master_bytes: 4 * (b.len() + t.len()),
        };
        Ok(CategoricalBridgeExport { native, receipt })
    }
}
