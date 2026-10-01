//! Separated content/context geometric address scores, offline CPU training only.
//!
//! The current context `u[t]` and OLD retained content `h[t-1]` are split into
//! contiguous four-coordinate lanes; there are no learned query/key maps here.
//! Each nonzero lane is placed at the signed nearest canonical 2I root (largest
//! signed dot; lowest canonical index wins ties). The hard relative element is
//! exactly table[inverse(query), key], with all 120 signed IDs retained. Content
//! and context stay separate. Their relative root coordinates receive learned
//! unary linear potentials and a 4-by-4 bilinear cross potential. These small
//! offline potentials could be evaluated on the finite domain and compiled into
//! tables later; this module neither exports nor qualifies four-bit serving.
//!
//! Each channel also has a 32-by-32 radial table and four presence coefficients.
//! Radius bins are powers 2^e, e=-16..15. Assignment minimizes PHYSICAL radius
//! distance, midpoint ties choose the smaller exponent, endpoints clip. Exactly
//! zero is a separate presence flag, never a semantic identity-root observation.
//! Direction/radius terms require both endpoints present; the presence state is
//! 2*query_present+key_present and is always scored.
//!
//! Backward is an explicit surrogate, not the derivative of discrete placement:
//! * direction: use the derivative of x/sqrt(|x|^2+eps^2), eps=2^-20, followed by
//!   smooth conjugate(query)*key evaluated at the HARD endpoint roots, then the
//!   potential derivative at the exact HARD relative root;
//! * radius: locally bilinear-interpolate detached radial coefficients between
//!   neighboring powers in physical radius; this supplies INPUT derivatives
//!   only. Exactly the hard selected cell receives the coefficient derivative.
//!   Input derivatives are zero at zero and at/outside both clipped endpoints;
//!   internal exact powers use the interval to their right;
//! * presence: hard Boolean, no surrogate derivative.
//! These rules avoid doubled table gradients. Higher-order gradients are not
//! implemented. Quantization and potential parameters remain F32 training data.
//!
//! The output is [batch, heads, query_time, key_time] before age/causal masking,
//! NoRead normalization or value mixing; those remain the caller's single path.
//! Saved config binds canonical F64 root bits, inverse/product bytes, ordering,
//! radius rules and surrogate version. A complete integer reader also needs an
//! actual compiled discrete code producer; float hidden states are not serving.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use candle_core::{CpuStorage, CustomOp3, DType, Layout, Shape, Tensor};
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::{
    embedding::canonical_h4_roots,
    group_table::{group_table, GROUP_ORDER, ROW_STRIDE},
};

use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-address/1";
pub const RADIUS_MIN_EXPONENT: i32 = -16;
pub const RADIUS_BINS: usize = 32;
pub const NORM_EPSILON: f64 = 1.0 / 1_048_576.0;
const RADIUS_RULE: &str = "physical-nearest-dyadic-midpoint-lower-clip[-16,15]-zero-separate/1";
const SURROGATE_RULE: &str = "hard-root-hamilton-normalized-ste;hard-radius-coefficient/local-physical-input;presence-stop/1";
const CONTENT_UNARY: usize = 0;
const CONTEXT_UNARY: usize = 4;
const PAIR: usize = 8;
const CONTENT_RADIUS: usize = 24;
const CONTEXT_RADIUS: usize = CONTENT_RADIUS + RADIUS_BINS * RADIUS_BINS;
const CONTENT_PRESENCE: usize = CONTEXT_RADIUS + RADIUS_BINS * RADIUS_BINS;
const CONTEXT_PRESENCE: usize = CONTENT_PRESENCE + 4;
const PARAMETERS_PER_LANE: usize = CONTEXT_PRESENCE + 4;
const NAMES: [&str; 7] = [
    "read.address.content_unary",
    "read.address.context_unary",
    "read.address.pair",
    "read.address.content_radius",
    "read.address.context_radius",
    "read.address.content_presence",
    "read.address.context_presence",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricAddressConfig {
    pub schema: String,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub root_table_sha256: String,
    pub radius_min_exponent: i32,
    pub radius_bins: usize,
    pub norm_epsilon: f64,
    pub radius_rule: String,
    pub surrogate_rule: String,
}

impl GeometricAddressConfig {
    pub fn new(width: usize, heads: usize) -> Result<Self> {
        if heads == 0 || heads > 64 || width == 0 || width % (4 * heads) != 0 {
            return Err(invalid(
                "geometric address requires width divisible by 4*heads, heads1..64",
            ));
        }
        let lanes_per_head = width / (4 * heads);
        if lanes_per_head > 32 {
            return Err(invalid("geometric address requires 1..32 lanes per head"));
        }
        Ok(Self {
            schema: SCHEMA.into(),
            heads,
            lanes_per_head,
            root_table_sha256: geometry_digest().to_owned(),
            radius_min_exponent: RADIUS_MIN_EXPONENT,
            radius_bins: RADIUS_BINS,
            norm_epsilon: NORM_EPSILON,
            radius_rule: RADIUS_RULE.into(),
            surrogate_rule: SURROGATE_RULE.into(),
        })
    }

    pub fn validate(&self, width: usize, heads: usize) -> Result<()> {
        if *self != Self::new(width, heads)? {
            return Err(invalid(
                "geometric address config, root ordering or surrogate binding differs",
            ));
        }
        Ok(())
    }
}

fn roots() -> &'static [[f64; 4]; GROUP_ORDER] {
    static ROOTS: OnceLock<[[f64; 4]; GROUP_ORDER]> = OnceLock::new();
    ROOTS.get_or_init(|| std::array::from_fn(|i| canonical_h4_roots()[i].to_array()))
}

