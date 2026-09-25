//! Learned neighboring integer-code choice on frozen dyadic parameter grids.
//!
//! This module is one self-contained offline learning mechanism. It keeps each
//! parameter's existing frozen representation exactly (bit width, per-output-row
//! dyadic scale, clamp interval) and adds one trainable per-coordinate decision
//! `alpha` that chooses between the two allowed *neighboring integer codes* of
//! the original parent value. It is an F32 offline learner/emulator stage: it
//! implements no optimizer, no CLI, no serving path and no integer execution
//! kernel, and it makes no language-quality claim.
//!
//! # Frozen data
//!
//! Construction reads the parent variables once through `as_detached_tensor`
//! and never mutates them or their `Var` identities. For every coordinate of
//! every parameter, with the parent shadow `w`, its frozen row scale
//! `s = 2^exponent` and the bit-width code interval `[q_min, q_max]`:
//!
//! - `ratio = w / s` is required finite and inside `[q_min, q_max]`; anything
//!   else fails closed with a request to project the parent first, because the
//!   authorized parents are already projected and a silent clamp here would
//!   hide an unreported representation change.
//! - `lower = floor(ratio)` and `upper = ceil(ratio)` are the two allowed
//!   integer-code neighbors, each clipped into `[q_min, q_max]` by the range
//!   check above (so the clip is inert for an in-range parent).
//! - A coordinate whose original value lies exactly on the dyadic grid has
//!   `upper == lower`: it has one legal neighbor only, stays fixed at that code
//!   for every `alpha`, and is removed from the penalty. No second neighbor is
//!   invented for it.
//! - `fraction = ratio - lower` and `alpha` is initialized to the exact inverse
//!   of the relaxation, so the soft value reconstructs the original parent
//!   value (up to F32 rounding) before the first update.
//!
//! All frozen data (lower codes, the 0/1 choice mask, the broadcast scales, the
//! per-parameter counts and the penalty denominator) is computed once, on the
//! host, and stored detached. Gradients never pass through a floor, ceil or
//! round; those are applied only on the host during initialization.
//!
//! # Relaxation, hard decision and penalty
//!
//! - Soft: `d = clamp(1.2 * sigmoid(alpha) - 0.1, 0, 1)` and
//!   `w_soft = s * (lower + choice * d)`, differentiable in `alpha`.
//! - Hard: detached, and always exactly one of the two legal grid points.
//!   `sigmoid(alpha) > 0.5` selects `upper`, `< 0.5` selects `lower`, and
//!   exactly `0.5` preserves the original nearest round-away-zero code
//!   (including negative half-steps). The threshold is applied to the sigmoid
//!   rather than to `d` because `1.2 * 0.5 - 0.1 == 0.5` exactly in real
//!   arithmetic while its F32 evaluation is knife-edge; `sigmoid(0) == 0.5`
//!   exactly in F32, so the initialization tie is deterministic and identical
//!   across backends. The selected code is multiplied by the frozen scale, so
//!   every hard value is an exact dyadic grid point with an integer code inside
//!   the parameter's own interval; signed zero is canonicalized to positive
//!   zero, matching the existing hard codec.
//! - Penalty: `coefficient(u) * sum((1 - |2d-1|^beta) * choice) / N` where `N`
//!   is the number of non-degenerate choices over the *whole* inventory, so it
//!   is a coordinate-weighted global mean and not a mean of per-tensor means.
//!   `coefficient(u)` is zero during warm-up and `regularization` afterwards;
//!   `beta(u)` follows a cosine from `beta_start` to `beta_end` across
//!   `[warmup_steps, steps]`. The returned tensor is the already-weighted term:
//!   add it to the language loss once. `beta >= 1` is required so the
//!   `|2d-1|^beta` gradient is finite at the centre and at both endpoints.
//! - Every `alpha` variable stays in the differentiable graph in every phase,
//!   including a parameter whose coordinates are all exactly on the grid and
//!   including the zero-coefficient warm-up. `NamedAdamW` rejects a variable
//!   without a gradient, and `read.no_read.bias` is fully degenerate in the
//!   authorized quaternion parent, so this is a correctness requirement rather
//!   than a convenience. The coefficient and the choice mask are applied as
//!   tensor multiplies, never as `affine(0.0, ..)`, because Candle prunes the
//!   argument of a zero-multiplier affine from the backward walk.
//!
//! Recipe constants are caller configuration (`RoundingConfig`); this module
//! selects no experiment constants and reads no data.

use std::collections::BTreeMap;

use candle_core::{DType, Device, Tensor, Var};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::joint_quantization::{
    ParameterQuantization, QuantizationSpec, MAX_EXPONENT, MIN_EXPONENT,
};
use crate::{invalid, Result};

/// Frozen mechanism identity, bound into `statistics()`.
const ROUNDING_SCHEMA: &str = "uor-r4.learned-rounding/1";
const STATISTICS_SCHEMA: &str = "uor-r4.learned-rounding-statistics/1";

/// `d = clamp(RECTIFIER_SCALE * sigmoid(alpha) + RECTIFIER_OFFSET, 0, 1)`.
const RECTIFIER_SCALE: f64 = 1.2;
const RECTIFIER_OFFSET: f64 = -0.1;
/// Bound on the sigmoid argument. Every exponential and reciprocal taken in
/// `sigmoid_tensor` is then finite for any finite input, so a saturated or
/// extreme `alpha` cannot produce a non-finite value or gradient. Saturating at
/// this bound is numerically inert: `d` is already clamped to `0`/`1` well
/// before it.
const ALPHA_LIMIT: f64 = 30.0;
/// Smallest exponent with a defined `|2d-1|^beta` gradient at the centre.
const MINIMUM_BETA: f64 = 1.0;

/// Mirrors the private `joint_quantization::limits`.
fn code_limits(bits: u8) -> Result<(i32, i32)> {
    match bits {
        4 => Ok((-7, 7)),
        16 => Ok((-32767, 32767)),
        _ => Err(invalid("learned rounding codes must have 4 or 16 bits")),
    }
}

/// Mirrors the private `joint_quantization::step`; both are exercised against
/// the public `QuantizationSpec::parameter` grid in the tests below.
fn dyadic_step(exponent: i16) -> Result<f32> {
    if !(MIN_EXPONENT..=MAX_EXPONENT).contains(&exponent) {
        return Err(invalid("learned rounding exponent outside [-24,16]"));
    }
    Ok(2f32.powi(i32::from(exponent)))
}

/// Caller-owned recipe. No experiment constant is chosen in this module.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundingConfig {
    /// Fixed total number of updates the schedule spans.
    pub steps: usize,
    /// Updates without regularization; the cosine runs after them.
    pub warmup_steps: usize,
    /// Commitment exponent at the first update after warm-up.
    pub beta_start: f64,
    /// Commitment exponent at `steps`.
    pub beta_end: f64,
    /// Penalty coefficient after warm-up; zero during warm-up.
    pub regularization: f64,
}

