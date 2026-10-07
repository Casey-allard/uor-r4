//! Offline input credit from frozen geometric consumers to a live q4 context.
//!
//! Hard state, observation and packet choices come from the same authoritative
//! native trace. Consumer coefficients are copied as fixed q/4 values, never
//! attached as trainable tensors. These are declared biased first-order input
//! surrogates, not derivatives of discrete argmax or the integer reducer.
//! Potential/NoRead scores use nat units; Q24/Q25/Q16 are reconstructed once at
//! their boundaries. The surrounding caller owns immutable artifact admission.

use std::collections::BTreeMap;
use std::sync::Arc;

use candle_core::{CpuStorage, CustomOp2, DType, Device, Layout, Shape, Tensor};
use uor_r4_integer::geometric_no_read::CANONICAL_BASIS_Q25;
use uor_r4_integer::geometric_potential::{AddressLane, NativePotentialTables, ENTRIES_PER_LANE};
use uor_r4_integer::geometric_potential_q4::{
    self, NativePotentialQ4, PotentialQ4Config, FAMILY_COUNTS,
};
use uor_r4_integer::geometric_value_q4::{NativeValueQ4, ValueQ4Config};
use uor_r4_integer::h4_tables::{H4Code, HistoricalH4Tables};

use crate::geometric_context::ContextQ4Output;
use crate::geometric_no_read::NoReadWeights;
use crate::geometric_potential_q4::{PotentialQ4Output, PotentialQ4Weights};
use crate::geometric_value_producer::{ValueProducerOutput, ValueProducerWeights};
use crate::{invalid, Result};

pub const SURROGATE: &str = "native-context-trace-conditioned;frozen-q4-consumer-inputs;potential-offdiagonal-ambient-directed-Hamilton/root120;self-angular-zero;category33-native-lane-score-counterfactual-both-self-roles;NoRead-all-lanes-root-linear/category33;value-native-choice-decoded-prototype-and-zero-anchor;nat-units/1";
const Q24: f64 = 16_777_216.;
const Q25: f64 = 33_554_432.;
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");

#[derive(Clone, Copy)]
struct Dimensions {
    batch: usize,
    time: usize,
    heads: usize,
    lanes: usize,
}
impl Dimensions {
    fn rows(self) -> usize {
        self.batch * self.time
    }
    fn width(self) -> usize {
        self.heads * self.lanes
    }
    fn count(self) -> usize {
        self.rows() * self.width()
    }
    fn index(self, b: usize, t: usize, h: usize, l: usize) -> usize {
        ((b * self.time + t) * self.heads + h) * self.lanes + l
    }
}
fn tensor_values(t: &Tensor, shape: &[usize]) -> Result<Vec<f32>> {
    if t.dims() != shape || t.dtype() != DType::F32 || !t.device().is_cpu() {
        return Err(invalid("context credit tensor shape/type/device differs"));
    }
    let v = t.flatten_all()?.to_vec1::<f32>()?;
    if v.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonfinite context credit tensor"));
    }
    Ok(v)
}
fn code(root: u8) -> Result<H4Code> {
    H4Code::try_from(root).map_err(|e| invalid(e.to_string()))
}
fn root(root: u8) -> [f64; 4] {
    CANONICAL_BASIS_Q25[usize::from(root)].map(|x| f64::from(x) / Q25)
}
fn observation(root: u8, category: usize) -> Result<AddressLane> {
    AddressLane::new(
        if category == 0 { 1 } else { root },
        if category == 0 {
            0
        } else {
            (category - 1) as u8
        },
        category != 0,
    )
    .map_err(|e| invalid(e.to_string()))
}
fn admit(context: &ContextQ4Output) -> Result<(Dimensions, Vec<H4Code>)> {
    let t = &context.trace;
    let d = Dimensions {
        batch: t.batch,
        time: t.time,
        heads: t.heads,
        lanes: t.lanes_per_head,
    };
    if d.batch == 0
        || d.time == 0
        || d.time > 128
        || !(1..=2).contains(&d.heads)
        || !(1..=4).contains(&d.lanes)
        || d.batch
            .checked_mul(d.time)
            .and_then(|v| v.checked_mul(d.heads))
            .and_then(|v| v.checked_mul(d.lanes))
            .and_then(|v| v.checked_mul(960))
            .is_none()
    {
        return Err(invalid("context credit dimensions differ"));
    }
    if t.states.len() != d.rows()
        || t.actions.len() != d.rows()
        || t.emitted_roots.len() != d.count()
        || t.categories.len() != d.count()
        || t.codes.len() != d.count()
    {
        return Err(invalid("context credit trace shape differs"));
    }
    let on_cpu = context.latent_roots.device().is_cpu();
    let device = context.latent_roots.device();
    for (tensor, shape) in [
        (
            &context.latent_roots,
            vec![d.batch, d.time, d.heads, d.lanes, 4],
        ),
        (
            &context.root_logits,
            vec![d.batch, d.time, d.heads, d.lanes, 120],
        ),
        (
            &context.category_logits,
            vec![d.batch, d.time, d.heads, d.lanes, 33],
        ),
    ] {
        if tensor.dims() != shape
            || tensor.dtype() != DType::F32
            || !tensor.device().same_device(device)
            || (!device.is_cpu() && !device.is_cuda())
        {
            return Err(invalid("context credit graph device/shape differs"));
        }
    }
    let latent = if on_cpu {
        tensor_values(
            &context.latent_roots,
            &[d.batch, d.time, d.heads, d.lanes, 4],
        )?
    } else {
        vec![]
    };
    if on_cpu {
        tensor_values(
            &context.root_logits,
            &[d.batch, d.time, d.heads, d.lanes, 120],
        )?;
        tensor_values(
            &context.category_logits,
            &[d.batch, d.time, d.heads, d.lanes, 33],
        )?;
    }
    let mut states = Vec::with_capacity(d.count());
    for (row, (s, a)) in t.states.iter().zip(&t.actions).enumerate() {
        if s.len() != d.width() || a.len() != d.width() {
            return Err(invalid("context credit state/action width differs"));
        }
        for (lane, (&s, &a)) in s.iter().zip(a).enumerate() {
            states.push(code(s)?);
            code(a)?;
            let at = row * d.width() + lane;
            for (axis, expected) in root(s).iter().enumerate() {
                if on_cpu && latent[at * 4 + axis].to_bits() != (*expected as f32).to_bits() {
                    return Err(invalid(
                        "context credit graph latent differs from native trace",
                    ));
                }
            }
            code(t.emitted_roots[at])?;
            if t.categories[at] > 32
                || t.codes[at] != observation(t.emitted_roots[at], usize::from(t.categories[at]))?
            {
                return Err(invalid(
                    "context credit raw observation differs from canonical native code",
                ));
            }
        }
    }
    Ok((d, states))
}
fn admit_tokens(
    ids: &[u32],
    d: Dimensions,
    vocabulary: usize,
    held: &[H4Code],
    span_valid: &[bool],
) -> Result<()> {
    if ids.len() != d.rows()
        || ids.iter().any(|&x| x as usize >= vocabulary)
        || held.len() != d.count()
        || span_valid.len() != d.rows()
    {
        return Err(invalid("context credit token/held/validity shape differs"));
    }
    Ok(())
}
fn probabilities(values: &[f32]) -> Vec<f64> {
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut p = values
        .iter()
        .map(|&x| (f64::from(x) - f64::from(max)).exp())
        .collect::<Vec<_>>();
    let sum: f64 = p.iter().sum();
    for x in &mut p {
        *x /= sum;
    }
    p
}
pub(crate) fn choice_pullback(logits: &[f32], credit: &[f64], out: &mut [f64]) {
    let p = probabilities(logits);
    let mean: f64 = p.iter().zip(credit).map(|(p, g)| p * g).sum();
    for ((out, p), g) in out.iter_mut().zip(p).zip(credit) {
        *out += p * (g - mean);
    }
}
fn checked(values: Vec<f64>) -> candle_core::Result<Vec<f32>> {
    if values
        .iter()
        .any(|x| !x.is_finite() || x.abs() > f64::from(f32::MAX))
    {
        candle_core::bail!("context credit overflow/nonfinite adjoint");
    }
    Ok(values.into_iter().map(|x| x as f32).collect())
}
fn contiguous<'a>(storage: &'a CpuStorage, layout: &Layout) -> candle_core::Result<&'a [f32]> {
    let (a, b) = layout.contiguous_offsets().ok_or_else(|| {
        candle_core::Error::Msg("context credit requires contiguous storage".into())
    })?;
    Ok(&storage.as_slice::<f32>()?[a..b])
}