/// Binds the existing canonical root ordering and exact finite multiplication.
pub fn geometry_digest() -> &'static str {
    static DIGEST: OnceLock<String> = OnceLock::new();
    DIGEST
        .get_or_init(|| {
            let table = group_table();
            let mut bytes = b"uor-r4.geometric-address.roots-and-algebra/1".to_vec();
            for root in roots() {
                for value in root {
                    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
                }
            }
            bytes.extend_from_slice(&(GROUP_ORDER as u32).to_le_bytes());
            bytes.extend_from_slice(&(ROW_STRIDE as u32).to_le_bytes());
            bytes.push(table.identity);
            bytes.extend_from_slice(&table.inverse);
            bytes.extend_from_slice(&table.product);
            sha256_bytes(&bytes)
        })
        .as_str()
}

pub fn parameter_shapes(width: usize, heads: usize) -> Result<BTreeMap<String, Vec<usize>>> {
    let c = GeometricAddressConfig::new(width, heads)?;
    let (h, l) = (c.heads, c.lanes_per_head);
    Ok(BTreeMap::from([
        (NAMES[0].into(), vec![h, l, 4]),
        (NAMES[1].into(), vec![h, l, 4]),
        (NAMES[2].into(), vec![h, l, 4, 4]),
        (NAMES[3].into(), vec![h, l, RADIUS_BINS, RADIUS_BINS]),
        (NAMES[4].into(), vec![h, l, RADIUS_BINS, RADIUS_BINS]),
        (NAMES[5].into(), vec![h, l, 4]),
        (NAMES[6].into(), vec![h, l, 4]),
    ]))
}

pub struct AddressWeights {
    pub content_unary: Tensor,
    pub context_unary: Tensor,
    pub pair: Tensor,
    pub content_radius: Tensor,
    pub context_radius: Tensor,
    pub content_presence: Tensor,
    pub context_presence: Tensor,
}

impl AddressWeights {
    fn pack(&self, config: &GeometricAddressConfig) -> Result<Tensor> {
        let h = config.heads;
        let l = config.lanes_per_head;
        let tensors = [
            &self.content_unary,
            &self.context_unary,
            &self.pair,
            &self.content_radius,
            &self.context_radius,
            &self.content_presence,
            &self.context_presence,
        ];
        let shapes = [
            vec![h, l, 4],
            vec![h, l, 4],
            vec![h, l, 4, 4],
            vec![h, l, 32, 32],
            vec![h, l, 32, 32],
            vec![h, l, 4],
            vec![h, l, 4],
        ];
        let mut packed = Vec::with_capacity(7);
        for (tensor, shape) in tensors.iter().zip(shapes) {
            if tensor.dims() != shape || tensor.dtype() != DType::F32 || !tensor.device().is_cpu() {
                return Err(invalid(
                    "geometric address potential shape/dtype/device differs",
                ));
            }
            packed.push(tensor.reshape((h, l, shape[2..].iter().product::<usize>()))?);
        }
        Ok(Tensor::cat(&packed, 2)?.contiguous()?)
    }
}

/// Hard lane code; bin0 is only a storage placeholder when `present == false`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressCode {
    pub root: u8,
    pub radius_bin: u8,
    pub present: bool,
}

#[derive(Clone, Copy, Debug)]
struct Lane {
    x: [f64; 4],
    radius: f64,
    code: AddressCode,
}

fn power(bin: usize) -> f64 {
    2f64.powi(RADIUS_MIN_EXPONENT + bin as i32)
}

