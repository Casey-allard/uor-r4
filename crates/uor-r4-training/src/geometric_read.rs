//! Optional finite geometric read kernel: a learned signed relative-group score.
//!
//! This module implements the first operator designed in
//! `docs/integration/fourth-lab-geometric-attention-2026-09-26.md` and frozen in
//! `docs/integration/geometric-read-plan-2026-09-27.md`: an absent-by-default
//! replacement for the dense query/key dot score. For each of 16 four-coordinate
//! lanes the query and key are L2-normalized, quantized to the signed nearest
//! element of the 120-root binary icosahedral group `2I`, composed through the
//! exact bound product/inverse table, and scored by a small per-lane MLP.
//!
//! Forward execution uses the exact hard relation; backward passes through a
//! smooth Hamilton composition and the score derivative (a straight-through
//! surrogate). The module is offline F32 training/evaluation only: quantization,
//! packed export, integer serving, precision views and rounding are rejected.
//!
//! The kernel is selected only for `AdmissionPolicy::Full` and the retained
//! `Dot` geometry, so the existing bounded-admission and radial paths are
//! unchanged when it is disabled. With the kernel off this module contributes
//! nothing to any serialized contract, parameter set or graph.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use candle_core::{DType, Device, Tensor, Var};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;
use uor_r4_core::native_geometric::learner::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};

use crate::joint_model::{Initializer, JointConfig};
use crate::{invalid, Result};

/// Versioned kernel configuration identifier.
pub const GEOMETRIC_READ_SCHEMA: &str = "uor-r4.geometric-read/1";
/// Fixed lane count for the 64-coordinate read width.
pub const GEOMETRIC_READ_LANES: usize = 16;
/// Fixed hidden width of each per-lane MLP.
pub const GEOMETRIC_READ_HIDDEN: usize = 8;
/// Coordinates per lane.
pub const GEOMETRIC_READ_LANE_WIDTH: usize = 4;
/// Default L2 normalization epsilon.
pub const DEFAULT_NORM_EPS: f64 = 1e-6;
/// Default uniform initialization bound for both MLP layers.
pub const DEFAULT_INIT_SCALE: f64 = 0.55;
/// Domain separator so kernel draws cannot replay the base parameter stream.
pub const KERNEL_SEED_DOMAIN: u64 = 0x9E37_79B9_7F4A_7C15;

/// Per-lane first-layer weight.
pub const RELATION_WEIGHT1: &str = "read.relation.weight1";
/// Per-lane first-layer bias.
pub const RELATION_BIAS1: &str = "read.relation.bias1";
/// Per-lane second-layer weight.
pub const RELATION_WEIGHT2: &str = "read.relation.weight2";
/// Per-lane second-layer bias.
pub const RELATION_BIAS2: &str = "read.relation.bias2";

fn default_lanes() -> usize {
    GEOMETRIC_READ_LANES
}
fn default_hidden() -> usize {
    GEOMETRIC_READ_HIDDEN
}
fn default_norm_eps() -> f64 {
    DEFAULT_NORM_EPS
}
fn default_init_scale() -> f64 {
    DEFAULT_INIT_SCALE
}

/// Versioned configuration of the optional finite geometric read kernel.
///
/// `lanes` and `hidden` are fixed by the frozen plan; `norm_eps` and
/// `init_scale` retain their declared defaults. [`Self::validate`] rejects any
/// other lane/hidden count, a non-positive epsilon or a non-positive scale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricReadConfig {
    pub schema: String,
    #[serde(default = "default_lanes")]
    pub lanes: usize,
    #[serde(default = "default_hidden")]
    pub hidden: usize,
    #[serde(default = "default_norm_eps")]
    pub norm_eps: f64,
    #[serde(default = "default_init_scale")]
    pub init_scale: f64,
}

impl Default for GeometricReadConfig {
    fn default() -> Self {
        Self {
            schema: GEOMETRIC_READ_SCHEMA.to_owned(),
            lanes: GEOMETRIC_READ_LANES,
            hidden: GEOMETRIC_READ_HIDDEN,
            norm_eps: DEFAULT_NORM_EPS,
            init_scale: DEFAULT_INIT_SCALE,
        }
    }
}

impl GeometricReadConfig {
    /// The frozen signed-2I kernel configuration.
    pub fn signed_2i() -> Self {
        Self::default()
    }

    /// Reject any configuration outside the frozen contract.
    pub fn validate(&self) -> Result<()> {
        if self.schema != GEOMETRIC_READ_SCHEMA {
            return Err(invalid(
                "geometric read schema must be uor-r4.geometric-read/1",
            ));
        }
        if self.lanes != GEOMETRIC_READ_LANES || self.hidden != GEOMETRIC_READ_HIDDEN {
            return Err(invalid(
                "geometric read kernel requires 16 lanes and hidden width 8",
            ));
        }
        if !self.norm_eps.is_finite() || self.norm_eps <= 0.0 {
            return Err(invalid(
                "geometric read norm_eps must be positive and finite",
            ));
        }
        if !self.init_scale.is_finite() || self.init_scale <= 0.0 {
            return Err(invalid(
                "geometric read init_scale must be positive and finite",
            ));
        }
        Ok(())
    }
}

/// The four kernel parameter names in initialization order.
pub fn parameter_names() -> [&'static str; 4] {
    [
        RELATION_WEIGHT1,
        RELATION_BIAS1,
        RELATION_WEIGHT2,
        RELATION_BIAS2,
    ]
}

/// Kernel tensor shapes, keyed by parameter name.
pub fn kernel_shapes(kernel: &GeometricReadConfig) -> BTreeMap<String, Vec<usize>> {
    let (lanes, hidden) = (kernel.lanes, kernel.hidden);
    BTreeMap::from([
        (RELATION_WEIGHT1.to_owned(), vec![lanes, hidden, 4]),
        (RELATION_BIAS1.to_owned(), vec![lanes, hidden]),
        (RELATION_WEIGHT2.to_owned(), vec![lanes, 1, hidden]),
        (RELATION_BIAS2.to_owned(), vec![lanes, 1]),
    ])
}

/// Base `JointConfig` parameter shapes plus the kernel shapes when enabled.
///
/// The base inventory is exactly `config.shapes()`, so a kernel-free checkpoint
/// keeps its historical names, tensor bytes and numerical contract.
pub fn parameter_shapes(
    config: &JointConfig,
    kernel: Option<&GeometricReadConfig>,
) -> BTreeMap<String, Vec<usize>> {
    let mut shapes = config.shapes();
    if let Some(kernel) = kernel {
        shapes.extend(kernel_shapes(kernel));
    }
    shapes
}

/// Draw kernel tensors from a separate initializer stream.
///
/// Both weight layers are `U(-init_scale, init_scale)`; both biases are zero.
/// The domain-separated seed and the separate stream mean the base parameter
/// draw order and values are identical with the kernel on or off.
pub(crate) fn initialize_kernel_variables(
    kernel: &GeometricReadConfig,
    config: &JointConfig,
    device: &Device,
) -> Result<BTreeMap<String, Var>> {
    kernel.validate()?;
    let mut rng = Initializer(config.seed ^ KERNEL_SEED_DOMAIN);
    let scale = kernel.init_scale as f32;
    let mut variables = BTreeMap::new();
    for (name, shape) in kernel_shapes(kernel) {
        let count: usize = shape.iter().product();
        let mut values = vec![0f32; count];
        if name == RELATION_WEIGHT1 || name == RELATION_WEIGHT2 {
            for value in &mut values {
                *value = rng.symmetric() * scale;
            }
        }
        variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
    }
    Ok(variables)
}