impl RoundingConfig {
    pub fn validate(&self) -> Result<()> {
        if self.steps == 0 {
            return Err(invalid("learned rounding requires at least one step"));
        }
        if self.warmup_steps >= self.steps {
            return Err(invalid(
                "learned rounding warm-up must precede the annealing window",
            ));
        }
        if !self.beta_start.is_finite() || !self.beta_end.is_finite() {
            return Err(invalid("learned rounding beta must be finite"));
        }
        if !(self.beta_start as f32).is_finite() || !(self.beta_end as f32).is_finite() {
            return Err(invalid("learned rounding beta must be finite in F32"));
        }
        if self.beta_start < MINIMUM_BETA || self.beta_end < MINIMUM_BETA {
            return Err(invalid(
                "learned rounding beta must be at least one for a defined power gradient",
            ));
        }
        if !self.regularization.is_finite() || self.regularization < 0.0 {
            return Err(invalid(
                "learned rounding regularization must be finite and nonnegative",
            ));
        }
        let coefficient = self.regularization as f32;
        if !coefficient.is_finite() || coefficient < 0.0 {
            return Err(invalid(
                "learned rounding regularization must be finite and nonnegative in F32",
            ));
        }
        Ok(())
    }

    /// Cosine schedule from `beta_start` at the first post-warm-up update to
    /// `beta_end` at `steps`, held at `beta_start` during warm-up. Safe for any
    /// `completed_updates`, including an unvalidated configuration.
    pub fn beta(&self, completed_updates: usize) -> f64 {
        if completed_updates < self.warmup_steps {
            return self.beta_start;
        }
        let Some(window) = self.steps.checked_sub(self.warmup_steps) else {
            return self.beta_end;
        };
        if window == 0 {
            return self.beta_end;
        }
        let elapsed = completed_updates - self.warmup_steps;
        let progress = if elapsed >= window {
            1.0
        } else {
            elapsed as f64 / window as f64
        };
        let shaped = 0.5 * (1.0 + (std::f64::consts::PI * progress).cos());
        self.beta_end + (self.beta_start - self.beta_end) * shaped
    }
}

/// Immutable per-parameter mechanism state.
#[derive(Debug)]
struct FrozenParameter {
    name: String,
    shape: Vec<usize>,
    /// Distinct frozen exponents, for reporting only.
    exponents: Vec<i16>,
    bits: u8,
    rows: usize,
    width: usize,
    elements: usize,
    choosable: usize,
    fixed_at_upper_code: usize,
    fixed_at_lower_code: usize,
    fixed_interior: usize,
    minimum_lower_code: i32,
    maximum_upper_code: i32,
    /// `[rows, 1]` for matrices, `[1]` for vectors.
    scales: Tensor,
    /// Frozen lower integer codes, as F32, same shape as the parameter.
    lower: Tensor,
    /// 1.0 where a choice exists, 0.0 for an exact-grid coordinate. This is
    /// exactly `upper - lower`, so it is also the penalty mask.
    choice: Tensor,
}

/// Offline learned rounding mechanism over one frozen parameter inventory.
#[derive(Debug)]
pub struct LearnedRounding {
    config: RoundingConfig,
    device: Device,
    variables: BTreeMap<String, Var>,
    parameters: Vec<FrozenParameter>,
    elements: usize,
    choosable_total: usize,
    initial_commitment: f64,
    denominator: Tensor,
    zero_coefficient: Tensor,
    regularization_coefficient: Tensor,
}

impl LearnedRounding {
    /// Freeze the parent representation and initialize the decisions.
    ///
    /// `parent` must be the projected parent variables bound to `spec`. The
    /// parent is only read; its values and `Var` identities are preserved.
    pub fn new(
        spec: &QuantizationSpec,
        parent: &BTreeMap<String, Var>,
        config: RoundingConfig,
    ) -> Result<Self> {
        config.validate()?;
        // Inventory, dtype, shape and the structural element/parameter limits.
        spec.validate(parent)?;
        let device = parent
            .values()
            .next()
            .ok_or_else(|| invalid("learned rounding requires a nonempty parent inventory"))?
            .device();
        let mut identities = Vec::with_capacity(parent.len());
        for (name, variable) in parent {
            if !device.same_device(variable.device())
                || !variable.is_contiguous()
                || identities.contains(&variable.id())
            {
                return Err(invalid(format!(
                    "learned rounding requires distinct contiguous parent variables on one device; \
                     a tied parameter must appear once: {name}"
                )));
            }
            identities.push(variable.id());
        }

        let mut variables = BTreeMap::new();
        let mut parameters = Vec::with_capacity(spec.parameters.len());
        let mut elements = 0usize;
        let mut choosable_total = 0usize;
        let mut commitment_sum = 0.0f64;
        for (name, parameter) in &spec.parameters {
            let variable = parent.get(name).ok_or_else(|| {
                invalid(format!("learned rounding parent inventory lacks {name}"))
            })?;
            let (frozen, alpha, commitment) =
                freeze_parameter(name, parameter, variable, &config, device)?;
            elements = elements
                .checked_add(frozen.elements)
                .ok_or_else(|| invalid("learned rounding element total overflow"))?;
            choosable_total = choosable_total
                .checked_add(frozen.choosable)
                .ok_or_else(|| invalid("learned rounding choice total overflow"))?;
            commitment_sum += commitment;
            parameters.push(frozen);
            variables.insert(name.clone(), alpha);
        }
        if elements == 0 || parameters.is_empty() {
            return Err(invalid("learned rounding requires nonempty parameters"));
        }

        // A fully exact-grid inventory has no defined mean; report zero instead
        // and let `statistics` mark it as not trainable. The choice count is
        // bounded by the spec's element limit, so verify the F32 round trip
        // rather than assuming it.
        let denominator = choosable_total.max(1) as f32;
        if f64::from(denominator) != choosable_total.max(1) as f64 {
            return Err(invalid("learned rounding choice count is inexact in F32"));
        }
        Ok(Self {
            denominator: Tensor::new(denominator, device)?,
            zero_coefficient: Tensor::new(0f32, device)?,
            regularization_coefficient: Tensor::new(config.regularization as f32, device)?,
            initial_commitment: if choosable_total > 0 {
                commitment_sum / choosable_total as f64
            } else {
                0.0
            },
            config,
            device: device.clone(),
            variables,
            parameters,
            elements,
            choosable_total,
        })
    }

    /// Trainable per-coordinate decisions: the parent inventory and shapes.
    pub fn variables(&self) -> &BTreeMap<String, Var> {
        &self.variables
    }

