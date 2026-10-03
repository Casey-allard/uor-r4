//! Offline capture credit through the retained native geometric reader.
//!
//! Native events/OLD-held codes and current residual age determine every hard
//! score, packet and reduced output. Frozen consumer coefficients supply only
//! input adjoints. The inherited span tangent/Boolean-stop policy is unchanged.
//! Real-softmax input credit ignores integer normalization/rounding derivatives.
use crate::geometric_composition::CompositionWeights;
use crate::geometric_composition_native::{
    CompiledComposition, ComposedReadRow, ComposedReadTrace,
};
use crate::geometric_context::NativeContextTrace;
use crate::geometric_context_credit::NoReadCreditOutput;
use crate::geometric_event::EventQ4Output;
use crate::geometric_no_read::NoReadWeights;
use crate::geometric_potential_q4::{PotentialQ4Output, PotentialQ4Weights};
use crate::geometric_read_native::CompiledGeometricRead;
use crate::geometric_span_native::NativeSpanTrace;
use crate::geometric_value_producer::{ValueProducerOutput, ValueProducerWeights};
use crate::{invalid, Result};
use candle_core::{CpuStorage, CustomOp1, DType, Device, Layout, Shape, Tensor};
use std::collections::BTreeMap;
use uor_r4_integer::{
    geometric_composed_read::NativeGeometricComposedRead,
    geometric_no_read::CANONICAL_BASIS_Q25,
    geometric_potential::AddressLane,
    geometric_potential_q4::{self, NativePotentialQ4, PotentialQ4Config, FAMILY_COUNTS},
    geometric_value_q4::{NativeValueQ4, ValueQ4Config},
    h4_tables::{H4Code, HistoricalH4Tables},
};

pub const POLICY: &str = "current-native-event-actions/OLD-held;frozen-q4-value-NoRead-potential/H4-bank-input-credit;span-tangent/validity-stop;native-current-age-and-read-plus-(real-softmax-surrogate-minus-detached-surrogate);same-row-angular-zero;no-frozen-parameter-gradient/1";
const Q24: f64 = 16_777_216.;
const Q25: f64 = 33_554_432.;
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");