fn lane(x: [f32; 4]) -> candle_core::Result<Lane> {
    if x.iter().any(|v| !v.is_finite()) {
        candle_core::bail!("nonfinite geometric address input");
    }
    let x = x.map(f64::from);
    let radius = x.iter().map(|v| v * v).sum::<f64>().sqrt();
    let present = radius != 0.;
    let mut root = group_table().identity;
    let mut radius_bin = 0u8;
    if present {
        let mut best_dot = f64::NEG_INFINITY;
        for (i, candidate) in roots().iter().enumerate() {
            let dot = (0..4).map(|j| x[j] * candidate[j]).sum::<f64>();
            if dot > best_dot {
                best_dot = dot;
                root = i as u8;
            }
        }
        if radius >= power(RADIUS_BINS - 1) {
            radius_bin = (RADIUS_BINS - 1) as u8;
        } else if radius > power(0) {
            let mut error = f64::INFINITY;
            for i in 0..RADIUS_BINS {
                let candidate = (radius - power(i)).abs();
                if candidate < error {
                    error = candidate;
                    radius_bin = i as u8;
                }
            }
        }
    }
    Ok(Lane {
        x,
        radius,
        code: AddressCode {
            root,
            radius_bin,
            present,
        },
    })
}

pub fn encode_lane(coordinates: [f32; 4]) -> Result<AddressCode> {
    Ok(lane(coordinates)?.code)
}

fn encode(values: &[f32]) -> candle_core::Result<Vec<Lane>> {
    if values.len() % 4 != 0 {
        candle_core::bail!("geometric address lane width");
    }
    values
        .chunks_exact(4)
        .map(|x| lane([x[0], x[1], x[2], x[3]]))
        .collect()
}

fn relative(query: Lane, key: Lane) -> [f64; 4] {
    let table = group_table();
    let code = table.product[usize::from(table.inverse[usize::from(query.code.root)]) * ROW_STRIDE
        + usize::from(key.code.root)];
    roots()[usize::from(code)]
}

fn presence(query: Lane, key: Lane) -> usize {
    2 * usize::from(query.code.present) + usize::from(key.code.present)
}

fn both(query: Lane, key: Lane) -> bool {
    query.code.present && key.code.present
}

fn hard_radius(weights: &[f32], query: Lane, key: Lane) -> f64 {
    if both(query, key) {
        f64::from(
            weights[usize::from(query.code.radius_bin) * RADIUS_BINS
                + usize::from(key.code.radius_bin)],
        )
    } else {
        0.
    }
}

fn hard_score(w: &[f32], cq: Lane, ck: Lane, rq: Lane, rk: Lane) -> f64 {
    let (dc, dr) = (relative(cq, ck), relative(rq, rk));
    let mut out = f64::from(w[CONTENT_PRESENCE + presence(cq, ck)])
        + f64::from(w[CONTEXT_PRESENCE + presence(rq, rk)]);
    if both(cq, ck) {
        out += (0..4)
            .map(|i| f64::from(w[CONTENT_UNARY + i]) * dc[i])
            .sum::<f64>();
        out += hard_radius(&w[CONTENT_RADIUS..CONTEXT_RADIUS], cq, ck);
    }
    if both(rq, rk) {
        out += (0..4)
            .map(|i| f64::from(w[CONTEXT_UNARY + i]) * dr[i])
            .sum::<f64>();
        out += hard_radius(&w[CONTEXT_RADIUS..CONTENT_PRESENCE], rq, rk);
    }
    if both(cq, ck) && both(rq, rk) {
        for i in 0..4 {
            for j in 0..4 {
                out += f64::from(w[PAIR + i * 4 + j]) * dc[i] * dr[j];
            }
        }
    }
    out
}

// Adjacent powers and physical interpolation derivative. At/outside clipped
// endpoints the local input surrogate is constant; zero has no radius state.
fn interval(radius: f64) -> (usize, usize, f64, f64) {
    if radius <= power(0) {
        return (0, 0, 0., 0.);
    }
    if radius >= power(31) {
        return (31, 31, 0., 0.);
    }
    for lo in 0..31 {
        if radius < power(lo + 1) {
            let delta = power(lo + 1) - power(lo);
            return (lo, lo + 1, (radius - power(lo)) / delta, 1. / delta);
        }
    }
    (31, 31, 0., 0.)
}

fn radial_surrogate(w: &[f32], q: Lane, k: Lane) -> (f64, f64, f64) {
    if !both(q, k) {
        return (0., 0., 0.);
    }
    let (ql, qh, qa, qd) = interval(q.radius);
    let (kl, kh, ka, kd) = interval(k.radius);
    let v00 = f64::from(w[ql * 32 + kl]);
    let v01 = f64::from(w[ql * 32 + kh]);
    let v10 = f64::from(w[qh * 32 + kl]);
    let v11 = f64::from(w[qh * 32 + kh]);
    let low = (1. - ka) * v00 + ka * v01;
    let high = (1. - ka) * v10 + ka * v11;
    (
        (1. - qa) * low + qa * high,
        qd * (high - low),
        kd * ((1. - qa) * (v01 - v00) + qa * (v11 - v10)),
    )
}