/// Frozen factor tensors retain input gradients, with no source Var in graph.
/// Native packet choices, including category-zero raw roots, are authoritative.
pub fn frozen_value_forward(
    weights: &ValueProducerWeights,
    ids: &[u32],
    context: &ContextQ4Output,
    held: &[H4Code],
    span_valid: &[bool],
    occurrence_valid: &[bool],
) -> Result<ValueProducerOutput> {
    let (d, states) = admit(context)?;
    let c = weights.config();
    if !weights.is_q4()
        || c.heads != d.heads
        || c.latent_lanes_per_head != d.lanes
        || occurrence_valid.len() != d.rows()
    {
        return Err(invalid("frozen q4 value/context config differs"));
    }
    admit_tokens(ids, d, c.vocab_size, held, span_valid)?;
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
    let slots = d.heads * 4 * 2;
    let mut roots = Vec::with_capacity(d.rows() * slots);
    let mut categories = Vec::with_capacity(d.rows() * slots);
    for row in 0..d.rows() {
        let first = row * d.width();
        let value = native
            .produce(
                ids[row] as usize,
                &states[first..first + d.width()],
                if span_valid[row] {
                    Some(&held[first..first + d.width()])
                } else {
                    None
                },
                occurrence_valid[row],
            )
            .map_err(|e| invalid(e.to_string()))?;
        roots.extend_from_slice(&value.root_choices[..slots]);
        categories.extend_from_slice(&value.categories[..slots]);
    }
    let fixed = weights
        .parameters()
        .iter()
        .map(|(name, var)| {
            let values = var
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .map(|x| (x * 4.).round() * 0.25)
                .collect::<Vec<_>>();
            Ok((
                name.clone(),
                Tensor::from_vec(values, var.shape(), &Device::Cpu)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let get = |name: &str| {
        fixed
            .get(name)
            .ok_or_else(|| invalid("frozen value factor absent"))
    };
    let held = Tensor::from_vec(
        held.iter()
            .flat_map(|r| root(r.index()).map(|x| x as f32))
            .collect::<Vec<_>>(),
        (d.batch, d.time, d.heads, d.lanes, 4),
        &Device::Cpu,
    )?;
    let validity = Tensor::from_vec(
        span_valid
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
                let own_feature = feature(&context.latent_roots, own)?;
                let neighbor = if d.lanes > 1 {
                    Some(feature(&context.latent_roots, (own + 1) % d.lanes)?)
                } else {
                    None
                };
                let held_feature = feature(&held, own)?.broadcast_mul(&validity)?;
                for atom in 0..2 {
                    let slot = (h * 4 + lane) * 2 + atom;
                    let mut score = tokens.narrow(1, slot * classes, classes)?;
                    for (factor, input) in [
                        ("own", Some(&own_feature)),
                        ("neighbor", neighbor.as_ref()),
                        ("span", Some(&held_feature)),
                    ] {
                        if let Some(input) = input {
                            let basis = get(&format!("{factor}_{family}"))?
                                .reshape((slots, classes, 4))?
                                .narrow(0, slot, 1)?
                                .squeeze(0)?;
                            score = (&score + input.matmul(&basis.t()?)?)?;
                        }
                    }
                    let bias = get(&format!("span_valid_{family}"))?
                        .reshape((slots, classes))?
                        .narrow(0, slot, 1)?;
                    pieces.push((&score + validity.broadcast_mul(&bias)?)?);
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
        occurrence_valid,
        roots,
        categories,
    )
}

pub struct NoReadCreditOutput {
    pub scores: Tensor,
    pub scores_q24: Vec<i64>,
}
pub fn frozen_no_read_forward(
    weights: &NoReadWeights,
    ids: &[u32],
    context: &ContextQ4Output,
    held: &[H4Code],
    span_valid: &[bool],
) -> Result<NoReadCreditOutput> {
    let (d, states) = admit(context)?;
    no_read_credit_forward(
        weights,
        ids,
        d,
        &states,
        &context.trace.codes,
        &context.latent_roots,
        &context.category_logits,
        held,
        span_valid,
    )
}
fn no_read_credit_forward(
    weights: &NoReadWeights,
    ids: &[u32],
    d: Dimensions,
    states: &[H4Code],
    codes: &[AddressLane],
    latent_roots: &Tensor,
    category_logits: &Tensor,
    held: &[H4Code],
    span_valid: &[bool],
) -> Result<NoReadCreditOutput> {
    let c = *weights.config();
    if c.heads != d.heads || c.latent_lanes_per_head != d.lanes {
        return Err(invalid("frozen NoRead/context config differs"));
    }
    admit_tokens(ids, d, c.vocabulary, held, span_valid)?;
    let native = weights.native()?;
    let mut hard = vec![0i64; d.rows() * d.heads];
    for row in 0..d.rows() {
        let first = row * d.width();
        let z = native
            .score(
                ids[row] as usize,
                &states[first..first + d.width()],
                &codes[first..first + d.width()],
                if span_valid[row] {
                    Some(&held[first..first + d.width()])
                } else {
                    None
                },
            )
            .map_err(|e| invalid(e.to_string()))?;
        for h in 0..d.heads {
            hard[((row / d.time) * d.heads + h) * d.time + row % d.time] = z[h];
        }
    }
    let coefficients = weights
        .parameters()
        .get("coefficients")
        .ok_or_else(|| invalid("NoRead coefficients absent"))?
        .flatten_all()?
        .to_vec1::<f32>()?
        .into_iter()
        .map(|x| f64::from((x * 4.).round() * 0.25))
        .collect();
    let op = NoReadCredit {
        d,
        vocabulary: c.vocabulary,
        coefficients,
        hard: hard.clone(),
    };
    let scores = if latent_roots.device().is_cuda() {
        op.device_graph(latent_roots, category_logits)?
    } else {
        latent_roots
            .contiguous()?
            .apply_op2(&category_logits.contiguous()?, op)?
    };
    Ok(NoReadCreditOutput {
        scores,
        scores_q24: hard,
    })
}
struct NoReadCredit {
    d: Dimensions,
    vocabulary: usize,
    coefficients: Vec<f64>,
    hard: Vec<i64>,
}
impl NoReadCredit {
    fn backward(
        &self,
        categories: &[f32],
        upstream: &[f32],
    ) -> candle_core::Result<(Vec<f32>, Vec<f32>)> {
        let d = self.d;
        if categories.len() != d.count() * 33
            || upstream.len() != d.rows() * d.heads
            || categories.iter().chain(upstream).any(|x| !x.is_finite())
        {
            candle_core::bail!("NoRead input credit shape/finite mismatch");
        }
        let mut latent = vec![0.; d.count() * 4];
        let mut category = vec![0.; d.count() * 33];
        let width = 1 + self.vocabulary + d.width() * 42;
        for row in 0..d.rows() {
            for h in 0..d.heads {
                let g = f64::from(upstream[((row / d.time) * d.heads + h) * d.time + row % d.time]);
                let start = h * width + 1 + self.vocabulary;
                for lane in 0..d.width() {
                    let at = row * d.width() + lane;
                    for axis in 0..4 {
                        latent[at * 4 + axis] += g * self.coefficients[start + lane * 4 + axis];
                    }
                    let f = &self.coefficients[start + d.width() * 4 + lane * 33
                        ..start + d.width() * 4 + (lane + 1) * 33];
                    let credit = f.iter().map(|x| g * x).collect::<Vec<_>>();
                    choice_pullback(
                        &categories[at * 33..(at + 1) * 33],
                        &credit,
                        &mut category[at * 33..(at + 1) * 33],
                    );
                }
            }
        }
        Ok((checked(latent)?, checked(category)?))
    }
}
impl CustomOp2 for NoReadCredit {
    fn name(&self) -> &'static str {
        "frozen-q4-NoRead-context-input-credit"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let a = contiguous(a, la)?;
        let b = contiguous(b, lb)?;
        if a.len() != self.d.count() * 4
            || b.len() != self.d.count() * 33
            || a.iter().chain(b).any(|x| !x.is_finite())
        {
            candle_core::bail!("NoRead context graph input differs");
        }
        Ok((
            CpuStorage::F32(self.hard.iter().map(|&x| (x as f64 / Q24) as f32).collect()),
            Shape::from((self.d.batch, self.d.heads, self.d.time)),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        _: &Tensor,
        g: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (da, db) = self.backward(
            &b.flatten_all()?.to_vec1::<f32>()?,
            &g.flatten_all()?.to_vec1::<f32>()?,
        )?;
        Ok((
            Some(Tensor::from_vec(da, a.shape(), a.device())?),
            Some(Tensor::from_vec(db, b.shape(), b.device())?),
        ))
    }
}

/// Potential hard output is the unchanged integer scorer. Only its context
/// observations are differentiable here; content and all coefficients freeze.
pub fn frozen_potential_forward(
    weights: &PotentialQ4Weights,
    content: &[AddressLane],
    context: &ContextQ4Output,
) -> Result<PotentialQ4Output> {
    let (d, _) = admit(context)?;
    potential_credit_forward(
        weights,
        content,
        d,
        &context.trace.codes,
        &context.trace.emitted_roots,
        &context.root_logits,
        &context.category_logits,
    )
}
fn potential_credit_forward(
    weights: &PotentialQ4Weights,
    content: &[AddressLane],
    d: Dimensions,
    codes: &[AddressLane],
    raw_roots: &[u8],
    root_logits: &Tensor,
    category_logits: &Tensor,
) -> Result<PotentialQ4Output> {
    if weights.config().heads != d.heads
        || weights.config().lanes_per_head != d.lanes
        || content.len() != d.count()
    {
        return Err(invalid("frozen potential/context shape differs"));
    }
    let packed = weights.packed_coefficients()?;
    let op = PotentialCredit::new(
        d,
        content.to_vec(),
        codes.to_vec(),
        raw_roots.to_vec(),
        &packed,
    )?;
    let scores_q24 = op.hard.clone();
    let scores = if root_logits.device().is_cuda() {
        op.device_graph(root_logits, category_logits)?
    } else {
        root_logits
            .contiguous()?
            .apply_op2(&category_logits.contiguous()?, op)?
    };
    Ok(PotentialQ4Output {
        scores,
        scores_q24,
        content_codes: content.to_vec(),
        context_codes: codes.to_vec(),
    })
}
struct PotentialCredit {
    d: Dimensions,
    content: Vec<AddressLane>,
    context: Vec<AddressLane>,
    raw_roots: Vec<u8>,
    algebra: Arc<HistoricalH4Tables>,
    lanes: Vec<NativePotentialTables>,
    coefficients: Vec<i8>,
    hard: Vec<i64>,
}
fn hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}
fn conjugate(a: [f64; 4]) -> [f64; 4] {
    [a[0], -a[1], -a[2], -a[3]]
}
fn dot(a: [f64; 4], b: [f64; 4]) -> f64 {
    a.into_iter().zip(b).map(|(x, y)| x * y).sum()
}
impl PotentialCredit {
    fn new(
        d: Dimensions,
        content: Vec<AddressLane>,
        context: Vec<AddressLane>,
        raw_roots: Vec<u8>,
        packed: &[u8],
    ) -> Result<Self> {
        let c = PotentialQ4Config {
            heads: d.heads,
            lanes_per_head: d.lanes,
        };
        let codec = NativePotentialQ4::new(c, packed).map_err(|e| invalid(e.to_string()))?;
        let coefficients = geometric_potential_q4::unpack_coefficients(
            c.coefficient_count().map_err(|e| invalid(e.to_string()))?,
            packed,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let algebra =
            Arc::new(HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?);
        let lanes = codec
            .expanded_q24()
            .chunks_exact(ENTRIES_PER_LANE)
            .map(|row| NativePotentialTables::new(1, 1, row).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let mut hard = vec![0; d.batch * d.heads * d.time * d.time];
        for b in 0..d.batch {
            for h in 0..d.heads {
                for q in 0..d.time {
                    for k in 0..=q {
                        let qi = d.index(b, q, h, 0);
                        let ki = d.index(b, k, h, 0);
                        hard[((b * d.heads + h) * d.time + q) * d.time + k] = codec
                            .score(
                                h,
                                &content[qi..qi + d.lanes],
                                &content[ki..ki + d.lanes],
                                &context[qi..qi + d.lanes],
                                &context[ki..ki + d.lanes],
                                &algebra,
                            )
                            .map_err(|e| invalid(e.to_string()))?;
                    }
                }
            }
        }
        Ok(Self {
            d,
            content,
            context,
            raw_roots,
            algebra,
            lanes,
            coefficients,
            hard,
        })
    }
    fn coefficient(&self, family: usize, lane: usize, index: usize) -> f64 {
        let offset = FAMILY_COUNTS[..family].iter().sum::<usize>() * self.d.width()
            + lane * FAMILY_COUNTS[family]
            + index;
        f64::from(self.coefficients[offset]) * 0.25
    }
    fn lane_score(
        &self,
        lane: usize,
        qi: usize,
        ki: usize,
        rq: AddressLane,
        rk: AddressLane,
    ) -> Result<f64> {
        Ok(self.lanes[lane]
            .score(
                0,
                &[self.content[qi]],
                &[self.content[ki]],
                &[rq],
                &[rk],
                &self.algebra,
            )
            .map_err(|e| invalid(e.to_string()))? as f64
            / Q24)
    }
    fn backward(
        &self,
        roots: &[f32],
        categories: &[f32],
        upstream: &[f32],
    ) -> Result<(Vec<f32>, Vec<f32>)> {
        let d = self.d;
        if roots.len() != d.count() * 120
            || categories.len() != d.count() * 33
            || upstream.len() != self.hard.len()
            || roots
                .iter()
                .chain(categories)
                .chain(upstream)
                .any(|x| !x.is_finite())
        {
            return Err(invalid("potential context credit shape/finite mismatch"));
        }
        let mut ambient = vec![[0.; 4]; d.count()];
        let mut category_credit = vec![0.; d.count() * 33];
        for b in 0..d.batch {
            for h in 0..d.heads {
                for q in 0..d.time {
                    for k in 0..=q {
                        let g = f64::from(upstream[((b * d.heads + h) * d.time + q) * d.time + k]);
                        if g == 0. {
                            continue;
                        }
                        for l in 0..d.lanes {
                            let lane = h * d.lanes + l;
                            let qi = d.index(b, q, h, l);
                            let ki = d.index(b, k, h, l);
                            let rq = self.context[qi];
                            let rk = self.context[ki];
                            // Exact self relative is identity. Ambient conjugation would
                            // produce a spurious radial adjoint, so omit it explicitly.
                            if q != k && rq.present() && rk.present() {
                                let mut v = std::array::from_fn(|i| self.coefficient(1, lane, i));
                                if self.content[qi].present() && self.content[ki].present() {
                                    let relative = self.algebra.relative(
                                        code(self.content[qi].root())?,
                                        code(self.content[ki].root())?,
                                    );
                                    let dc = root(relative.index());
                                    for j in 0..4 {
                                        for (i, &x) in dc.iter().enumerate() {
                                            v[j] += x * self.coefficient(2, lane, i * 4 + j);
                                        }
                                    }
                                }
                                let a = root(rq.root());
                                let z = root(rk.root());
                                for axis in 0..4 {
                                    let mut unit = [0.; 4];
                                    unit[axis] = 1.;
                                    ambient[qi][axis] += g * dot(v, hamilton(conjugate(unit), z));
                                    ambient[ki][axis] += g * dot(v, hamilton(conjugate(a), unit));
                                }
                            }
                            for category in 0..33 {
                                let qalt = observation(self.raw_roots[qi], category)?;
                                if q == k {
                                    category_credit[qi * 33 + category] +=
                                        g * self.lane_score(lane, qi, ki, qalt, qalt)?;
                                } else {
                                    let kalt = observation(self.raw_roots[ki], category)?;
                                    category_credit[qi * 33 + category] +=
                                        g * self.lane_score(lane, qi, ki, qalt, rk)?;
                                    category_credit[ki * 33 + category] +=
                                        g * self.lane_score(lane, qi, ki, rq, kalt)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut dr = vec![0.; d.count() * 120];
        let mut dc = vec![0.; d.count() * 33];
        for at in 0..d.count() {
            let credit = (0..120)
                .map(|r| dot(ambient[at], root(r)))
                .collect::<Vec<_>>();
            choice_pullback(
                &roots[at * 120..(at + 1) * 120],
                &credit,
                &mut dr[at * 120..(at + 1) * 120],
            );
            choice_pullback(
                &categories[at * 33..(at + 1) * 33],
                &category_credit[at * 33..(at + 1) * 33],
                &mut dc[at * 33..(at + 1) * 33],
            );
        }
        Ok((checked(dr)?, checked(dc)?))
    }
}
impl CustomOp2 for PotentialCredit {
    fn name(&self) -> &'static str {
        "frozen-q4-potential-context-input-credit"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let a = contiguous(a, la)?;
        let b = contiguous(b, lb)?;
        if a.len() != self.d.count() * 120
            || b.len() != self.d.count() * 33
            || a.iter().chain(b).any(|x| !x.is_finite())
        {
            candle_core::bail!("potential context graph input differs");
        }
        Ok((
            CpuStorage::F32(self.hard.iter().map(|&x| (x as f64 / Q24) as f32).collect()),
            Shape::from((self.d.batch, self.d.heads, self.d.time, self.d.time)),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        _: &Tensor,
        g: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (da, db) = self
            .backward(
                &a.flatten_all()?.to_vec1::<f32>()?,
                &b.flatten_all()?.to_vec1::<f32>()?,
                &g.flatten_all()?.to_vec1::<f32>()?,
            )
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        Ok((
            Some(Tensor::from_vec(da, a.shape(), a.device())?),
            Some(Tensor::from_vec(db, b.shape(), b.device())?),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context::NativeContextTrace;
    use crate::geometric_value_native::ValuePacketStatus;
    use candle_core::Var;

    fn mock(
        d: Dimensions,
        states: Vec<u8>,
        emitted: Vec<u8>,
        categories: Vec<u8>,
    ) -> Result<(ContextQ4Output, Var, Var, Var)> {
        let latent = Var::from_vec(
            states
                .iter()
                .flat_map(|&s| root(s).map(|x| x as f32))
                .collect::<Vec<_>>(),
            (d.batch, d.time, d.heads, d.lanes, 4),
            &Device::Cpu,
        )?;
        let r = Var::zeros(
            (d.batch, d.time, d.heads, d.lanes, 120),
            DType::F32,
            &Device::Cpu,
        )?;
        let c = Var::zeros(
            (d.batch, d.time, d.heads, d.lanes, 33),
            DType::F32,
            &Device::Cpu,
        )?;
        let trace = NativeContextTrace {
            batch: d.batch,
            time: d.time,
            heads: d.heads,
            lanes_per_head: d.lanes,
            states: states.chunks(d.width()).map(|v| v.to_vec()).collect(),
            actions: vec![vec![1; d.width()]; d.rows()],
            codes: emitted
                .iter()
                .zip(&categories)
                .map(|(&r, &c)| observation(r, usize::from(c)))
                .collect::<Result<_>>()?,
            emitted_roots: emitted,
            categories,
            coefficient_reads: 0,
        };
        Ok((
            ContextQ4Output {
                // These legacy observed-input fixtures do not consume hidden-state scores.
                state_logits: Tensor::zeros(
                    (d.batch, d.time, d.heads, d.lanes, 120),
                    DType::F32,
                    &Device::Cpu,
                )?,
                latent_roots: latent.as_tensor().clone(),
                // Legacy observation fixture does not consume retained choices.
                state_choices: Tensor::zeros(
                    (d.batch, d.time, d.heads, d.lanes, 120),
                    DType::F32,
                    &Device::Cpu,
                )?,
                root_logits: r.as_tensor().clone(),
                category_logits: c.as_tensor().clone(),
                trace,
            },
            latent,
            r,
            c,
        ))
    }
    fn packed(d: Dimensions, entries: &[(usize, usize, i8)]) -> Result<Vec<u8>> {
        let cfg = PotentialQ4Config {
            heads: d.heads,
            lanes_per_head: d.lanes,
        };
        let mut q = vec![
            0;
            cfg.coefficient_count()
                .map_err(|e| invalid(e.to_string()))?
        ];
        for &(family, within, value) in entries {
            let start = FAMILY_COUNTS[..family].iter().sum::<usize>() * d.width();
            q[start + within] = value;
        }
        geometric_potential_q4::pack_coefficients(&q).map_err(|e| invalid(e.to_string()))
    }
    #[test]
    fn context_credit_potential_self_alias_and_absence_use_actual_category_scores() -> Result<()> {
        let d = Dimensions {
            batch: 1,
            time: 1,
            heads: 1,
            lanes: 1,
        };
        let present = observation(3, 1)?;
        let absent = observation(1, 0)?;
        let source = packed(
            d,
            &[(1, 0, 4), (4, 0, 4), (4, 33, -4), (6, 0, -2), (6, 3, 3)],
        )?;
        let op = PotentialCredit::new(d, vec![absent], vec![present], vec![3], &source)?;
        let (dr, dc) = op.backward(&vec![0.; 120], &vec![0.; 33], &[1.])?;
        assert!(
            dr.iter().all(|x| *x == 0.),
            "self relative must have no directional credit"
        );
        let scores = (0..33)
            .map(|c| {
                let alt = observation(3, c)?;
                op.lane_score(0, 0, 0, alt, alt)
            })
            .collect::<Result<Vec<_>>>()?;
        let mean = scores.iter().sum::<f64>() / 33.;
        for (g, score) in dc.iter().zip(scores) {
            assert!((f64::from(*g) - (score - mean) / 33.).abs() < 1e-7);
        }
        assert!(
            dc[0].abs() > 0.001,
            "absence participates in the finite categorical credit"
        );
        assert_ne!(
            dc[1], dc[2],
            "both radial diagonal roles must change together"
        );
        Ok(())
    }

    #[test]
    fn context_credit_potential_both_directions_future_mask_and_raw_absent_root() -> Result<()> {
        let d = Dimensions {
            batch: 1,
            time: 2,
            heads: 1,
            lanes: 1,
        };
        let present = observation(1, 17)?;
        let absent = observation(1, 0)?;
        let source = packed(d, &[(1, 0, 4), (1, 1, 2), (2, 0, 3), (6, 1, 2), (6, 2, -3)])?;
        let op = PotentialCredit::new(d, vec![present; 2], vec![present; 2], vec![1; 2], &source)?;
        let r = vec![0.; 240];
        let c = vec![0.; 66];
        let (dr, _) = op.backward(&r, &c, &[0., 0., 1., 0.])?;
        assert!(dr[..120].iter().any(|x| x.abs() > 1e-5));
        assert!(dr[120..].iter().any(|x| x.abs() > 1e-5));
        assert!(
            dr[0] < 0. && dr[1] > 0. && dr[120] < 0. && dr[121] > 0.,
            "finite antipodal credit survives for both endpoints"
        );
        // Finite differences of the declared fixed-other-endpoint smooth
        // Hamilton/expected-root surrogate, not the discrete lookup function.
        let smooth = |z: &[f32], query: bool| {
            let mut expected = [0.; 4];
            for (i, p) in probabilities(z).into_iter().enumerate() {
                for (axis, v) in root(i as u8).into_iter().enumerate() {
                    expected[axis] += p * v;
                }
            }
            let relative = if query {
                hamilton(conjugate(expected), root(1))
            } else {
                hamilton(conjugate(root(1)), expected)
            };
            dot([1.75, 0.5, 0., 0.], relative)
        };
        for query in [false, true] {
            for at in [0, 1, 3] {
                let mut hi = vec![0.; 120];
                let mut lo = hi.clone();
                hi[at] = 0.001;
                lo[at] = -0.001;
                let numeric = (smooth(&hi, query) - smooth(&lo, query))
                    / (f64::from(hi[at]) - f64::from(lo[at]));
                assert!((numeric - f64::from(dr[usize::from(query) * 120 + at])).abs() < 1e-6);
            }
        }
        let (future_r, future_c) = op.backward(&r, &c, &[0., 1., 0., 0.])?;
        assert!(future_r.iter().chain(&future_c).all(|x| *x == 0.));
        let left = PotentialCredit::new(
            d,
            vec![present; 2],
            vec![present, absent],
            vec![1, 1],
            &source,
        )?;
        let right = PotentialCredit::new(
            d,
            vec![present; 2],
            vec![present, absent],
            vec![1, 0],
            &source,
        )?;
        assert_eq!(
            left.hard, right.hard,
            "absence hard score does not expose raw direction"
        );
        let (_, lc) = left.backward(&r, &c, &[0., 0., 1., 0.])?;
        let (_, rc) = right.backward(&r, &c, &[0., 0., 1., 0.])?;
        assert_ne!(
            lc[33], rc[33],
            "absent-to-present uses retained raw root, not canonical identity"
        );
        Ok(())
    }

    #[test]
    fn context_credit_no_read_all_lanes_category_zero_and_frozen_coefficients() -> Result<()> {
        let d = Dimensions {
            batch: 1,
            time: 2,
            heads: 2,
            lanes: 1,
        };
        let (ctx, latent, _, category) = mock(d, vec![1; 4], vec![1; 4], vec![0; 4])?;
        let weights = NoReadWeights::new(4, 2, 1)?;
        let width = weights.config().coefficients_per_head();
        let mut w = vec![0f32; width * 2];
        let start = 5;
        w[start + 4] = 0.5; // head-zero depends on other head's latent root.
        w[start + 8 + 33] = 1.;
        w[start + 8 + 33 + 17] = -0.75;
        weights.parameters()["coefficients"].set(&Tensor::from_vec(
            w,
            (2, width),
            &Device::Cpu,
        )?)?;
        let held = vec![H4Code::IDENTITY; 4];
        let out = frozen_no_read_forward(&weights, &[0, 1], &ctx, &held, &[false; 2])?;
        assert_eq!(
            out.scores_q24,
            vec![(1.5 * Q24) as i64; 2]
                .into_iter()
                .chain([0, 0])
                .collect::<Vec<_>>()
        );
        let grad = out
            .scores
            .narrow(1, 0, 1)?
            .narrow(2, 1, 1)?
            .sum_all()?
            .backward()?;
        let g = grad
            .get(latent.as_tensor())
            .ok_or_else(|| invalid("NoRead latent gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(g[12], 0.5);
        assert!(g[..12].iter().all(|x| *x == 0.));
        let gc = grad
            .get(category.as_tensor())
            .ok_or_else(|| invalid("NoRead category gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(gc[99] > 0. && gc[116] < 0.);
        assert!(gc[..99].iter().all(|x| *x == 0.));
        assert!(weights
            .parameters()
            .values()
            .all(|v| grad.get(v.as_tensor()).is_none()));
        Ok(())
    }

    #[test]
    fn context_credit_value_native_zero_input_gradient_and_structural_absence() -> Result<()> {
        let d = Dimensions {
            batch: 1,
            time: 2,
            heads: 1,
            lanes: 1,
        };
        let (ctx, latent, _, _) = mock(d, vec![1; 2], vec![1; 2], vec![17; 2])?;
        let weights = ValueProducerWeights::new(4, 1, 1, 17)?.into_q4()?;
        for variable in weights.parameters().values() {
            variable.set(&Tensor::zeros(variable.shape(), DType::F32, &Device::Cpu)?)?;
        }
        let variable = &weights.parameters()["own_root"];
        let mut w = vec![0f32; variable.elem_count()];
        for slot in 0..8 {
            w[(slot * 120) * 4] = -1.;
            w[(slot * 120 + 1) * 4] = 1.;
        }
        variable.set(&Tensor::from_vec(w, variable.shape(), &Device::Cpu)?)?;
        let held = vec![H4Code::IDENTITY; 2];
        let out =
            frozen_value_forward(&weights, &[0, 1], &ctx, &held, &[false; 2], &[true, false])?;
        assert!(out.trace.values_q16.iter().all(|x| *x == 0));
        for packet in out.trace.packets[..4].iter().flatten() {
            assert_eq!(packet.status, ValuePacketStatus::PresentZero);
        }
        for packet in out.trace.packets[4..].iter().flatten() {
            assert_eq!(packet.status, ValuePacketStatus::Absent);
        }
        let grad = out.values.sum_all()?.backward()?;
        let g = grad
            .get(latent.as_tensor())
            .ok_or_else(|| invalid("value latent gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(g[0] > 0.);
        assert!(g[4..].iter().all(|x| *x == 0.));
        assert!(weights
            .parameters()
            .values()
            .all(|v| grad.get(v.as_tensor()).is_none()));
        let native = weights.forward_q4_codes(
            &[0, 1],
            1,
            2,
            &[H4Code::IDENTITY; 2],
            &held,
            &[false; 2],
            &[true, false],
        )?;
        assert_eq!(native.trace.packets, out.trace.packets);
        assert_eq!(native.trace.values_q16, out.trace.values_q16);
        Ok(())
    }

    #[test]
    fn context_credit_trace_mismatch_rejects_and_no_read_scalar_surrogate_difference() -> Result<()>
    {
        let d = Dimensions {
            batch: 1,
            time: 1,
            heads: 1,
            lanes: 1,
        };
        let (mut ctx, _, _, _) = mock(d, vec![1], vec![0], vec![0])?;
        admit(&ctx)?;
        ctx.trace.codes[0] = observation(1, 1)?;
        assert!(admit(&ctx).is_err());
        ctx.trace.codes[0] = observation(1, 0)?;
        ctx.latent_roots = Tensor::zeros((1, 1, 1, 1, 4), DType::F32, &Device::Cpu)?;
        assert!(admit(&ctx).is_err());
        let mut coefficients = vec![0.; 1 + 4 + 42];
        coefficients[5] = 0.75;
        coefficients[9] = -1.;
        coefficients[9 + 32] = 1.5;
        let op = NoReadCredit {
            d,
            vocabulary: 4,
            coefficients: coefficients.clone(),
            hard: vec![0],
        };
        let c = (0..33)
            .map(|i| (i as f32 - 16.) * 0.031)
            .collect::<Vec<_>>();
        let (_, dc) = op.backward(&c, &[0.7])?;
        let loss = |z: &[f32]| -> f64 {
            0.7 * probabilities(z)
                .iter()
                .zip(&coefficients[9..42])
                .map(|(p, f)| p * f)
                .sum::<f64>()
        };
        let epsilon = 0.001;
        for at in [0, 7, 32] {
            let mut hi = c.clone();
            let mut lo = c.clone();
            hi[at] += epsilon;
            lo[at] -= epsilon;
            let numeric = (loss(&hi) - loss(&lo)) / (f64::from(hi[at]) - f64::from(lo[at]));
            assert!((numeric - f64::from(dc[at])).abs() < 2e-6);
        }
        Ok(())
    }
}

/// Source observations plus one refined query snapshot, not a token replay.
/// Source-side tensors are detached; only the final query carries input credit.
pub struct SnapshotCreditOutput {
    d: Dimensions,
    states: Vec<H4Code>,
    codes: Vec<AddressLane>,
    raw_roots: Vec<u8>,
    latent_roots: Tensor,
    root_logits: Tensor,
    category_logits: Tensor,
}

/// Apply frozen readouts to a refined NEW state without another transition.
/// Original source rows are copied from the real causal replay; its query/prefix
/// rows are omitted. The appended row is explicitly an observation-only snapshot.
pub fn frozen_snapshot_observation(
    weights: &crate::geometric_context::ContextWeights,
    codec: &uor_r4_integer::geometric_context_q4::NativeContextQ4,
    original: &uor_r4_integer::geometric_source_realizer::SerializableContextReplay,
    source_count: usize,
    snapshot: &uor_r4_integer::geometric_read_feedback::QuerySnapshotReport,
    refined_latent: &Tensor,
) -> Result<SnapshotCreditOutput> {
    let c = weights.config();
    let width = c.heads * c.lanes_per_head;
    if !weights.is_q4()
        || original.heads != c.heads
        || original.lanes_per_head != c.lanes_per_head
        || source_count == 0
        || source_count >= original.tokens.len()
        || snapshot.heads != c.heads
        || snapshot.lanes_per_head != c.lanes_per_head
        || snapshot.last_token_id
            != *original
                .tokens
                .last()
                .ok_or_else(|| invalid("empty original replay"))?
        || original.states.len() != original.tokens.len()
        || original.raw_roots.len() != original.tokens.len() * width
        || original.categories.len() != original.raw_roots.len()
        || original.codes.len() != original.raw_roots.len()
        || snapshot.states.len() != width
        || snapshot.raw_roots.len() != width
        || snapshot.categories.len() != width
        || snapshot.codes.len() != width
    {
        return Err(invalid(
            "refined observation source/snapshot dimensions differ",
        ));
    }
    if codec.config()
        != (uor_r4_integer::geometric_context_q4::ContextQ4Config {
            vocab_size: c.vocab_size,
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
        })
    {
        return Err(invalid("snapshot frozen codec config differs"));
    }
    let typed = snapshot
        .states
        .iter()
        .map(|&r| code(r))
        .collect::<Result<Vec<_>>>()?;
    let observed = uor_r4_integer::geometric_context::NativeContextState::observe_states(
        snapshot.last_token_id as usize,
        &typed,
        codec.native(),
    )
    .map_err(|e| invalid(e.to_string()))?;
    for lane in 0..width {
        if observed.readout_roots[lane].index() != snapshot.raw_roots[lane]
            || observed.categories[lane] != snapshot.categories[lane]
            || observed.output[lane]
                != AddressLane::new(
                    snapshot.codes[lane].root,
                    snapshot.codes[lane].radius_bin,
                    snapshot.codes[lane].present,
                )
                .map_err(|e| invalid(e.to_string()))?
        {
            return Err(invalid(
                "refined snapshot does not match frozen native observation",
            ));
        }
    }
    let actual = tensor_values(refined_latent, &[width, 4])?;
    for (lane, &s) in snapshot.states.iter().enumerate() {
        for (axis, x) in root(s).iter().enumerate() {
            if actual[lane * 4 + axis].to_bits() != (*x as f32).to_bits() {
                return Err(invalid("refined graph latent differs"));
            }
        }
    }
    let mut states = Vec::new();
    let mut codes = Vec::new();
    let mut raw_roots = Vec::new();
    let mut source_latent = Vec::new();
    for row in 0..source_count {
        if original.states[row].len() != width {
            return Err(invalid("source latent width differs"));
        }
        for lane in 0..width {
            let at = row * width + lane;
            let s = code(original.states[row][lane])?;
            states.push(s);
            source_latent.extend(root(s.index()).map(|x| x as f32));
            let r = original.raw_roots[at];
            code(r)?;
            let category = usize::from(original.categories[at]);
            if category > 32 {
                return Err(invalid("source category differs"));
            }
            let q = observation(r, category)?;
            let saved = &original.codes[at];
            if q != AddressLane::new(saved.root, saved.radius_bin, saved.present)
                .map_err(|e| invalid(e.to_string()))?
            {
                return Err(invalid("source observation differs"));
            }
            codes.push(q);
            raw_roots.push(r);
        }
    }
    states.extend_from_slice(&typed);
    codes.extend_from_slice(&observed.output[..width]);
    raw_roots.extend_from_slice(&snapshot.raw_roots);
    let latent_roots = Tensor::cat(
        &[
            Tensor::from_vec(source_latent, (source_count, width, 4), &Device::Cpu)?,
            refined_latent.reshape((1, width, 4))?,
        ],
        0,
    )?
    .reshape((1, source_count + 1, c.heads, c.lanes_per_head, 4))?;
    let family_logits = |family: &str, classes: usize| -> Result<Tensor> {
        let fixed = |name: &str| -> Result<Vec<f32>> {
            Ok(weights
                .parameters()
                .get(name)
                .ok_or_else(|| invalid("observation coefficient absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .map(|x| (x * 4.).round() * 0.25)
                .collect())
        };
        let own = fixed(&format!("self_{family}"))?;
        let neighbor = if c.lanes_per_head > 1 {
            Some(fixed(&format!("neighbor_{family}"))?)
        } else {
            None
        };
        let tables = codec.table_slices();
        let (token_table, own_table, neighbor_table, stride) = if family == "root" {
            (
                tables.token_root,
                tables.self_root,
                tables.neighbor_root,
                128,
            )
        } else {
            (
                tables.token_category,
                tables.self_category,
                tables.neighbor_category,
                64,
            )
        };
        let mut output = Vec::new();
        for lane in 0..width {
            let next = lane / c.lanes_per_head * c.lanes_per_head
                + (lane % c.lanes_per_head + 1) % c.lanes_per_head;
            let weight = Tensor::from_vec(
                own[lane * classes * 4..(lane + 1) * classes * 4].to_vec(),
                (classes, 4),
                &Device::Cpu,
            )?;
            let mut surrogate = refined_latent
                .narrow(0, lane, 1)?
                .matmul(&weight.t()?)?
                .reshape(classes)?;
            if let Some(n) = &neighbor {
                let weight = Tensor::from_vec(
                    n[lane * classes * 4..(lane + 1) * classes * 4].to_vec(),
                    (classes, 4),
                    &Device::Cpu,
                )?;
                surrogate = (&surrogate
                    + refined_latent
                        .narrow(0, next, 1)?
                        .matmul(&weight.t()?)?
                        .reshape(classes)?)?;
            }
            let hard = (0..classes)
                .map(|class| {
                    let mut q = i64::from(
                        token_table
                            [(snapshot.last_token_id as usize * width + lane) * stride + class],
                    ) + i64::from(
                        own_table[(lane * 128 + snapshot.states[lane] as usize) * stride + class],
                    );
                    if let Some(n) = neighbor_table {
                        q += i64::from(
                            n[(lane * 128 + snapshot.states[next] as usize) * stride + class],
                        );
                    }
                    (q as f64 / Q24) as f32
                })
                .collect::<Vec<_>>();
            output.push(
                (&Tensor::from_vec(hard, classes, &Device::Cpu)?
                    + (&surrogate - surrogate.detach())?)?,
            );
        }
        let query = Tensor::stack(&output, 0)?.reshape((1, width, classes))?;
        // Frozen source logits affect only a declared backward softmax; their
        // native choices/codes remain authoritative and they have no graph path.
        let source = Tensor::zeros((source_count, width, classes), DType::F32, &Device::Cpu)?;
        Ok(Tensor::cat(&[source, query], 0)?.reshape((
            1,
            source_count + 1,
            c.heads,
            c.lanes_per_head,
            classes,
        ))?)
    };
    Ok(SnapshotCreditOutput {
        d: Dimensions {
            batch: 1,
            time: source_count + 1,
            heads: c.heads,
            lanes: c.lanes_per_head,
        },
        states,
        codes,
        raw_roots,
        latent_roots,
        root_logits: family_logits("root", 120)?,
        category_logits: family_logits("category", 33)?,
    })
}
pub fn frozen_snapshot_potential_forward(
    weights: &PotentialQ4Weights,
    snapshot: &SnapshotCreditOutput,
) -> Result<PotentialQ4Output> {
    let absent = AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?;
    potential_credit_forward(
        weights,
        &vec![absent; snapshot.d.count()],
        snapshot.d,
        &snapshot.codes,
        &snapshot.raw_roots,
        &snapshot.root_logits,
        &snapshot.category_logits,
    )
}
pub fn frozen_snapshot_no_read_forward(
    weights: &NoReadWeights,
    ids: &[u32],
    snapshot: &SnapshotCreditOutput,
) -> Result<NoReadCreditOutput> {
    no_read_credit_forward(
        weights,
        ids,
        snapshot.d,
        &snapshot.states,
        &snapshot.codes,
        &snapshot.latent_roots,
        &snapshot.category_logits,
        &vec![H4Code::IDENTITY; snapshot.d.count()],
        &vec![false; snapshot.d.rows()],
    )
}

// PATCH DRAFT: append to training/src/geometric_context_credit.rs.
// Reuses private admit/code/choice_pullback/checked/contiguous/Q24/ALGEBRA.
// Offline only. Hard native cue output with a declared finite-choice adjoint.
use uor_r4_integer::geometric_cue_carrier::{
    CarrierState, CueAngularConfig, CueCarrierTrace, CueScoreMode,
};

pub struct CueRootCreditOutput {
    pub scores: Tensor,
    pub scores_q24: Vec<Vec<i64>>,
}

fn cue_final_roots(context: &ContextQ4Output, packet: &CarrierState) -> Result<Tensor> {
    let (d, _) = admit(context)?;
    if d.batch != 1 || packet.token_ids.len() != d.time {
        return Err(invalid(
            "cue root credit requires one actual nonempty local encoding",
        ));
    }
    let at = (d.time - 1) * d.width();
    let codes = context.trace.codes[at..at + d.width()]
        .iter()
        .copied()
        .map(uor_r4_integer::geometric_source_realizer::ObservedCode::from)
        .collect::<Vec<_>>();
    if context.trace.states[d.time - 1] != packet.states
        || context.trace.emitted_roots[at..at + d.width()] != packet.raw_roots
        || context.trace.categories[at..at + d.width()] != packet.categories
        || codes != packet.codes
    {
        return Err(invalid(
            "cue local final packet differs from native carrier",
        ));
    }
    Ok(context
        .root_logits
        .narrow(1, d.time - 1, 1)?
        .reshape((d.width(), 120))?
        .contiguous()?)
}

/// Caller verifies actual token IDs/reset-from-identity replay and enclosing
/// artifact identity. Final raw/observed packet and payload binding are checked
/// here. No category credit or latent substitution; absent endpoints remain zero.
/// Output heads x candidates matches the native cue sidecar, in nat units.
pub fn frozen_cue_root_forward(
    config: CueAngularConfig,
    packed: &[u8],
    carrier: &CueCarrierTrace,
    query: &ContextQ4Output,
    cues: &[ContextQ4Output],
) -> Result<CueRootCreditOutput> {
    use sha2::{Digest, Sha256};
    if config != carrier.metadata.potential
        || hex::encode(Sha256::digest(packed)) != carrier.metadata.potential_packed_sha256
        || cues.len() != carrier.cues.len()
        || query.trace.heads != config.heads
        || query.trace.lanes_per_head != config.lanes_per_head
    {
        return Err(invalid(
            "cue credit config/payload/local encoding binding differs",
        ));
    }
    let q = cue_final_roots(query, &carrier.query)?;
    let mut local = Vec::with_capacity(cues.len());
    for (context, packet) in cues.iter().zip(&carrier.cues) {
        if context.trace.heads != config.heads
            || context.trace.lanes_per_head != config.lanes_per_head
        {
            return Err(invalid("cue credit local encoder dimensions differ"));
        }
        local.push(cue_final_roots(context, &packet.state)?);
    }
    // No-cue banks need a graph-compatible empty second input, not a fake cue.
    let k = if local.is_empty() {
        Tensor::zeros(
            (0, config.heads * config.lanes_per_head, 120),
            DType::F32,
            q.device(),
        )?
    } else {
        Tensor::stack(&local, 0)?.contiguous()?
    };
    let op = CueRootCredit::new(config, packed, carrier)?;
    let scores_q24 = op.hard.clone();
    let scores = if q.device().is_cuda() {
        op.device_graph(&q, &k)?
    } else {
        q.apply_op2(&k, op)?
    };
    Ok(CueRootCreditOutput { scores, scores_q24 })
}

struct CueRootCredit {
    config: CueAngularConfig,
    coefficients: Vec<i8>,
    algebra: HistoricalH4Tables,
    query: Vec<AddressLane>,
    cues: Vec<Vec<AddressLane>>,
    candidate_cues: Vec<Option<usize>>,
    hard: Vec<Vec<i64>>,
}
impl CueRootCredit {
    fn new(config: CueAngularConfig, packed: &[u8], trace: &CueCarrierTrace) -> Result<Self> {
        let coefficients = geometric_potential_q4::unpack_coefficients(
            config
                .coefficient_count()
                .map_err(|e| invalid(e.to_string()))?,
            packed,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let addresses = |p: &CarrierState| -> Result<Vec<AddressLane>> {
            p.codes
                .iter()
                .map(|c| {
                    AddressLane::new(c.root, c.radius_bin, c.present)
                        .map_err(|e| invalid(e.to_string()))
                })
                .collect()
        };
        let width = config.heads * config.lanes_per_head;
        let query = addresses(&trace.query)?;
        let cues = trace
            .cues
            .iter()
            .map(|x| addresses(&x.state))
            .collect::<Result<Vec<_>>>()?;
        if query.len() != width
            || cues.iter().any(|x| x.len() != width)
            || trace
                .candidate_cue_indices
                .iter()
                .flatten()
                .any(|&x| x >= cues.len())
            || trace.copy_q24.len() != config.heads
            || trace
                .copy_q24
                .iter()
                .any(|x| x.len() != trace.candidate_cue_indices.len())
        {
            return Err(invalid("cue credit native trace shape differs"));
        }
        let op = Self {
            config,
            coefficients,
            algebra: HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?,
            query,
            cues,
            candidate_cues: trace.candidate_cue_indices.clone(),
            hard: trace.copy_q24.clone(),
        };
        for h in 0..config.heads {
            for n in 0..op.candidate_cues.len() {
                let mut exact = 0i64;
                if let Some(s) = op.candidate_cues[n] {
                    for l in 0..config.lanes_per_head {
                        let lane = h * config.lanes_per_head + l;
                        exact = exact
                            .checked_add(op.score(lane, op.query[lane], op.cues[s][lane])?)
                            .ok_or_else(|| invalid("cue hard score overflow"))?;
                    }
                }
                if exact != op.hard[h][n] {
                    return Err(invalid("cue native hard Q24 differs"));
                }
            }
        }
        Ok(op)
    }
    fn score(&self, lane: usize, q: AddressLane, k: AddressLane) -> Result<i64> {
        if !q.present() || !k.present() {
            return Ok(0);
        }
        // Compute directed relation in both arms, matching the native contract.
        let relative = self
            .algebra
            .relative(code(q.root())?, code(k.root())?)
            .index();
        let index = match self.config.mode {
            CueScoreMode::DirectedRelative => relative,
            CueScoreMode::CueUnary => k.root(),
        };
        Ok(i64::from(self.coefficients[lane * 120 + usize::from(index)]) << 22)
    }
    fn backward(&self, q: &[f32], k: &[f32], g: &[f32]) -> Result<(Vec<f32>, Vec<f32>)> {
        let width = self.query.len();
        let n = self.candidate_cues.len();
        if q.len() != width * 120
            || k.len() != self.cues.len() * width * 120
            || g.len() != self.config.heads * n
            || q.iter().chain(k).chain(g).any(|x| !x.is_finite())
        {
            return Err(invalid("cue root adjoint shape/nonfinite input"));
        }
        let mut qcredit = vec![0f64; q.len()];
        let mut kcredit = vec![0f64; k.len()];
        for h in 0..self.config.heads {
            for candidate in 0..n {
                let Some(s) = self.candidate_cues[candidate] else {
                    continue;
                };
                let adj = f64::from(g[h * n + candidate]);
                for l in 0..self.config.lanes_per_head {
                    let lane = h * self.config.lanes_per_head + l;
                    let rq = self.query[lane];
                    let rk = self.cues[s][lane];
                    if !rq.present() || !rk.present() {
                        continue;
                    }
                    for alternative in 0..120u8 {
                        let aq = AddressLane::new(alternative, rq.radius_bin(), true)
                            .map_err(|e| invalid(e.to_string()))?;
                        let ak = AddressLane::new(alternative, rk.radius_bin(), true)
                            .map_err(|e| invalid(e.to_string()))?;
                        qcredit[lane * 120 + usize::from(alternative)] +=
                            adj * self.score(lane, aq, rk)? as f64 / Q24;
                        kcredit[(s * width + lane) * 120 + usize::from(alternative)] +=
                            adj * self.score(lane, rq, ak)? as f64 / Q24;
                    }
                }
            }
        }
        // Choice credit is invariant to a rowwise constant. Remove it exactly
        // before pullback so unary-query and constant-table ties are EXACT zero,
        // rather than tiny numerical residuals of summing120 probabilities.
        for row in qcredit.chunks_exact_mut(120) {
            let anchor = row[0];
            for x in row {
                *x -= anchor;
            }
        }
        for row in kcredit.chunks_exact_mut(120) {
            let anchor = row[0];
            for x in row {
                *x -= anchor;
            }
        }
        let mut dq = vec![0f64; q.len()];
        let mut dk = vec![0f64; k.len()];
        // Sum all candidate contributions sharing a cue BEFORE the softmax adjoint.
        for lane in 0..width {
            let at = lane * 120;
            choice_pullback(
                &q[at..at + 120],
                &qcredit[at..at + 120],
                &mut dq[at..at + 120],
            );
        }
        for at in (0..k.len()).step_by(120) {
            choice_pullback(
                &k[at..at + 120],
                &kcredit[at..at + 120],
                &mut dk[at..at + 120],
            );
        }
        Ok((checked(dq)?, checked(dk)?))
    }
}
impl CustomOp2 for CueRootCredit {
    fn name(&self) -> &'static str {
        "frozen-cue-native-root120-choice-credit"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let q = contiguous(a, la)?;
        let k = contiguous(b, lb)?;
        if q.len() != self.query.len() * 120
            || k.len() != self.cues.len() * self.query.len() * 120
            || q.iter().chain(k).any(|x| !x.is_finite())
        {
            candle_core::bail!("cue credit forward inputs differ");
        }
        Ok((
            CpuStorage::F32(
                self.hard
                    .iter()
                    .flatten()
                    .map(|&x| (x as f64 / Q24) as f32)
                    .collect(),
            ),
            Shape::from((self.config.heads, self.candidate_cues.len())),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        _: &Tensor,
        g: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (dq, dk) = self
            .backward(
                &a.flatten_all()?.to_vec1::<f32>()?,
                &b.flatten_all()?.to_vec1::<f32>()?,
                &g.flatten_all()?.to_vec1::<f32>()?,
            )
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        Ok((
            Some(Tensor::from_vec(dq, a.shape(), a.device())?),
            Some(Tensor::from_vec(dk, b.shape(), b.device())?),
        ))
    }
}

#[cfg(test)]
mod cue_root_credit_draft_tests {
    use super::*;
    use candle_core::Var;
    fn fixture(mode: CueScoreMode, present: bool) -> Result<CueRootCredit> {
        let config = CueAngularConfig {
            heads: 1,
            lanes_per_head: 1,
            mode,
        };
        let q = if present {
            AddressLane::new(4, 7, true)
        } else {
            AddressLane::new(H4Code::IDENTITY.index(), 0, false)
        }
        .map_err(|e| invalid(e.to_string()))?;
        let k = AddressLane::new(9, 3, true).map_err(|e| invalid(e.to_string()))?;
        let mut op = CueRootCredit {
            config,
            coefficients: (0..120).map(|r| (r % 15) as i8 - 7).collect(),
            algebra: HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?,
            query: vec![q],
            cues: vec![vec![k]],
            candidate_cues: vec![Some(0), Some(0), None],
            hard: vec![vec![0; 3]],
        };
        let exact = op.score(0, q, k)?;
        op.hard[0] = vec![exact, exact, 0];
        Ok(op)
    }
    #[test]
    fn cue_root_credit_native_hard_shared_candidate_adjoint_and_finite_choice() -> Result<()> {
        let op = fixture(CueScoreMode::DirectedRelative, true)?;
        let hard = op.hard.clone();
        let q = Var::from_vec(vec![0f32; 120], (1, 120), &Device::Cpu)?;
        let k = Var::from_vec(vec![0f32; 120], (1, 1, 120), &Device::Cpu)?;
        let out = q.as_tensor().apply_op2(k.as_tensor(), op)?;
        assert_eq!(
            out.to_vec2::<f32>()?,
            hard.iter()
                .map(|r| r
                    .iter()
                    .map(|&x| (x as f64 / Q24) as f32)
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
        let grad = (out * Tensor::from_vec(vec![1f32, 2., 0.], (1, 3), &Device::Cpu)?)?
            .sum_all()?
            .backward()?;
        let dq = grad
            .get(q.as_tensor())
            .ok_or_else(|| invalid("query root graph disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let dk = grad
            .get(k.as_tensor())
            .ok_or_else(|| invalid("cue root graph disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(dq.iter().any(|x| x.abs() > 1e-6) && dk.iter().any(|x| x.abs() > 1e-6));
        let op = fixture(CueScoreMode::DirectedRelative, true)?;
        let (singleq, singlek) = op.backward(&vec![0f32; 120], &vec![0f32; 120], &[1., 0., 0.])?;
        for i in 0..120 {
            assert!((dq[i] - 3. * singleq[i]).abs() < 1e-6);
            assert!((dk[i] - 3. * singlek[i]).abs() < 1e-6);
        }
        // Finite difference of the declared smooth expectation, NOT hard argmax.
        let mut logits = vec![0f32; 120];
        let axis = 17;
        let epsilon = 0.001f32;
        let expectation = |xs: &[f32]| -> Result<f64> {
            probabilities(xs)
                .iter()
                .enumerate()
                .try_fold(0., |sum, (r, p)| {
                    let aq = AddressLane::new(r as u8, op.query[0].radius_bin(), true)
                        .map_err(|e| invalid(e.to_string()))?;
                    Ok(sum + 3. * p * op.score(0, aq, op.cues[0][0])? as f64 / Q24)
                })
        };
        logits[axis] = epsilon;
        let plus = expectation(&logits)?;
        logits[axis] = -epsilon;
        let minus = expectation(&logits)?;
        assert!(((plus - minus) / (2. * f64::from(epsilon)) - f64::from(dq[axis])).abs() < 1e-5);
        Ok(())
    }
    #[test]
    fn cue_root_credit_masks_absence_and_unary_query_and_preserves_tied_hard_forward() -> Result<()>
    {
        for mode in [CueScoreMode::DirectedRelative, CueScoreMode::CueUnary] {
            let absent = fixture(mode, false)?;
            assert_eq!(absent.hard, vec![vec![0i64; 3]]);
            let (q, k) = absent.backward(&vec![0f32; 120], &vec![0f32; 120], &[1., 2., 7.])?;
            assert!(q.iter().chain(&k).all(|&x| x == 0.));
        }
        for mode in [CueScoreMode::DirectedRelative, CueScoreMode::CueUnary] {
            let mut absent_cue = fixture(mode, true)?;
            absent_cue.cues[0][0] = AddressLane::new(H4Code::IDENTITY.index(), 0, false)
                .map_err(|e| invalid(e.to_string()))?;
            absent_cue.hard = vec![vec![0i64; 3]];
            let (q, k) = absent_cue.backward(&vec![0f32; 120], &vec![0f32; 120], &[1., 2., 7.])?;
            assert!(q.iter().chain(&k).all(|&x| x == 0.));
        }
        let unary = fixture(CueScoreMode::CueUnary, true)?;
        let (q, k) = unary.backward(&vec![0f32; 120], &vec![0f32; 120], &[1., 0., 0.])?;
        assert!(q.iter().all(|&x| x == 0.));
        assert!(k.iter().any(|&x| x != 0.));
        let mut tied = fixture(CueScoreMode::DirectedRelative, true)?;
        tied.coefficients.fill(3);
        tied.hard = vec![vec![3i64 << 22, 3i64 << 22, 0]];
        let (q, k) = tied.backward(&vec![0f32; 120], &vec![0f32; 120], &[1., 2., 0.])?;
        assert!(q.iter().chain(&k).all(|&x| x == 0.));
        // No tie winner is invented: all120 alternatives equal, actual hard packet retained.
        let q = Tensor::zeros((1, 120), DType::F32, &Device::Cpu)?;
        let k = Tensor::zeros((1, 1, 120), DType::F32, &Device::Cpu)?;
        assert_eq!(
            q.apply_op2(&k, tied)?.to_vec2::<f32>()?,
            vec![vec![0.75, 0.75, 0.]]
        );
        Ok(())
    }
}

/// Full observed-state credit from the frozen native cue scorer. Root120 and
/// category33 are finite-choice first-order surrogates with other lanes held to
/// their factual packets. Hard output remains exact native unary plus ordered
/// joint Q24; coefficient tensors are never trainable here.
/// Final packets and dimensions are checked here. ContextQ4Output does not
/// carry input token IDs: the caller binds each actual local token sequence and
/// reset-from-identity encoding to the frozen context artifact before this call.
pub fn frozen_cue_state_forward(
    config: CueAngularConfig,
    packed: &[u8],
    joint: Option<&uor_r4_integer::geometric_cue_carrier::CueJointQ4>,
    carrier: &CueCarrierTrace,
    query: &ContextQ4Output,
    cues: &[ContextQ4Output],
) -> Result<CueRootCreditOutput> {
    use sha2::{Digest, Sha256};
    if config != carrier.metadata.potential
        || hex::encode(Sha256::digest(packed)) != carrier.metadata.potential_packed_sha256
        || cues.len() != carrier.cues.len()
        || query.trace.heads != config.heads
        || query.trace.lanes_per_head != config.lanes_per_head
    {
        return Err(invalid(
            "cue state credit artifact/config/encoding binding differs",
        ));
    }
    validate_cue_state_joint_payload(joint, &carrier.metadata.joint)?;
    let q = cue_final_state_logits(query, &carrier.query)?;
    let mut local = Vec::with_capacity(cues.len());
    for (context, packet) in cues.iter().zip(&carrier.cues) {
        if context.trace.heads != config.heads
            || context.trace.lanes_per_head != config.lanes_per_head
        {
            return Err(invalid("cue state local encoder dimensions differ"));
        }
        local.push(cue_final_state_logits(context, &packet.state)?);
    }
    let k = if local.is_empty() {
        Tensor::zeros(
            (0, config.heads * config.lanes_per_head, 153),
            DType::F32,
            q.device(),
        )?
    } else {
        Tensor::stack(&local, 0)?.contiguous()?
    };
    let op = CueStateCredit::new(config, packed, joint, carrier)?;
    let scores_q24 = op.base.hard.clone();
    let scores = if q.device().is_cuda() {
        op.device_graph(&q, &k)?
    } else {
        q.apply_op2(&k, op)?
    };
    Ok(CueRootCreditOutput { scores, scores_q24 })
}
fn cue_final_state_logits(context: &ContextQ4Output, packet: &CarrierState) -> Result<Tensor> {
    let roots = cue_final_roots(context, packet)?;
    let (d, _) = admit(context)?;
    let categories = context
        .category_logits
        .narrow(1, d.time - 1, 1)?
        .reshape((d.width(), 33))?;
    Ok(Tensor::cat(&[&roots, &categories], 1)?.contiguous()?)
}

struct CueStateCredit {
    base: CueRootCredit,
    query_raw: Vec<u8>,
    cues_raw: Vec<Vec<u8>>,
    joint: Option<(
        uor_r4_integer::geometric_cue_carrier::CueJointConfig,
        Vec<i32>,
    )>,
}
impl CueStateCredit {
    fn new(
        config: CueAngularConfig,
        packed: &[u8],
        joint: Option<&uor_r4_integer::geometric_cue_carrier::CueJointQ4>,
        trace: &CueCarrierTrace,
    ) -> Result<Self> {
        let coefficients = geometric_potential_q4::unpack_coefficients(
            config
                .coefficient_count()
                .map_err(|e| invalid(e.to_string()))?,
            packed,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let addresses = |p: &CarrierState| {
            p.codes
                .iter()
                .map(|c| {
                    AddressLane::new(c.root, c.radius_bin, c.present)
                        .map_err(|e| invalid(e.to_string()))
                })
                .collect::<Result<Vec<_>>>()
        };
        let query = addresses(&trace.query)?;
        let cues = trace
            .cues
            .iter()
            .map(|p| addresses(&p.state))
            .collect::<Result<Vec<_>>>()?;
        let width = config.heads * config.lanes_per_head;
        if query.len() != width
            || trace.query.raw_roots.len() != width
            || cues.iter().any(|s| s.len() != width)
            || trace.cues.iter().any(|s| s.state.raw_roots.len() != width)
            || trace
                .candidate_cue_indices
                .iter()
                .flatten()
                .any(|&s| s >= cues.len())
        {
            return Err(invalid("cue state native packet shape differs"));
        }
        for r in trace
            .query
            .raw_roots
            .iter()
            .chain(trace.cues.iter().flat_map(|s| s.state.raw_roots.iter()))
        {
            code(*r)?;
        }
        let algebra =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let base = CueRootCredit {
            config,
            coefficients,
            algebra,
            query,
            cues,
            candidate_cues: trace.candidate_cue_indices.clone(),
            hard: trace.copy_q24.clone(),
        };
        let joint = joint
            .map(|j| {
                let c = j.config();
                if config.mode != CueScoreMode::DirectedRelative
                    || c.head >= config.heads
                    || c.left_lane >= config.lanes_per_head
                    || c.right_lane >= config.lanes_per_head
                {
                    return Err(invalid("cue state joint lanes/mode differ"));
                }
                let t = geometric_potential_q4::compile_ordered_pair_q4(j.packed_coefficients())
                    .map_err(|e| invalid(e.to_string()))?;
                Ok((c, t))
            })
            .transpose()?;
        let op = Self {
            base,
            query_raw: trace.query.raw_roots.clone(),
            cues_raw: trace
                .cues
                .iter()
                .map(|s| s.state.raw_roots.clone())
                .collect(),
            joint,
        };
        if op.base.hard.len() != config.heads
            || op
                .base
                .hard
                .iter()
                .any(|v| v.len() != op.base.candidate_cues.len())
        {
            return Err(invalid("cue state hard score shape differs"));
        }
        for h in 0..config.heads {
            for (i, s) in op.base.candidate_cues.iter().enumerate() {
                let exact = if let Some(s) = s {
                    op.score(h, &op.base.query, &op.base.cues[*s])?
                } else {
                    0
                };
                if exact != op.base.hard[h][i] {
                    return Err(invalid("cue state full native hard Q24 differs"));
                }
            }
        }
        Ok(op)
    }
    fn score(&self, h: usize, q: &[AddressLane], k: &[AddressLane]) -> Result<i64> {
        let mut total = 0i64;
        for l in 0..self.base.config.lanes_per_head {
            let at = h * self.base.config.lanes_per_head + l;
            total = total
                .checked_add(self.base.score(at, q[at], k[at])?)
                .ok_or_else(|| invalid("cue state unary sum overflow"))?;
        }
        if let Some((c, t)) = &self.joint {
            if c.head == h {
                let left = h * self.base.config.lanes_per_head + c.left_lane;
                let right = h * self.base.config.lanes_per_head + c.right_lane;
                if q[left].present()
                    && k[left].present()
                    && q[right].present()
                    && k[right].present()
                {
                    let a = self
                        .base
                        .algebra
                        .relative(code(q[left].root())?, code(k[left].root())?)
                        .index();
                    let b = self
                        .base
                        .algebra
                        .relative(code(q[right].root())?, code(k[right].root())?)
                        .index();
                    total = total
                        .checked_add(i64::from(t[(usize::from(a) << 7) + usize::from(b)]))
                        .ok_or_else(|| invalid("cue state joint sum overflow"))?;
                }
            }
        }
        Ok(total)
    }
    fn backward(&self, q: &[f32], k: &[f32], g: &[f32]) -> Result<(Vec<f32>, Vec<f32>)> {
        let width = self.base.query.len();
        let n = self.base.candidate_cues.len();
        if q.len() != width * 153
            || k.len() != self.base.cues.len() * width * 153
            || g.len() != self.base.config.heads * n
            || q.iter().chain(k).chain(g).any(|x| !x.is_finite())
        {
            return Err(invalid("cue state adjoint shape/nonfinite"));
        }
        let mut qc = vec![0f64; q.len()];
        let mut kc = vec![0f64; k.len()];
        // Alias upstream credit is summed once for each authentic cue/head.
        let mut upstream = vec![0f64; self.base.config.heads * self.base.cues.len()];
        for h in 0..self.base.config.heads {
            for (i, s) in self.base.candidate_cues.iter().enumerate() {
                if let Some(s) = s {
                    upstream[h * self.base.cues.len() + s] += f64::from(g[h * n + i]);
                }
            }
        }
        for h in 0..self.base.config.heads {
            for s in 0..self.base.cues.len() {
                let adj = upstream[h * self.base.cues.len() + s];
                if adj == 0. {
                    continue;
                }
                for l in 0..self.base.config.lanes_per_head {
                    let lane = h * self.base.config.lanes_per_head + l;
                    for family in 0..2 {
                        let (start, classes) = if family == 0 { (0, 120) } else { (120, 33) };
                        for a in 0..classes {
                            let mut qq = self.base.query.clone();
                            let mut kk = self.base.cues[s].clone();
                            if family == 0 {
                                // Absent packets have canonical root1/bin0;
                                // root-only alternatives cannot reactivate them.
                                if qq[lane].present() {
                                    qq[lane] =
                                        AddressLane::new(a as u8, qq[lane].radius_bin(), true)
                                            .map_err(|e| invalid(e.to_string()))?;
                                }
                                if kk[lane].present() {
                                    kk[lane] =
                                        AddressLane::new(a as u8, kk[lane].radius_bin(), true)
                                            .map_err(|e| invalid(e.to_string()))?;
                                }
                            } else {
                                qq[lane] = observation(self.query_raw[lane], a)?;
                                kk[lane] = observation(self.cues_raw[s][lane], a)?;
                            }
                            qc[lane * 153 + start + a] +=
                                adj * self.score(h, &qq, &self.base.cues[s])? as f64 / Q24;
                            kc[(s * width + lane) * 153 + start + a] +=
                                adj * self.score(h, &self.base.query, &kk)? as f64 / Q24;
                        }
                    }
                }
            }
        }
        let pull = |logits: &[f32], credit: &mut [f64]| {
            let mut out = vec![0f64; logits.len()];
            for at in (0..logits.len()).step_by(153) {
                for (start, classes) in [(0, 120), (120, 33)] {
                    let row = &mut credit[at + start..at + start + classes];
                    let anchor = row[0];
                    for x in row.iter_mut() {
                        *x -= anchor;
                    }
                    choice_pullback(
                        &logits[at + start..at + start + classes],
                        row,
                        &mut out[at + start..at + start + classes],
                    );
                }
            }
            out
        };
        Ok((checked(pull(q, &mut qc))?, checked(pull(k, &mut kc))?))
    }
}
impl CustomOp2 for CueStateCredit {
    fn name(&self) -> &'static str {
        "frozen-cue-native-root120-category33-state-credit"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let q = contiguous(a, la)?;
        let k = contiguous(b, lb)?;
        if q.len() != self.base.query.len() * 153
            || k.len() != self.base.cues.len() * self.base.query.len() * 153
            || q.iter().chain(k).any(|x| !x.is_finite())
        {
            candle_core::bail!("cue state forward input differs");
        }
        Ok((
            CpuStorage::F32(
                self.base
                    .hard
                    .iter()
                    .flatten()
                    .map(|&x| (x as f64 / Q24) as f32)
                    .collect(),
            ),
            Shape::from((self.base.config.heads, self.base.candidate_cues.len())),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        _: &Tensor,
        g: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (dq, dk) = self
            .backward(
                &a.flatten_all()?.to_vec1::<f32>()?,
                &b.flatten_all()?.to_vec1::<f32>()?,
                &g.flatten_all()?.to_vec1::<f32>()?,
            )
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        Ok((
            Some(Tensor::from_vec(dq, a.shape(), a.device())?),
            Some(Tensor::from_vec(dk, b.shape(), b.device())?),
        ))
    }
}

fn validate_cue_state_joint_payload(
    joint: Option<&uor_r4_integer::geometric_cue_carrier::CueJointQ4>,
    expected: &Option<uor_r4_integer::geometric_cue_carrier::CueJointMetadata>,
) -> Result<()> {
    if &joint.map(|j| j.metadata()) != expected {
        return Err(invalid("cue state joint payload/metadata differs"));
    }
    Ok(())
}

#[cfg(test)]
mod cue_state_credit_tests {
    use super::*;
    use uor_r4_integer::geometric_cue_carrier::{CueJointConfig, CueJointQ4};
    fn fixture(
        joint_enabled: bool,
        query_absent: bool,
        cue_absent: bool,
    ) -> Result<CueStateCredit> {
        let qraw = vec![4, 7];
        let kraw = vec![9, 13];
        let q = qraw
            .iter()
            .map(|&r| observation(r, if query_absent { 0 } else { 4 }))
            .collect::<Result<Vec<_>>>()?;
        let k = kraw
            .iter()
            .map(|&r| observation(r, if cue_absent { 0 } else { 3 }))
            .collect::<Result<Vec<_>>>()?;
        let c = CueJointConfig {
            head: 0,
            left_lane: 0,
            right_lane: 1,
        };
        let quarters = (0..16).map(|i| (i % 15) as i8 - 7).collect::<Vec<_>>();
        let packed = geometric_potential_q4::pack_coefficients(&quarters)
            .map_err(|e| invalid(e.to_string()))?;
        let jt = geometric_potential_q4::compile_ordered_pair_q4(&packed)
            .map_err(|e| invalid(e.to_string()))?;
        let mut op = CueStateCredit {
            base: CueRootCredit {
                config: CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 2,
                    mode: CueScoreMode::DirectedRelative,
                },
                coefficients: vec![0; 240],
                algebra: HistoricalH4Tables::from_bytes(ALGEBRA)
                    .map_err(|e| invalid(e.to_string()))?,
                query: q,
                cues: vec![k],
                candidate_cues: vec![Some(0), Some(0), None],
                hard: vec![vec![0; 3]],
            },
            query_raw: qraw,
            cues_raw: vec![kraw],
            joint: if joint_enabled { Some((c, jt)) } else { None },
        };
        let v = op.score(0, &op.base.query, &op.base.cues[0])?;
        op.base.hard[0] = vec![v, v, 0];
        Ok(op)
    }
    #[test]
    fn cue_state_joint_ordered_lanes_query_cue_credit_and_exact_forward() -> Result<()> {
        let op = fixture(true, false, false)?;
        let q = Tensor::zeros((2, 153), DType::F32, &Device::Cpu)?;
        let k = Tensor::zeros((1, 2, 153), DType::F32, &Device::Cpu)?;
        let hard = op.base.hard.clone();
        let (dq, dk) = op.backward(&vec![0.; 306], &vec![0.; 306], &[1., 0., 0.])?;
        for lane in 0..2 {
            assert!(dq[lane * 153..lane * 153 + 120]
                .iter()
                .any(|x| x.abs() > 1e-8));
            assert!(dk[lane * 153..lane * 153 + 120]
                .iter()
                .any(|x| x.abs() > 1e-8));
        }
        let a = op
            .base
            .algebra
            .relative(
                code(op.base.query[0].root())?,
                code(op.base.cues[0][0].root())?,
            )
            .index();
        let b = op
            .base
            .algebra
            .relative(
                code(op.base.query[1].root())?,
                code(op.base.cues[0][1].root())?,
            )
            .index();
        let (_, table) = op
            .joint
            .as_ref()
            .ok_or_else(|| invalid("fixture joint absent"))?;
        assert_eq!(
            hard[0][0],
            i64::from(table[(usize::from(a) << 7) + usize::from(b)])
        );
        assert_eq!(
            q.apply_op2(&k, op)?.to_vec2::<f32>()?,
            vec![hard[0]
                .iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>()]
        );
        Ok(())
    }
    #[test]
    fn cue_state_category_absence_reactivation_uses_raw_roots() -> Result<()> {
        // One absent lane leaves the other joint lane factual/present so the
        // single-category counterfactual can reveal its causal influence.
        for query_side in [true, false] {
            let mut op = fixture(true, false, false)?;
            if query_side {
                op.base.query[0] = observation(op.query_raw[0], 0)?;
            } else {
                op.base.cues[0][0] = observation(op.cues_raw[0][0], 0)?;
            }
            let (dq, dk) = op.backward(&vec![0.; 306], &vec![0.; 306], &[1., 0., 0.])?;
            let credit = if query_side { &dq } else { &dk };
            assert!(credit[120..153].iter().any(|x| x.abs() > 1e-8));
            let mut restored_q = op.base.query.clone();
            let mut restored_k = op.base.cues[0].clone();
            if query_side {
                restored_q[0] = observation(op.query_raw[0], 1)?;
            } else {
                restored_k[0] = observation(op.cues_raw[0][0], 1)?;
            }
            let factual_present = fixture(true, false, false)?;
            assert_eq!(
                op.score(0, &restored_q, &restored_k)?,
                factual_present.score(
                    0,
                    &factual_present.base.query,
                    &factual_present.base.cues[0]
                )?
            );
        }
        // Both lanes absent require a simultaneous intervention; this declared
        // first-order conditional surrogate correctly has no joint-only credit.
        let op = fixture(true, true, false)?;
        let (dq, dk) = op.backward(&vec![0.; 306], &vec![0.; 306], &[1., 0., 0.])?;
        assert!(dq.iter().chain(&dk).all(|&x| x == 0.));
        Ok(())
    }
    #[test]
    fn cue_state_aliases_sum_before_pullback_and_unary_query_zero() -> Result<()> {
        let op = fixture(true, false, false)?;
        let (a, b) = op.backward(&vec![0.; 306], &vec![0.; 306], &[1., 2., 7.])?;
        let (x, y) = op.backward(&vec![0.; 306], &vec![0.; 306], &[3., 0., 0.])?;
        assert_eq!(a, x);
        assert_eq!(b, y);
        let mut unary = fixture(false, false, false)?;
        unary.base.config.mode = CueScoreMode::CueUnary;
        unary.base.coefficients = (0..240).map(|i| (i % 15) as i8 - 7).collect();
        let (q, k) = unary.backward(&vec![0.; 306], &vec![0.; 306], &[1., 0., 0.])?;
        for lane in 0..2 {
            assert!(q[lane * 153..lane * 153 + 120].iter().all(|&x| x == 0.));
        }
        assert!(k.iter().any(|&x| x != 0.));
        Ok(())
    }
    #[test]
    fn cue_state_joint_payload_binding_rejects_mismatch() -> Result<()> {
        let c = CueJointConfig {
            head: 0,
            left_lane: 0,
            right_lane: 1,
        };
        let a = CueJointQ4::new(c, vec![0; 8]).map_err(|e| invalid(e.to_string()))?;
        let b = CueJointQ4::new(c, vec![0x11; 8]).map_err(|e| invalid(e.to_string()))?;
        validate_cue_state_joint_payload(Some(&a), &Some(a.metadata()))?;
        assert!(validate_cue_state_joint_payload(Some(&b), &Some(a.metadata())).is_err());
        assert!(validate_cue_state_joint_payload(None, &Some(a.metadata())).is_err());
        assert!(validate_cue_state_joint_payload(Some(&a), &None).is_err());
        Ok(())
    }
}

// CUDA credit uses ordinary differentiable Candle operations. Frozen native
// trace/table utility vectors are staged from CPU once per call. Dynamic logits,
// normalization, alias gathers and all adjoints stay on the live device. CPU
// CustomOps remain the numerical reference; GPU F32 reduction differences are
// measured with explicit tolerances, not called bitwise gradient parity.
fn gathered_device_utility(
    logits: &Tensor,
    classes: usize,
    indices: &[u32],
    utility: Vec<f32>,
    edges: usize,
    lanes: usize,
    normalized: bool,
) -> Result<Tensor> {
    if indices.len() != edges * lanes || utility.len() != edges * lanes * classes {
        return Err(invalid("device credit static utility shape differs"));
    }
    let device = logits.device();
    if edges == 0 {
        return Ok(Tensor::zeros(0, DType::F32, device)?);
    }
    let choices = logits.reshape((logits.elem_count() / classes, classes))?;
    let choices = if normalized {
        candle_nn::ops::softmax(&choices, 1)?
    } else {
        choices
    };
    let index = Tensor::from_vec(indices.to_vec(), indices.len(), device)?;
    let selected = choices.index_select(&index, 0)?;
    let table = Tensor::from_vec(utility, (edges * lanes, classes), device)?;
    Ok((selected * table)?
        .sum(1)?
        .reshape((edges, lanes))?
        .sum(1)?)
}
fn device_hard_anchor(hard: &[i64], credit: Tensor) -> Result<Tensor> {
    let authoritative = Tensor::from_vec(
        hard.iter()
            .map(|&x| (x as f64 / Q24) as f32)
            .collect::<Vec<_>>(),
        hard.len(),
        credit.device(),
    )?;
    Ok((authoritative + (&credit - credit.detach())?)?)
}
impl NoReadCredit {
    fn device_graph(&self, latent: &Tensor, categories: &Tensor) -> Result<Tensor> {
        if !latent.device().same_device(categories.device()) {
            return Err(invalid("NoRead graph devices differ"));
        }
        let d = self.d;
        let edges = d.rows() * d.heads;
        let lanes = d.width();
        let coefficient_width = 1 + self.vocabulary + lanes * 42;
        let mut index = Vec::with_capacity(edges * lanes);
        let mut ambient = Vec::with_capacity(edges * lanes * 4);
        let mut categorical = Vec::with_capacity(edges * lanes * 33);
        // Output order is B,H,T, matching exact native hard vectors.
        for b in 0..d.batch {
            for h in 0..d.heads {
                for t in 0..d.time {
                    let start = h * coefficient_width + 1 + self.vocabulary;
                    for lane in 0..lanes {
                        index.push(((b * d.time + t) * lanes + lane) as u32);
                        ambient.extend(
                            self.coefficients[start + lane * 4..start + (lane + 1) * 4]
                                .iter()
                                .map(|&x| x as f32),
                        );
                        let row = &self.coefficients
                            [start + lanes * 4 + lane * 33..start + lanes * 4 + (lane + 1) * 33];
                        let anchor = row[0];
                        categorical.extend(row.iter().map(|&x| (x - anchor) as f32));
                    }
                }
            }
        }
        let credit = (gathered_device_utility(latent, 4, &index, ambient, edges, lanes, false)?
            + gathered_device_utility(categories, 33, &index, categorical, edges, lanes, true)?)?;
        Ok(device_hard_anchor(&self.hard, credit)?.reshape((d.batch, d.heads, d.time))?)
    }
}
impl PotentialCredit {
    fn device_graph(&self, roots: &Tensor, categories: &Tensor) -> Result<Tensor> {
        if !roots.device().same_device(categories.device()) {
            return Err(invalid("potential graph devices differ"));
        }
        let d = self.d;
        let edges = d.batch * d.heads * d.time * (d.time + 1) / 2;
        let lanes = d.lanes;
        let mut qi_indices = Vec::with_capacity(edges * lanes);
        let mut ki_indices = Vec::with_capacity(edges * lanes);
        let mut qr = Vec::with_capacity(edges * lanes * 120);
        let mut kr = Vec::with_capacity(edges * lanes * 120);
        let mut qc = Vec::with_capacity(edges * lanes * 33);
        let mut kc = Vec::with_capacity(edges * lanes * 33);
        let mut output = Vec::with_capacity(edges);
        for b in 0..d.batch {
            for h in 0..d.heads {
                for q in 0..d.time {
                    for k in 0..=q {
                        output.push((((b * d.heads + h) * d.time + q) * d.time + k) as u32);
                        for l in 0..lanes {
                            let lane = h * lanes + l;
                            let qi = d.index(b, q, h, l);
                            let ki = d.index(b, k, h, l);
                            qi_indices.push(qi as u32);
                            ki_indices.push(ki as u32);
                            let rq = self.context[qi];
                            let rk = self.context[ki];
                            let mut aq = [0f64; 4];
                            let mut ak = [0f64; 4];
                            if q != k && rq.present() && rk.present() {
                                let mut v = std::array::from_fn(|i| self.coefficient(1, lane, i));
                                if self.content[qi].present() && self.content[ki].present() {
                                    let dc = root(
                                        self.algebra
                                            .relative(
                                                code(self.content[qi].root())?,
                                                code(self.content[ki].root())?,
                                            )
                                            .index(),
                                    );
                                    for j in 0..4 {
                                        for (i, &x) in dc.iter().enumerate() {
                                            v[j] += x * self.coefficient(2, lane, i * 4 + j);
                                        }
                                    }
                                }
                                let a = root(rq.root());
                                let z = root(rk.root());
                                for axis in 0..4 {
                                    let mut unit = [0.; 4];
                                    unit[axis] = 1.;
                                    aq[axis] = dot(v, hamilton(conjugate(unit), z));
                                    ak[axis] = dot(v, hamilton(conjugate(a), unit));
                                }
                            }
                            let qanchor = dot(aq, root(0));
                            let kanchor = dot(ak, root(0));
                            for r in 0..120 {
                                qr.push((dot(aq, root(r)) - qanchor) as f32);
                                kr.push((dot(ak, root(r)) - kanchor) as f32);
                            }
                            let qanchor = self.lane_score(
                                lane,
                                qi,
                                ki,
                                observation(self.raw_roots[qi], 0)?,
                                if q == k {
                                    observation(self.raw_roots[qi], 0)?
                                } else {
                                    rk
                                },
                            )?;
                            let kanchor = if q == k {
                                0.
                            } else {
                                self.lane_score(
                                    lane,
                                    qi,
                                    ki,
                                    rq,
                                    observation(self.raw_roots[ki], 0)?,
                                )?
                            };
                            for category in 0..33 {
                                let qalt = observation(self.raw_roots[qi], category)?;
                                qc.push(
                                    (self.lane_score(
                                        lane,
                                        qi,
                                        ki,
                                        qalt,
                                        if q == k { qalt } else { rk },
                                    )? - qanchor) as f32,
                                );
                                kc.push(if q == k {
                                    0.
                                } else {
                                    (self.lane_score(
                                        lane,
                                        qi,
                                        ki,
                                        rq,
                                        observation(self.raw_roots[ki], category)?,
                                    )? - kanchor) as f32
                                });
                            }
                        }
                    }
                }
            }
        }
        let credit =
            (((gathered_device_utility(roots, 120, &qi_indices, qr, edges, lanes, true)?
                + gathered_device_utility(roots, 120, &ki_indices, kr, edges, lanes, true)?)?
                + gathered_device_utility(categories, 33, &qi_indices, qc, edges, lanes, true)?)?
                + gathered_device_utility(categories, 33, &ki_indices, kc, edges, lanes, true)?)?;
        let indices = Tensor::from_vec(output, edges, roots.device())?;
        let full = Tensor::zeros(self.hard.len(), DType::F32, roots.device())?
            .index_add(&indices, &credit, 0)?;
        Ok(device_hard_anchor(&self.hard, full)?.reshape((d.batch, d.heads, d.time, d.time))?)
    }
}
impl CueStateCredit {
    fn device_graph(&self, q: &Tensor, k: &Tensor) -> Result<Tensor> {
        if !q.device().same_device(k.device()) {
            return Err(invalid("cue state graph devices differ"));
        }
        let width = self.base.query.len();
        let heads = self.base.config.heads;
        let lanes = self.base.config.lanes_per_head;
        let sources = self.base.cues.len();
        let edges = heads * sources;
        let mut qi = Vec::with_capacity(edges * lanes);
        let mut ki = Vec::with_capacity(edges * lanes);
        let mut qr = Vec::with_capacity(edges * lanes * 120);
        let mut kr = Vec::with_capacity(edges * lanes * 120);
        let mut qc = Vec::with_capacity(edges * lanes * 33);
        let mut kc = Vec::with_capacity(edges * lanes * 33);
        for h in 0..heads {
            for s in 0..sources {
                for l in 0..lanes {
                    let lane = h * lanes + l;
                    qi.push(lane as u32);
                    ki.push((s * width + lane) as u32);
                    for (classes, root_family) in [(120, true), (33, false)] {
                        let mut qrow = Vec::with_capacity(classes);
                        let mut krow = Vec::with_capacity(classes);
                        for a in 0..classes {
                            let mut qq = self.base.query.clone();
                            let mut kk = self.base.cues[s].clone();
                            if root_family {
                                if qq[lane].present() {
                                    qq[lane] =
                                        AddressLane::new(a as u8, qq[lane].radius_bin(), true)
                                            .map_err(|e| invalid(e.to_string()))?;
                                }
                                if kk[lane].present() {
                                    kk[lane] =
                                        AddressLane::new(a as u8, kk[lane].radius_bin(), true)
                                            .map_err(|e| invalid(e.to_string()))?;
                                }
                            } else {
                                qq[lane] = observation(self.query_raw[lane], a)?;
                                kk[lane] = observation(self.cues_raw[s][lane], a)?;
                            }
                            qrow.push(self.score(h, &qq, &self.base.cues[s])? as f64 / Q24);
                            krow.push(self.score(h, &self.base.query, &kk)? as f64 / Q24);
                        }
                        let qa = qrow[0];
                        let ka = krow[0];
                        let qrow = qrow.into_iter().map(|x| (x - qa) as f32);
                        let krow = krow.into_iter().map(|x| (x - ka) as f32);
                        if root_family {
                            qr.extend(qrow);
                            kr.extend(krow);
                        } else {
                            qc.extend(qrow);
                            kc.extend(krow);
                        }
                    }
                }
            }
        }
        let qroots = q.narrow(1, 0, 120)?.contiguous()?;
        let qcats = q.narrow(1, 120, 33)?.contiguous()?;
        let kroots = k.narrow(2, 0, 120)?.contiguous()?;
        let kcats = k.narrow(2, 120, 33)?.contiguous()?;
        let credit = (((gathered_device_utility(&qroots, 120, &qi, qr, edges, lanes, true)?
            + gathered_device_utility(&kroots, 120, &ki, kr, edges, lanes, true)?)?
            + gathered_device_utility(&qcats, 33, &qi, qc, edges, lanes, true)?)?
            + gathered_device_utility(&kcats, 33, &ki, kc, edges, lanes, true)?)?
        .reshape((heads, sources))?;
        // One authentic cue utility reused by all its candidate aliases. Index
        // selection backward performs the alias accumulation on GPU.
        let credit = Tensor::cat(
            &[&credit, &Tensor::zeros((heads, 1), DType::F32, q.device())?],
            1,
        )?;
        let aliases = self
            .base
            .candidate_cues
            .iter()
            .map(|s| s.unwrap_or(sources) as u32)
            .collect::<Vec<_>>();
        let n = aliases.len();
        let aliases = Tensor::from_vec(aliases, n, q.device())?;
        let chosen = credit.index_select(&aliases, 1)?.flatten_all()?;
        let hard = self.base.hard.iter().flatten().copied().collect::<Vec<_>>();
        Ok(device_hard_anchor(&hard, chosen)?.reshape((heads, n))?)
    }
}

impl CueRootCredit {
    fn device_graph(&self, q: &Tensor, k: &Tensor) -> Result<Tensor> {
        if !q.device().same_device(k.device()) {
            return Err(invalid("cue root graph devices differ"));
        }
        let heads = self.config.heads;
        let lanes = self.config.lanes_per_head;
        let sources = self.cues.len();
        let edges = heads * sources;
        let width = self.query.len();
        let mut qi = Vec::with_capacity(edges * lanes);
        let mut ki = Vec::with_capacity(edges * lanes);
        let mut qr = Vec::with_capacity(edges * lanes * 120);
        let mut kr = Vec::with_capacity(edges * lanes * 120);
        for h in 0..heads {
            for s in 0..sources {
                for l in 0..lanes {
                    let lane = h * lanes + l;
                    qi.push(lane as u32);
                    ki.push((s * width + lane) as u32);
                    let rq = self.query[lane];
                    let rk = self.cues[s][lane];
                    let mut qrow = vec![0f64; 120];
                    let mut krow = vec![0f64; 120];
                    if rq.present() && rk.present() {
                        for a in 0..120 {
                            qrow[a] = self.score(
                                lane,
                                AddressLane::new(a as u8, rq.radius_bin(), true)
                                    .map_err(|e| invalid(e.to_string()))?,
                                rk,
                            )? as f64
                                / Q24;
                            krow[a] = self.score(
                                lane,
                                rq,
                                AddressLane::new(a as u8, rk.radius_bin(), true)
                                    .map_err(|e| invalid(e.to_string()))?,
                            )? as f64
                                / Q24;
                        }
                    }
                    let qa = qrow[0];
                    let ka = krow[0];
                    qr.extend(qrow.into_iter().map(|x| (x - qa) as f32));
                    kr.extend(krow.into_iter().map(|x| (x - ka) as f32));
                }
            }
        }
        let credit = (gathered_device_utility(q, 120, &qi, qr, edges, lanes, true)?
            + gathered_device_utility(k, 120, &ki, kr, edges, lanes, true)?)?
        .reshape((heads, sources))?;
        let credit = Tensor::cat(
            &[&credit, &Tensor::zeros((heads, 1), DType::F32, q.device())?],
            1,
        )?;
        let aliases = self
            .candidate_cues
            .iter()
            .map(|s| s.unwrap_or(sources) as u32)
            .collect::<Vec<_>>();
        let n = aliases.len();
        let chosen = credit
            .index_select(&Tensor::from_vec(aliases, n, q.device())?, 1)?
            .flatten_all()?;
        let hard = self.hard.iter().flatten().copied().collect::<Vec<_>>();
        Ok(device_hard_anchor(&hard, chosen)?.reshape((heads, n))?)
    }
}

#[cfg(all(test, feature = "cuda"))]
mod cuda_credit_parity_tests {
    use super::*;
    use candle_core::Var;
    fn close(actual: &[f32], expected: &[f32]) -> Result<()> {
        if actual.len() != expected.len() {
            return Err(invalid("CUDA credit parity lengths differ"));
        }
        for (&a, &e) in actual.iter().zip(expected) {
            if !a.is_finite() || (a - e).abs() > 2e-5 + 2e-5 * e.abs() {
                return Err(invalid(format!("CUDA credit adjoint mismatch {a} vs {e}")));
            }
        }
        Ok(())
    }
    fn check<F>(
        device: &Device,
        a: Vec<f32>,
        b: Vec<f32>,
        ashape: &[usize],
        bshape: &[usize],
        hard: &[i64],
        da: Vec<f32>,
        db: Vec<f32>,
        graph: F,
    ) -> Result<()>
    where
        F: FnOnce(&Tensor, &Tensor) -> Result<Tensor>,
    {
        let av = Var::from_vec(a, ashape, device)?;
        let bv = Var::from_vec(b, bshape, device)?;
        let output = graph(av.as_tensor(), bv.as_tensor())?;
        assert!(output.device().same_device(device));
        assert_eq!(
            output.flatten_all()?.to_vec1::<f32>()?,
            hard.iter()
                .map(|&v| (v as f64 / Q24) as f32)
                .collect::<Vec<_>>()
        );
        let grad = output.sum_all()?.backward()?;
        close(
            &grad
                .get(av.as_tensor())
                .ok_or_else(|| invalid("CUDA first credit graph disconnected"))?
                .flatten_all()?
                .to_vec1::<f32>()?,
            &da,
        )?;
        close(
            &grad
                .get(bv.as_tensor())
                .ok_or_else(|| invalid("CUDA second credit graph disconnected"))?
                .flatten_all()?
                .to_vec1::<f32>()?,
            &db,
        )?;
        Ok(())
    }
    #[test]
    fn cuda_frozen_no_read_potential_and_cue_credit_match_cpu_reference() -> Result<()> {
        // Explicit CUDA request: absence is an error, never a passing skip.
        let device = Device::new_cuda(0)?;
        let d = Dimensions {
            batch: 1,
            time: 2,
            heads: 1,
            lanes: 1,
        };
        let logits = |count: usize| {
            (0..count)
                .map(|i| ((i % 17) as f32 - 8.) * 0.07)
                .collect::<Vec<_>>()
        };
        let nr = NoReadCredit {
            d,
            vocabulary: 2,
            coefficients: (0..45).map(|i| ((i % 7) as f64 - 3.) * 0.25).collect(),
            hard: vec![1 << 22, -2 << 22],
        };
        let a = logits(8);
        let b = logits(66);
        let (da, db) = nr.backward(&b, &[1., 1.])?;
        check(
            &device,
            a,
            b,
            &[1, 2, 1, 1, 4],
            &[1, 2, 1, 1, 33],
            &nr.hard,
            da,
            db,
            |a, b| nr.device_graph(a, b),
        )?;
        let config = PotentialQ4Config {
            heads: 1,
            lanes_per_head: 1,
        };
        let q = (0..config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?)
            .map(|i| (i % 15) as i8 - 7)
            .collect::<Vec<_>>();
        let packed =
            geometric_potential_q4::pack_coefficients(&q).map_err(|e| invalid(e.to_string()))?;
        let present = AddressLane::new(10, 4, true).map_err(|e| invalid(e.to_string()))?;
        let absent = AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?;
        let pot = PotentialCredit::new(
            d,
            vec![absent; 2],
            vec![
                present,
                AddressLane::new(20, 6, true).map_err(|e| invalid(e.to_string()))?,
            ],
            vec![10, 20],
            &packed,
        )?;
        let a = logits(240);
        let b = logits(66);
        let (da, db) = pot.backward(&a, &b, &[1.; 4])?;
        check(
            &device,
            a,
            b,
            &[1, 2, 1, 1, 120],
            &[1, 2, 1, 1, 33],
            &pot.hard,
            da,
            db,
            |a, b| pot.device_graph(a, b),
        )?;
        let base = CueRootCredit {
            config: CueAngularConfig {
                heads: 1,
                lanes_per_head: 1,
                mode: CueScoreMode::DirectedRelative,
            },
            coefficients: (0..120).map(|i| (i % 15) as i8 - 7).collect(),
            algebra: HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?,
            query: vec![present],
            cues: vec![vec![present], vec![absent]],
            candidate_cues: vec![Some(0), Some(0), Some(1), None],
            hard: vec![vec![0; 4]],
        };
        let cue = CueStateCredit {
            base,
            query_raw: vec![10],
            cues_raw: vec![vec![10], vec![30]],
            joint: None,
        };
        let a = logits(153);
        let b = logits(306);
        let (da, db) = cue.backward(&a, &b, &[1.; 4])?;
        let hard = cue.base.hard.iter().flatten().copied().collect::<Vec<_>>();
        check(
            &device,
            a,
            b,
            &[1, 153],
            &[2, 1, 153],
            &hard,
            da,
            db,
            |a, b| cue.device_graph(a, b),
        )?;
        Ok(())
    }
}