pub struct EventAgeCreditOutput {
    pub event: EventQ4Output,
    pub context: NativeContextTrace,
    pub span: NativeSpanTrace,
    pub values: ValueProducerOutput,
    pub potential: PotentialQ4Output,
    pub no_read: NoReadCreditOutput,
    pub read: ComposedReadTrace,
    pub age_q24: Vec<i64>,
    pub policy: &'static str,
}
#[derive(Clone, Copy)]
struct Dims {
    batch: usize,
    time: usize,
    heads: usize,
    lanes: usize,
}
impl Dims {
    fn rows(self) -> usize {
        self.batch * self.time
    }
    fn width(self) -> usize {
        self.heads * self.lanes
    }
    fn count(self) -> usize {
        self.rows() * self.width()
    }
    fn at(self, b: usize, t: usize, h: usize, l: usize) -> usize {
        ((b * self.time + t) * self.heads + h) * self.lanes + l
    }
}
fn code(v: u8) -> Result<H4Code> {
    H4Code::try_from(v).map_err(|e| invalid(e.to_string()))
}
fn root(v: u8) -> [f64; 4] {
    CANONICAL_BASIS_Q25[usize::from(v)].map(|x| f64::from(x) / Q25)
}
fn finite(t: &Tensor, shape: &[usize]) -> Result<Vec<f32>> {
    if t.dims() != shape || t.dtype() != DType::F32 || !t.device().is_cpu() {
        return Err(invalid("held credit tensor shape/type/device differs"));
    }
    let v = t.flatten_all()?.to_vec1::<f32>()?;
    if v.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonfinite held credit tensor"));
    }
    Ok(v)
}
struct Inputs {
    d: Dims,
    states: Vec<H4Code>,
    held: Vec<H4Code>,
    valid: Vec<bool>,
    content: Vec<AddressLane>,
}
fn admit(
    ids: &[u32],
    ctx: &NativeContextTrace,
    span: &NativeSpanTrace,
    held: &Tensor,
) -> Result<Inputs> {
    let d = Dims {
        batch: ctx.batch,
        time: ctx.time,
        heads: ctx.heads,
        lanes: ctx.lanes_per_head,
    };
    if d.batch == 0
        || d.batch > 32
        || d.time == 0
        || d.time > 128
        || d.heads != 2
        || d.lanes != 4
        || ids.len() != d.rows()
        || ctx.states.len() != d.rows()
        || ctx.codes.len() != d.count()
        || ctx.emitted_roots.len() != d.count()
        || ctx.categories.len() != d.count()
        || span.batch != d.batch
        || span.time != d.time
        || span.lanes != d.width()
        || span.prior_codes.len() != d.rows()
    {
        return Err(invalid("held credit trace dimensions differ"));
    }
    let actual = finite(held, &[d.batch, d.time, d.heads, d.lanes, 4])?;
    let mut states = Vec::with_capacity(d.count());
    let mut codes = Vec::with_capacity(d.count());
    let mut content = Vec::with_capacity(d.count());
    let mut valid = Vec::with_capacity(d.rows());
    for row in 0..d.rows() {
        if ctx.states[row].len() != d.width()
            || span.prior_codes[row]
                .as_ref()
                .is_some_and(|x| x.len() != d.width())
        {
            return Err(invalid("held credit lane width differs"));
        }
        let present = span.prior_codes[row].is_some();
        valid.push(present);
        for l in 0..d.width() {
            let at = row * d.width() + l;
            states.push(code(ctx.states[row][l])?);
            let raw = ctx.emitted_roots[at];
            code(raw)?;
            let cat = ctx.categories[at];
            if cat > 32
                || ctx.codes[at]
                    != AddressLane::new(
                        if cat == 0 { 1 } else { raw },
                        if cat == 0 { 0 } else { cat - 1 },
                        cat != 0,
                    )
                    .map_err(|e| invalid(e.to_string()))?
            {
                return Err(invalid("context observation record differs"));
            }
            let r = span.prior_codes[row].as_ref().map(|x| x[l]).unwrap_or(1);
            codes.push(code(r)?);
            content.push(
                AddressLane::new(r, if present { 16 } else { 0 }, present)
                    .map_err(|e| invalid(e.to_string()))?,
            );
            let expected = if present { root(r) } else { [0.; 4] };
            if (0..4).any(|i| actual[at * 4 + i].to_bits() != (expected[i] as f32).to_bits()) {
                return Err(invalid(
                    "connected held tensor differs from authoritative OLD native span",
                ));
            }
        }
    }
    Ok(Inputs {
        d,
        states,
        held: codes,
        valid,
        content,
    })
}