fn normalization_pullback(value: Lane, gradient: [f64; 4]) -> [f64; 4] {
    if !value.code.present {
        return [0.; 4];
    }
    let d = (value.radius * value.radius + NORM_EPSILON * NORM_EPSILON).sqrt();
    let inner = (0..4).map(|i| value.x[i] * gradient[i]).sum::<f64>();
    std::array::from_fn(|i| gradient[i] / d - value.x[i] * inner / (d * d * d))
}

// Jacobians of conjugate(q)*k evaluated at hard endpoint root coordinates.
fn relative_pullback(q: Lane, k: Lane, g: [f64; 4]) -> ([f64; 4], [f64; 4]) {
    let a = roots()[usize::from(q.code.root)];
    let b = roots()[usize::from(k.code.root)];
    let dq = [
        g[0] * b[0] + g[1] * b[1] + g[2] * b[2] + g[3] * b[3],
        g[0] * b[1] - g[1] * b[0] + g[2] * b[3] - g[3] * b[2],
        g[0] * b[2] - g[1] * b[3] - g[2] * b[0] + g[3] * b[1],
        g[0] * b[3] + g[1] * b[2] - g[2] * b[1] - g[3] * b[0],
    ];
    let dk = [
        g[0] * a[0] - g[1] * a[1] - g[2] * a[2] - g[3] * a[3],
        g[0] * a[1] + g[1] * a[0] - g[2] * a[3] + g[3] * a[2],
        g[0] * a[2] + g[1] * a[3] + g[2] * a[0] - g[3] * a[1],
        g[0] * a[3] - g[1] * a[2] + g[2] * a[1] + g[3] * a[0],
    ];
    (normalization_pullback(q, dq), normalization_pullback(k, dk))
}

fn add_radius(g: &mut [f64; 4], value: Lane, derivative: f64) {
    if value.code.present && derivative != 0. {
        for (i, part) in g.iter_mut().enumerate() {
            *part += derivative * value.x[i] / value.radius;
        }
    }
}

struct Pullback {
    cq: [f64; 4],
    ck: [f64; 4],
    rq: [f64; 4],
    rk: [f64; 4],
}

fn pullback(
    w: &[f32],
    dw: &mut [f64],
    cq: Lane,
    ck: Lane,
    rq: Lane,
    rk: Lane,
    scale: f64,
) -> Pullback {
    let (dc, dr) = (relative(cq, ck), relative(rq, rk));
    let (mut gc, mut gr) = ([0f64; 4], [0f64; 4]);
    dw[CONTENT_PRESENCE + presence(cq, ck)] += scale;
    dw[CONTEXT_PRESENCE + presence(rq, rk)] += scale;
    if both(cq, ck) {
        for i in 0..4 {
            dw[CONTENT_UNARY + i] += scale * dc[i];
            gc[i] += scale * f64::from(w[CONTENT_UNARY + i]);
        }
    }
    if both(rq, rk) {
        for i in 0..4 {
            dw[CONTEXT_UNARY + i] += scale * dr[i];
            gr[i] += scale * f64::from(w[CONTEXT_UNARY + i]);
        }
    }
    if both(cq, ck) && both(rq, rk) {
        for i in 0..4 {
            for j in 0..4 {
                dw[PAIR + i * 4 + j] += scale * dc[i] * dr[j];
                gc[i] += scale * f64::from(w[PAIR + i * 4 + j]) * dr[j];
                gr[j] += scale * f64::from(w[PAIR + i * 4 + j]) * dc[i];
            }
        }
    }
    let (mut gcq, mut gck) = relative_pullback(cq, ck, gc);
    let (mut grq, mut grk) = relative_pullback(rq, rk, gr);
    for (offset, q, k, gq, gk) in [
        (CONTENT_RADIUS, cq, ck, &mut gcq, &mut gck),
        (CONTEXT_RADIUS, rq, rk, &mut grq, &mut grk),
    ] {
        if both(q, k) {
            dw[offset + usize::from(q.code.radius_bin) * 32 + usize::from(k.code.radius_bin)] +=
                scale;
            let (_, dq, dk) = radial_surrogate(&w[offset..offset + 32 * 32], q, k);
            add_radius(gq, q, scale * dq);
            add_radius(gk, k, scale * dk);
        }
    }
    Pullback {
        cq: gcq,
        ck: gck,
        rq: grq,
        rk: grk,
    }
}

fn contiguous<'a>(storage: &'a CpuStorage, layout: &Layout) -> candle_core::Result<&'a [f32]> {
    let values = storage.as_slice::<f32>()?;
    match layout.contiguous_offsets() {
        Some((start, end)) => Ok(&values[start..end]),
        None => candle_core::bail!("geometric address needs contiguous CPU F32 inputs"),
    }
}