    /// Parameter tensors for the model graph. `hard = false` returns the
    /// differentiable soft values; `hard = true` returns detached legal grid
    /// values with exactly one integer code chosen per coordinate.
    pub fn parameters(&self, hard: bool) -> Result<BTreeMap<String, Tensor>> {
        let mut parameters = BTreeMap::new();
        for parameter in &self.parameters {
            let alpha = self.alpha(&parameter.name)?;
            let value = if hard {
                hard_values(parameter, alpha)?
            } else {
                soft_values(parameter, alpha)?
            };
            parameters.insert(parameter.name.clone(), value);
        }
        Ok(parameters)
    }

    /// Weighted commitment term at `completed_updates`; add it to the loss once.
    pub fn penalty(&self, completed_updates: usize) -> Result<Tensor> {
        let beta = self.config.beta(completed_updates);
        let mut contributions = Vec::with_capacity(self.parameters.len());
        for parameter in &self.parameters {
            let decision = decision(self.alpha(&parameter.name)?)?;
            // `|2d-1|^beta` is finite and differentiable at both endpoints, and
            // its gradient vanishes at the centre for the required `beta >= 1`.
            let centre = decision.affine(2.0, -1.0)?.abs()?;
            let term = centre.powf(beta)?.affine(-1.0, 1.0)?;
            contributions.push(term.mul(&parameter.choice)?.sum_all()?);
        }
        let coefficient = if completed_updates < self.config.warmup_steps {
            &self.zero_coefficient
        } else {
            &self.regularization_coefficient
        };
        Ok(Tensor::stack(&contributions, 0)?
            .sum_all()?
            .div(&self.denominator)?
            .mul(coefficient)?)
    }

    pub fn config(&self) -> &RoundingConfig {
        &self.config
    }

    /// Frozen-inventory diagnostics. Host-only; no device read.
    pub fn statistics(&self) -> Result<Value> {
        let mut parameters = Vec::with_capacity(self.parameters.len());
        for parameter in &self.parameters {
            let fixed = parameter
                .fixed_at_upper_code
                .checked_add(parameter.fixed_at_lower_code)
                .and_then(|total| total.checked_add(parameter.fixed_interior))
                .ok_or_else(|| invalid("learned rounding fixed count overflow"))?;
            parameters.push(json!({
                "name": parameter.name.clone(),
                "shape": parameter.shape.clone(),
                "bits": parameter.bits,
                "scale_rows": parameter.rows,
                "row_width": parameter.width,
                "exponents": parameter.exponents.clone(),
                "elements": parameter.elements,
                "choosable_coordinates": parameter.choosable,
                "fixed_coordinates": fixed,
                "fixed_at_upper_code": parameter.fixed_at_upper_code,
                "fixed_at_lower_code": parameter.fixed_at_lower_code,
                "fixed_interior": parameter.fixed_interior,
                "minimum_lower_code": parameter.minimum_lower_code,
                "maximum_upper_code": parameter.maximum_upper_code,
            }));
        }
        Ok(json!({
            "schema": STATISTICS_SCHEMA,
            "mechanism": ROUNDING_SCHEMA,
            "parameters": parameters,
            "config": {
                "steps": self.config.steps,
                "warmup_steps": self.config.warmup_steps,
                "beta_start": self.config.beta_start,
                "beta_end": self.config.beta_end,
                "regularization": self.config.regularization,
            },
            "totals": {
                "parameters": self.parameters.len(),
                "elements": self.elements,
                "choosable_coordinates": self.choosable_total,
                "fixed_coordinates": self.elements - self.choosable_total,
                "penalty_denominator": self.choosable_total.max(1),
            },
            "trainable": self.choosable_total > 0,
            "initial_commitment_mean": self.initial_commitment,
            "device": format!("{:?}", self.device),
            "semantics": {
                "neighbours": "clamped integer lower=floor(w/s) and upper=ceil(w/s) of the original \
                               parent value at its frozen row scale; exact-grid coordinates have \
                               upper==lower, stay fixed and are excluded from the penalty",
                "alpha": "one Var per parent coordinate, same inventory and shape as the parent; \
                          initialized as the inverse relaxation of the original fraction",
                "relaxation": "d = clamp(1.2*sigmoid(alpha) - 0.1, 0, 1); soft value is \
                               s*(lower + choice*d)",
                "hard": "detached; alpha>0 selects upper, alpha<0 selects lower, exactly \
                         zero preserves the original round-away-zero code; signed zero canonical",
                "penalty": "coefficient(completed_updates) * sum over nondegenerate coordinates of \
                            (1 - |2d-1|^beta) / nondegenerate coordinate count; zero during warm-up",
                "beta": "cosine from beta_start to beta_end across [warmup_steps, steps]",
                "scope": "offline F32 learner only; no optimizer, CLI or integer serving path",
            },
        }))
    }

    fn alpha(&self, name: &str) -> Result<&Tensor> {
        Ok(self
            .variables
            .get(name)
            .ok_or_else(|| invalid(format!("missing learned rounding decision {name}")))?
            .as_tensor())
    }
}

