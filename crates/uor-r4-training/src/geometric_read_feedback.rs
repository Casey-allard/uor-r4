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
pub const ROUTE_SURROGATE: &str = "native-Q31-factual-CE-forward;T1-all120-conditional-native-token-probability-mixture-adjoint;mean-global-lanes;quarter-shadow-STE;frozen-producer-stage1-sourcekeys-readouts;no-floor-no-zero-alternative-mask/1";
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
    pub(crate) fn policy_logits(&self, trace: &FeedbackTrace) -> Result<Vec<Tensor>> {
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
        let mut policies = Vec::new();
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
            policies.push(logits);
        }
        Ok(policies)
    }
    /// Historical state-expectation adjoint retained unchanged in its own API.
    pub(crate) fn refined_latent(&self, trace: &FeedbackTrace) -> Result<Tensor> {
        let logits = self.policy_logits(trace)?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let basis = canonical_basis_q25();
        let mut refined = Vec::with_capacity(logits.len());
        for (lane, logits) in logits.iter().enumerate() {
            let old =
                H4Code::try_from(trace.before.states[lane]).map_err(|e| invalid(e.to_string()))?;
            let updated =
                H4Code::try_from(trace.after.states[lane]).map_err(|e| invalid(e.to_string()))?;
            let expected = finite_transport_expectation(logits, old, &geometry)?;
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

/// Detached native alias probabilities are mixed before taking the logarithm.
/// Zero alternatives are legitimate; only zero/nonfinite total support fails.
pub(crate) fn native_probability_mixture(logits: &Tensor, probabilities: &[f64]) -> Result<Tensor> {
    if logits.dims() != [120]
        || probabilities.len() != 120
        || probabilities
            .iter()
            .any(|u| !u.is_finite() || !(0.0..=1.0).contains(u))
        || logits.to_vec1::<f32>()?.iter().any(|z| !z.is_finite())
    {
        return Err(invalid(
            "native route policy/probability dimensions or bounds differ",
        ));
    }
    let u = Tensor::from_vec(
        probabilities.iter().map(|u| *u as f32).collect(),
        120,
        &Device::Cpu,
    )?;
    let support = (candle_nn::ops::softmax(logits, 0)? * u)?.sum_all()?;
    let value = support.to_scalar::<f32>()?;
    if !value.is_finite() || value <= 0.0 {
        return Err(invalid(
            "native route mixture has zero or nonfinite support; no floor",
        ));
    }
    Ok(support.log()?.neg()?)
}

/// Anchor at CE, not probability: the derivative denominator remains mixture support.
pub(crate) fn anchor_native_route_loss(
    native_probability: f64,
    mixture: &Tensor,
) -> Result<Tensor> {
    if !native_probability.is_finite()
        || !(0.0..=1.0).contains(&native_probability)
        || native_probability == 0.0
    {
        return Err(invalid(
            "native route factual target has zero/nonfinite support",
        ));
    }
    let probability = native_probability as f32;
    if !probability.is_finite() || probability <= 0.0 {
        return Err(invalid(
            "native route factual probability cannot be represented",
        ));
    }
    let native_ce = Tensor::new(probability, &Device::Cpu)?.log()?.neg()?;
    Ok((&native_ce + (mixture - mixture.detach())?)?)
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
    fn native_route_mixture_adjoint_matches_probability_marginal_with_zero_alternatives(
    ) -> Result<()> {
        let z = (0..120).map(|a| (a as f32 - 60.) / 47.).collect::<Vec<_>>();
        let u = (0..120)
            .map(|a| {
                if a % 7 == 0 {
                    0.0
                } else {
                    (a % 19 + 1) as f64 / 40.
                }
            })
            .collect::<Vec<_>>();
        let var = Var::from_vec(z.clone(), 120, &Device::Cpu)?;
        let mixture = native_probability_mixture(var.as_tensor(), &u)?;
        let grad = mixture
            .backward()?
            .get(var.as_tensor())
            .ok_or_else(|| invalid("native mixture policy disconnected"))?
            .to_vec1::<f32>()?;
        let smooth = |z: &[f64]| {
            let max = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let exp = z.iter().map(|z| (z - max).exp()).collect::<Vec<_>>();
            -(exp.iter().zip(&u).map(|(p, u)| p * u).sum::<f64>() / exp.iter().sum::<f64>()).ln()
        };
        let base = z.iter().map(|z| f64::from(*z)).collect::<Vec<_>>();
        let max = base.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exp = base.iter().map(|z| (z - max).exp()).collect::<Vec<_>>();
        let denominator = exp.iter().sum::<f64>();
        let support = exp.iter().zip(&u).map(|(p, u)| p * u).sum::<f64>() / denominator;
        for a in 0..120 {
            let p = exp[a] / denominator;
            let analytic = p - p * u[a] / support;
            let mut left = base.clone();
            let mut right = base.clone();
            left[a] -= 1e-5;
            right[a] += 1e-5;
            let numerical = (smooth(&right) - smooth(&left)) / 2e-5;
            assert!((f64::from(grad[a]) - analytic).abs() < 3e-7);
            assert!((f64::from(grad[a]) - numerical).abs() < 3e-7);
            if u[a] == 0.0 {
                assert!(grad[a] > 0.0);
            }
        }
        assert!(native_probability_mixture(var.as_tensor(), &[0.0; 120]).is_err());
        Ok(())
    }
    #[test]
    fn native_route_loss_anchor_preserves_mixture_gradient_independent_of_factual_probability(
    ) -> Result<()> {
        let var = Var::from_vec(vec![0f32; 120], 120, &Device::Cpu)?;
        let mut u = vec![0.0; 120];
        u[1] = 0.4;
        u[7] = 0.8;
        let mixture = native_probability_mixture(var.as_tensor(), &u)?;
        let raw = mixture
            .backward()?
            .get(var.as_tensor())
            .ok_or_else(|| invalid("mixture gradient absent"))?
            .to_vec1::<f32>()?;
        for factual in [0.01, 0.4, 0.9] {
            let anchored = anchor_native_route_loss(factual, &mixture)?;
            let expected = Tensor::new(factual as f32, &Device::Cpu)?.log()?.neg()?;
            assert_eq!(
                anchored.to_scalar::<f32>()?.to_bits(),
                expected.to_scalar::<f32>()?.to_bits()
            );
            let grad = anchored
                .backward()?
                .get(var.as_tensor())
                .ok_or_else(|| invalid("anchored gradient absent"))?
                .to_vec1::<f32>()?;
            assert_eq!(grad, raw);
        }
        assert!(anchor_native_route_loss(0.0, &mixture).is_err());
        Ok(())
    }
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