struct AddressOp {
    batch: usize,
    time: usize,
    heads: usize,
    lanes: usize,
}
impl AddressOp {
    fn row(&self, b: usize, t: usize, h: usize, l: usize) -> usize {
        ((b * self.time + t) * self.heads + h) * self.lanes + l
    }
    fn validate(&self, current: &[f32], prior: &[f32], weights: &[f32]) -> candle_core::Result<()> {
        if current.len() != self.batch * self.time * self.heads * self.lanes * 4
            || prior.len() != current.len()
            || weights.len() != self.heads * self.lanes * PARAMETERS_PER_LANE
        {
            candle_core::bail!("geometric address packed shape differs");
        }
        if weights.iter().any(|x| !x.is_finite()) {
            candle_core::bail!("nonfinite geometric address potential");
        }
        Ok(())
    }
}

impl CustomOp3 for AddressOp {
    fn name(&self) -> &'static str {
        "geometric-address-paired-relative"
    }
    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
        s3: &CpuStorage,
        l3: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (current, prior, weights) = (
            contiguous(s1, l1)?,
            contiguous(s2, l2)?,
            contiguous(s3, l3)?,
        );
        self.validate(current, prior, weights)?;
        let (r, c) = (encode(current)?, encode(prior)?);
        let mut out = vec![0f32; self.batch * self.heads * self.time * self.time];
        for b in 0..self.batch {
            for h in 0..self.heads {
                for t in 0..self.time {
                    for j in 0..self.time {
                        let mut value = 0.;
                        for l in 0..self.lanes {
                            let q = self.row(b, t, h, l);
                            let k = self.row(b, j, h, l);
                            let offset = (h * self.lanes + l) * PARAMETERS_PER_LANE;
                            value += hard_score(
                                &weights[offset..offset + PARAMETERS_PER_LANE],
                                c[q],
                                c[k],
                                r[q],
                                r[k],
                            );
                        }
                        let value = value as f32;
                        if !value.is_finite() {
                            candle_core::bail!("nonfinite geometric address score");
                        }
                        out[((b * self.heads + h) * self.time + t) * self.time + j] = value;
                    }
                }
            }
        }
        Ok((
            CpuStorage::F32(out),
            Shape::from((self.batch, self.heads, self.time, self.time)),
        ))
    }
    fn bwd(
        &self,
        current: &Tensor,
        prior: &Tensor,
        weights: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        let u = current.flatten_all()?.to_vec1::<f32>()?;
        let p = prior.flatten_all()?.to_vec1::<f32>()?;
        let w = weights.flatten_all()?.to_vec1::<f32>()?;
        let ds = grad.flatten_all()?.to_vec1::<f32>()?;
        self.validate(&u, &p, &w)?;
        if ds.len() != self.batch * self.heads * self.time * self.time
            || ds.iter().any(|x| !x.is_finite())
        {
            candle_core::bail!("nonfinite or invalid geometric address upstream gradient");
        }
        let (r, c) = (encode(&u)?, encode(&p)?);
        let (mut du, mut dp, mut dw) = (
            vec![0f64; u.len()],
            vec![0f64; p.len()],
            vec![0f64; w.len()],
        );
        for b in 0..self.batch {
            for h in 0..self.heads {
                for t in 0..self.time {
                    for j in 0..self.time {
                        let scale =
                            f64::from(ds[((b * self.heads + h) * self.time + t) * self.time + j]);
                        if scale == 0. {
                            continue;
                        }
                        for l in 0..self.lanes {
                            let q = self.row(b, t, h, l);
                            let k = self.row(b, j, h, l);
                            let offset = (h * self.lanes + l) * PARAMETERS_PER_LANE;
                            let d = pullback(
                                &w[offset..offset + PARAMETERS_PER_LANE],
                                &mut dw[offset..offset + PARAMETERS_PER_LANE],
                                c[q],
                                c[k],
                                r[q],
                                r[k],
                                scale,
                            );
                            for i in 0..4 {
                                dp[q * 4 + i] += d.cq[i];
                                dp[k * 4 + i] += d.ck[i];
                                du[q * 4 + i] += d.rq[i];
                                du[k * 4 + i] += d.rk[i];
                            }
                        }
                    }
                }
            }
        }
        let checked = |values: Vec<f64>| -> candle_core::Result<Vec<f32>> {
            let values = values.into_iter().map(|x| x as f32).collect::<Vec<_>>();
            if values.iter().any(|x| !x.is_finite()) {
                candle_core::bail!("nonfinite geometric address surrogate gradient");
            }
            Ok(values)
        };
        Ok((
            Some(Tensor::from_vec(
                checked(du)?,
                current.shape(),
                current.device(),
            )?),
            Some(Tensor::from_vec(
                checked(dp)?,
                prior.shape(),
                prior.device(),
            )?),
            Some(Tensor::from_vec(
                checked(dw)?,
                weights.shape(),
                weights.device(),
            )?),
        ))
    }
}

