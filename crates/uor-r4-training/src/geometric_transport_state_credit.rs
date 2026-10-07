//! Offline input credit from frozen prefix/end tables to retained H4 states.
//!
//! Native metadata, actual retained roots and integer scores remain authoritative.
//! Primary credit retains all 120 exact local root counterfactual table scores
//! and applies the existing choice pullback to actual POSTSTATE choice logits.
//! The other endpoint and prior recurrent state stay factual: this is conditional
//! local input credit, not an argmax derivative or a global joint posterior.
//! Earlier recurrent state propagation still uses its declared four-coordinate
//! approximation. A separately named least-squares ambient control is retained;
//! it projects 120 scores to four coordinates and can discard higher harmonics.
//! An explicit opt-in temporal-utility path attaches the same exact120 tables
//! directly to retained hard state_choices; ContextOp owns its single pullback
//! and full120 factual-action carry. Existing APIs keep their old default.
//! None of these paths changes the training normalizer.
//! Frozen coefficients and the factual selected-source branch have no gradient.
//! Empty local prefixes use constant IDENTITY and receive no fabricated credit.
//!
//! ContextQ4Output contains no token IDs or encoder digest. The enclosing caller
//! must bind its actual token IDs, reset-from-identity replay and current encoder
//! artifact. CPU admission also verifies exact latent tensor values against
//! native roots. CUDA admission checks shapes/devices and the authoritative
//! native trace without downloading dynamic tensors; the fresh device producer
//! and enclosing replay bind live values. Neither path proves arbitrary graphs
//! encoded the caller's token IDs merely because their roots agree.

use candle_core::{DType, Device, Tensor};
use uor_r4_integer::{
    geometric_no_read::CANONICAL_BASIS_Q25,
    geometric_potential_q4::unpack_coefficients,
    geometric_prefix_transport::{
        NativePrefixTransport, PrefixScoreMode, PrefixState, PrefixTransportTrace,
    },
    h4_tables::{H4Code, HistoricalH4Tables, ROOT_COUNT, TRUSTED_MATHEMATICAL_SHA256},
};

use crate::{geometric_context::ContextQ4Output, invalid, sha256_bytes, Result};

pub const SURROGATE: &str = "frozen-prefix-end-Q4-tables;actual-retained-POSTSTATE-choice-logits120;full-local-conditional120-exact-table-counterfactuals;existing-choice-pullback;other-endpoint-and-prior-state-factual;earlier-recurrence-ambient4-remains;subtraction-first-zero-forward;no-observed-root-substitution;identity-empty-no-credit;factual-selected-source-stop-gradient;nat-units/2";
pub const TEMPORAL_UTILITY_SURROGATE: &str = "opt-in-frozen-prefix-Q4;actual-hard-retained-state-onehot120;full120-exact-conditional-utility;context-owned-single-action-pullback-and-temporal-carry;no-local-softmax;subtraction-first-zero-forward;identity-empty-no-credit;nat-units/1";
pub const LS_CONTROL_SURROGATE: &str = "frozen-prefix-end-Q4-tables;120-counterfactuals-projected-to-four-coordinate-least-squares;higher-harmonic-nullspace;explicit-control-not-primary/1";
#[derive(Clone, Copy)]
enum CreditMode {
    FullLocalChoices,
    FullTemporalUtilities,
    AmbientLeastSquaresControl,
}
const Q24: f64 = 16_777_216.;
const Q25: f64 = 33_554_432.;
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");

pub struct PrefixStateCreditOutput {
    pub scores: Tensor,
    pub scores_q24: Vec<Vec<i64>>,
}

fn code(root: u8) -> Result<H4Code> {
    H4Code::try_from(root).map_err(|e| invalid(e.to_string()))
}
fn basis(root: u8) -> Result<[f64; 4]> {
    code(root)?;
    Ok(CANONICAL_BASIS_Q25[usize::from(root)].map(|v| f64::from(v) / Q25))
}
fn geometry(binding: &str) -> Result<HistoricalH4Tables> {
    if TRUSTED_MATHEMATICAL_SHA256 != Some(binding) {
        return Err(invalid("transport state credit algebra binding differs"));
    }
    HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))
}