/// The four kernel weight tensors for one score evaluation.
pub(crate) struct KernelWeights {
    pub weight1: Tensor,
    pub bias1: Tensor,
    pub weight2: Tensor,
    pub bias2: Tensor,
}

#[derive(Clone)]
struct GeometricReadUsage {
    key_counts: Vec<[u64; GROUP_ORDER]>,
    relation_counts: Vec<[u64; GROUP_ORDER]>,
    relation_total: u64,
    non_identity_relations: u64,
}

impl GeometricReadUsage {
    fn new(lanes: usize) -> Self {
        Self {
            key_counts: vec![[0u64; GROUP_ORDER]; lanes],
            relation_counts: vec![[0u64; GROUP_ORDER]; lanes],
            relation_total: 0,
            non_identity_relations: 0,
        }
    }
}

/// Detached hard-path caches for one read: integer key codes and their exact
/// root coordinates, both shaped `[batch, candidates, lanes]` (codes, `u32`)
/// and `[batch, candidates, lanes, 4]` (coordinates, `f32`). They are built
/// once per written key and gathered per read, so the 120-root nearest search
/// runs on the `batch * lanes` newly written keys rather than on every
/// candidate of every position. Forward semantics are unchanged: the codes and
/// coordinates equal a direct recompute on the same written rows.
#[derive(Clone)]
pub(crate) struct GeometricReadKeyCache {
    pub codes: Tensor,
    pub hard_coords: Tensor,
}

/// Cached immutable kernel state attached to a [`crate::JointModel`].
///
/// Holds the versioned configuration, the four parameter names, the cached
/// constant root tensor used to materialize hard coordinates, the flat exact
/// product table used to compose relative codes without a host loop, and
/// integer usage counters accumulated during enabled forwards. Cloning shares
/// the usage mailbox so prepared/detached views report one histogram.
#[derive(Clone)]
pub struct GeometricReadState {
    config: GeometricReadConfig,
    parameter_names: Vec<String>,
    roots: Box<[[f32; 4]; GROUP_ORDER]>,
    root_tensor: Tensor,
    /// Flat `product[a * ROW_STRIDE + b]` table as `u32`, gathered per relation.
    product: Tensor,
    usage: Arc<Mutex<GeometricReadUsage>>,
}

impl GeometricReadState {
    /// Build the cached kernel state on `device`.
    pub fn new(config: GeometricReadConfig, device: &Device) -> Result<Self> {
        config.validate()?;
        let canonical = canonical_h4_roots();
        let mut roots = [[0f32; 4]; GROUP_ORDER];
        let mut flat = Vec::with_capacity(GROUP_ORDER * 4);
        for (index, root) in canonical.iter().enumerate() {
            let array = root.to_array();
            roots[index] = [
                array[0] as f32,
                array[1] as f32,
                array[2] as f32,
                array[3] as f32,
            ];
            flat.extend_from_slice(&roots[index]);
        }
        let root_tensor = Tensor::from_vec(flat, (GROUP_ORDER, 4), device)?;
        let table = group_table();
        let product = Tensor::from_vec(
            table
                .product
                .iter()
                .map(|&code| code as u32)
                .collect::<Vec<u32>>(),
            (GROUP_ORDER * ROW_STRIDE,),
            device,
        )?;
        Ok(Self {
            config,
            parameter_names: parameter_names().iter().map(|n| (*n).to_owned()).collect(),
            roots: Box::new(roots),
            root_tensor,
            product,
            usage: Arc::new(Mutex::new(GeometricReadUsage::new(GEOMETRIC_READ_LANES))),
        })
    }

    /// The bound kernel configuration.
    pub fn config(&self) -> &GeometricReadConfig {
        &self.config
    }

    /// The four kernel parameter names.
    pub fn parameter_names(&self) -> &[String] {
        &self.parameter_names
    }

    /// Signed nearest-root quantization: `argmax` of the signed dot product,
    /// with a strict comparison so the lowest canonical index wins ties and
    /// antipodes stay distinct (never folded with `abs`).
    fn quantize_signed(&self, coordinates: [f32; 4]) -> usize {
        let mut best = 0usize;
        let mut best_dot = f32::NEG_INFINITY;
        for (index, root) in self.roots.iter().enumerate() {
            let dot = root[0] * coordinates[0]
                + root[1] * coordinates[1]
                + root[2] * coordinates[2]
                + root[3] * coordinates[3];
            if dot > best_dot {
                best_dot = dot;
                best = index;
            }
        }
        best
    }

    /// Quantize `rows` normalized lane rows of `[rows, lanes, 4]` to signed
    /// nearest-root codes on the host. Used for the query and for the
    /// write-time cache; the candidate history is never re-coded per read.
    fn quantize_flat(&self, host: &[f32], rows: usize) -> Vec<u32> {
        let lanes = self.config.lanes;
        let lane_width = GEOMETRIC_READ_LANE_WIDTH;
        let mut codes = vec![0u32; rows * lanes];
        for row in 0..rows {
            for lane in 0..lanes {
                let base = (row * lanes + lane) * lane_width;
                codes[row * lanes + lane] = self.quantize_signed([
                    host[base],
                    host[base + 1],
                    host[base + 2],
                    host[base + 3],
                ]) as u32;
            }
        }
        codes
    }

    /// Encode one written key batch `[batch, read_width]` into the detached
    /// hard-path cache: integer codes `[batch, 1, lanes]` and their exact root
    /// coordinates `[batch, 1, lanes, 4]`. The row uses the same 4-D lane
    /// normalization as the read path, so these codes equal a direct recompute
    /// of the corresponding row of the differentiable history.
    pub(crate) fn encode_keys(&self, key: &Tensor) -> Result<(Tensor, Tensor)> {
        let (batch, read_width) = key.dims2()?;
        let lanes = self.config.lanes;
        let lane_width = GEOMETRIC_READ_LANE_WIDTH;
        if read_width != lanes * lane_width {
            return Err(invalid("geometric read key shape mismatch"));
        }
        let device = key.device();
        let eps_sq = self.config.norm_eps * self.config.norm_eps;
        let key_lanes = key.reshape((batch, 1, lanes, lane_width))?.detach();
        let unit = normalize_lane_4d(&key_lanes, eps_sq, device)?;
        let host = unit.flatten_all()?.to_vec1::<f32>()?;
        let codes = self.quantize_flat(&host, batch);
        let codes_tensor = Tensor::from_vec(codes.clone(), (batch, 1, lanes), device)?;
        let coords = self
            .root_tensor
            .index_select(&Tensor::from_vec(codes, (batch * lanes,), device)?, 0)?
            .reshape((batch, 1, lanes, lane_width))?;
        Ok((codes_tensor, coords))
    }

