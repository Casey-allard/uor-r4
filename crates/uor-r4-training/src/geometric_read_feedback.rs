//! Offline bridge-only credit for the checked native two-read mechanism.
//! Hard forward is native; the T=1 finite-action surrogate is explicitly biased.
//! Producer, provisional occurrence selection, before-state and source keys are
//! constants. Only the existing 83-coefficient finite action rows are Vars.
use crate::{invalid, Result};
use candle_core::{Device, Tensor, Var};
use std::collections::BTreeMap;
use uor_r4_integer::{
    geometric_context_q4::ContextQ4Config,
    geometric_read_feedback::{FeedbackTrace, NativeReadFeedback},
    geometric_source_realizer::NativeArtifactBinding,
    geometric_value_q4::{canonical_basis_q25, pack_coefficients, unpack_coefficients},
    h4_tables::{H4Code, HistoricalH4Tables},
};
pub const SURROGATE:&str="native-identity-first-q4-hard-action-forward;T1-softmax-all120-exact-right-composed-native-root-adjoint;quarter-shadow-STE;frozen-producer-stage1-sourcekeys-readouts;final-token-loss-only/1";
const ROW: usize = 83;
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");

pub struct FeedbackBridgeWeights {
    parent: NativeArtifactBinding,
    context: ContextQ4Config,
    value_packed: Vec<u8>,
    bridge: Var,
}
impl FeedbackBridgeWeights {
    /// Imports compatible producer bytes and finite action coefficients from a
    /// checked parent-bound native feedback artifact; no historical value import.
    pub fn from_native(native: &NativeReadFeedback) -> Result<Self> {
        let context = native.metadata().context;
        let count = NativeReadFeedback::bridge_coefficient_count(context)?;
        let q = unpack_coefficients(count, native.bridge_packed())
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            parent: native.metadata().parent_artifact.clone(),
            context,
            value_packed: native.value_packed().to_vec(),
            bridge: Var::from_vec(
                q.into_iter().map(|q| f32::from(q) * 0.25).collect(),
                (context.heads * context.lanes_per_head, 120, ROW),
                &Device::Cpu,
            )?,
        })
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        BTreeMap::from([("bridge.coefficients".into(), self.bridge.clone())])
    }
    pub fn context_config(&self) -> ContextQ4Config {
        self.context
    }
    pub fn parent_binding(&self) -> &NativeArtifactBinding {
        &self.parent
    }
    pub fn value_packed(&self) -> &[u8] {
        &self.value_packed
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        let values = self.bridge.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite bridge shadow"));
        }
        self.bridge.set(&Tensor::from_vec(
            values.into_iter().map(|x| x.clamp(-1.75, 1.75)).collect(),
            self.bridge.shape(),
            &Device::Cpu,
        )?)?;
        Ok(())
    }
    pub fn packed_coefficients(&self) -> Result<Vec<u8>> {
        let v = self.bridge.flatten_all()?.to_vec1::<f32>()?;
        if v.iter()
            .any(|x| !x.is_finite() || !(-1.75..=1.75).contains(x))
        {
            return Err(invalid("invalid bridge shadow range"));
        }
        pack_coefficients(
            &v.into_iter()
                .map(|x| (x * 4.).round() as i8)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn compile(&self) -> Result<NativeReadFeedback> {
        Ok(NativeReadFeedback::compile(
            self.parent.clone(),
            self.context,
            &self.value_packed,
            &self.packed_coefficients()?,
        )?)
    }
    /// Refined latent tensor [global_lane,4]. Actual producer atoms and selected
    /// actions come from the native trace; no loss target is accepted here.
    pub(crate) fn refined_latent(&self, trace: &FeedbackTrace) -> Result<Tensor> {
        let c = self.context;
        let width = c.heads * c.lanes_per_head;
        if trace.before.heads != c.heads
            || trace.before.lanes_per_head != c.lanes_per_head
            || trace.before.states.len() != width
            || trace.after.states.len() != width
            || trace.actions.len() != width
            || trace.value.heads != c.heads
            || trace.value.packets.len() != c.heads * 4
        {
            return Err(invalid("bridge native trace dimensions differ"));
        }
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let basis = canonical_basis_q25();
        let q = unpack_coefficients(
            NativeReadFeedback::bridge_coefficient_count(c)?,
            &self.packed_coefficients()?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let hard = Tensor::from_vec(
            q.iter().map(|&q| f32::from(q) * 0.25).collect(),
            self.bridge.shape(),
            &Device::Cpu,
        )?;
        let coefficients = (&hard + (self.bridge.as_tensor() - self.bridge.detach())?)?;
        let mut refined = Vec::new();
        for lane in 0..width {
            let next = lane / c.lanes_per_head * c.lanes_per_head
                + (lane % c.lanes_per_head + 1) % c.lanes_per_head;
            let value_lane = lane / c.lanes_per_head * 4 + lane % c.lanes_per_head;
            let old =
                H4Code::try_from(trace.before.states[lane]).map_err(|e| invalid(e.to_string()))?;
            let neighbor =
                H4Code::try_from(trace.before.states[next]).map_err(|e| invalid(e.to_string()))?;
            let mut features = [0f32; ROW];
            features[0] = 1.;
            for axis in 0..4 {
                features[1 + axis] = basis[old.index() as usize][axis] as f32 / 33554432.;
                features[5 + axis] = basis[neighbor.index() as usize][axis] as f32 / 33554432.;
            }
            let atoms = &trace.value.packets[value_lane];
            for (atom, p) in atoms.iter().enumerate() {
                let (r, cat) = if atom == 0 { (9, 13) } else { (46, 50) };
                let category = match p.state {
                    "Absent" => 0,
                    "PresentZero" => 1,
                    "PresentNonzero" if p.radius_bin <= 30 => 2 + p.radius_bin as usize,
                    _ => return Err(invalid("bridge atom status/radius differs")),
                };
                features[cat + category] = 1.;
                if p.state == "PresentNonzero" {
                    let root = H4Code::try_from(p.root).map_err(|e| invalid(e.to_string()))?;
                    for axis in 0..4 {
                        features[r + axis] = basis[root.index() as usize][axis] as f32 / 33554432.;
                    }
                }
            }
            let surrogate = coefficients
                .narrow(0, lane, 1)?
                .reshape((120, ROW))?
                .matmul(&Tensor::from_vec(
                    features.to_vec(),
                    (ROW, 1),
                    &Device::Cpu,
                )?)?
                .reshape(120)?;
            let mut scores = Vec::new();
            let mut integer_scores = Vec::new();
            for action in 0..120 {
                let row = &q[(lane * 120 + action) * ROW..(lane * 120 + action + 1) * ROW];
                let mut score = i64::from(row[0]) << 22;
                score += factor_q24(&row[1..5], &basis[old.index() as usize]);
                score += factor_q24(&row[5..9], &basis[neighbor.index() as usize]);
                for (atom, p) in atoms.iter().enumerate() {
                    let (r, cat) = if atom == 0 { (9, 13) } else { (46, 50) };
                    let category = if p.state == "Absent" {
                        0
                    } else if p.state == "PresentZero" {
                        1
                    } else {
                        2 + p.radius_bin as usize
                    };
                    score += i64::from(row[cat + category]) << 22;
                    if p.state == "PresentNonzero" {
                        score += factor_q24(&row[r..r + 4], &basis[p.root as usize]);
                    }
                }
                integer_scores.push(score);
                scores.push((score as f64 / 16777216.) as f32);
            }
            let mut selected = 1;
            for action in (0..120).filter(|a| *a != 1) {
                if integer_scores[action] > integer_scores[selected] {
                    selected = action;
                }
            }
            let a = H4Code::try_from(trace.actions[lane]).map_err(|e| invalid(e.to_string()))?;
            let updated = geometry.compose(old, a);
            // Compare integer score selection, not rounded-F32 score ties.
            // Native trace action is validated by replay in loss_dependent;
            // exact updated code and authoritative action are checked here.
            if selected != a.index() as usize || updated.index() != trace.after.states[lane] {
                return Err(invalid("bridge native right-composed state differs"));
            }
            let logits = (&Tensor::from_vec(scores, 120, &Device::Cpu)?
                + (&surrogate - surrogate.detach())?)?;
            let expected = finite_transport_expectation(&logits, old, &geometry)?;
            let native = Tensor::from_vec(
                basis[updated.index() as usize]
                    .map(|x| x as f32 / 33554432.)
                    .to_vec(),
                4,
                &Device::Cpu,
            )?;
            refined.push((&native + (&expected - expected.detach())?)?);
        }
        Ok(Tensor::stack(&refined, 0)?)
    }
}
fn finite_transport_expectation(
    logits: &Tensor,
    old: H4Code,
    geometry: &HistoricalH4Tables,
) -> Result<Tensor> {
    let basis = canonical_basis_q25();
    let mut transported = Vec::with_capacity(480);
    for action in 0..120 {
        let action = H4Code::try_from(action as u8).map_err(|e| invalid(e.to_string()))?;
        transported.extend(
            basis[geometry.compose(old, action).index() as usize].map(|x| x as f32 / 33554432.),
        );
    }
    Ok(candle_nn::ops::softmax(logits, 0)?
        .reshape((1, 120))?
        .matmul(&Tensor::from_vec(transported, (120, 4), &Device::Cpu)?)?
        .reshape(4)?)
}
fn factor_q24(coefficients: &[i8], basis: &[i32; 4]) -> i64 {
    let sum = coefficients
        .iter()
        .zip(basis)
        .map(|(&q, &x)| i64::from(q) * i64::from(x))
        .sum::<i64>();
    let magnitude = (sum.unsigned_abs() + 4) >> 3;
    if sum < 0 {
        -(magnitude as i64)
    } else {
        magnitude as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependent_finite_action_adjoint_matches_declared_smooth_transport() -> Result<()> {
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let basis = canonical_basis_q25();
        let old = H4Code::try_from(17).map_err(|e| invalid(e.to_string()))?;
        let logits = (0..120)
            .map(|i| ((i * 37 % 31) as f32 - 15.) / 20.)
            .collect::<Vec<_>>();
        let transported = (0..120)
            .map(|i| {
                let a = H4Code::try_from(i as u8).map_err(|e| invalid(e.to_string()))?;
                Ok(basis[geometry.compose(old, a).index() as usize].map(|x| x as f64 / 33554432.))
            })
            .collect::<Result<Vec<_>>>()?;
        assert!((0..120).any(|i| {
            let a = H4Code::try_from(i as u8).ok();
            a.is_some_and(|a| geometry.compose(old, a) != geometry.compose(a, old))
        }));
        let g = [0.7f64, -0.3, 0.2, 1.1];
        let credit = transported
            .iter()
            .map(|v| v.iter().zip(g).map(|(v, g)| v * g).sum::<f64>())
            .collect::<Vec<_>>();
        let var = Var::from_vec(logits.clone(), 120, &Device::Cpu)?;
        let loss = (finite_transport_expectation(var.as_tensor(), old, &geometry)?
            * Tensor::from_vec(g.map(|x| x as f32).to_vec(), 4, &Device::Cpu)?)?
        .sum_all()?;
        let grad = loss
            .backward()?
            .get(var.as_tensor())
            .ok_or_else(|| invalid("action adjoint disconnected"))?
            .to_vec1::<f32>()?;
        let smooth = |z: &[f64]| {
            let max = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let p = z.iter().map(|z| (z - max).exp()).collect::<Vec<_>>();
            p.iter().zip(&credit).map(|(p, c)| p * c).sum::<f64>() / p.iter().sum::<f64>()
        };
        let base = logits.iter().map(|x| f64::from(*x)).collect::<Vec<_>>();
        for i in 0..120 {
            let mut left = base.clone();
            let mut right = base.clone();
            left[i] -= 1e-5;
            right[i] += 1e-5;
            let numerical = (smooth(&right) - smooth(&left)) / 2e-5;
            assert!((f64::from(grad[i]) - numerical).abs() < 2e-7, "action{i}");
        }
        Ok(())
    }
}