fn admit_context(context: &ContextQ4Output, time: usize, heads: usize, lanes: usize) -> Result<()> {
    let trace = &context.trace;
    let width = heads * lanes;
    if time == 0
        || time > 128
        || trace.batch != 1
        || trace.time != time
        || trace.heads != heads
        || trace.lanes_per_head != lanes
        || trace.states.len() != time
        || trace.states.iter().any(|s| s.len() != width)
        || context.latent_roots.dims() != [1, time, heads, lanes, 4]
        || context.state_logits.dims() != [1, time, heads, lanes, ROOT_COUNT]
        || context.state_logits.dtype() != DType::F32
        || !context
            .state_logits
            .device()
            .same_device(context.latent_roots.device())
        || context.latent_roots.dtype() != DType::F32
        || (!context.latent_roots.device().is_cpu() && !context.latent_roots.device().is_cuda())
    {
        return Err(invalid(
            "transport state credit local context shape differs",
        ));
    }
    // CUDA producer/caller binds the fresh native trace; do not extract live
    // tensors to host for admission. CPU reference additionally checks values.
    if !context.latent_roots.device().is_cpu() {
        return Ok(());
    }
    let choices = context.state_logits.flatten_all()?.to_vec1::<f32>()?;
    if choices.iter().any(|x| !x.is_finite()) {
        return Err(invalid("transport retained state choice logits nonfinite"));
    }
    // Offline float logits may choose differently from quantized native scores.
    // Native roots, not the surrogate float argmax, remain authoritative.
    let values = context.latent_roots.flatten_all()?.to_vec1::<f32>()?;
    for (row, roots) in trace.states.iter().enumerate() {
        for (lane, &root) in roots.iter().enumerate() {
            for (axis, expected) in basis(root)?.iter().enumerate() {
                if values[(row * width + lane) * 4 + axis].to_bits() != (*expected as f32).to_bits()
                {
                    return Err(invalid(
                        "transport state credit graph root differs from native trace",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn admit_temporal_choices(context: &ContextQ4Output, mode: CreditMode) -> Result<()> {
    if !matches!(mode, CreditMode::FullTemporalUtilities) {
        return Ok(());
    }
    let trace = &context.trace;
    let shape = [1, trace.time, trace.heads, trace.lanes_per_head, ROOT_COUNT];
    if context.state_choices.dims() != shape
        || context.state_choices.dtype() != DType::F32
        || !context
            .state_choices
            .device()
            .same_device(context.latent_roots.device())
    {
        return Err(invalid(
            "transport temporal state utility channel shape/device differs",
        ));
    }
    if context.state_choices.device().is_cpu() {
        let values = context.state_choices.flatten_all()?.to_vec1::<f32>()?;
        let width = trace.heads * trace.lanes_per_head;
        for (time, roots) in trace.states.iter().enumerate() {
            for (lane, &selected) in roots.iter().enumerate() {
                code(selected)?;
                for state in 0..ROOT_COUNT {
                    let expected = if state == usize::from(selected) {
                        1f32
                    } else {
                        0f32
                    };
                    if values[(time * width + lane) * ROOT_COUNT + state].to_bits()
                        != expected.to_bits()
                    {
                        return Err(invalid(
                            "transport temporal onehot differs from native retained state",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

struct Endpoint {
    device: Device,
    roots: Vec<u8>,
    live: Option<Tensor>,
    choices: Option<Tensor>,
}
fn retained(context: &ContextQ4Output, time_index: usize) -> Result<Tensor> {
    let width = context.trace.heads * context.trace.lanes_per_head;
    Ok(context
        .latent_roots
        .narrow(1, time_index, 1)?
        .reshape((width, 4))?
        .contiguous()?)
}
fn retained_choices(
    context: &ContextQ4Output,
    time_index: usize,
    mode: CreditMode,
) -> Result<Tensor> {
    let width = context.trace.heads * context.trace.lanes_per_head;
    let choices = if matches!(mode, CreditMode::FullTemporalUtilities) {
        &context.state_choices
    } else {
        &context.state_logits
    };
    Ok(choices
        .narrow(1, time_index, 1)?
        .reshape((width, ROOT_COUNT))?
        .contiguous()?)
}
fn response(
    packet: &PrefixState,
    context: Option<&ContextQ4Output>,
    heads: usize,
    lanes: usize,
    device: &Device,
    mode: CreditMode,
) -> Result<Endpoint> {
    let width = heads * lanes;
    if packet.states.len() != width {
        return Err(invalid("transport response state width differs"));
    }
    if packet.token_ids.is_empty() {
        if context.is_some() || packet.states.iter().any(|&r| r != H4Code::IDENTITY.index()) {
            return Err(invalid(
                "empty transport response must be constant IDENTITY without context graph",
            ));
        }
        return Ok(Endpoint {
            device: device.clone(),
            roots: packet.states.clone(),
            live: None,
            choices: None,
        });
    }
    let context = context.ok_or_else(|| invalid("nonempty transport response graph absent"))?;
    admit_context(context, packet.token_ids.len(), heads, lanes)?;
    admit_temporal_choices(context, mode)?;
    let at = packet.token_ids.len() - 1;
    if context.trace.states[at] != packet.states {
        return Err(invalid("transport response retained state differs"));
    }
    if !context.latent_roots.device().same_device(device) {
        return Err(invalid("transport endpoint device differs"));
    }
    Ok(Endpoint {
        device: device.clone(),
        roots: packet.states.clone(),
        live: Some(retained(context, at)?),
        choices: Some(retained_choices(context, at, mode)?),
    })
}

fn index(
    geometry: &HistoricalH4Tables,
    directed: bool,
    response: u8,
    source: u8,
) -> Result<(u8, u8)> {
    let relative = geometry.relative(code(response)?, code(source)?).index();
    Ok((if directed { relative } else { source }, relative))
}

/// Least-squares finite-choice extension: g minimizes sum_r
/// (g·(basis(r)-basis(selected)) - (f(r)-f(selected)))².
/// Anchoring eliminates constant-table credit, while full ambient coordinates
/// retain radial/antipodal information for the existing finite-choice context.
fn ambient_gradient(
    table: &[i8],
    geometry: &HistoricalH4Tables,
    directed: bool,
    response: u8,
    source: u8,
    response_endpoint: bool,
) -> Result<[f32; 4]> {
    if table.len() != ROOT_COUNT {
        return Err(invalid("transport credit table width differs"));
    }
    if response_endpoint && !directed {
        return Ok([0.; 4]);
    }
    let selected = if response_endpoint { response } else { source };
    let anchor = basis(selected)?;
    let selected_index = index(geometry, directed, response, source)?.0;
    let f0 = f64::from(table[usize::from(selected_index)]) * 0.25;
    let mut system = [[0f64; 5]; 4];
    for alternative in 0..ROOT_COUNT {
        let alt = alternative as u8;
        let (q, k) = if response_endpoint {
            (alt, source)
        } else {
            (response, alt)
        };
        let f = f64::from(table[usize::from(index(geometry, directed, q, k)?.0)]) * 0.25;
        let point = basis(alt)?;
        let dx = std::array::from_fn::<_, 4, _>(|axis| point[axis] - anchor[axis]);
        for row in 0..4 {
            for col in 0..4 {
                system[row][col] += dx[row] * dx[col];
            }
            system[row][4] += dx[row] * (f - f0);
        }
    }
    for col in 0..4 {
        let mut pivot = col;
        for row in col + 1..4 {
            if system[row][col].abs() > system[pivot][col].abs() {
                pivot = row;
            }
        }
        if !system[pivot][col].is_finite() || system[pivot][col].abs() < 1e-12 {
            return Err(invalid("transport root interpolation is singular"));
        }
        system.swap(col, pivot);
        let divisor = system[col][col];
        for entry in col..5 {
            system[col][entry] /= divisor;
        }
        for row in 0..4 {
            if row == col {
                continue;
            }
            let multiplier = system[row][col];
            for entry in col..5 {
                system[row][entry] -= multiplier * system[col][entry];
            }
        }
    }
    let mut result = [0f32; 4];
    for axis in 0..4 {
        let value = system[axis][4];
        if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
            return Err(invalid("transport state adjoint is nonfinite"));
        }
        result[axis] = value as f32;
    }
    Ok(result)
}
fn attach(live: &Option<Tensor>, lane: usize, gradient: [f32; 4]) -> Result<Tensor> {
    let Some(live) = live else {
        return Ok(Tensor::new(0f32, &Device::Cpu)?);
    };
    let value = live.narrow(0, lane, 1)?.reshape(4)?;
    // Subtract first: every live term is exactly zero in the forward path.
    Ok(((value.clone() - value.detach())?
        * Tensor::from_vec(gradient.to_vec(), 4, value.device())?)?
    .sum_all()?)
}
fn attach_choice(
    live: &Option<Tensor>,
    lane: usize,
    table: &[i8],
    geometry: &HistoricalH4Tables,
    directed: bool,
    response: u8,
    source: u8,
    response_endpoint: bool,
    temporal: bool,
) -> Result<Tensor> {
    let Some(live) = live else {
        return Ok(Tensor::new(0f32, &Device::Cpu)?);
    };
    let value = live.narrow(0, lane, 1)?.reshape(ROOT_COUNT)?;

    let mut credit = Vec::with_capacity(ROOT_COUNT);
    for alt in 0..ROOT_COUNT {
        let (q, k) = if response_endpoint {
            (alt as u8, source)
        } else {
            (response, alt as u8)
        };
        let bin = index(geometry, directed, q, k)?.0;
        credit.push(f64::from(table[usize::from(bin)]) * 0.25);
    }
    // All 120 scores retained; centering avoids constant-table numerical credit.
    let anchor = credit[0];
    for x in &mut credit {
        *x -= anchor;
    }
    if temporal {
        let utility = Tensor::from_vec(
            credit.iter().map(|&x| x as f32).collect::<Vec<_>>(),
            ROOT_COUNT,
            value.device(),
        )?;
        return Ok(((value.clone() - value.detach())? * utility)?.sum_all()?);
    }
    if value.device().is_cuda() {
        let utility = Tensor::from_vec(
            credit.iter().map(|&x| x as f32).collect::<Vec<_>>(),
            ROOT_COUNT,
            value.device(),
        )?;
        let expected = (candle_nn::ops::softmax(&value, 0)? * utility)?.sum_all()?;
        return Ok((&expected - expected.detach())?);
    }
    let logits = value.to_vec1::<f32>()?;
    let mut gradient = vec![0f64; ROOT_COUNT];
    crate::geometric_context_credit::choice_pullback(&logits, &credit, &mut gradient);
    if gradient
        .iter()
        .any(|x| !x.is_finite() || x.abs() > f64::from(f32::MAX))
    {
        return Err(invalid("transport choice adjoint nonfinite"));
    }
    let gradient = gradient.into_iter().map(|x| x as f32).collect::<Vec<_>>();
    Ok(((value.clone() - value.detach())?
        * Tensor::from_vec(gradient, ROOT_COUNT, value.device())?)?
    .sum_all()?)
}
fn endpoint_credit(
    mode: CreditMode,
    endpoint: &Endpoint,
    lane: usize,
    table: &[i8],
    geometry: &HistoricalH4Tables,
    directed: bool,
    response: u8,
    source: u8,
    is_response: bool,
) -> Result<Tensor> {
    if endpoint.choices.is_none() && endpoint.live.is_none() {
        return Ok(Tensor::new(0f32, &endpoint.device)?);
    }
    match mode {
        CreditMode::FullLocalChoices | CreditMode::FullTemporalUtilities => attach_choice(
            &endpoint.choices,
            lane,
            table,
            geometry,
            directed,
            response,
            source,
            is_response,
            matches!(mode, CreditMode::FullTemporalUtilities),
        ),
        CreditMode::AmbientLeastSquaresControl => attach(
            &endpoint.live,
            lane,
            ambient_gradient(table, geometry, directed, response, source, is_response)?,
        ),
    }
}
fn score(
    table: &[i8],
    geometry: &HistoricalH4Tables,
    directed: bool,
    response: &Endpoint,
    source: &Endpoint,
    head: usize,
    lanes: usize,
    mode: CreditMode,
) -> Result<(i64, Tensor)> {
    let mut hard = 0i64;
    let mut credit = Tensor::new(0f32, &source.device)?;
    for lane in 0..lanes {
        let global = head * lanes + lane;
        let row = &table[global * ROOT_COUNT..(global + 1) * ROOT_COUNT];
        let q = response.roots[global];
        let k = source.roots[global];
        let bin = index(geometry, directed, q, k)?.0;
        hard = hard
            .checked_add(i64::from(row[usize::from(bin)]) << 22)
            .ok_or_else(|| invalid("transport native score sum overflow"))?;
        credit = (credit
            + endpoint_credit(mode, source, global, row, geometry, directed, q, k, false)?)?;
        if directed {
            credit = (credit
                + endpoint_credit(mode, response, global, row, geometry, true, q, k, true)?)?;
        }
    }
    Ok((
        hard,
        (Tensor::new((hard as f64 / Q24) as f32, &source.device)? + credit)?,
    ))
}

/// Output H×C in nat units; caller supplies the full local source encodes.
/// Candidate offset j uses post-state j-1, never token j or a future token.
pub fn frozen_prefix_state_forward(
    native: &NativePrefixTransport<'_>,
    trace: &PrefixTransportTrace,
    sources: &[ContextQ4Output],
    response_context: Option<&ContextQ4Output>,
) -> Result<PrefixStateCreditOutput> {
    frozen_prefix_state_forward_mode(
        native,
        trace,
        sources,
        response_context,
        CreditMode::FullLocalChoices,
    )
}

/// Explicit opt-in full120 temporal utility input credit. Native forward is unchanged.
/// Utilities attach to hard retained state_choices; ContextOp owns the single
/// choice pullback and temporal carry. No local-logit credit is also attached.
pub fn frozen_prefix_state_forward_temporal_utility(
    native: &NativePrefixTransport<'_>,
    trace: &PrefixTransportTrace,
    sources: &[ContextQ4Output],
    response_context: Option<&ContextQ4Output>,
) -> Result<PrefixStateCreditOutput> {
    frozen_prefix_state_forward_mode(
        native,
        trace,
        sources,
        response_context,
        CreditMode::FullTemporalUtilities,
    )
}

/// Explicit biased ambient control; never the default transport credit.

fn frozen_prefix_state_forward_mode(
    native: &NativePrefixTransport<'_>,
    trace: &PrefixTransportTrace,
    sources: &[ContextQ4Output],
    response_context: Option<&ContextQ4Output>,
    mode: CreditMode,
) -> Result<PrefixStateCreditOutput> {
    let metadata = native.metadata();
    let c = metadata.potential;
    let count = trace.candidate_source_indices.len();
    let width = c.heads * c.lanes_per_head;
    if count > 128
        || sources.len() > 128
        || trace.metadata != *metadata
        || sha256_bytes(native.packed_coefficients()) != metadata.potential_packed_sha256
        || sources.len() != trace.sources.len()
        || trace.candidate_offsets.len() != count
        || trace.copy_q24.len() != c.heads
        || trace.copy_q24.iter().any(|r| r.len() != count)
        || trace.angular_indices.len() != width
        || trace.relative_roots.len() != width
        || trace
            .angular_indices
            .iter()
            .chain(&trace.relative_roots)
            .any(|r| r.len() != count)
    {
        return Err(invalid(
            "prefix state credit metadata/payload/shape differs",
        ));
    }
    let geometry = geometry(&metadata.algebra_sha256)?;
    let table = unpack_coefficients(
        c.coefficient_count().map_err(|e| invalid(e.to_string()))?,
        native.packed_coefficients(),
    )
    .map_err(|e| invalid(e.to_string()))?;
    let directed = c.mode == PrefixScoreMode::DirectedRelative;
    let fallback_device = Device::Cpu;
    let device = sources
        .first()
        .map(|s| s.latent_roots.device())
        .or_else(|| response_context.map(|s| s.latent_roots.device()))
        .unwrap_or(&fallback_device);
    if sources
        .iter()
        .any(|s| !s.latent_roots.device().same_device(device))
    {
        return Err(invalid("transport source graph devices differ"));
    }
    let response = response(
        &trace.response,
        response_context,
        c.heads,
        c.lanes_per_head,
        device,
        mode,
    )?;
    for (packet, context) in trace.sources.iter().zip(sources) {
        admit_context(context, packet.token_ids.len(), c.heads, c.lanes_per_head)?;
        admit_temporal_choices(context, mode)?;
        if packet.states_before.len() != packet.token_ids.len()
            || packet.states_before.iter().any(|r| r.len() != width)
        {
            return Err(invalid("prefix state before-candidate shape differs"));
        }
        for (offset, states) in packet.states_before.iter().enumerate() {
            let expected = if offset == 0 {
                vec![H4Code::IDENTITY.index(); width]
            } else {
                context.trace.states[offset - 1].clone()
            };
            if states != &expected {
                return Err(invalid("prefix state before-candidate root differs"));
            }
        }
    }
    let mut rows = Vec::with_capacity(c.heads);
    for h in 0..c.heads {
        let mut values = Vec::with_capacity(count);
        for candidate in 0..count {
            let source_index = trace.candidate_source_indices[candidate];
            let offset = trace.candidate_offsets[candidate];
            let packet = trace
                .sources
                .get(source_index)
                .ok_or_else(|| invalid("prefix candidate source ordinal absent"))?;
            let roots = packet
                .states_before
                .get(offset)
                .ok_or_else(|| invalid("prefix candidate offset absent"))?
                .clone();
            let live = if offset == 0 {
                None
            } else {
                Some(retained(&sources[source_index], offset - 1)?)
            };
            let choices = if offset == 0 {
                None
            } else {
                Some(retained_choices(&sources[source_index], offset - 1, mode)?)
            };
            let source = Endpoint {
                device: device.clone(),
                roots,
                live,
                choices,
            };
            for lane in 0..c.lanes_per_head {
                let global = h * c.lanes_per_head + lane;
                let (bin, relative) = index(
                    &geometry,
                    directed,
                    response.roots[global],
                    source.roots[global],
                )?;
                if trace.angular_indices[global][candidate] != bin
                    || trace.relative_roots[global][candidate] != relative
                {
                    return Err(invalid(
                        "prefix state credit native angular address differs",
                    ));
                }
            }
            let (hard, value) = score(
                &table,
                &geometry,
                directed,
                &response,
                &source,
                h,
                c.lanes_per_head,
                mode,
            )?;
            if hard != trace.copy_q24[h][candidate] {
                return Err(invalid("prefix state credit native hard score differs"));
            }
            values.push(value);
        }
        rows.push(if count == 0 {
            Tensor::zeros(0, DType::F32, device)?
        } else {
            Tensor::stack(&values, 0)?
        });
    }
    Ok(PrefixStateCreditOutput {
        scores: Tensor::stack(&rows, 0)?,
        scores_q24: trace.copy_q24.clone(),
    })
}

/// Frozen selected-source branch: gradients reach that source's full retained
/// end and the actual response prefix, never the discrete route decision.
/// Caller binds factual bank occurrence -> source ordinal mapping; this trace
/// only permits validating the earliest maximum Copy bank occurrence itself.

/// Explicit biased ambient control; never the default transport credit.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context::NativeContextTrace;
    use candle_core::Var;
    use uor_r4_integer::{
        geometric_context_q4::{ContextQ4Config, NativeContextQ4},
        geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, CueScoreMode, NativeCueCarrier},
        geometric_potential::AddressLane,
        geometric_potential_q4::pack_coefficients,
        geometric_prefix_transport::{
            PrefixAngularConfig, PrefixAngularQ4, PrefixTransportCosts, SourcePrefixTrace,
        },
        geometric_source_realizer::{ArtifactIdentity, NativeArtifactBinding},
    };

    // These are explicit arithmetic graph fixtures, not a token-encoding or
    // language-learning claim. Integration exercises actual ContextWeights.
    fn live(roots: &[u8]) -> Result<(Var, ContextQ4Output)> {
        let time = roots.len();
        let values = roots
            .iter()
            .map(|&r| basis(r))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .map(|x| x as f32)
            .collect::<Vec<_>>();
        let latent = Tensor::from_vec(values, (1, time, 1, 1, 4), &Device::Cpu)?;
        let variable = Var::zeros((1, time, 1, 1, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let codes = roots
            .iter()
            .map(|&r| AddressLane::new(r, 0, true).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let output = ContextQ4Output {
            latent_roots: latent,
            state_logits: variable.as_tensor().clone(),
            // This fixture consumes only the legacy logit-credit interface.
            state_choices: Tensor::zeros((1, time, 1, 1, ROOT_COUNT), DType::F32, &Device::Cpu)?,
            root_logits: Tensor::zeros((1, time, 1, 1, 120), DType::F32, &Device::Cpu)?,
            category_logits: Tensor::zeros((1, time, 1, 1, 33), DType::F32, &Device::Cpu)?,
            trace: NativeContextTrace {
                batch: 1,
                time,
                heads: 1,
                lanes_per_head: 1,
                states: roots.iter().map(|&r| vec![r]).collect(),
                actions: vec![vec![1]; time],
                emitted_roots: roots.to_vec(),
                categories: vec![1; time],
                codes,
                coefficient_reads: 0,
            },
        };
        Ok((variable, output))
    }
    fn components() -> Result<(NativeContextQ4, HistoricalH4Tables, NativeArtifactBinding)> {
        let config = ContextQ4Config {
            vocab_size: 8,
            heads: 1,
            lanes_per_head: 1,
        };
        let context = NativeContextQ4::new(
            config,
            &vec![
                0;
                config
                    .coefficient_count()
                    .map_err(|e| invalid(e.to_string()))?
                    / 2
            ],
        )
        .map_err(|e| invalid(e.to_string()))?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let parent = NativeArtifactBinding {
            metadata_sha256: "a".repeat(64),
            identity: ArtifactIdentity {
                tokenizer_sha256: "b".repeat(64),
                parent_checkpoint_manifest_sha256: "c".repeat(64),
                parent_model_sha256: "d".repeat(64),
                parent_config_sha256: "e".repeat(64),
            },
        };
        Ok((context, geometry, parent))
    }
    fn packed(axis: usize) -> Result<Vec<u8>> {
        let values = (0..ROOT_COUNT)
            .map(|r| basis(r as u8).map(|v| (v[axis] * 7.).round() as i8))
            .collect::<Result<Vec<_>>>()?;
        pack_coefficients(&values).map_err(|e| invalid(e.to_string()))
    }
    fn prefix_trace(
        native: &NativePrefixTransport<'_>,
        geometry: &HistoricalH4Tables,
        directed: bool,
        response: u8,
        source: u8,
        empty_response: bool,
    ) -> Result<PrefixTransportTrace> {
        let q = unpack_coefficients(ROOT_COUNT, native.packed_coefficients())
            .map_err(|e| invalid(e.to_string()))?;
        let mut bins = Vec::new();
        let mut relative = Vec::new();
        let mut scores = Vec::new();
        for k in [1, source] {
            let (bin, rel) = index(geometry, directed, response, k)?;
            bins.push(bin);
            relative.push(rel);
            scores.push(i64::from(q[usize::from(bin)]) << 22);
        }
        Ok(PrefixTransportTrace {
            metadata: native.metadata().clone(),
            response: PrefixState {
                token_ids: if empty_response { vec![] } else { vec![6] },
                states: vec![response],
            },
            sources: vec![SourcePrefixTrace {
                source_segment_index: 3,
                token_ids: vec![4, 5],
                states_before: vec![vec![1], vec![source]],
            }],
            candidate_source_indices: vec![0, 0],
            candidate_offsets: vec![0, 1],
            angular_indices: vec![bins],
            relative_roots: vec![relative],
            copy_q24: vec![scores],
            costs: PrefixTransportCosts::default(),
        })
    }
    fn norm(values: &[f32]) -> f32 {
        values.iter().map(|v| v.abs()).sum()
    }

    fn temporal_prefix_actual_context(device: &Device, reset: bool) -> Result<Vec<f32>> {
        use crate::geometric_context::ContextWeights;
        let cpu = ContextWeights::new(4, 4, 1, 241)?.into_q4()?;
        for variable in cpu.parameters().values() {
            variable.set(&Tensor::zeros(variable.shape(), DType::F32, &Device::Cpu)?)?;
        }
        let transition = &cpu.parameters()["token_transition"];
        let mut values = vec![0f32; 4 * ROOT_COUNT];
        for token in 1..4 {
            values[token * ROOT_COUNT + 1] = 1.;
        }
        transition.set(&Tensor::from_vec(values, transition.shape(), &Device::Cpu)?)?;
        let codec = NativeContextQ4::new(
            ContextQ4Config {
                vocab_size: 4,
                heads: 1,
                lanes_per_head: 1,
            },
            &cpu.packed_coefficients()?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let (_, _, parent) = components()?;
        let cue = NativeCueCarrier::compile(
            parent.clone(),
            &codec,
            &geometry,
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: CueScoreMode::DirectedRelative,
                },
                &vec![0; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        // Even utility has zero four-coordinate moment but nonzero120 content.
        let table = (0..ROOT_COUNT)
            .map(|r| basis(r as u8).map(|v| if v[1].abs() > 0.75 { 4i8 } else { 0i8 }))
            .collect::<Result<Vec<_>>>()?;
        let native = NativePrefixTransport::compile(
            parent,
            &codec,
            &geometry,
            cue.metadata().clone(),
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &pack_coefficients(&table).map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let weights = cpu.to_device(device)?;
        let source = weights.forward_q4(&[2, 3], 1, 2, false)?;
        let response = weights.forward_q4(&[0, 1], 1, 2, reset)?;
        let mut trace = prefix_trace(
            &native,
            &geometry,
            true,
            response.trace.states[1][0],
            source.trace.states[0][0],
            false,
        )?;
        trace.response.token_ids = vec![0, 1];
        trace.sources[0].token_ids = vec![2, 3];
        let sources = [source];
        let old = frozen_prefix_state_forward(&native, &trace, &sources, Some(&response))?;
        let new = frozen_prefix_state_forward_temporal_utility(
            &native,
            &trace,
            &sources,
            Some(&response),
        )?;
        assert_eq!(old.scores_q24, new.scores_q24);
        assert_eq!(new.scores_q24, trace.copy_q24);
        assert_eq!(old.scores.to_vec2::<f32>()?, new.scores.to_vec2::<f32>()?);
        let variable = &weights.parameters()["token_transition"];
        let gradients = new.scores.narrow(1, 1, 1)?.sum_all()?.backward()?;
        let actual = gradients
            .get(variable.as_tensor())
            .ok_or_else(|| invalid("temporal prefix transition gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        if reset {
            assert!(actual[..ROOT_COUNT].iter().all(|&v| v == 0.));
        } else {
            assert!(norm(&actual[..ROOT_COUNT]) > 1e-3);
            let roots = (0..ROOT_COUNT)
                .map(|r| basis(r as u8))
                .collect::<Result<Vec<_>>>()?;
            for axis in 0..4 {
                let moment = (0..ROOT_COUNT)
                    .map(|r| f64::from(actual[r]) * roots[r][axis])
                    .sum::<f64>();
                assert!(
                    moment.abs() < 1e-6,
                    "even utility moment axis {axis}: {moment}"
                );
            }
            let gradients = old.scores.narrow(1, 1, 1)?.sum_all()?.backward()?;
            let legacy = gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid("local prefix transition gradient absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(legacy[..ROOT_COUNT].iter().all(|&v| v == 0.));
        }
        assert!(norm(&actual[ROOT_COUNT..2 * ROOT_COUNT]) > 1e-3);
        // Offset1 excludes the following Source token; it receives no credit.
        assert!(actual[3 * ROOT_COUNT..].iter().all(|&v| v == 0.));
        let mut empty_trace = prefix_trace(
            &native,
            &geometry,
            true,
            H4Code::IDENTITY.index(),
            sources[0].trace.states[0][0],
            true,
        )?;
        empty_trace.sources[0].token_ids = vec![2, 3];
        let empty =
            frozen_prefix_state_forward_temporal_utility(&native, &empty_trace, &sources, None)?;
        assert_eq!(empty.scores_q24, empty_trace.copy_q24);
        let empty_gradients = empty.scores.narrow(1, 1, 1)?.sum_all()?.backward()?;
        let empty_values = empty_gradients
            .get(variable.as_tensor())
            .ok_or_else(|| invalid("empty-prefix source gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(empty_values[..2 * ROOT_COUNT].iter().all(|&v| v == 0.));
        assert!(empty_values[3 * ROOT_COUNT..].iter().all(|&v| v == 0.));
        Ok(actual)
    }

    #[test]
    fn prefix_temporal_utility_actual_context_preserves_native_and_carries_even_history(
    ) -> Result<()> {
        temporal_prefix_actual_context(&Device::Cpu, false)?;
        temporal_prefix_actual_context(&Device::Cpu, true)?;
        let (_, mut malformed) = live(&[1, 1])?;
        assert!(admit_temporal_choices(&malformed, CreditMode::FullTemporalUtilities).is_err());
        // Existing controls deliberately do not require the opt-in channel.
        admit_temporal_choices(&malformed, CreditMode::FullLocalChoices)?;
        let mut onehot = vec![0f32; 2 * ROOT_COUNT];
        onehot[1] = 1.;
        onehot[ROOT_COUNT + 1] = 1.;
        malformed.state_choices = Tensor::from_vec(onehot, (1, 2, 1, 1, ROOT_COUNT), &Device::Cpu)?;
        admit_temporal_choices(&malformed, CreditMode::FullTemporalUtilities)?;
        malformed.state_choices = Tensor::zeros((1, 2, 1, 1, 119), DType::F32, &Device::Cpu)?;
        assert!(admit_temporal_choices(&malformed, CreditMode::FullTemporalUtilities).is_err());
        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn prefix_temporal_utility_actual_context_cuda_parity_requires_gpu() -> Result<()> {
        // No skip-as-PASS: invoking this CUDA test requires a real device.
        let device = Device::new_cuda(0)?;
        for reset in [false, true] {
            let cpu = temporal_prefix_actual_context(&Device::Cpu, reset)?;
            let cuda = temporal_prefix_actual_context(&device, reset)?;
            for (a, b) in cpu.iter().zip(cuda) {
                assert!((a - b).abs() < 1e-5, "CPU {a}, CUDA {b}");
            }
        }
        Ok(())
    }

    #[test]
    fn full_local_choices_preserve_table_modes_discarded_by_ambient_control() -> Result<()> {
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let identity = H4Code::IDENTITY.index();
        let mut pairs = Vec::new();
        for r in 0..ROOT_COUNT {
            let b = CANONICAL_BASIS_Q25[r];
            let opposite = CANONICAL_BASIS_Q25
                .iter()
                .position(|x| *x == b.map(|v| -v))
                .ok_or_else(|| invalid("canonical root antipode absent"))?;
            if r < opposite && r != usize::from(identity) && opposite != usize::from(identity) {
                pairs.push((r, opposite));
            }
        }
        let mut table = vec![0i8; ROOT_COUNT];
        for (pair, value) in [(pairs[0], 1), (pairs[1], -1)] {
            table[pair.0] = value;
            table[pair.1] = value;
        }
        // Even antipodal, zero-mean table mode: exactly no ambient linear moment.
        assert!(
            norm(&ambient_gradient(
                &table, &geometry, false, identity, identity, false
            )?) < 1e-7
        );
        let latent = Var::from_vec(
            basis(identity)?.map(|x| x as f32).to_vec(),
            (1, 4),
            &Device::Cpu,
        )?;
        let choices = Var::zeros((1, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let source = Endpoint {
            device: Device::Cpu,
            roots: vec![identity],
            live: Some(latent.as_tensor().clone()),
            choices: Some(choices.as_tensor().clone()),
        };
        let response = Endpoint {
            device: Device::Cpu,
            roots: vec![identity],
            live: None,
            choices: None,
        };
        let (hard, full) = score(
            &table,
            &geometry,
            false,
            &response,
            &source,
            0,
            1,
            CreditMode::FullLocalChoices,
        )?;
        let (ls_hard, ls) = score(
            &table,
            &geometry,
            false,
            &response,
            &source,
            0,
            1,
            CreditMode::AmbientLeastSquaresControl,
        )?;
        assert_eq!(hard, ls_hard);
        assert_eq!(full.to_scalar::<f32>()?, ls.to_scalar::<f32>()?);
        let gradients = full.backward()?;
        let gradient = gradients
            .get(choices.as_tensor())
            .ok_or_else(|| invalid("full local choice disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(norm(&gradient) > 1e-5);
        for (actual, &coefficient) in gradient.iter().zip(&table) {
            assert!(
                (f64::from(*actual) - f64::from(coefficient) * 0.25 / ROOT_COUNT as f64).abs()
                    < 1e-8
            );
        }
        assert!(gradients.get(latent.as_tensor()).is_none());
        let ls_gradients = ls.backward()?;
        let ambient = ls_gradients
            .get(latent.as_tensor())
            .ok_or_else(|| invalid("LS control disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(norm(&ambient) < 1e-7);
        assert!(ls_gradients.get(choices.as_tensor()).is_none());
        Ok(())
    }
    #[test]
    fn frozen_prefix_directed_credit_reaches_both_endpoints_not_future_token() -> Result<()> {
        let (context, geometry, parent) = components()?;
        let cue_config = CueAngularConfig {
            heads: 1,
            lanes_per_head: 1,
            mode: CueScoreMode::DirectedRelative,
        };
        let cue = NativeCueCarrier::compile(
            parent.clone(),
            &context,
            &geometry,
            CueAngularQ4::new(cue_config, &vec![0; 60]).map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let native = NativePrefixTransport::compile(
            parent,
            &context,
            &geometry,
            cue.metadata().clone(),
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &packed(0)?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let trace = prefix_trace(&native, &geometry, true, 20, 10, false)?;
        let (source_var, source) = live(&[10, 30])?;
        let (response_var, response) = live(&[20])?;
        let output = frozen_prefix_state_forward(&native, &trace, &[source], Some(&response))?;
        assert_eq!(output.scores_q24, trace.copy_q24);
        assert_eq!(
            output.scores.to_vec2::<f32>()?,
            trace
                .copy_q24
                .iter()
                .map(|r| r
                    .iter()
                    .map(|&v| (v as f64 / Q24) as f32)
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
        // Only the second candidate is scored for this backward assertion.
        let gradients = output.scores.narrow(1, 1, 1)?.sum_all()?.backward()?;
        let source_gradient = gradients
            .get(source_var.as_tensor())
            .ok_or_else(|| invalid("source latent state disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let response_gradient = gradients
            .get(response_var.as_tensor())
            .ok_or_else(|| invalid("response latent state disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(norm(&source_gradient[..ROOT_COUNT]) > 1e-5);
        assert_eq!(&source_gradient[ROOT_COUNT..], &[0.; ROOT_COUNT]);
        assert!(norm(&response_gradient) > 1e-5);
        let table = unpack_coefficients(ROOT_COUNT, native.packed_coefficients())
            .map_err(|e| invalid(e.to_string()))?;
        for (actual, is_response) in [
            (&source_gradient[..ROOT_COUNT], false),
            (&response_gradient[..], true),
        ] {
            let scores = (0..ROOT_COUNT)
                .map(|r| {
                    let (q, k) = if is_response {
                        (r as u8, 10)
                    } else {
                        (20, r as u8)
                    };
                    index(&geometry, true, q, k)
                        .map(|(bin, _)| f64::from(table[usize::from(bin)]) * 0.25)
                })
                .collect::<Result<Vec<_>>>()?;
            let mean = scores.iter().sum::<f64>() / ROOT_COUNT as f64;
            for (&value, expected) in actual.iter().zip(scores) {
                assert!((f64::from(value) - (expected - mean) / ROOT_COUNT as f64).abs() < 1e-7);
            }
        }

        Ok(())
    }
    #[test]
    fn frozen_prefix_unary_has_zero_response_credit_and_empty_identity_no_fake_credit() -> Result<()>
    {
        let (context, geometry, parent) = components()?;
        let cue = NativeCueCarrier::compile(
            parent.clone(),
            &context,
            &geometry,
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: CueScoreMode::DirectedRelative,
                },
                &vec![0; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let native = NativePrefixTransport::compile(
            parent,
            &context,
            &geometry,
            cue.metadata().clone(),
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: PrefixScoreMode::SourcePrefixUnary,
                },
                &packed(0)?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let trace = prefix_trace(&native, &geometry, false, 20, 10, false)?;
        let (source_var, source) = live(&[10, 30])?;
        let (response_var, response) = live(&[20])?;
        let output = frozen_prefix_state_forward(&native, &trace, &[source], Some(&response))?;
        let gradients = output.scores.sum_all()?.backward()?;
        assert!(gradients.get(response_var.as_tensor()).is_none());
        assert!(
            norm(
                &gradients
                    .get(source_var.as_tensor())
                    .ok_or_else(|| invalid("unary source latent state disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?
            ) > 1e-5
        );
        let mut empty = prefix_trace(&native, &geometry, false, 1, 10, true)?;
        empty.candidate_source_indices.truncate(1);
        empty.candidate_offsets.truncate(1);
        empty.angular_indices[0].truncate(1);
        empty.relative_roots[0].truncate(1);
        empty.copy_q24[0].truncate(1);
        let (unused_var, source) = live(&[10, 30])?;
        let output = frozen_prefix_state_forward(&native, &empty, &[source], None)?;
        let gradients = output.scores.sum_all()?.backward()?;
        assert!(gradients.get(unused_var.as_tensor()).is_none());
        let (_, source) = live(&[10, 30])?;
        assert!(frozen_prefix_state_forward(&native, &empty, &[source], Some(&response)).is_err());
        Ok(())
    }
    #[test]
    fn frozen_prefix_rejects_stale_payload_scores_and_wrong_before_candidate_root() -> Result<()> {
        let (context, geometry, parent) = components()?;
        let cue = NativeCueCarrier::compile(
            parent.clone(),
            &context,
            &geometry,
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: CueScoreMode::DirectedRelative,
                },
                &vec![0; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let native = NativePrefixTransport::compile(
            parent,
            &context,
            &geometry,
            cue.metadata().clone(),
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &packed(0)?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let trace = prefix_trace(&native, &geometry, true, 20, 10, false)?;
        for mutation in 0..3 {
            let mut bad = trace.clone();
            match mutation {
                0 => bad.metadata.potential_packed_sha256 = "0".repeat(64),
                1 => bad.copy_q24[0][1] += 1,
                _ => bad.sources[0].states_before[1][0] = 30,
            }
            let (_, source) = live(&[10, 30])?;
            let (_, response) = live(&[20])?;
            assert!(
                frozen_prefix_state_forward(&native, &bad, &[source], Some(&response)).is_err()
            );
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "cuda"))]
mod cuda_transport_parity {
    use super::*;
    use candle_core::Var;
    #[test]
    fn cuda_full120_transport_credit_matches_cpu_both_endpoints() -> Result<()> {
        // Explicit CUDA qualification test: missing CUDA returns an error.
        let device = Device::new_cuda(0)?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let table = (0..ROOT_COUNT)
            .map(|r| (r % 15) as i8 - 7)
            .collect::<Vec<_>>();
        let values = (0..ROOT_COUNT)
            .map(|r| ((r % 13) as f32 - 6.) * 0.09)
            .collect::<Vec<_>>();
        for directed in [false, true] {
            let cpuq = Var::from_vec(values.clone(), (1, ROOT_COUNT), &Device::Cpu)?;
            let cpuk = Var::from_vec(values.clone(), (1, ROOT_COUNT), &Device::Cpu)?;
            let gpuq = Var::from_vec(values.clone(), (1, ROOT_COUNT), &device)?;
            let gpuk = Var::from_vec(values.clone(), (1, ROOT_COUNT), &device)?;
            let endpoint = |root: u8, variable: &Var, device: &Device| Endpoint {
                device: device.clone(),
                roots: vec![root],
                live: None,
                choices: Some(variable.as_tensor().clone()),
            };
            let (_, cpu) = score(
                &table,
                &geometry,
                directed,
                &endpoint(20, &cpuq, &Device::Cpu),
                &endpoint(10, &cpuk, &Device::Cpu),
                0,
                1,
                CreditMode::FullLocalChoices,
            )?;
            let (_, gpu) = score(
                &table,
                &geometry,
                directed,
                &endpoint(20, &gpuq, &device),
                &endpoint(10, &gpuk, &device),
                0,
                1,
                CreditMode::FullLocalChoices,
            )?;
            assert_eq!(cpu.to_scalar::<f32>()?, gpu.to_scalar::<f32>()?);
            let cg = cpu.backward()?;
            let gg = gpu.backward()?;
            for (cv, gv, required) in [(&cpuk, &gpuk, true), (&cpuq, &gpuq, directed)] {
                if !required {
                    assert!(cg.get(cv.as_tensor()).is_none());
                    assert!(gg.get(gv.as_tensor()).is_none());
                    continue;
                }
                let c = cg
                    .get(cv.as_tensor())
                    .ok_or_else(|| invalid("CPU transport parity gradient disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let g = gg
                    .get(gv.as_tensor())
                    .ok_or_else(|| invalid("CUDA transport parity gradient disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                assert_eq!(c.len(), g.len());
                for (&a, &b) in c.iter().zip(&g) {
                    assert!(b.is_finite());
                    assert!((a - b).abs() < 2e-5 + 2e-5 * a.abs());
                }
            }
        }
        // Constant-identity endpoints never introduce an artificial graph.
        let fixed = Endpoint {
            device: device.clone(),
            roots: vec![1],
            live: None,
            choices: None,
        };
        let (_, value) = score(
            &table,
            &geometry,
            true,
            &fixed,
            &fixed,
            0,
            1,
            CreditMode::FullLocalChoices,
        )?;
        assert!(value.device().same_device(&device));
        Ok(())
    }
}