    /// Learned signed relative-group score, shaped `[batch, candidates]`.
    ///
    /// When `caches` is present the hard branch reads the detached write-time
    /// codes and coordinates; when absent it recomputes them from `keys` with
    /// the identical quantizer. Both paths produce the same hard tensors, and
    /// the smooth branch always differentiates the current `keys` history.
    pub(crate) fn score(
        &self,
        query: &Tensor,
        keys: &Tensor,
        caches: Option<&GeometricReadKeyCache>,
        weights: &KernelWeights,
    ) -> Result<Tensor> {
        let device = query.device();
        let (batch, read_width) = query.dims2()?;
        let (key_batch, candidates, key_width) = keys.dims3()?;
        let lanes = self.config.lanes;
        let lane_width = GEOMETRIC_READ_LANE_WIDTH;
        if read_width != key_width
            || read_width != lanes * lane_width
            || key_batch != batch
            || weights.weight1.dims() != [lanes, self.config.hidden, lane_width].as_slice()
            || weights.bias1.dims() != [lanes, self.config.hidden].as_slice()
            || weights.weight2.dims() != [lanes, 1, self.config.hidden].as_slice()
            || weights.bias2.dims() != [lanes, 1].as_slice()
        {
            return Err(invalid("geometric read kernel shape mismatch"));
        }
        let eps_sq = self.config.norm_eps * self.config.norm_eps;
        let query_lanes = query.reshape((batch, lanes, lane_width))?;
        let key_lanes = keys.reshape((batch, candidates, lanes, lane_width))?;
        let query_unit = normalize_lane_3d(&query_lanes, eps_sq, device)?;
        let key_unit = normalize_lane_4d(&key_lanes, eps_sq, device)?;

        // Query coding is per position and small (batch * lanes).
        let query_host = query_unit.detach().flatten_all()?.to_vec1::<f32>()?;
        let query_codes = self.quantize_flat(&query_host, batch);

        // Hard key codes: the detached write-time cache when present, otherwise
        // the declared scalar recompute (reference and equality tests).
        let (key_codes, key_hard) = match caches {
            Some(cache)
                if cache.codes.dims() == [batch, candidates, lanes].as_slice()
                    && cache.hard_coords.dims()
                        == [batch, candidates, lanes, lane_width].as_slice() =>
            {
                (cache.codes.clone(), cache.hard_coords.clone())
            }
            Some(_) => return Err(invalid("geometric read key cache shape mismatch")),
            None => {
                let key_host = key_unit.detach().flatten_all()?.to_vec1::<f32>()?;
                let codes = self.quantize_flat(&key_host, batch * candidates);
                let codes_tensor =
                    Tensor::from_vec(codes.clone(), (batch, candidates, lanes), device)?;
                let hard = self
                    .root_tensor
                    .index_select(
                        &Tensor::from_vec(codes, (batch * candidates * lanes,), device)?,
                        0,
                    )?
                    .reshape((batch, candidates, lanes, lane_width))?;
                (codes_tensor, hard)
            }
        };

        // Query hard coordinates, then the exact relative code per candidate:
        // `inverse(q_code) * ROW_STRIDE + k_code` gathered from the flat product
        // table (no host loop over candidates).
        let query_hard = self
            .root_tensor
            .index_select(
                &Tensor::from_vec(query_codes.clone(), (batch * lanes,), device)?,
                0,
            )?
            .reshape((batch, lanes, lane_width))?;
        let table = group_table();
        let mut query_shift = vec![0u32; batch * lanes];
        for (row, lane_codes) in query_codes.chunks(lanes).enumerate() {
            for (lane, &code) in lane_codes.iter().enumerate() {
                query_shift[row * lanes + lane] =
                    table.inverse[code as usize] as u32 * ROW_STRIDE as u32;
            }
        }
        let relation_codes = self
            .product
            .index_select(
                &Tensor::from_vec(query_shift, (batch, lanes), device)?
                    .unsqueeze(1)?
                    .broadcast_add(&key_codes)?
                    .flatten_all()?,
                0,
            )?
            .reshape((batch, candidates, lanes))?;
        let relation_hard = self
            .root_tensor
            .index_select(&relation_codes.flatten_all()?, 0)?
            .reshape((batch, candidates, lanes, lane_width))?;
        self.record_usage(&key_codes, &relation_codes, batch * candidates)?;

        // Straight-through surrogates: forward is the exact hard value, the
        // derivative is the identity through the smooth branch.
        let query_st = query_unit.add(&query_hard.broadcast_sub(&query_unit.detach())?.detach())?;
        let key_st = key_unit.add(&key_hard.broadcast_sub(&key_unit.detach())?.detach())?;
        let smooth = smooth_relative(&query_st, &key_st)?;
        let relation_st = smooth.add(&relation_hard.sub(&smooth.detach())?.detach())?;

        // Batch every lane into one broadcast matmul pair instead of 16 sets of
        // narrow/matmul/broadcast_add launches. Lane accumulation order is kept
        // (0..lanes) so the sum is bitwise the declared per-lane sequence.
        let permuted = relation_st.permute((2, 0, 1, 3))?.contiguous()?;
        let flat = permuted.reshape((lanes, batch * candidates, lane_width))?;
        let hidden = flat
            .broadcast_matmul(&weights.weight1.transpose(1, 2)?)?
            .broadcast_add(&weights.bias1.unsqueeze(1)?)?
            .tanh()?;
        let lane_scores = hidden
            .broadcast_matmul(&weights.weight2.transpose(1, 2)?)?
            .broadcast_add(&weights.bias2.reshape((lanes, 1, 1))?)?;
        let mut total: Option<Tensor> = None;
        for lane in 0..lanes {
            let score = lane_scores.narrow(0, lane, 1)?.squeeze(0)?.contiguous()?;
            total = Some(match total {
                None => score,
                Some(previous) => previous.add(&score)?,
            });
        }
        Ok(total
            .ok_or_else(|| invalid("geometric read kernel has no lanes"))?
            .reshape((batch, candidates))?)
    }

    fn record_usage(&self, key_codes: &Tensor, relation_codes: &Tensor, rows: usize) -> Result<()> {
        let lanes = self.config.lanes;
        let identity = group_table().identity as usize;
        let key_host = key_codes.flatten_all()?.to_vec1::<u32>()?;
        let relation_host = relation_codes.flatten_all()?.to_vec1::<u32>()?;
        let mut usage = self
            .usage
            .lock()
            .map_err(|_| invalid("geometric read usage lock poisoned"))?;
        for index in 0..rows * lanes {
            let lane = index % lanes;
            usage.key_counts[lane][key_host[index] as usize] += 1;
            let relation = relation_host[index] as usize;
            usage.relation_counts[lane][relation] += 1;
            usage.relation_total += 1;
            if relation != identity {
                usage.non_identity_relations += 1;
            }
        }
        Ok(())
    }