fn frozen_value(
    weights: &ValueProducerWeights,
    ids: &[u32],
    ctx: &NativeContextTrace,
    held: &Tensor,
    a: &Inputs,
) -> Result<ValueProducerOutput> {
    let d = a.d;
    let c = weights.config();
    if !weights.is_q4()
        || c.heads != d.heads
        || c.latent_lanes_per_head != d.lanes
        || ids.iter().any(|&x| x as usize >= c.vocab_size)
    {
        return Err(invalid("held value config differs"));
    }
    let native = NativeValueQ4::new(
        ValueQ4Config {
            vocab_size: c.vocab_size,
            heads: d.heads,
            latent_lanes_per_head: d.lanes,
        },
        &weights.packed_coefficients()?,
    )
    .and_then(|x| x.into_native())
    .map_err(|e| invalid(e.to_string()))?;
    let slots = d.heads * 8;
    let mut roots = Vec::new();
    let mut cats = Vec::new();
    for row in 0..d.rows() {
        let first = row * d.width();
        let v = native
            .produce(
                ids[row] as usize,
                &a.states[first..first + d.width()],
                if a.valid[row] {
                    Some(&a.held[first..first + d.width()])
                } else {
                    None
                },
                true,
            )
            .map_err(|e| invalid(e.to_string()))?;
        roots.extend_from_slice(&v.root_choices[..slots]);
        cats.extend_from_slice(&v.categories[..slots]);
    }
    let fixed = weights
        .parameters()
        .iter()
        .map(|(name, var)| {
            Ok((
                name.clone(),
                Tensor::from_vec(
                    var.flatten_all()?
                        .to_vec1::<f32>()?
                        .into_iter()
                        .map(|x| (x * 4.).round() * 0.25)
                        .collect::<Vec<_>>(),
                    var.shape(),
                    &Device::Cpu,
                )?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let get = |n: &str| {
        fixed
            .get(n)
            .ok_or_else(|| invalid("frozen held value factor absent"))
    };
    let latent = Tensor::from_vec(
        ctx.states
            .iter()
            .flatten()
            .flat_map(|&r| root(r).map(|x| x as f32))
            .collect::<Vec<_>>(),
        (d.batch, d.time, d.heads, d.lanes, 4),
        &Device::Cpu,
    )?;
    let valid = Tensor::from_vec(
        a.valid
            .iter()
            .map(|&x| if x { 1f32 } else { 0. })
            .collect::<Vec<_>>(),
        (d.rows(), 1),
        &Device::Cpu,
    )?;
    let token_ids = Tensor::from_vec(ids.to_vec(), d.rows(), &Device::Cpu)?;
    let mut logits = Vec::new();
    for (family, classes) in [("root", 120), ("category", 32)] {
        let tokens = get(&format!("token_{family}"))?
            .reshape((c.vocab_size, slots * classes))?
            .index_select(&token_ids, 0)?;
        let mut pieces = Vec::new();
        for h in 0..d.heads {
            for lane in 0..4 {
                let own = lane % d.lanes;
                let feature = |t: &Tensor, l| -> Result<Tensor> {
                    Ok(t.narrow(2, h, 1)?.narrow(3, l, 1)?.reshape((d.rows(), 4))?)
                };
                let own_feature = feature(&latent, own)?;
                let neighbor = feature(&latent, (own + 1) % d.lanes)?;
                let held_feature = feature(held, own)?.broadcast_mul(&valid)?;
                for atom in 0..2 {
                    let slot = (h * 4 + lane) * 2 + atom;
                    let mut score = tokens.narrow(1, slot * classes, classes)?;
                    for (factor, input) in [
                        ("own", &own_feature),
                        ("neighbor", &neighbor),
                        ("span", &held_feature),
                    ] {
                        let basis = get(&format!("{factor}_{family}"))?
                            .reshape((slots, classes, 4))?
                            .narrow(0, slot, 1)?
                            .squeeze(0)?;
                        score = (&score + input.matmul(&basis.t()?)?)?;
                    }
                    let bias = get(&format!("span_valid_{family}"))?
                        .reshape((slots, classes))?
                        .narrow(0, slot, 1)?;
                    pieces.push((&score + valid.broadcast_mul(&bias)?)?);
                }
            }
        }
        logits.push(
            Tensor::cat(&pieces, 1)?
                .reshape(vec![d.batch, d.time, d.heads, 4, 2, classes])?
                .contiguous()?,
        );
    }
    weights.emit_q4_native_choices(
        &logits[0],
        &logits[1],
        d.batch,
        d.time,
        &vec![true; d.rows()],
        roots,
        cats,
    )
}
fn mul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}
fn conj(a: [f64; 4]) -> [f64; 4] {
    [a[0], -a[1], -a[2], -a[3]]
}
// Fixed per-output derivatives into OLD-held coordinates. SpanOp supplies its
// inherited tangent and selected-branch recurrence after these contributions sum.
struct HeldLinear {
    shape: Vec<usize>,
    input_count: usize,
    hard: Vec<i64>,
    edges: Vec<(usize, usize, [f64; 4])>,
}
impl CustomOp1 for HeldLinear {
    fn name(&self) -> &'static str {
        "frozen-native-held-input-credit"
    }
    fn cpu_fwd(&self, s: &CpuStorage, l: &Layout) -> candle_core::Result<(CpuStorage, Shape)> {
        let (lo, hi) = l
            .contiguous_offsets()
            .ok_or_else(|| candle_core::Error::Msg("held input must be contiguous".into()))?;
        let v = &s.as_slice::<f32>()?[lo..hi];
        if v.len() != self.input_count || v.iter().any(|x| !x.is_finite()) {
            candle_core::bail!("held input shape/finite mismatch");
        }
        Ok((
            CpuStorage::F32(self.hard.iter().map(|&x| (x as f64 / Q24) as f32).collect()),
            Shape::from(self.shape.clone()),
        ))
    }
    fn bwd(&self, arg: &Tensor, _: &Tensor, g: &Tensor) -> candle_core::Result<Option<Tensor>> {
        let upstream = g.flatten_all()?.to_vec1::<f32>()?;
        if upstream.len() != self.hard.len() || upstream.iter().any(|x| !x.is_finite()) {
            candle_core::bail!("held upstream shape/finite mismatch");
        }
        let mut out = vec![0f64; self.input_count];
        for &(row, at, derivative) in &self.edges {
            for i in 0..4 {
                out[at * 4 + i] += f64::from(upstream[row]) * derivative[i];
            }
        }
        if out
            .iter()
            .any(|x| !x.is_finite() || x.abs() > f64::from(f32::MAX))
        {
            candle_core::bail!("held adjoint overflow");
        }
        Ok(Some(Tensor::from_vec(
            out.into_iter().map(|x| x as f32).collect::<Vec<_>>(),
            arg.shape(),
            arg.device(),
        )?))
    }
}
fn frozen_null(
    weights: &NoReadWeights,
    ids: &[u32],
    ctx: &NativeContextTrace,
    held: &Tensor,
    a: &Inputs,
) -> Result<NoReadCreditOutput> {
    let d = a.d;
    let c = weights.config();
    if c.heads != d.heads || c.latent_lanes_per_head != d.lanes {
        return Err(invalid("held NoRead config differs"));
    }
    let native = weights.native()?;
    let coeff = weights.parameters()["coefficients"]
        .flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(|x| f64::from((x * 4.).round() * 0.25))
        .collect::<Vec<_>>();
    let mut hard = vec![0; d.rows() * d.heads];
    let mut edges = Vec::new();
    let stride = 1 + c.vocabulary + d.width() * 42;
    for row in 0..d.rows() {
        let first = row * d.width();
        let z = native
            .score(
                ids[row] as usize,
                &a.states[first..first + d.width()],
                &ctx.codes[first..first + d.width()],
                if a.valid[row] {
                    Some(&a.held[first..first + d.width()])
                } else {
                    None
                },
            )
            .map_err(|e| invalid(e.to_string()))?;
        for h in 0..d.heads {
            let index = ((row / d.time) * d.heads + h) * d.time + row % d.time;
            hard[index] = z[h];
            if a.valid[row] {
                for lane in 0..d.width() {
                    let start = h * stride + 1 + c.vocabulary + d.width() * 37 + lane * 4;
                    edges.push((
                        index,
                        first + lane,
                        std::array::from_fn(|axis| coeff[start + axis]),
                    ));
                }
            }
        }
    }
    let scores = held.contiguous()?.apply_op1(HeldLinear {
        shape: vec![d.batch, d.heads, d.time],
        input_count: d.count() * 4,
        hard: hard.clone(),
        edges,
    })?;
    Ok(NoReadCreditOutput {
        scores,
        scores_q24: hard,
    })
}
fn frozen_potential(
    weights: &PotentialQ4Weights,
    ctx: &NativeContextTrace,
    held: &Tensor,
    a: &Inputs,
) -> Result<PotentialQ4Output> {
    let d = a.d;
    let cfg = PotentialQ4Config {
        heads: d.heads,
        lanes_per_head: d.lanes,
    };
    if *weights.config() != cfg {
        return Err(invalid("held potential config differs"));
    }
    frozen_potential_packed(cfg, &weights.packed_coefficients()?, ctx, held, a)
}
fn frozen_potential_packed(
    cfg: PotentialQ4Config,
    packed: &[u8],
    ctx: &NativeContextTrace,
    held: &Tensor,
    a: &Inputs,
) -> Result<PotentialQ4Output> {
    let d = a.d;
    let codec = NativePotentialQ4::new(cfg, packed).map_err(|e| invalid(e.to_string()))?;
    let coeff = geometric_potential_q4::unpack_coefficients(
        cfg.coefficient_count()
            .map_err(|e| invalid(e.to_string()))?,
        packed,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let algebra = HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
    let coefficient = |f: usize, l: usize, i: usize| {
        f64::from(
            coeff[FAMILY_COUNTS[..f].iter().sum::<usize>() * d.width() + l * FAMILY_COUNTS[f] + i],
        ) * 0.25
    };
    let mut hard = vec![0; d.batch * d.heads * d.time * d.time];
    let mut edges = Vec::new();
    for b in 0..d.batch {
        for h in 0..d.heads {
            for q in 0..d.time {
                for k in 0..=q {
                    let qi = d.at(b, q, h, 0);
                    let ki = d.at(b, k, h, 0);
                    let index = ((b * d.heads + h) * d.time + q) * d.time + k;
                    hard[index] = codec
                        .score(
                            h,
                            &a.content[qi..qi + d.lanes],
                            &a.content[ki..ki + d.lanes],
                            &ctx.codes[qi..qi + d.lanes],
                            &ctx.codes[ki..ki + d.lanes],
                            &algebra,
                        )
                        .map_err(|e| invalid(e.to_string()))?;
                    // Only same OCCURRENCE aliases have constant angular self-relative.
                    if q == k || !a.valid[b * d.time + q] || !a.valid[b * d.time + k] {
                        continue;
                    }
                    for l in 0..d.lanes {
                        let x = qi + l;
                        let y = ki + l;
                        let lane = h * d.lanes + l;
                        let mut v = std::array::from_fn(|i| coefficient(0, lane, i));
                        if ctx.codes[x].present() && ctx.codes[y].present() {
                            let dr = root(
                                algebra
                                    .relative(
                                        code(ctx.codes[x].root())?,
                                        code(ctx.codes[y].root())?,
                                    )
                                    .index(),
                            );
                            for i in 0..4 {
                                for j in 0..4 {
                                    v[i] += coefficient(2, lane, i * 4 + j) * dr[j];
                                }
                            }
                        }
                        edges.push((index, x, mul(root(a.held[y].index()), conj(v))));
                        edges.push((index, y, mul(root(a.held[x].index()), v)));
                    }
                }
            }
        }
    }
    let scores = held.contiguous()?.apply_op1(HeldLinear {
        shape: vec![d.batch, d.heads, d.time, d.time],
        input_count: d.count() * 4,
        hard: hard.clone(),
        edges,
    })?;
    Ok(PotentialQ4Output {
        scores,
        scores_q24: hard,
        content_codes: a.content.clone(),
        context_codes: ctx.codes.clone(),
    })
}

/// Actual current native hard read plus a zero-forward real-softmax input
/// surrogate. Return the checked native trace, never reconstruct raw Q16 from F32.
#[allow(clippy::too_many_arguments)]
pub(crate) fn forward(
    ids: &[u32],
    ctx: NativeContextTrace,
    event: EventQ4Output,
    span: NativeSpanTrace,
    held: &Tensor,
    age_shadow: &Tensor,
    value_weights: &ValueProducerWeights,
    null_weights: &NoReadWeights,
    potential_weights: &PotentialQ4Weights,
    bank: &CompositionWeights,
    composition: &CompiledComposition,
    reducer: &CompiledGeometricRead,
    routes: [bool; 4],
) -> Result<(Tensor, EventAgeCreditOutput)> {
    let a = admit(ids, &ctx, &span, held)?;
    let d = a.d;
    let mut values = frozen_value(value_weights, ids, &ctx, held, &a)?;
    let mut null = frozen_null(null_weights, ids, &ctx, held, &a)?;
    let mut potential = frozen_potential(potential_weights, &ctx, held, &a)?;
    if !routes[0] {
        values.values = values.values.detach();
    }
    if !routes[1] {
        null.scores = null.scores.detach();
    }
    if !routes[2] {
        potential.scores = potential.scores.detach();
    }
    let mut age = crate::geometric_read_native::q4_age_residual_training_view(
        age_shadow,
        d.heads,
        reducer.metadata().context,
    )?;
    let age_q24 = finite(&age, &[d.heads, reducer.metadata().context])?
        .iter()
        .map(|&x| (f64::from(x) * Q24) as i64)
        .collect::<Vec<_>>();
    if !routes[3] {
        age = age.detach();
    }
    let composed = bank.forward_frozen_values(&values.values, &values.trace)?;
    let raw_values = composition.compose_trace(&values.trace)?;
    let mut kernel =
        NativeGeometricComposedRead::new(reducer.metadata().context, reducer.exp_q31())
            .map_err(|e| invalid(e.to_string()))?;
    let mut rows = Vec::with_capacity(d.rows() * d.heads);
    let mut fixed_scores = vec![f32::NEG_INFINITY; d.batch * d.heads * d.time * d.time];
    for b in 0..d.batch {
        for h in 0..d.heads {
            let first = (b * d.heads + h) * d.time;
            for q in 0..d.time {
                let ag = (0..=q)
                    .map(|k| age_q24[h * reducer.metadata().context + q - k])
                    .collect::<Vec<_>>();
                let at = (first + q) * d.time;
                for (k, &v) in ag.iter().enumerate() {
                    let sum = potential.scores_q24[at + k]
                        .checked_add(v)
                        .ok_or_else(|| invalid("held score+age overflow"))?;
                    fixed_scores[at + k] = (sum as f64 / Q24) as f32;
                }
                let r = kernel
                    .reduce(
                        &potential.scores_q24[at..at + q + 1],
                        &ag,
                        null.scores_q24[first + q],
                        &raw_values[first * 32..(first + q + 1) * 32],
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                rows.push(ComposedReadRow {
                    output_q16: r.output_q16.to_vec(),
                    occurrence_weights_q31: r.occurrence_weights_q31.to_vec(),
                    no_read_weight_q31: r.no_read_weight_q31,
                    total_weight_q31: r.total_weight_q31,
                    max_score_q24: r.max_score_q24,
                });
            }
        }
    }
    let mut native = Vec::with_capacity(d.rows() * 32);
    for b in 0..d.batch {
        for t in 0..d.time {
            for i in 0..32 {
                let x = rows[(b * d.heads) * d.time + t].output_q16[i]
                    .checked_add(rows[(b * d.heads + 1) * d.time + t].output_q16[i])
                    .ok_or_else(|| invalid("held head sum overflow"))?;
                native.push((x as f64 / 65536.) as f32);
            }
        }
    }
    let hard = Tensor::from_vec(native, (d.batch, d.time, 32), &Device::Cpu)?;
    let lag = Tensor::from_vec(
        (0..d.time)
            .flat_map(|q| (0..d.time).map(move |k| q.saturating_sub(k) as u32))
            .collect::<Vec<_>>(),
        d.time * d.time,
        &Device::Cpu,
    )?;
    let age = age
        .index_select(&lag, 1)?
        .reshape((1, d.heads, d.time, d.time))?
        .broadcast_as((d.batch, d.heads, d.time, d.time))?;
    let live = (&potential.scores + &age)?;
    let scores = (&Tensor::from_vec(
        fixed_scores,
        (d.batch, d.heads, d.time, d.time),
        &Device::Cpu,
    )? + (&live - live.detach())?)?;
    let null_tensor = null.scores.unsqueeze(3)?;
    let joined = Tensor::cat(&[&null_tensor, &scores], 3)?;
    let zero = Tensor::zeros((d.batch, d.heads, 1, 32), DType::F32, &Device::Cpu)?;
    let surrogate = candle_nn::ops::softmax(&joined, 3)?
        .matmul(&Tensor::cat(&[&zero, &composed], 2)?)?
        .sum(1)?;
    finite(&surrogate, &[d.batch, d.time, 32])?;
    let residual = (&hard + (&surrogate - surrogate.detach())?)?;
    let read = ComposedReadTrace {
        batch: d.batch,
        heads: d.heads,
        time: d.time,
        value_width: 32,
        no_read_q24: null.scores_q24.clone(),
        values_q16: raw_values,
        rows,
    };
    Ok((
        residual,
        EventAgeCreditOutput {
            event,
            context: ctx,
            span,
            values,
            potential,
            no_read: null,
            read,
            age_q24,
            policy: POLICY,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Var;
    fn fixture(present: bool) -> Result<(NativeContextTrace, NativeSpanTrace, Var)> {
        let lane = AddressLane::new(1, 16, true).map_err(|e| invalid(e.to_string()))?;
        let ctx = NativeContextTrace {
            batch: 1,
            time: 2,
            heads: 2,
            lanes_per_head: 4,
            states: vec![vec![1; 8]; 2],
            actions: vec![vec![1; 8]; 2],
            emitted_roots: vec![1; 16],
            categories: vec![17; 16],
            codes: vec![lane; 16],
            coefficient_reads: 0,
        };
        let span = NativeSpanTrace {
            batch: 1,
            time: 2,
            lanes: 8,
            actions: vec![0; 2],
            prior_codes: vec![if present { Some(vec![2; 8]) } else { None }; 2],
        };
        let coordinates = (0..16)
            .flat_map(|_| {
                if present {
                    root(2).map(|x| x as f32)
                } else {
                    [0f32; 4]
                }
            })
            .collect::<Vec<_>>();
        Ok((
            ctx,
            span,
            Var::from_vec(coordinates, (1, 2, 2, 4, 4), &Device::Cpu)?,
        ))
    }
    #[test]
    fn held_credit_distinct_equal_codes_keep_endpoint_adjoint_but_self_and_future_do_not(
    ) -> Result<()> {
        let (ctx, span, held) = fixture(true)?;
        let a = admit(&[0, 0], &ctx, &span, held.as_tensor())?;
        let c = PotentialQ4Config {
            heads: 2,
            lanes_per_head: 4,
        };
        let mut q = vec![0i8; c.coefficient_count().map_err(|e| invalid(e.to_string()))?];
        for lane in 0..8 {
            q[lane * 4 + 2] = 1;
        }
        let packed =
            geometric_potential_q4::pack_coefficients(&q).map_err(|e| invalid(e.to_string()))?;
        let out = frozen_potential_packed(c, &packed, &ctx, held.as_tensor(), &a)?;
        let g = out.scores.flatten_all()?.get(2)?.backward()?;
        let d = g
            .get(held.as_tensor())
            .ok_or_else(|| invalid("held offdiagonal credit absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(d.iter().any(|x| *x != 0.));
        for i in 0..32 {
            assert_eq!(
                d[i] + d[32 + i],
                0.,
                "equal roots cancel only after both occurrence adjoints"
            );
        }
        for index in [0usize, 1, 3] {
            // self, future, self
            let g = out.scores.flatten_all()?.get(index)?.backward()?;
            assert!(g
                .get(held.as_tensor())
                .ok_or_else(|| invalid("zero held graph absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .all(|x| *x == 0.));
        }
        Ok(())
    }
    #[test]
    fn held_credit_noread_uses_all_lanes_and_absence_stops_input_credit() -> Result<()> {
        let weights = NoReadWeights::new(1, 2, 4)?;
        let p = &weights.parameters()["coefficients"];
        let mut v = vec![0f32; p.elem_count()];
        let stride = weights.config().coefficients_per_head();
        // Output head0 reads a held lane in head1; validity bias is deliberately nonzero.
        v[2 + 8 * 37 + 7 * 4 + 1] = 0.5;
        v[2 + 8 * 41 + 7] = 0.75;
        v[stride + 2 + 8 * 37 + 0] = 0.25;
        p.set(&Tensor::from_vec(v, p.shape(), &Device::Cpu)?)?;
        for present in [true, false] {
            let (ctx, span, held) = fixture(present)?;
            let a = admit(&[0, 0], &ctx, &span, held.as_tensor())?;
            let out = frozen_null(&weights, &[0, 0], &ctx, held.as_tensor(), &a)?;
            let g = out.scores.flatten_all()?.get(0)?.backward()?;
            let d = g
                .get(held.as_tensor())
                .ok_or_else(|| invalid("held null graph absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert_eq!(d[7 * 4 + 1], if present { 0.5 } else { 0. });
            assert_eq!(d.iter().filter(|x| **x != 0.).count(), usize::from(present));
            assert!(g.get(p.as_tensor()).is_none());
            if !present {
                assert_eq!(out.scores_q24, vec![0; 4]);
            }
        }
        Ok(())
    }
}