pub fn score(
    current: &Tensor,
    prior: &Tensor,
    config: &GeometricAddressConfig,
    weights: &AddressWeights,
) -> Result<Tensor> {
    let (batch, time, width) = current.dims3()?;
    config.validate(width, config.heads)?;
    if batch == 0
        || time == 0
        || prior.shape() != current.shape()
        || current.dtype() != DType::F32
        || prior.dtype() != DType::F32
        || !current.device().is_cpu()
        || !prior.device().is_cpu()
    {
        return Err(invalid(
            "geometric address requires nonempty matching CPU F32 [B,T,W] inputs",
        ));
    }
    let packed = weights.pack(config)?;
    Ok(current.contiguous()?.apply_op3(
        &prior.contiguous()?,
        &packed,
        AddressOp {
            batch,
            time,
            heads: config.heads,
            lanes: config.lanes_per_head,
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Var};

    fn weights() -> Vec<f32> {
        (0..PARAMETERS_PER_LANE)
            .map(|i| ((i % 17) as f32 - 8.) * 0.03)
            .collect()
    }
    fn axis(value: f32) -> Lane {
        lane([value, 0., 0., 0.]).unwrap()
    }
    fn inputs() -> [Lane; 4] {
        [
            lane([0.9, 0.2, -0.1, 0.3]).unwrap(),
            lane([1.6, -0.2, 0.4, 0.1]).unwrap(),
            lane([0.3, 1.1, 0.2, -0.1]).unwrap(),
            lane([1.1, 0.4, -0.3, 0.2]).unwrap(),
        ]
    }

    #[test]
    fn geometric_address_codes_bind_signed_roots_radius_zero_and_config() {
        for (i, root) in roots().iter().enumerate() {
            assert_eq!(encode_lane(root.map(|x| x as f32)).unwrap().root, i as u8);
        }
        assert_ne!(axis(1.).code.root, axis(-1.).code.root);
        assert_eq!(axis(1.).code.radius_bin, 16);
        assert_eq!(axis(1.5).code.radius_bin, 16); // physical midpoint, tie lower
        assert_eq!(axis(1.5001).code.radius_bin, 17);
        assert_eq!(axis(2.).code.radius_bin, 17);
        assert_eq!(axis(f32::MAX).code.radius_bin, 31);
        assert_eq!(axis(f32::from_bits(1)).code.radius_bin, 0);
        assert!(!axis(0.).code.present);
        assert!(axis(f32::from_bits(1)).code.present);
        assert_eq!(axis(0.).code.root, group_table().identity);
        assert!(encode_lane([f32::NAN, 0., 0., 0.]).is_err());
        let config = GeometricAddressConfig::new(32, 2).unwrap();
        let bytes = serde_json::to_vec(&config).unwrap();
        let mut loaded: GeometricAddressConfig = serde_json::from_slice(&bytes).unwrap();
        loaded.validate(32, 2).unwrap();
        loaded.root_table_sha256.push('0');
        assert!(loaded.validate(32, 2).is_err());
        assert!(GeometricAddressConfig::new(30, 2).is_err());
        assert_eq!(parameter_shapes(32, 2).unwrap()[NAMES[2]], vec![2, 4, 4, 4]);
    }

    #[test]
    fn geometric_address_directed_relation_and_cross_factor_are_load_bearing() {
        let (one, minus) = (axis(1.), axis(-1.));
        let mut w = vec![0f32; PARAMETERS_PER_LANE];
        w[PAIR] = 1.;
        let table = [
            hard_score(&w, one, one, one, one),
            hard_score(&w, one, one, one, minus),
            hard_score(&w, one, minus, one, one),
            hard_score(&w, one, minus, one, minus),
        ];
        assert_eq!(table, [1., -1., -1., 1.]);
        // Independent unary functions always have zero mixed difference.
        assert_eq!(table[0] + table[3] - table[1] - table[2], 4.);
        w[PAIR] = 0.;
        w[CONTENT_UNARY + 3] = 1.;
        let i = lane([0., 1., 0., 0.]).unwrap();
        let j = lane([0., 0., 1., 0.]).unwrap();
        assert_eq!(hard_score(&w, i, j, one, one), -1.);
        assert_eq!(hard_score(&w, j, i, one, one), 1.);
        w[CONTENT_UNARY + 3] = 0.;
        w[CONTENT_RADIUS + 16 * 32 + 17] = 3.;
        w[CONTENT_PRESENCE + 1] = 7.;
        assert_eq!(hard_score(&w, one, axis(2.), one, one), 3.);
        assert_eq!(hard_score(&w, axis(0.), axis(2.), one, one), 7.);
    }

    fn conjugate_product(q: [f64; 4], k: [f64; 4]) -> [f64; 4] {
        [
            q[0] * k[0] + q[1] * k[1] + q[2] * k[2] + q[3] * k[3],
            q[0] * k[1] - q[1] * k[0] - q[2] * k[3] + q[3] * k[2],
            q[0] * k[2] + q[1] * k[3] - q[2] * k[0] - q[3] * k[1],
            q[0] * k[3] - q[1] * k[2] + q[2] * k[1] - q[3] * k[0],
        ]
    }
    fn moved_root(base: Lane, x: [f64; 4]) -> [f64; 4] {
        let d = (x.iter().map(|v| v * v).sum::<f64>() + NORM_EPSILON * NORM_EPSILON).sqrt();
        let original = (base.radius * base.radius + NORM_EPSILON * NORM_EPSILON).sqrt();
        let root = roots()[usize::from(base.code.root)];
        std::array::from_fn(|i| root[i] + x[i] / d - base.x[i] / original)
    }
    fn frozen_relative(q: Lane, k: Lane, qx: [f64; 4], kx: [f64; 4]) -> [f64; 4] {
        let hard = relative(q, k);
        let at_base = conjugate_product(
            roots()[usize::from(q.code.root)],
            roots()[usize::from(k.code.root)],
        );
        let moved = conjugate_product(moved_root(q, qx), moved_root(k, kx));
        std::array::from_fn(|i| hard[i] + moved[i] - at_base[i])
    }
    // This is the documented local surrogate with codes/coefficients frozen,
    // NOT a finite difference of the piecewise-constant hard quantizer.
    fn surrogate(w: &[f32], base: [Lane; 4], coordinates: [[f64; 4]; 4]) -> f64 {
        let dc = frozen_relative(base[0], base[1], coordinates[0], coordinates[1]);
        let dr = frozen_relative(base[2], base[3], coordinates[2], coordinates[3]);
        let mut result = 0.;
        for i in 0..4 {
            result +=
                f64::from(w[CONTENT_UNARY + i]) * dc[i] + f64::from(w[CONTEXT_UNARY + i]) * dr[i];
            for j in 0..4 {
                result += f64::from(w[PAIR + i * 4 + j]) * dc[i] * dr[j];
            }
        }
        for (offset, begin) in [(CONTENT_RADIUS, 0), (CONTEXT_RADIUS, 2)] {
            let updated = |i: usize| Lane {
                x: coordinates[i],
                radius: coordinates[i].iter().map(|v| v * v).sum::<f64>().sqrt(),
                code: base[i].code,
            };
            result += radial_surrogate(
                &w[offset..offset + 1024],
                updated(begin),
                updated(begin + 1),
            )
            .0;
        }
        result
    }

    #[test]
    fn geometric_address_input_surrogate_matches_frozen_branch_finite_difference() {
        let w = weights();
        let base = inputs();
        let [cq, ck, rq, rk] = base;
        let mut dw = vec![0.; PARAMETERS_PER_LANE];
        let d = pullback(&w, &mut dw, cq, ck, rq, rk, 1.);
        let gradients = [d.cq, d.ck, d.rq, d.rk];
        let original = base.map(|v| v.x);
        for endpoint in 0..4 {
            for coordinate in 0..4 {
                let (mut plus, mut minus) = (original, original);
                plus[endpoint][coordinate] += 1e-5;
                minus[endpoint][coordinate] -= 1e-5;
                let numerical = (surrogate(&w, base, plus) - surrogate(&w, base, minus)) / (2e-5);
                assert!(
                    (numerical - gradients[endpoint][coordinate]).abs() < 2e-7,
                    "endpoint{endpoint} coordinate{coordinate}: numerical{numerical} analytic{}",
                    gradients[endpoint][coordinate]
                );
            }
        }
        for gradient in gradients {
            assert!(gradient.iter().any(|v| v.abs() > 1e-7));
        }
        // Hard coefficient gradients are exact, including only the selected radial cell.
        let selected =
            CONTENT_RADIUS + usize::from(cq.code.radius_bin) * 32 + usize::from(ck.code.radius_bin);
        assert_eq!(dw[selected], 1.);
        assert_eq!(
            dw[CONTENT_RADIUS..CONTEXT_RADIUS]
                .iter()
                .filter(|x| **x != 0.)
                .count(),
            1
        );
        for index in [
            CONTENT_UNARY + 1,
            CONTEXT_UNARY + 2,
            PAIR + 7,
            selected,
            CONTENT_PRESENCE + 3,
        ] {
            let (mut plus, mut minus) = (w.clone(), w.clone());
            plus[index] += 0.001;
            minus[index] -= 0.001;
            let numerical = (hard_score(&plus, cq, ck, rq, rk)
                - hard_score(&minus, cq, ck, rq, rk))
                / f64::from(plus[index] - minus[index]);
            assert!((numerical - dw[index]).abs() < 2e-7);
        }
    }

    #[test]
    fn geometric_address_radial_surrogate_does_not_double_coefficients_or_move_zero() {
        let mut w = vec![0f32; PARAMETERS_PER_LANE];
        for q in 0..32 {
            for k in 0..32 {
                w[CONTENT_RADIUS + q * 32 + k] = (q + 2 * k) as f32;
            }
        }
        let (q, k, r) = (axis(1.2), axis(1.7), axis(1.));
        let mut dw = vec![0.; PARAMETERS_PER_LANE];
        let d = pullback(&w, &mut dw, q, k, r, r, 1.);
        assert!((d.cq[0] - 1.).abs() < 1e-10);
        assert!((d.ck[0] - 2.).abs() < 1e-10);
        assert_eq!(dw[CONTENT_RADIUS..CONTEXT_RADIUS].iter().sum::<f64>(), 1.);
        for q in [
            axis(0.),
            axis(f32::from_bits(1)),
            axis(32768.),
            axis(f32::MAX),
        ] {
            let mut dw = vec![0.; PARAMETERS_PER_LANE];
            let d = pullback(&w, &mut dw, q, k, r, r, 1.);
            assert_eq!(d.cq, [0.; 4]);
            if !q.code.present {
                assert_eq!(dw[CONTENT_RADIUS..CONTEXT_RADIUS].iter().sum::<f64>(), 0.);
            }
        }
    }

    fn weight_vars(w: &[f32]) -> (AddressWeights, Vec<Var>) {
        let specs = [
            (0, 4, vec![1, 1, 4]),
            (4, 8, vec![1, 1, 4]),
            (8, 24, vec![1, 1, 4, 4]),
            (24, 1048, vec![1, 1, 32, 32]),
            (1048, 2072, vec![1, 1, 32, 32]),
            (2072, 2076, vec![1, 1, 4]),
            (2076, 2080, vec![1, 1, 4]),
        ];
        let vars = specs
            .iter()
            .map(|(a, b, shape)| {
                Var::from_vec(w[*a..*b].to_vec(), shape.as_slice(), &Device::Cpu).unwrap()
            })
            .collect::<Vec<_>>();
        let tensors = AddressWeights {
            content_unary: vars[0].as_tensor().clone(),
            context_unary: vars[1].as_tensor().clone(),
            pair: vars[2].as_tensor().clone(),
            content_radius: vars[3].as_tensor().clone(),
            context_radius: vars[4].as_tensor().clone(),
            content_presence: vars[5].as_tensor().clone(),
            context_presence: vars[6].as_tensor().clone(),
        };
        (tensors, vars)
    }

    #[test]
    fn geometric_address_autograd_reaches_both_inputs_and_all_seven_tensors() {
        let base = inputs();
        let w = weights();
        let (weights, vars) = weight_vars(&w);
        let current = Var::from_vec(
            base[2..]
                .iter()
                .flat_map(|v| v.x.map(|x| x as f32))
                .collect::<Vec<_>>(),
            (1, 2, 4),
            &Device::Cpu,
        )
        .unwrap();
        let prior = Var::from_vec(
            base[..2]
                .iter()
                .flat_map(|v| v.x.map(|x| x as f32))
                .collect::<Vec<_>>(),
            (1, 2, 4),
            &Device::Cpu,
        )
        .unwrap();
        let config = GeometricAddressConfig::new(4, 1).unwrap();
        let result = score(&current, &prior, &config, &weights).unwrap();
        assert_eq!(result.dims(), [1, 1, 2, 2]);
        let selected = result
            .get(0)
            .unwrap()
            .get(0)
            .unwrap()
            .get(0)
            .unwrap()
            .get(1)
            .unwrap();
        assert!(
            (f64::from(selected.to_scalar::<f32>().unwrap())
                - hard_score(&w, base[0], base[1], base[2], base[3]))
            .abs()
                < 1e-6
        );
        let grads = selected.backward().unwrap();
        for variable in [&current, &prior].into_iter().chain(vars.iter()) {
            let gradient = grads
                .get(variable)
                .unwrap()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap();
            assert!(gradient.iter().all(|x| x.is_finite()));
            assert!(gradient.iter().any(|x| x.abs() > 1e-7));
        }
        let mut expected_weights = vec![0.; PARAMETERS_PER_LANE];
        let expected = pullback(
            &w,
            &mut expected_weights,
            base[0],
            base[1],
            base[2],
            base[3],
            1.,
        );
        let actual = grads
            .get(&prior)
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        for (a, b) in actual
            .iter()
            .zip(expected.cq.into_iter().chain(expected.ck))
        {
            assert!((f64::from(*a) - b).abs() < 1e-6);
        }
        // An upstream causal consumer can exclude a future slot exactly: this
        // scorer itself neither applies a second mask nor invents a NoRead term.
        let masked = result
            .narrow(2, 1, 1)
            .unwrap()
            .narrow(3, 0, 1)
            .unwrap()
            .sum_all()
            .unwrap();
        assert!(masked.backward().is_ok());
    }
}