    /// Integer-only hard-path usage snapshot. No tensors are retained.
    pub fn usage_snapshot(&self) -> Result<Value> {
        let usage = self
            .usage
            .lock()
            .map_err(|_| invalid("geometric read usage lock poisoned"))?;
        let relation_per_lane: Vec<usize> = usage
            .relation_counts
            .iter()
            .map(|row| row.iter().filter(|&&count| count > 0).count())
            .collect();
        let key_per_lane: Vec<usize> = usage
            .key_counts
            .iter()
            .map(|row| row.iter().filter(|&&count| count > 0).count())
            .collect();
        let mut occupied = [false; GROUP_ORDER];
        for row in &usage.relation_counts {
            for (index, &count) in row.iter().enumerate() {
                if count > 0 {
                    occupied[index] = true;
                }
            }
        }
        let occupied_codes = occupied.iter().filter(|&&seen| seen).count();
        Ok(json!({
            "schema":"uor-r4.geometric-read-usage/1",
            "lanes": self.config.lanes,
            "relation_distinct_codes_per_lane": relation_per_lane,
            "key_distinct_codes_per_lane": key_per_lane,
            "relation_table_occupied_codes": occupied_codes,
            "relation_table_occupancy": occupied_codes as f64 / GROUP_ORDER as f64,
            "relation_observations": usage.relation_total,
            "non_identity_relations": usage.non_identity_relations,
            "non_identity_relation_share": if usage.relation_total > 0 {
                Value::from(usage.non_identity_relations as f64 / usage.relation_total as f64)
            } else {
                Value::Null
            },
            "scope":"Integer counts only, accumulated during enabled Full-admission forwards; no tensor retained. Offline training instrumentation, not a serving metric."
        }))
    }
}

/// Normalize `[batch, lanes, 4]` lanes. A lane below `sqrt(eps_sq)` becomes the
/// identity root; otherwise it is divided by `sqrt(sum_sq + eps_sq)`.
fn normalize_lane_3d(input: &Tensor, eps_sq: f64, device: &Device) -> Result<Tensor> {
    let norm_sq = input.sqr()?.sum_keepdim(2)?;
    let unit = input.broadcast_div(&norm_sq.affine(1.0, eps_sq)?.sqrt()?)?;
    let mask = norm_sq.lt(eps_sq)?.to_dtype(DType::F32)?;
    let keep = mask.affine(-1.0, 1.0)?;
    let identity = Tensor::from_vec(vec![1f32, 0., 0., 0.], (1, 1, 4), device)?;
    Ok(unit
        .broadcast_mul(&keep)?
        .broadcast_add(&identity.broadcast_mul(&mask)?)?)
}

/// As [`normalize_lane_3d`] for `[batch, candidates, lanes, 4]`.
fn normalize_lane_4d(input: &Tensor, eps_sq: f64, device: &Device) -> Result<Tensor> {
    let norm_sq = input.sqr()?.sum_keepdim(3)?;
    let unit = input.broadcast_div(&norm_sq.affine(1.0, eps_sq)?.sqrt()?)?;
    let mask = norm_sq.lt(eps_sq)?.to_dtype(DType::F32)?;
    let keep = mask.affine(-1.0, 1.0)?;
    let identity = Tensor::from_vec(vec![1f32, 0., 0., 0.], (1, 1, 1, 4), device)?;
    Ok(unit
        .broadcast_mul(&keep)?
        .broadcast_add(&identity.broadcast_mul(&mask)?)?)
}

/// Explicit Hamilton product of conj(query) and key, broadcast over candidates.
/// Components use the shared `(w, x, y, z)` convention.
fn smooth_relative(query: &Tensor, key: &Tensor) -> Result<Tensor> {
    let query = query.unsqueeze(1)?;
    let conjugate = [
        query.narrow(3, 0, 1)?,
        query.narrow(3, 1, 1)?.neg()?,
        query.narrow(3, 2, 1)?.neg()?,
        query.narrow(3, 3, 1)?.neg()?,
    ];
    let product = [
        key.narrow(3, 0, 1)?,
        key.narrow(3, 1, 1)?,
        key.narrow(3, 2, 1)?,
        key.narrow(3, 3, 1)?,
    ];
    hamilton_components(
        [&conjugate[0], &conjugate[1], &conjugate[2], &conjugate[3]],
        [&product[0], &product[1], &product[2], &product[3]],
    )
}

/// Hamilton product of four matching broadcast component slices.
fn hamilton_components(a: [&Tensor; 4], b: [&Tensor; 4]) -> Result<Tensor> {
    let (aw, ax, ay, az) = (a[0], a[1], a[2], a[3]);
    let (bw, bx, by, bz) = (b[0], b[1], b[2], b[3]);
    let w = aw
        .broadcast_mul(bw)?
        .sub(&ax.broadcast_mul(bx)?)?
        .sub(&ay.broadcast_mul(by)?)?
        .sub(&az.broadcast_mul(bz)?)?;
    let x = aw
        .broadcast_mul(bx)?
        .add(&ax.broadcast_mul(bw)?)?
        .add(&ay.broadcast_mul(bz)?)?
        .sub(&az.broadcast_mul(by)?)?;
    let y = aw
        .broadcast_mul(by)?
        .sub(&ax.broadcast_mul(bz)?)?
        .add(&ay.broadcast_mul(bw)?)?
        .add(&az.broadcast_mul(bx)?)?;
    let z = aw
        .broadcast_mul(bz)?
        .add(&ax.broadcast_mul(by)?)?
        .sub(&ay.broadcast_mul(bx)?)?
        .add(&az.broadcast_mul(bw)?)?;
    Tensor::cat(&[&w, &x, &y, &z], 3).map_err(Into::into)
}