/// Freeze one parameter and build its decision variable.
fn freeze_parameter(
    name: &str,
    parameter: &ParameterQuantization,
    variable: &Var,
    config: &RoundingConfig,
    device: &Device,
) -> Result<(FrozenParameter, Var, f64)> {
    let shape = parameter.shape.clone();
    if !matches!(shape.len(), 1 | 2) {
        return Err(invalid(format!(
            "learned rounding requires vectors or matrices: {name}"
        )));
    }
    let (rows, width) = if shape.len() == 2 {
        (shape[0], shape[1])
    } else {
        (1, shape[0])
    };
    if rows == 0 || width == 0 {
        return Err(invalid(format!(
            "learned rounding requires nonempty parameters: {name}"
        )));
    }
    let elements = shape
        .iter()
        .try_fold(1usize, |total, &dimension| total.checked_mul(dimension))
        .ok_or_else(|| invalid(format!("learned rounding shape overflow for {name}")))?;
    if parameter.bits == 16 && shape.len() != 1 {
        return Err(invalid(format!(
            "learned rounding additive parameters require one vector scale: {name}"
        )));
    }
    let (minimum_code, maximum_code) = code_limits(parameter.bits)?;
    let minimum_code_f = minimum_code as f32;
    let maximum_code_f = maximum_code as f32;
    if parameter.row_exponents.len() != rows {
        return Err(invalid(format!(
            "learned rounding scale-row count differs for {name}"
        )));
    }
    let scale_values = parameter
        .row_exponents
        .iter()
        .map(|&exponent| dyadic_step(exponent))
        .collect::<Result<Vec<f32>>>()?;

    let values = variable
        .as_detached_tensor()
        .flatten_all()?
        .to_vec1::<f32>()?;
    if values.len() != elements {
        return Err(invalid(format!(
            "learned rounding element count differs for {name}"
        )));
    }

    let mut lower_codes = vec![0f32; elements];
    let mut choices = vec![0f32; elements];
    let mut alpha_values = vec![0f32; elements];
    let mut choosable = 0usize;
    let mut fixed_at_upper_code = 0usize;
    let mut fixed_at_lower_code = 0usize;
    let mut fixed_interior = 0usize;
    let mut minimum_lower_code = i32::MAX;
    let mut maximum_upper_code = i32::MIN;
    let mut commitment = 0.0f64;

    for (row, row_values) in values.chunks_exact(width).enumerate() {
        let scale = scale_values
            .get(row)
            .copied()
            .ok_or_else(|| invalid(format!("learned rounding scale row missing for {name}")))?;
        for (column, &value) in row_values.iter().enumerate() {
            let index = row * width + column;
            if !value.is_finite() {
                return Err(invalid(format!(
                    "nonfinite learned rounding parent value in {name}"
                )));
            }
            let ratio = value / scale;
            if !ratio.is_finite() {
                return Err(invalid(format!(
                    "nonfinite learned rounding ratio in {name}"
                )));
            }
            let lower = ratio.floor();
            let upper = ratio.ceil();
            // The clip is inert for an in-range parent; a coordinate outside the
            // fixed interval is a representation change and is rejected instead
            // of being absorbed silently. Both neighbours are exact integers in
            // F32 because |code| <= 32767.
            if lower < minimum_code_f || upper > maximum_code_f {
                return Err(invalid(format!(
                    "learned rounding parent {name} lies outside its fixed dyadic range; \
                     project the parent first"
                )));
            }
            lower_codes[index] = lower;
            let selectable = if upper > lower { 1.0f32 } else { 0.0f32 };
            choices[index] = selectable;
            if selectable == 0.0 {
                if ratio == maximum_code_f {
                    fixed_at_upper_code += 1;
                } else if ratio == minimum_code_f {
                    fixed_at_lower_code += 1;
                } else {
                    fixed_interior += 1;
                }
            } else {
                choosable += 1;
                let fraction = f64::from(ratio) - f64::from(lower);
                // Invert `d = clamp(1.2*sigmoid(alpha) - 0.1, 0, 1)` so the soft
                // value reconstructs the parent at initialization. The affine
                // keeps the target strictly inside (0,1) for every fraction.
                let target = (fraction - RECTIFIER_OFFSET) / RECTIFIER_SCALE;
                if !(target > 0.0 && target < 1.0) {
                    return Err(invalid(format!(
                        "learned rounding initialization is undefined in {name}"
                    )));
                }
                alpha_values[index] = ((target / (1.0 - target)).ln()) as f32;
                let centre = (2.0 * fraction - 1.0).abs();
                commitment += 1.0 - centre.powf(config.beta_start);
            }
            let lower_code = lower as i32;
            let upper_code = upper as i32;
            minimum_lower_code = minimum_lower_code.min(lower_code);
            maximum_upper_code = maximum_upper_code.max(upper_code);
        }
    }

    let scale_shape = if shape.len() == 2 {
        vec![shape[0], 1]
    } else {
        vec![1]
    };
    let mut exponents = parameter.row_exponents.clone();
    exponents.sort_unstable();
    exponents.dedup();
    let frozen = FrozenParameter {
        name: name.to_string(),
        shape: shape.clone(),
        exponents,
        bits: parameter.bits,
        rows,
        width,
        elements,
        choosable,
        fixed_at_upper_code,
        fixed_at_lower_code,
        fixed_interior,
        minimum_lower_code,
        maximum_upper_code,
        scales: Tensor::from_vec(scale_values, scale_shape.clone(), device)?,
        lower: Tensor::from_vec(lower_codes, shape.clone(), device)?,
        choice: Tensor::from_vec(choices, shape.clone(), device)?,
    };
    let alpha = Var::from_vec(alpha_values, shape.clone(), device)?;
    Ok((frozen, alpha, commitment))
}

/// Overflow-free logistic. Both exponentials are taken on an argument bounded
/// by [`ALPHA_LIMIT`], so every intermediate and every gradient is finite.
fn sigmoid_tensor(alpha: &Tensor) -> Result<Tensor> {
    let bounded = alpha.clamp(-ALPHA_LIMIT, ALPHA_LIMIT)?;
    let positive = bounded.neg()?.exp()?.affine(1.0, 1.0)?.recip()?;
    let negative = bounded.exp()?;
    let negative = negative.mul(&negative.affine(1.0, 1.0)?.recip()?)?;
    Ok(bounded.ge(0f64)?.where_cond(&positive, &negative)?)
}

/// Differentiable relaxed decision `d`.
fn decision(alpha: &Tensor) -> Result<Tensor> {
    Ok(sigmoid_tensor(alpha)?
        .affine(RECTIFIER_SCALE, RECTIFIER_OFFSET)?
        .clamp(0f64, 1f64)?)
}

/// Differentiable soft parameter value.
fn soft_values(parameter: &FrozenParameter, alpha: &Tensor) -> Result<Tensor> {
    let decision = decision(alpha)?;
    let code = parameter.lower.add(&parameter.choice.mul(&decision)?)?;
    Ok(code.broadcast_mul(&parameter.scales)?)
}

/// Detached hard parameter value: one legal integer code per coordinate.
fn hard_values(parameter: &FrozenParameter, alpha: &Tensor) -> Result<Tensor> {
    let alpha = alpha.detach();
    let ones = alpha.ones_like()?;
    let zeros = alpha.zeros_like()?;
    // A tie is reachable exactly at `alpha == 0`, which is the initialization of
    // every half-step coordinate. `lower >= 0` selects `upper`, so the original
    // nearest round-away-zero code is preserved: `+x.5 -> x+1` and `-x.5 -> -x-1`.
    let tie = parameter.lower.ge(0f64)?.to_dtype(DType::F32)?;
    let selection = alpha
        .gt(0f64)?
        .where_cond(&ones, &alpha.lt(0f64)?.where_cond(&zeros, &tie)?)?;
    let code = parameter.lower.add(&parameter.choice.mul(&selection)?)?;
    let value = code.broadcast_mul(&parameter.scales)?.detach();
    // Packed signed integer zero has no sign bit; canonicalize negative zero.
    Ok(value
        .eq(0f64)?
        .where_cond(&value.zeros_like()?, &value)?
        .detach())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint_quantization::SPEC_SCHEMA;
    use crate::{
        FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE, FINITE_DIFFERENCE_EPSILON,
        FINITE_DIFFERENCE_RELATIVE_TOLERANCE,
    };

    fn values(tensor: &Tensor) -> Result<Vec<f32>> {
        Ok(tensor.flatten_all()?.to_vec1::<f32>()?)
    }

    /// A constant F32 tensor of the given shape.
    fn filled(value: f32, shape: &[usize], device: &Device) -> Result<Tensor> {
        let count = shape
            .iter()
            .try_fold(1usize, |total, &dimension| total.checked_mul(dimension))
            .ok_or_else(|| invalid("test shape overflow"))?;
        Ok(Tensor::from_vec(
            vec![value; count],
            shape.to_vec(),
            device,
        )?)
    }

    /// Set every decision of one mechanism to the same value.
    fn set_all(rounding: &LearnedRounding, value: f32) -> Result<()> {
        for variable in rounding.variables().values() {
            variable.set(&filled(value, variable.dims(), &Device::Cpu)?)?;
        }
        Ok(())
    }

    /// The mechanism's own objective: weighted commitment plus the soft values.
    fn learning_loss(rounding: &LearnedRounding, completed_updates: usize) -> Result<Tensor> {
        let mut total = rounding.penalty(completed_updates)?;
        for tensor in rounding.parameters(false)?.values() {
            total = total.add(&tensor.sum_all()?)?;
        }
        Ok(total)
    }

    fn loss_value(rounding: &LearnedRounding, completed_updates: usize) -> Result<f64> {
        Ok(f64::from(
            learning_loss(rounding, completed_updates)?.to_scalar::<f32>()?,
        ))
    }

    fn parameter(shape: Vec<usize>, bits: u8, row_exponents: Vec<i16>) -> ParameterQuantization {
        ParameterQuantization {
            shape,
            bits,
            row_exponents,
        }
    }

    /// Two bit widths, both shape forms, ties in both signs, both code
    /// endpoints, an interior exact-grid value, an exact zero (including a
    /// signed zero) and ordinary interior values.
    fn fixture() -> Result<(QuantizationSpec, BTreeMap<String, Var>)> {
        let spec = QuantizationSpec {
            schema: SPEC_SCHEMA.into(),
            parameters: BTreeMap::from([
                (
                    "layer.weight".to_string(),
                    parameter(vec![2, 4], 4, vec![-1, -1]),
                ),
                ("layer.bias".to_string(), parameter(vec![4], 16, vec![-4])),
                ("read.age".to_string(), parameter(vec![4], 16, vec![-12])),
            ]),
        };
        let device = Device::Cpu;
        let parent = BTreeMap::from([
            (
                "layer.weight".to_string(),
                Var::from_vec(
                    vec![-1.75f32, -0.25, 0.25, 1.75, -3.5, 3.5, 1.5, 3.1],
                    (2, 4),
                    &device,
                )?,
            ),
            (
                "layer.bias".to_string(),
                Var::from_vec(vec![-0.15625f32, 2047.9375, -2047.9375, 0.0], (4,), &device)?,
            ),
            (
                "read.age".to_string(),
                Var::from_vec(vec![1e-6f32, 0.000732421875, 0.0, -0.0], (4,), &device)?,
            ),
        ]);
        spec.validate(&parent)?;
        Ok((spec, parent))
    }

    fn fixture_config() -> RoundingConfig {
        RoundingConfig {
            steps: 8,
            warmup_steps: 2,
            beta_start: 2.0,
            beta_end: 2.0,
            regularization: 1.0,
        }
    }

    /// Deterministic pseudo-random parent, calibrated by the crate's own
    /// public calibration instead of a hand-built grid.
    fn calibrated() -> Result<(QuantizationSpec, BTreeMap<String, Var>)> {
        let device = Device::Cpu;
        let mut state = 0x9e3779b97f4a7c15u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state >> 11) as f64 / (1u64 << 53) as f64) - 0.5
        };
        let mut first = Vec::with_capacity(6 * 4);
        let mut second = Vec::with_capacity(3 * 4);
        for _ in 0..24 {
            first.push((next() * 0.5) as f32);
        }
        for _ in 0..12 {
            second.push((next() * 0.02) as f32);
        }
        let parent = BTreeMap::from([
            (
                "read.value.weight".to_string(),
                Var::from_vec(first, (6, 4), &device)?,
            ),
            (
                "read.value.bias".to_string(),
                Var::from_vec(second, (12,), &device)?,
            ),
        ]);
        let spec = crate::joint_quantization::calibrate(&parent)?;
        Ok((spec, parent))
    }

    #[test]
    fn hard_initialization_reproduces_the_existing_hard_codes_bit_exactly() -> Result<()> {
        let (spec, parent) = fixture()?;
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        let hard = rounding.parameters(true)?;
        assert_eq!(
            hard.keys().collect::<Vec<_>>(),
            parent.keys().collect::<Vec<_>>()
        );
        for (name, variable) in &parent {
            // Public oracle: the crate's existing clamp-then-round-away-zero
            // grid. Covers scale derivation, both code endpoints, the tie rule
            // in both signs, exact-grid values and signed zero together.
            let oracle = spec.parameter(name, &variable.as_detached_tensor(), 1.0, false)?;
            let mine = values(&hard[name])?;
            let theirs = values(&oracle)?;
            assert_eq!(
                mine.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
                theirs
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                "hard initialization differs from the retained grid for {name}"
            );
        }
        // The signed-zero coordinate is canonicalized in both paths.
        assert!(values(&hard["read.age"])?[3].is_sign_positive());
        Ok(())
    }

    #[test]
    fn representable_values_on_both_sides_of_half_do_not_become_false_ties() -> Result<()> {
        let values = vec![
            f32::from_bits((-0.5f32).to_bits() - 1),
            -0.5,
            f32::from_bits((-0.5f32).to_bits() + 1),
            f32::from_bits(0.5f32.to_bits() - 1),
            0.5,
            f32::from_bits(0.5f32.to_bits() + 1),
        ];
        let parent = BTreeMap::from([(
            "edge.weight".into(),
            Var::from_vec(values, (6,), &Device::Cpu)?,
        )]);
        let spec = QuantizationSpec {
            schema: SPEC_SCHEMA.into(),
            parameters: BTreeMap::from([("edge.weight".into(), parameter(vec![6], 4, vec![0]))]),
        };
        let learner = LearnedRounding::new(&spec, &parent, fixture_config())?;
        let expected =
            spec.parameter("edge.weight", parent["edge.weight"].as_tensor(), 1.0, false)?;
        assert_eq!(
            learner.parameters(true)?["edge.weight"].to_vec1::<f32>()?,
            expected.to_vec1::<f32>()?
        );
        Ok(())
    }

    #[test]
    fn signed_ties_endpoints_and_exact_values_follow_the_declared_codes() -> Result<()> {
        let (spec, parent) = fixture()?;
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        let hard = values(&rounding.parameters(true)?["layer.weight"])?;
        // r = -3.5 -> -4, -0.5 -> -1, 0.5 -> +1, 3.5 -> +4 (round away from zero).
        // r = -7 and r = 7 are the code endpoints; r = 3 is exact on the grid.
        assert_eq!(hard, vec![-2.0f32, -0.5, 0.5, 2.0, -3.5, 3.5, 1.5, 3.0]);
        let bias = values(&rounding.parameters(true)?["layer.bias"])?;
        assert_eq!(bias, vec![-0.1875f32, 2047.9375, -2047.9375, 0.0]);
        let age = values(&rounding.parameters(true)?["read.age"])?;
        assert_eq!(age, vec![0.0f32, 0.000732421875, 0.0, 0.0]);
        Ok(())
    }

    #[test]
    fn exact_grid_coordinates_stay_fixed_and_hard_values_stay_legal_for_any_alpha() -> Result<()> {
        let (spec, parent) = fixture()?;
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        // Sweep including the saturated limits and an out-of-range value.
        for alpha in [-60f32, -30.0, -2.5, -0.5, 0.0, 0.5, 2.5, 30.0, 60.0] {
            set_all(&rounding, alpha)?;
            let hard = rounding.parameters(true)?;
            for parameter in &rounding.parameters {
                let scale = values(&parameter.scales)?;
                let lower = values(&parameter.lower)?;
                let choice = values(&parameter.choice)?;
                let mut seen_fixed = 0usize;
                for (index, value) in values(&hard[&parameter.name])?.iter().enumerate() {
                    let row = if parameter.shape.len() == 2 {
                        index / parameter.width
                    } else {
                        0
                    };
                    let code = value / scale[row];
                    assert_eq!(code, code.round(), "hard value off grid");
                    let (minimum, maximum) = code_limits(parameter.bits)?;
                    assert!(code >= minimum as f32 && code <= maximum as f32);
                    if choice[index] == 0.0 {
                        // Exact-grid coordinates keep the frozen single neighbor.
                        assert_eq!(code, lower[index], "fixed coordinate moved");
                        seen_fixed += 1;
                    } else {
                        // The other neighbour is always one code away.
                        assert!(code == lower[index] || code == lower[index] + 1.0);
                    }
                }
                assert_eq!(seen_fixed, parameter.elements - parameter.choosable);
            }
        }
        Ok(())
    }

    #[test]
    fn soft_values_reconstruct_the_parent_and_fixed_coordinates_are_exact() -> Result<()> {
        let (spec, parent) = fixture()?;
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        let soft = rounding.parameters(false)?;
        let scales = BTreeMap::from([
            ("layer.weight", 0.5f32),
            ("layer.bias", 0.0625),
            ("read.age", 0.000244140625),
        ]);
        for (name, variable) in &parent {
            let original = values(&variable.as_detached_tensor())?;
            let relaxed = values(&soft[name])?;
            let scale = scales[name.as_str()];
            for (index, (&before, &after)) in original.iter().zip(relaxed.iter()).enumerate() {
                if before == 0.0 {
                    assert_eq!(after, before, "zero coordinate changed for {name}");
                    continue;
                }
                let allowed = 8.0 * f32::EPSILON * before.abs().max(scale);
                assert!(
                    (after - before).abs() <= allowed,
                    "soft reconstruction {name}[{index}]: {before} vs {after}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn soft_values_are_differentiable_and_match_the_closed_form() -> Result<()> {
        let (spec, parent) = fixture()?;
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        // d/dalpha of s*(lower + choice*d) is s*choice*1.2*sigmoid*(1-sigmoid)
        // while the rectifier stays strictly inside its clamp.
        for alpha in [0f32, 1.0, -1.0] {
            set_all(&rounding, alpha)?;
            let gradients = learning_loss(&rounding, 0)?.backward()?;
            let sigmoid = 1.0f64 / (1.0 + (-f64::from(alpha)).exp());
            for parameter in &rounding.parameters {
                let gradient = values(
                    gradients
                        .get(rounding.variables()[&parameter.name].as_tensor())
                        .ok_or_else(|| invalid("missing soft gradient"))?,
                )?;
                let scale = values(&parameter.scales)?;
                let choice = values(&parameter.choice)?;
                for (index, &value) in gradient.iter().enumerate() {
                    assert!(value.is_finite(), "nonfinite soft gradient");
                    let row = if parameter.shape.len() == 2 {
                        index / parameter.width
                    } else {
                        0
                    };
                    let expected = f64::from(choice[index])
                        * f64::from(scale[row])
                        * 1.2
                        * sigmoid
                        * (1.0 - sigmoid);
                    assert!(
                        (f64::from(value) - expected).abs() <= 1e-5 * expected.abs().max(1e-6),
                        "soft gradient {}/{}: {value} vs {expected}",
                        parameter.name,
                        index
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn penalty_is_a_coordinate_weighted_global_mean_not_a_mean_of_tensor_means() -> Result<()> {
        let device = Device::Cpu;
        // 8 choosable coordinates against 1: the global mean is 1/9, while the
        // mean of the two tensor means would be 1/2.
        let spec = QuantizationSpec {
            schema: SPEC_SCHEMA.into(),
            parameters: BTreeMap::from([
                (
                    "wide.weight".to_string(),
                    parameter(vec![1, 8], 4, vec![-1]),
                ),
                ("narrow.bias".to_string(), parameter(vec![1], 16, vec![-4])),
            ]),
        };
        let parent = BTreeMap::from([
            (
                "wide.weight".to_string(),
                Var::from_vec(vec![0.25f32; 8], (1, 8), &device)?,
            ),
            (
                "narrow.bias".to_string(),
                Var::from_vec(vec![0.03125f32], (1,), &device)?,
            ),
        ]);
        spec.validate(&parent)?;
        let config = RoundingConfig {
            steps: 8,
            warmup_steps: 2,
            beta_start: 2.0,
            beta_end: 2.0,
            regularization: 1.0,
        };
        let rounding = LearnedRounding::new(&spec, &parent, config)?;
        let statistics = rounding.statistics()?;
        assert_eq!(statistics["totals"]["choosable_coordinates"], json!(9));
        assert_eq!(statistics["trainable"], json!(true));
        // Committed (d = 1, term 0) for the wide parameter, centered (d = 0.5,
        // term 1) for the narrow one.
        rounding.variables()["wide.weight"].set(&filled(30f32, &[1, 8], &device)?)?;
        rounding.variables()["narrow.bias"].set(&filled(0f32, &[1], &device)?)?;
        let penalty = rounding.penalty(4)?.to_scalar::<f32>()?;
        assert!((penalty - 1.0 / 9.0).abs() < 1e-6, "penalty {penalty}");
        assert!(
            (penalty - 0.5).abs() > 0.3,
            "mean of tensor means leaked in"
        );
        // Both endpoints are fully committed, and the warm-up is exactly zero.
        rounding.variables()["narrow.bias"].set(&filled(-30f32, &[1], &device)?)?;
        assert_eq!(rounding.penalty(4)?.to_scalar::<f32>()?, 0.0);
        rounding.variables()["narrow.bias"].set(&filled(0f32, &[1], &device)?)?;
        assert_eq!(rounding.penalty(0)?.to_scalar::<f32>()?, 0.0);
        assert_eq!(rounding.penalty(1)?.to_scalar::<f32>()?, 0.0);
        Ok(())
    }

    #[test]
    fn penalty_matches_the_independent_host_formula_at_initialization() -> Result<()> {
        let (spec, parent) = fixture()?;
        let config = RoundingConfig {
            steps: 8,
            warmup_steps: 0,
            beta_start: 3.0,
            beta_end: 3.0,
            regularization: 2.0,
        };
        let rounding = LearnedRounding::new(&spec, &parent, config)?;
        // Independent host recomputation from the frozen parent values.
        let mut total = 0.0f64;
        let mut count = 0usize;
        for (name, variable) in &parent {
            let parameter = &spec.parameters[name];
            let scale = dyadic_step(parameter.row_exponents[0])?;
            for &value in values(&variable.as_detached_tensor())?.iter() {
                let ratio = value / scale;
                let lower = ratio.floor();
                if ratio.ceil() == lower {
                    continue;
                }
                let fraction = f64::from(ratio) - f64::from(lower);
                let centre = (2.0 * fraction - 1.0).abs();
                total += 1.0 - centre.powf(3.0);
                count += 1;
            }
        }
        let expected = 2.0 * total / count as f64;
        let penalty = f64::from(rounding.penalty(0)?.to_scalar::<f32>()?);
        assert!(
            (penalty - expected).abs() <= 1e-4 * expected.abs(),
            "{penalty} vs {expected}"
        );
        Ok(())
    }

    #[test]
    fn penalty_gradient_is_finite_zero_at_the_centre_and_matches_finite_differences() -> Result<()>
    {
        let device = Device::Cpu;
        let spec = QuantizationSpec {
            schema: SPEC_SCHEMA.into(),
            parameters: BTreeMap::from([(
                "pair.weight".to_string(),
                parameter(vec![1, 2], 4, vec![-1]),
            )]),
        };
        let parent = BTreeMap::from([(
            "pair.weight".to_string(),
            Var::from_vec(vec![0.25f32, 0.75], (1, 2), &device)?,
        )]);
        spec.validate(&parent)?;
        let config = RoundingConfig {
            steps: 4,
            warmup_steps: 0,
            beta_start: 2.0,
            beta_end: 2.0,
            regularization: 1.0,
        };
        let rounding = LearnedRounding::new(&spec, &parent, config)?;
        let variable = &rounding.variables()["pair.weight"];

        // Exactly at the centre (alpha = 0) the power gradient vanishes.
        variable.set(&filled(0f32, &[1, 2], &device)?)?;
        let gradients = rounding.penalty(1)?.backward()?;
        let centre = values(
            gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid("missing centre gradient"))?,
        )?;
        assert_eq!(centre, vec![0f32, 0f32]);

        // Saturated limits vanish too, so alpha cannot run away.
        variable.set(&filled(60f32, &[1, 2], &device)?)?;
        let gradients = rounding.penalty(1)?.backward()?;
        assert_eq!(
            values(
                gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid("missing saturated gradient"))?
            )?,
            vec![0f32, 0f32]
        );

        // Independent central finite differences of the same loss.
        let base = [1.0f32, -0.5];
        variable.set(&Tensor::from_vec(base.to_vec(), (1, 2), &device)?)?;
        let gradients = learning_loss(&rounding, 1)?.backward()?;
        let analytic = values(
            gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid("missing analytic gradient"))?,
        )?;
        assert!(analytic.iter().all(|value| value.is_finite()));
        for index in 0..base.len() {
            let mut plus = base;
            plus[index] += FINITE_DIFFERENCE_EPSILON;
            let mut minus = base;
            minus[index] -= FINITE_DIFFERENCE_EPSILON;
            variable.set(&Tensor::from_vec(plus.to_vec(), (1, 2), &device)?)?;
            let high = loss_value(&rounding, 1)?;
            variable.set(&Tensor::from_vec(minus.to_vec(), (1, 2), &device)?)?;
            let low = loss_value(&rounding, 1)?;
            let numeric = (high - low) / (2.0 * f64::from(FINITE_DIFFERENCE_EPSILON));
            let allowed = FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE
                .max(FINITE_DIFFERENCE_RELATIVE_TOLERANCE * numeric.abs());
            assert!(
                (f64::from(analytic[index]) - numeric).abs() <= allowed,
                "gradient {index}: {} vs {numeric}",
                analytic[index]
            );
        }
        Ok(())
    }

    #[test]
    fn every_decision_keeps_a_gradient_including_a_fully_degenerate_parameter() -> Result<()> {
        let device = Device::Cpu;
        // `fixed.bias` is exactly on the grid at every coordinate, so it has no
        // choice at all; `open.bias` has one. NamedAdamW rejects a variable with
        // no gradient, so both must appear in the store, also during warm-up.
        let spec = QuantizationSpec {
            schema: SPEC_SCHEMA.into(),
            parameters: BTreeMap::from([
                ("fixed.bias".to_string(), parameter(vec![2], 16, vec![-4])),
                ("open.bias".to_string(), parameter(vec![2], 16, vec![-4])),
            ]),
        };
        let parent = BTreeMap::from([
            (
                "fixed.bias".to_string(),
                Var::from_vec(vec![0.0f32, 0.0625], (2,), &device)?,
            ),
            (
                "open.bias".to_string(),
                Var::from_vec(vec![0.03125f32, 0.09375], (2,), &device)?,
            ),
        ]);
        spec.validate(&parent)?;
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        let statistics = rounding.statistics()?;
        assert_eq!(statistics["trainable"], json!(true));
        let per_parameter = statistics["parameters"]
            .as_array()
            .ok_or_else(|| invalid("array"))?;
        let fixed = per_parameter
            .iter()
            .find(|entry| entry["name"] == json!("fixed.bias"))
            .ok_or_else(|| invalid("missing"))?;
        assert_eq!(fixed["choosable_coordinates"], json!(0));
        assert_eq!(fixed["fixed_coordinates"], json!(2));

        for completed in [0usize, 3] {
            let hard = rounding.parameters(true)?;
            let soft = rounding.parameters(false)?;
            assert!(!hard["fixed.bias"].track_op());
            assert!(!hard["open.bias"].track_op());
            let mut loss = rounding.penalty(completed)?;
            for tensor in soft.values() {
                loss = loss.add(&tensor.sum_all()?)?;
            }
            let gradients = loss.backward()?;
            for (name, variable) in rounding.variables() {
                let gradient = gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("missing gradient for {name}")))?;
                assert!(values(gradient)?.iter().all(|value| value.is_finite()));
            }
            // The fully degenerate parameter never moves its hard value.
            assert_eq!(values(&hard["fixed.bias"])?, vec![0f32, 0.0625]);
        }
        Ok(())
    }

    #[test]
    fn parent_variables_are_never_mutated() -> Result<()> {
        let (spec, parent) = fixture()?;
        let before: BTreeMap<String, Vec<u32>> = parent
            .iter()
            .map(|(name, variable)| {
                Ok((
                    name.clone(),
                    values(&variable.as_detached_tensor())?
                        .iter()
                        .map(|value| value.to_bits())
                        .collect(),
                ))
            })
            .collect::<Result<_>>()?;
        let identities: Vec<_> = parent.values().map(|variable| variable.id()).collect();
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        let _ = rounding.parameters(true)?;
        let _ = rounding.parameters(false)?;
        let _ = rounding.penalty(3)?;
        for (name, variable) in &parent {
            let after: Vec<u32> = values(&variable.as_detached_tensor())?
                .iter()
                .map(|value| value.to_bits())
                .collect();
            assert_eq!(before[name], after, "parent {name} was mutated");
        }
        assert_eq!(
            identities,
            parent
                .values()
                .map(|variable| variable.id())
                .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn decision_inventory_matches_the_parent_and_round_trips_through_the_spec() -> Result<()> {
        let (spec, parent) = calibrated()?;
        // Calibration chooses scales; the production parents additionally
        // project their shadows into the fixed representable range.
        spec.project_parameters(&parent)?;
        let rounding = LearnedRounding::new(&spec, &parent, fixture_config())?;
        // Exactly the integration check used by the model's rounding view.
        spec.validate(rounding.variables())?;
        assert_eq!(
            rounding.variables().keys().collect::<Vec<_>>(),
            spec.parameters.keys().collect::<Vec<_>>()
        );
        assert_eq!(rounding.variables().len(), spec.parameters.len());
        for (name, variable) in rounding.variables() {
            assert_eq!(variable.dims(), spec.parameters[name].shape.as_slice());
            assert_eq!(variable.dtype(), DType::F32);
        }
        let soft = rounding.parameters(false)?;
        assert_eq!(
            soft.keys().collect::<Vec<_>>(),
            spec.parameters.keys().collect::<Vec<_>>()
        );
        // A calibrated parent must still initialize at its own hard codes.
        let hard = rounding.parameters(true)?;
        for (name, variable) in &parent {
            let oracle = spec.parameter(name, &variable.as_detached_tensor(), 1.0, false)?;
            assert_eq!(values(&hard[name])?, values(&oracle)?, "{name}");
        }
        Ok(())
    }

    #[test]
    fn a_duplicated_tied_parameter_is_rejected() -> Result<()> {
        let (spec, parent) = fixture()?;
        let mut duplicated = BTreeMap::new();
        for (name, variable) in &parent {
            duplicated.insert(
                name.clone(),
                Var::from_tensor(&variable.as_detached_tensor())?,
            );
        }
        // Same values under distinct identities: accepted.
        let first = duplicated["layer.bias"].id();
        assert!(LearnedRounding::new(&spec, &duplicated, fixture_config()).is_ok());
        // One identity under two names is a tied parameter that must appear once.
        // The parent inventory already carries its tied input/output embedding as
        // a single entry, so the mechanism requires the same convention.
        duplicated.insert(
            "layer.bias.alias".to_string(),
            duplicated["layer.bias"].clone(),
        );
        let mut aliased_spec = spec.clone();
        aliased_spec.parameters.insert(
            "layer.bias.alias".to_string(),
            parameter(vec![4], 16, vec![-4]),
        );
        assert_eq!(first, duplicated["layer.bias.alias"].id());
        assert!(LearnedRounding::new(&aliased_spec, &duplicated, fixture_config()).is_err());
        Ok(())
    }

    #[test]
    fn out_of_range_and_nonfinite_parents_fail_closed() -> Result<()> {
        let (spec, _) = fixture()?;
        let device = Device::Cpu;
        let mut parent = BTreeMap::new();
        parent.insert(
            "layer.weight".to_string(),
            Var::from_vec(vec![0f32; 8], (2, 4), &device)?,
        );
        parent.insert(
            "layer.bias".to_string(),
            Var::from_vec(vec![0f32; 4], (4,), &device)?,
        );
        parent.insert(
            "read.age".to_string(),
            Var::from_vec(vec![0f32; 4], (4,), &device)?,
        );
        // Scale 0.5: 4.0 is r = 8, outside [-7,7].
        parent.insert(
            "layer.weight".to_string(),
            Var::from_vec(vec![4f32; 8], (2, 4), &device)?,
        );
        let error = LearnedRounding::new(&spec, &parent, fixture_config());
        assert!(error.is_err());
        parent.insert(
            "layer.weight".to_string(),
            Var::from_vec(vec![f32::NAN; 8], (2, 4), &device)?,
        );
        assert!(LearnedRounding::new(&spec, &parent, fixture_config()).is_err());
        Ok(())
    }

    #[test]
    fn config_validates_beta_regularization_and_warmup_and_schedules_beta() -> Result<()> {
        let valid = RoundingConfig {
            steps: 20,
            warmup_steps: 4,
            beta_start: 20.0,
            beta_end: 2.0,
            regularization: 1e-3,
        };
        valid.validate()?;
        // Cosine endpoints, warm-up hold and monotone decrease.
        assert_eq!(valid.beta(0), 20.0);
        assert_eq!(valid.beta(3), 20.0);
        assert_eq!(valid.beta(4), 20.0);
        assert!((valid.beta(20) - 2.0).abs() < 1e-12);
        let mut previous = valid.beta(4);
        for completed in 5..=20 {
            let current = valid.beta(completed);
            assert!(current <= previous + 1e-12 && current >= 2.0 - 1e-12);
            previous = current;
        }
        // Saturated schedule and the degenerate window stay defined.
        let saturated = RoundingConfig {
            warmup_steps: 4,
            beta_start: 2.0,
            beta_end: 2.0,
            ..valid.clone()
        };
        assert_eq!(saturated.beta(0), 2.0);
        assert_eq!(saturated.beta(usize::MAX), 2.0);
        let empty = RoundingConfig {
            steps: 4,
            warmup_steps: 4,
            ..valid.clone()
        };
        assert!(empty.validate().is_err());
        assert_eq!(empty.beta(4), 2.0);

        for invalid_config in [
            RoundingConfig {
                steps: 0,
                ..valid.clone()
            },
            RoundingConfig {
                warmup_steps: 20,
                ..valid.clone()
            },
            RoundingConfig {
                beta_start: 0.5,
                ..valid.clone()
            },
            RoundingConfig {
                beta_end: 0.0,
                ..valid.clone()
            },
            RoundingConfig {
                beta_start: f64::NAN,
                ..valid.clone()
            },
            RoundingConfig {
                beta_end: f64::INFINITY,
                ..valid.clone()
            },
            RoundingConfig {
                regularization: -1.0,
                ..valid.clone()
            },
            RoundingConfig {
                regularization: f64::NAN,
                ..valid.clone()
            },
        ] {
            assert!(
                invalid_config.validate().is_err(),
                "accepted {invalid_config:?}"
            );
        }
        // Serde round-trip, unknown-field rejection and equality.
        let encoded = serde_json::to_string(&valid)?;
        assert_eq!(serde_json::from_str::<RoundingConfig>(&encoded)?, valid);
        let unknown = encoded.replace("{\"steps\"", "{\"extra\":1,\"steps\"");
        assert!(serde_json::from_str::<RoundingConfig>(&unknown).is_err());
        Ok(())
    }
}