/// Add the kernel declaration to a numerical contract only when enabled.
/// With no kernel the contract is returned unchanged, so existing contract
/// bytes and checkpoint identities are preserved exactly.
pub fn with_geometric_read_contract(
    mut contract: Value,
    kernel: Option<&GeometricReadConfig>,
) -> Value {
    let Some(kernel) = kernel else {
        return contract;
    };
    contract["geometric_read"] = json!({
        "schema": GEOMETRIC_READ_SCHEMA,
        "lanes": kernel.lanes,
        "lane_width": GEOMETRIC_READ_LANE_WIDTH,
        "hidden": kernel.hidden,
        "norm_eps": kernel.norm_eps,
        "init_scale": kernel.init_scale,
        "parameters":{
            RELATION_WEIGHT1:[kernel.lanes, kernel.hidden, GEOMETRIC_READ_LANE_WIDTH],
            RELATION_BIAS1:[kernel.lanes, kernel.hidden],
            RELATION_WEIGHT2:[kernel.lanes, 1, kernel.hidden],
            RELATION_BIAS2:[kernel.lanes, 1]
        },
        "initialization":"Both weight layers are initialized U(-init_scale, init_scale) from a domain-separated stream; both biases are zero. Base parameters keep their historical draw order and values.",
        "normalization":"Per-lane L2 with denominator sqrt(sum_sq+norm_eps^2); a lane whose norm is below norm_eps normalizes to the identity root (1,0,0,0) and codes to the identity index",
        "quantizer":"Nearest signed H4 root by signed dot product; strict greater-than comparison so the lowest canonical index wins ties; q and -q remain distinct and are never folded",
        "relation":"r = conj(q_code) * k_code through the bound 2I inverse and product tables (left factor query)",
        "score":"per-lane E_l = W2_l . tanh(W1_l r_ST_l + b1_l) + b2_l, summed over 16 lanes; added to the unchanged learned age, NoRead, softmax and value path",
        "surrogate":"Straight-through: forward uses the exact hard codes and exact bound relation; backward differentiates the smooth Hamilton composition and the score",
        "read":"Query/Key from RMS-normalized provisional/written states; Value=tanh(affine(normalized written state)); score=sum_l E_l(signed relative 2I element)+learned age, competing with learned NoRead",
        "scope":"Offline F32 training and evaluation only, Full admission; quantization, packed export, integer serving, precision views and rounding are rejected"
    });
    contract
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint_admission::AdmissionPolicy;
    use crate::joint_model::{JointConfig, JointModel, PrecisionMode, ReadMode};
    use crate::joint_optimizer::{AdamConfig, NamedAdamW};

    fn nanos() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    }

    fn tiny_config(geometry: crate::joint_model::ReadGeometry) -> JointConfig {
        JointConfig {
            width: 128,
            context: 8,
            seed: 11,
            read_geometry: geometry,
            ..JointConfig::default()
        }
    }

    fn kernel_model() -> Result<(JointModel, GeometricReadConfig)> {
        let kernel = GeometricReadConfig::signed_2i();
        let model = JointModel::new_with_geometric_read(
            tiny_config(crate::joint_model::ReadGeometry::Dot),
            Some(kernel.clone()),
            &Device::Cpu,
        )?;
        Ok((model, kernel))
    }

    fn random_ids(count: usize, seed: u64) -> Vec<u32> {
        let mut state = seed;
        (0..count)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state % 4096) as u32
            })
            .collect()
    }

    fn kernel_tensor(model: &JointModel, name: &str) -> Result<Tensor> {
        Ok(model
            .variables()
            .get(name)
            .ok_or_else(|| invalid(format!("missing kernel parameter {name}")))?
            .as_tensor()
            .clone())
    }

    fn same_bits(left: &Tensor, right: &Tensor) -> Result<bool> {
        let a = left.flatten_all()?.to_vec1::<f32>()?;
        let b = right.flatten_all()?.to_vec1::<f32>()?;
        Ok(a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()))
    }

    /// Independent scalar reference for the declared hard path.
    fn reference_hard_scores(
        query: &[f32],
        keys: &[f32],
        candidates: usize,
        weights: &KernelWeights,
    ) -> Result<Vec<f32>> {
        let lanes = GEOMETRIC_READ_LANES;
        let lane_width = GEOMETRIC_READ_LANE_WIDTH;
        let roots = canonical_h4_roots();
        let quantize = |coordinates: [f32; 4]| -> usize {
            let mut best = 0usize;
            let mut best_dot = f32::NEG_INFINITY;
            for (index, root) in roots.iter().enumerate() {
                let array = root.to_array();
                let dot = array[0] as f32 * coordinates[0]
                    + array[1] as f32 * coordinates[1]
                    + array[2] as f32 * coordinates[2]
                    + array[3] as f32 * coordinates[3];
                if dot > best_dot {
                    best_dot = dot;
                    best = index;
                }
            }
            best
        };
        let normalize = |raw: &[f32]| -> [f32; 4] {
            let sum: f32 = raw.iter().map(|value| value * value).sum();
            if sum < 1e-12 {
                return [1.0, 0.0, 0.0, 0.0];
            }
            let denom = (sum + 1e-12).sqrt();
            [
                raw[0] / denom,
                raw[1] / denom,
                raw[2] / denom,
                raw[3] / denom,
            ]
        };
        let table = group_table();
        let weight1 = weights.weight1.flatten_all()?.to_vec1::<f32>()?;
        let bias1 = weights.bias1.flatten_all()?.to_vec1::<f32>()?;
        let weight2 = weights.weight2.flatten_all()?.to_vec1::<f32>()?;
        let bias2 = weights.bias2.flatten_all()?.to_vec1::<f32>()?;
        let hidden = GEOMETRIC_READ_HIDDEN;
        let mut scores = vec![0f32; candidates];
        for (candidate, score) in scores.iter_mut().enumerate() {
            let mut total = 0f32;
            for lane in 0..lanes {
                let query_unit = normalize(&query[lane * lane_width..(lane + 1) * lane_width]);
                let key_base = (candidate * lanes + lane) * lane_width;
                let key_unit = normalize(&keys[key_base..key_base + lane_width]);
                let query_code = quantize(query_unit);
                let key_code = quantize(key_unit);
                let relation = table.product
                    [table.inverse[query_code] as usize * ROW_STRIDE + key_code]
                    as usize;
                let root = roots[relation].to_array();
                let mut hidden_values = [0f32; GEOMETRIC_READ_HIDDEN];
                for (h, value) in hidden_values.iter_mut().enumerate() {
                    let mut sum = bias1[lane * hidden + h];
                    for coordinate in 0..lane_width {
                        sum += weight1[(lane * hidden + h) * lane_width + coordinate]
                            * root[coordinate] as f32;
                    }
                    *value = sum.tanh();
                }
                let mut lane_score = bias2[lane];
                for (h, value) in hidden_values.iter().enumerate() {
                    lane_score += weight2[lane * hidden + h] * value;
                }
                total += lane_score;
            }
            *score = total;
        }
        Ok(scores)
    }

    #[test]
    fn geometric_read_algebra_matches_hamilton_reference() -> Result<()> {
        // Random unit quaternions; the tensor Hamilton product must agree with
        // the scalar reference on every coordinate.
        let query = Tensor::from_vec(
            vec![0.5f32, -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.0],
            (1, 2, 4),
            &Device::Cpu,
        )?;
        let key = Tensor::from_vec(
            vec![
                0.5f32, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, -0.5, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.0,
            ],
            (1, 2, 2, 4),
            &Device::Cpu,
        )?;
        let relative = smooth_relative(&query, &key)?;
        assert_eq!(relative.dims().to_vec(), vec![1usize, 2, 2, 4]);
        let values = relative.flatten_all()?.to_vec1::<f32>()?;
        let query_values = query.flatten_all()?.to_vec1::<f32>()?;
        let key_values = key.flatten_all()?.to_vec1::<f32>()?;
        let hamilton = |a: [f64; 4], b: [f64; 4]| -> [f64; 4] {
            [
                a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
                a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
                a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
                a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
            ]
        };
        let mut index = 0usize;
        for candidate in 0..2 {
            for lane in 0..2 {
                let q_base = lane * 4;
                let k_base = (candidate * 2 + lane) * 4;
                let q = [
                    query_values[q_base] as f64,
                    query_values[q_base + 1] as f64,
                    query_values[q_base + 2] as f64,
                    query_values[q_base + 3] as f64,
                ];
                let k = [
                    key_values[k_base] as f64,
                    key_values[k_base + 1] as f64,
                    key_values[k_base + 2] as f64,
                    key_values[k_base + 3] as f64,
                ];
                let expected = hamilton([q[0], -q[1], -q[2], -q[3]], k);
                for coordinate in 0..4 {
                    let actual = values[index + coordinate] as f64;
                    assert!(
                        (actual - expected[coordinate]).abs() < 1e-6,
                        "candidate {candidate} lane {lane} coordinate {coordinate}: {actual} vs {}",
                        expected[coordinate]
                    );
                }
                index += 4;
            }
        }
        Ok(())
    }

    #[test]
    fn geometric_read_hard_forward_is_exact() -> Result<()> {
        let (model, _) = kernel_model()?;
        let query = Tensor::from_vec(
            (0..64)
                .map(|i| ((i * 37) % 29) as f32 * 0.1 - 1.0)
                .collect::<Vec<f32>>(),
            (1, 64),
            &Device::Cpu,
        )?;
        let candidates = 5usize;
        let keys = Tensor::from_vec(
            (0..candidates * 64)
                .map(|i| ((i * 53 + 7) % 31) as f32 * 0.1 - 1.5)
                .collect::<Vec<f32>>(),
            (1, candidates, 64),
            &Device::Cpu,
        )?;
        let weights = KernelWeights {
            weight1: kernel_tensor(&model, RELATION_WEIGHT1)?,
            bias1: kernel_tensor(&model, RELATION_BIAS1)?,
            weight2: kernel_tensor(&model, RELATION_WEIGHT2)?,
            bias2: kernel_tensor(&model, RELATION_BIAS2)?,
        };
        let state = GeometricReadState::new(GeometricReadConfig::signed_2i(), &Device::Cpu)?;
        let actual = state
            .score(&query, &keys, None, &weights)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let expected = reference_hard_scores(
            &query.flatten_all()?.to_vec1::<f32>()?,
            &keys.flatten_all()?.to_vec1::<f32>()?,
            candidates,
            &weights,
        )?;
        assert_eq!(actual.len(), expected.len());
        for (index, (a, e)) in actual.iter().zip(&expected).enumerate() {
            assert!(
                (a - e).abs() < 1e-5,
                "candidate {index}: kernel {a} vs hard reference {e}"
            );
        }
        Ok(())
    }

    #[test]
    fn geometric_read_gradients_connect() -> Result<()> {
        let (model, _) = kernel_model()?;
        let ids = random_ids(2 * 8, 0xABCDEF);
        let targets = random_ids(2 * 8, 0x123456);
        let output = model.forward(&ids, 2, 8, ReadMode::Enabled, true)?;
        let loss = output.loss(&targets)?;
        let gradients = loss.backward()?;
        for name in parameter_names() {
            let variable = model
                .variables()
                .get(name)
                .ok_or_else(|| invalid(format!("missing kernel parameter {name}")))?;
            let gradient = gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("kernel gradient missing {name}")))?;
            assert!(
                gradient
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .all(|value| value.is_finite()),
                "nonfinite kernel gradient for {name}"
            );
            let magnitude = gradient.abs()?.max_all()?.to_scalar::<f32>()?;
            assert!(magnitude > 0.0, "zero kernel gradient for {name}");
        }
        Ok(())
    }

    #[test]
    fn geometric_read_default_off_is_unchanged() -> Result<()> {
        let config = tiny_config(crate::joint_model::ReadGeometry::Dot);
        let base = JointModel::new(config.clone(), &Device::Cpu)?;
        let none = JointModel::new_with_geometric_read(config.clone(), None, &Device::Cpu)?;
        assert_eq!(base.variables().len(), none.variables().len());
        for (name, variable) in base.variables() {
            let other = none
                .variables()
                .get(name)
                .ok_or_else(|| invalid(format!("default-off model missing {name}")))?;
            assert!(
                same_bits(variable.as_tensor(), other.as_tensor())?,
                "default-off parameter differs for {name}"
            );
        }
        assert_eq!(
            serde_json::to_string(&base.numerical_contract())?,
            serde_json::to_string(&none.numerical_contract())?
        );
        let ids = random_ids(2 * 8, 0x55AA);
        let left = base.forward(&ids, 2, 8, ReadMode::Enabled, false)?;
        let right = none.forward(&ids, 2, 8, ReadMode::Enabled, false)?;
        assert!(same_bits(&left.probabilities, &right.probabilities)?);
        assert!(same_bits(&left.read_masses, &right.read_masses)?);
        assert!(none.geometric_read_config().is_none());
        Ok(())
    }

    #[test]
    fn geometric_read_zero_lane_uses_identity() -> Result<()> {
        let state = GeometricReadState::new(GeometricReadConfig::signed_2i(), &Device::Cpu)?;
        let input = Tensor::from_vec(
            vec![0f32, 0., 0., 0., 0., 2., 0., 0.],
            (1, 2, 4),
            &Device::Cpu,
        )?;
        let normalized = normalize_lane_3d(&input, 1e-12, &Device::Cpu)?;
        let values = normalized.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(&values[0..4], &[1.0f32, 0.0, 0.0, 0.0][..]);
        assert_eq!(&values[4..8], &[0.0f32, 1.0, 0.0, 0.0][..]);
        assert_eq!(state.quantize_signed([1.0, 0.0, 0.0, 0.0]), 1);
        Ok(())
    }

    #[test]
    fn geometric_read_ties_pick_lowest_index() -> Result<()> {
        let state = GeometricReadState::new(GeometricReadConfig::signed_2i(), &Device::Cpu)?;
        // The zero vector ties every root at dot 0; the strict comparison must
        // keep the lowest canonical index rather than the last one seen.
        assert_eq!(state.quantize_signed([0.0, 0.0, 0.0, 0.0]), 0);
        // A root direction has a unique maximum and selects that index.
        assert_eq!(state.quantize_signed([1.0, 0.0, 0.0, 0.0]), 1);
        assert_eq!(group_table().identity, 1u8);
        Ok(())
    }

    #[test]
    fn geometric_read_antipodes_stay_distinct() -> Result<()> {
        let state = GeometricReadState::new(GeometricReadConfig::signed_2i(), &Device::Cpu)?;
        let positive = state.quantize_signed([1.0, 0.0, 0.0, 0.0]);
        let negative = state.quantize_signed([-1.0, 0.0, 0.0, 0.0]);
        assert_eq!(positive, 1);
        assert_eq!(negative, 0);
        assert_ne!(positive, negative);
        // conj(q) * k is not folded: the antipodal key gives the negated
        // relative element, distinct from the identity.
        let table = group_table();
        let identity_relation =
            table.product[table.inverse[positive] as usize * ROW_STRIDE + positive];
        let antipodal_relation =
            table.product[table.inverse[positive] as usize * ROW_STRIDE + negative];
        assert_eq!(identity_relation as usize, table.identity as usize);
        assert_ne!(antipodal_relation, table.identity);
        Ok(())
    }

    #[test]
    fn geometric_read_serialization_round_trip() -> Result<()> {
        let (model, kernel) = kernel_model()?;
        let ids = random_ids(2 * 8, 0x0F0F);
        let _ = model.forward(&ids, 2, 8, ReadMode::Enabled, false)?;
        let directory = std::env::temp_dir().join(format!(
            "uor-geometric-read-{}-{}",
            std::process::id(),
            nanos()
        ));
        std::fs::create_dir(&directory)?;
        model.save(&directory)?;
        let loaded = JointModel::load(&directory, &Device::Cpu)?;
        assert_eq!(loaded.geometric_read_config(), Some(&kernel));
        assert_eq!(loaded.variables().len(), model.variables().len());
        for (name, variable) in model.variables() {
            let other = loaded
                .variables()
                .get(name)
                .ok_or_else(|| invalid(format!("round-trip model missing {name}")))?;
            assert!(
                same_bits(variable.as_tensor(), other.as_tensor())?,
                "round-trip parameter differs for {name}"
            );
        }
        assert_eq!(
            serde_json::to_string(&model.numerical_contract())?,
            serde_json::to_string(&loaded.numerical_contract())?
        );
        assert_eq!(loaded.parameter_count(), model.parameter_count());
        let usage = loaded.geometric_read_usage()?;
        assert!(usage["relation_observations"].as_u64().unwrap_or(0) == 0);
        // A kernel checkpoint refuses a target that disables the kernel.
        assert!(JointModel::load_with_geometric_read(&directory, &Device::Cpu, None).is_err());
        std::fs::remove_dir_all(&directory)?;
        Ok(())
    }

    #[test]
    fn geometric_read_resume_adds_kernel_with_zero_moments() -> Result<()> {
        let config = tiny_config(crate::joint_model::ReadGeometry::Dot);
        let parent = JointModel::new(config.clone(), &Device::Cpu)?;
        let optimizer_config = AdamConfig::default();
        let parent_optimizer = NamedAdamW::new(parent.variables(), optimizer_config.clone())?;
        let directory = std::env::temp_dir().join(format!(
            "uor-geometric-read-resume-{}-{}",
            std::process::id(),
            nanos()
        ));
        std::fs::create_dir(&directory)?;
        parent.save(&directory)?;
        parent_optimizer.save(&directory)?;
        let kernel = GeometricReadConfig::signed_2i();
        let loaded = JointModel::load_with_geometric_read(&directory, &Device::Cpu, Some(&kernel))?;
        assert_eq!(loaded.geometric_read_config(), Some(&kernel));
        for (name, variable) in parent.variables() {
            let other = loaded
                .variables()
                .get(name)
                .ok_or_else(|| invalid(format!("resumed model missing base {name}")))?;
            assert!(
                same_bits(variable.as_tensor(), other.as_tensor())?,
                "resume changed base parameter {name}"
            );
        }
        // The saved optimizer has no kernel rows; loading it without the fresh
        // allowance must fail, and the fresh allowance must add them at zero.
        assert!(NamedAdamW::load(&directory, loaded.variables(), &optimizer_config).is_err());
        let fresh: std::collections::BTreeSet<String> = parameter_names()
            .iter()
            .map(|name| (*name).to_owned())
            .collect();
        let resumed =
            NamedAdamW::load_with_fresh(&directory, loaded.variables(), &optimizer_config, &fresh)?;
        assert_eq!(resumed.step_count(), parent_optimizer.step_count());
        let ids = random_ids(2 * 8, 0x2468);
        let targets = random_ids(2 * 8, 0x1357);
        let gradients =
            crate::joint_parallel::batch_gradients(&loaded, &ids, &targets, None, 2, 8, 1)?;
        let mut resumed = resumed;
        resumed.step(loaded.variables(), &gradients.gradients)?;
        for name in parameter_names() {
            assert!(
                loaded
                    .variables()
                    .get(name)
                    .ok_or_else(|| invalid(format!("missing {name}")))?
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .all(|value| value.is_finite()),
                "nonfinite fresh kernel parameter after one update: {name}"
            );
        }
        std::fs::remove_dir_all(&directory)?;
        Ok(())
    }

    #[test]
    fn geometric_read_rejects_quantized_paths() -> Result<()> {
        let (mut model, _) = kernel_model()?;
        assert!(model.configure_quantization(0, 1).is_err());
        assert!(model
            .set_admission_policy(AdmissionPolicy::Recent64)
            .is_err());
        assert!(model.precision_view(PrecisionMode::FF).is_err());
        let directory = std::env::temp_dir().join(format!(
            "uor-geometric-read-hard-{}-{}",
            std::process::id(),
            nanos()
        ));
        std::fs::create_dir(&directory)?;
        assert!(model.save_hard(&directory).is_err());
        std::fs::remove_dir_all(&directory)?;
        // A non-Dot geometry cannot carry the kernel.
        assert!(JointModel::new_with_geometric_read(
            tiny_config(crate::joint_model::ReadGeometry::Lorentz),
            Some(GeometricReadConfig::signed_2i()),
            &Device::Cpu,
        )
        .is_err());
        // Configuration validation rejects any non-frozen shape.
        let mut wrong = GeometricReadConfig::signed_2i();
        wrong.lanes = 8;
        assert!(wrong.validate().is_err());
        let mut wrong = GeometricReadConfig::signed_2i();
        wrong.hidden = 4;
        assert!(wrong.validate().is_err());
        let mut wrong = GeometricReadConfig::signed_2i();
        wrong.schema = "other".into();
        assert!(wrong.validate().is_err());
        let mut wrong = GeometricReadConfig::signed_2i();
        wrong.norm_eps = 0.0;
        assert!(wrong.validate().is_err());
        let mut wrong = GeometricReadConfig::signed_2i();
        wrong.init_scale = -1.0;
        assert!(wrong.validate().is_err());
        Ok(())
    }

    #[test]
    fn geometric_read_two_update_smoke_is_finite() -> Result<()> {
        let (mut model, _) = kernel_model()?;
        let optimizer_config = AdamConfig {
            learning_rate: 1e-3,
            ..AdamConfig::default()
        };
        let mut optimizer = NamedAdamW::new(model.variables(), optimizer_config)?;
        let mut previous = f32::INFINITY;
        for step in 0..2u64 {
            let ids = random_ids(2 * 8, 0x1000 + step);
            let targets = random_ids(2 * 8, 0x2000 + step);
            let gradients =
                crate::joint_parallel::batch_gradients(&model, &ids, &targets, None, 2, 8, 1)?;
            assert!(gradients.mean_nll.is_finite(), "nonfinite smoke loss");
            assert!(gradients.mean_nll <= previous + 1.0);
            previous = gradients.mean_nll;
            optimizer.step(model.variables(), &gradients.gradients)?;
            model.set_completed_step(step as usize + 1)?;
        }
        for (name, variable) in model.variables() {
            assert!(
                variable
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .all(|value| value.is_finite()),
                "nonfinite parameter after smoke updates: {name}"
            );
        }
        // Per-lane hard-path usage on a random multi-candidate input: every
        // lane must see more than one distinct key/relation code.
        let state = GeometricReadState::new(GeometricReadConfig::signed_2i(), &Device::Cpu)?;
        let query = Tensor::from_vec(
            random_ids(64, 0x7777)
                .into_iter()
                .map(|id| (id as f32 - 2048.0) / 2048.0)
                .collect::<Vec<f32>>(),
            (1, 64),
            &Device::Cpu,
        )?;
        let candidates = 32usize;
        let keys = Tensor::from_vec(
            random_ids(candidates * 64, 0x8888)
                .into_iter()
                .map(|id| (id as f32 - 2048.0) / 2048.0)
                .collect::<Vec<f32>>(),
            (1, candidates, 64),
            &Device::Cpu,
        )?;
        let weights = KernelWeights {
            weight1: kernel_tensor(&model, RELATION_WEIGHT1)?,
            bias1: kernel_tensor(&model, RELATION_BIAS1)?,
            weight2: kernel_tensor(&model, RELATION_WEIGHT2)?,
            bias2: kernel_tensor(&model, RELATION_BIAS2)?,
        };
        let scores = state.score(&query, &keys, None, &weights)?;
        assert_eq!(scores.dims().to_vec(), vec![1usize, candidates]);
        assert!(scores
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|value| value.is_finite()));
        let usage = state.usage_snapshot()?;
        let per_lane = usage["key_distinct_codes_per_lane"]
            .as_array()
            .ok_or_else(|| invalid("usage snapshot missing key histogram"))?;
        for (lane, count) in per_lane.iter().enumerate() {
            assert!(
                count.as_u64().unwrap_or(0) > 1,
                "lane {lane} collapsed to <=1 key code: {count}"
            );
        }
        let relation_per_lane = usage["relation_distinct_codes_per_lane"]
            .as_array()
            .ok_or_else(|| invalid("usage snapshot missing relation histogram"))?;
        for (lane, count) in relation_per_lane.iter().enumerate() {
            assert!(
                count.as_u64().unwrap_or(0) > 1,
                "lane {lane} collapsed to <=1 relation code: {count}"
            );
        }
        assert!(usage["relation_observations"].as_u64().unwrap_or(0) > 0);
        Ok(())
    }

    fn fixed_query(seed: u64) -> Result<Tensor> {
        let values = random_ids(64, seed)
            .into_iter()
            .map(|id| (id as f32 - 2048.0) / 2048.0)
            .collect::<Vec<f32>>();
        Ok(Tensor::from_vec(values, (1, 64), &Device::Cpu)?)
    }

    fn fixed_keys(candidates: usize, seed: u64) -> Result<Tensor> {
        let values = random_ids(candidates * 64, seed)
            .into_iter()
            .map(|id| (id as f32 - 2048.0) / 2048.0)
            .collect::<Vec<f32>>();
        Ok(Tensor::from_vec(values, (1, candidates, 64), &Device::Cpu)?)
    }

    /// Build the detached write-time cache row by row, exactly as `core_step`
    /// appends it, and return it with the flattened code vector.
    fn encode_row_by_row(
        state: &GeometricReadState,
        keys: &Tensor,
        candidates: usize,
    ) -> Result<(GeometricReadKeyCache, Vec<u32>)> {
        let mut codes: Option<Tensor> = None;
        let mut coords: Option<Tensor> = None;
        for candidate in 0..candidates {
            let row = keys.narrow(1, candidate, 1)?.squeeze(1)?;
            let (row_codes, row_coords) = state.encode_keys(&row)?;
            codes = Some(match codes {
                Some(previous) => Tensor::cat(&[&previous, &row_codes], 1)?,
                None => row_codes,
            });
            coords = Some(match coords {
                Some(previous) => Tensor::cat(&[&previous, &row_coords], 1)?,
                None => row_coords,
            });
        }
        let codes = codes.ok_or_else(|| invalid("no candidates"))?;
        let coords = coords.ok_or_else(|| invalid("no candidates"))?;
        let flat = codes.flatten_all()?.to_vec1::<u32>()?;
        Ok((
            GeometricReadKeyCache {
                codes,
                hard_coords: coords,
            },
            flat,
        ))
    }

    fn recompute_codes(
        state: &GeometricReadState,
        keys: &Tensor,
        candidates: usize,
    ) -> Result<Vec<u32>> {
        let lanes = GEOMETRIC_READ_LANES;
        let lane_width = GEOMETRIC_READ_LANE_WIDTH;
        let key_lanes = keys.reshape((1, candidates, lanes, lane_width))?;
        let unit = normalize_lane_4d(&key_lanes, 1e-12, &Device::Cpu)?;
        let host = unit.flatten_all()?.to_vec1::<f32>()?;
        Ok(state.quantize_flat(&host, candidates))
    }

    #[test]
    fn geometric_read_cached_codes_match_recompute() -> Result<()> {
        let state = GeometricReadState::new(GeometricReadConfig::signed_2i(), &Device::Cpu)?;
        let candidates = 7usize;
        let lanes = GEOMETRIC_READ_LANES;
        let query = fixed_query(0x5EED_0001)?;
        let keys = fixed_keys(candidates, 0x5EED_0002)?;
        let (cache, cached_flat) = encode_row_by_row(&state, &keys, candidates)?;
        assert_eq!(cache.codes.dims().to_vec(), vec![1usize, candidates, lanes]);
        assert_eq!(
            cache.hard_coords.dims().to_vec(),
            vec![1usize, candidates, lanes, GEOMETRIC_READ_LANE_WIDTH]
        );
        let recomputed = recompute_codes(&state, &keys, candidates)?;
        assert_eq!(
            cached_flat, recomputed,
            "cached and recomputed key codes differ"
        );

        // The write-time hard coordinates equal the root of the recomputed key
        // codes, and the relation rows agree candidate by candidate and lane by
        // lane.
        let query_host = normalize_lane_3d(&query.reshape((1, lanes, 4))?, 1e-12, &Device::Cpu)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let query_codes = state.quantize_flat(&query_host, 1);
        let table = group_table();
        let relation_from = |codes: &[u32]| -> Vec<u32> {
            (0..candidates * lanes)
                .map(|index| {
                    let lane = index % lanes;
                    let query_code = query_codes[lane] as usize;
                    let key_code = codes[index] as usize;
                    table.product[table.inverse[query_code] as usize * ROW_STRIDE + key_code] as u32
                })
                .collect()
        };
        let cached_relations = relation_from(&cached_flat);
        let recomputed_relations = relation_from(&recomputed);
        assert_eq!(
            cached_relations, recomputed_relations,
            "cached and recomputed hard relation codes differ"
        );

        let key_coords = cache.hard_coords.flatten_all()?.to_vec1::<f32>()?;
        let expected_key_coords = state
            .root_tensor
            .index_select(
                &Tensor::from_vec(recomputed.clone(), (candidates * lanes,), &Device::Cpu)?,
                0,
            )?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let relation_coords = state
            .root_tensor
            .index_select(
                &Tensor::from_vec(cached_relations, (candidates * lanes,), &Device::Cpu)?,
                0,
            )?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let expected_relation_coords = state
            .root_tensor
            .index_select(
                &Tensor::from_vec(recomputed_relations, (candidates * lanes,), &Device::Cpu)?,
                0,
            )?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(key_coords.len(), expected_key_coords.len());
        assert_eq!(relation_coords.len(), expected_relation_coords.len());
        for (index, (actual, wanted)) in key_coords.iter().zip(&expected_key_coords).enumerate() {
            assert_eq!(
                actual.to_bits(),
                wanted.to_bits(),
                "hard key coordinate {index} differs"
            );
        }
        for (index, (actual, wanted)) in relation_coords
            .iter()
            .zip(&expected_relation_coords)
            .enumerate()
        {
            assert_eq!(
                actual.to_bits(),
                wanted.to_bits(),
                "hard relation coordinate {index} differs"
            );
        }
        Ok(())
    }

    #[test]
    fn geometric_read_cached_path_matches_uncached_scores() -> Result<()> {
        let (model, _) = kernel_model()?;
        let state = GeometricReadState::new(GeometricReadConfig::signed_2i(), &Device::Cpu)?;
        let candidates = 6usize;
        let query = fixed_query(0xC0FF_0001)?;
        let keys = fixed_keys(candidates, 0xC0FF_0002)?;
        let (cache, _) = encode_row_by_row(&state, &keys, candidates)?;
        let weights = KernelWeights {
            weight1: kernel_tensor(&model, RELATION_WEIGHT1)?,
            bias1: kernel_tensor(&model, RELATION_BIAS1)?,
            weight2: kernel_tensor(&model, RELATION_WEIGHT2)?,
            bias2: kernel_tensor(&model, RELATION_BIAS2)?,
        };
        let cached = state.score(&query, &keys, Some(&cache), &weights)?;
        let uncached = state.score(&query, &keys, None, &weights)?;
        assert_eq!(cached.dims(), uncached.dims());
        assert!(
            same_bits(&cached, &uncached)?,
            "cached and uncached scores differ"
        );
        Ok(())
    }
}
