//! Learned finite geometric event controller, with explicit biased autodiff.
//!
//! Each context lane is a signed 2I root. Selected token rows and quaternion
//! potentials of the OLD own/neighbor roots score 120 right actions. Earliest
//! argmax and exact group lookup update every lane synchronously. Four event
//! logits then read the NEW roots plus a selected token-event row. No contextual
//! float trunk, q/k map, event label or span label enters this controller.
//!
//! Backward tangent-projects each new-root adjoint, applies the Hamilton
//! Jacobian at the hard old/action roots, and replaces the action choice's
//! derivative by that of the softmax-weighted canonical roots (temperature 1).
//! State-score derivatives through BOTH own and old-neighbor roots are retained.
//! This local surrogate is biased; it is not the derivative of argmax or a
//! distribution over 120^L states. Only first-order gradients are implemented.
//!
//! Training may use F32 parameters/F64 arithmetic. Native tables compile the
//! same finite potentials to signed Q24, nearest/ties-away, rejecting overflow.
//! The native controller returns integer SpanAction values directly. Float
//! one-hot reconstruction exists only as a labelled span replay interface.
//! Q24 lookup entries are not a qualified four-bit coefficient codec.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;

use candle_core::{CpuStorage, CustomOp2, DType, Device, Layout, Shape, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::{
    embedding::canonical_h4_roots,
    group_table::{group_table, GROUP_ORDER, ROW_STRIDE},
    prefix_artifact::historical_roots,
};
use uor_r4_integer::geometric_event::{NativeEventState, NativeEventTables, FRACTIONAL_BITS};
use uor_r4_integer::geometric_span::SpanAction;
use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::h4_tables::{
    mathematical_sha256, H4Code, HistoricalH4Tables, TRUSTED_MATHEMATICAL_SHA256,
};

use crate::geometric_address::{encode_lane, geometry_digest};
use crate::geometric_stack::{
    StackConfig, StackModel, GEOMETRIC_ADDRESS_RECORD, GEOMETRIC_SPAN_RECORD,
};
use crate::{invalid, sha256_bytes, Result};

pub const SCHEMA: &str = "uor-r4.geometric-event/1";
const SOURCE_SCHEMA: &str = "uor-r4.geometric-event-source/1";
const COMPILED_SCHEMA: &str = "uor-r4.geometric-event-native/1";
const RULE:&str="old-own+old-next-neighbor;120-earliest-argmax;right-group-update;synchronous;new-state-four-events;identity-reset/1";
const SURROGATE: &str =
    "new-root-tangent;hard-hamilton;120-softmax-expected-root-T1;own-and-neighbor-score-credit/1";
const COEFFICIENT_POLICY: &str =
    "i32-Q24;nearest-ties-away;no-centering;overflow-reject;integer-events/1";
const TT: &str = "token_transition";
const ST: &str = "self_transition";
const NT: &str = "neighbor_transition";
const TE: &str = "token_event";
const SE: &str = "state_event";
const SOURCE_FILES: [&str; 4] = [
    "metadata.json",
    "event-parameters.safetensors",
    "h4-tables.bin",
    "tokenizer-identity.bin",
];
const NATIVE_FILES: [&str; 4] = [
    "metadata.json",
    "event-tables-i32le.bin",
    "h4-tables.bin",
    "tokenizer-identity.bin",
];
const PINNED: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricEventConfig {
    pub schema: String,
    pub vocab_size: usize,
    pub lanes: usize,
    pub seed: u64,
    pub geometry_sha256: String,
    pub rule: String,
    pub surrogate: String,
    pub temperature: f64,
}
impl GeometricEventConfig {
    pub fn new(vocab_size: usize, lanes: usize, seed: u64) -> Result<Self> {
        if vocab_size == 0 || vocab_size > 4096 || lanes == 0 || lanes > 4 {
            return Err(invalid(
                "event controller requires vocab1..4096 and lanes1..4",
            ));
        }
        Ok(Self {
            schema: SCHEMA.into(),
            vocab_size,
            lanes,
            seed,
            geometry_sha256: geometry_digest().into(),
            rule: RULE.into(),
            surrogate: SURROGATE.into(),
            temperature: 1.,
        })
    }
    pub fn validate(&self) -> Result<()> {
        if *self != Self::new(self.vocab_size, self.lanes, self.seed)? {
            return Err(invalid(
                "event controller configuration/geometry/surrogate differs",
            ));
        }
        Ok(())
    }
    fn shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let (v, l) = (self.vocab_size, self.lanes);
        let mut out = BTreeMap::from([
            (TT.into(), vec![v, l, 120]),
            (ST.into(), vec![l, 120, 4]),
            (TE.into(), vec![v, 4]),
            (SE.into(), vec![l, 4, 4]),
        ]);
        if l > 1 {
            out.insert(NT.into(), vec![l, 120, 4]);
        }
        out
    }
}
fn roots() -> &'static [[f64; 4]; 120] {
    static R: OnceLock<[[f64; 4]; 120]> = OnceLock::new();
    R.get_or_init(|| std::array::from_fn(|i| canonical_h4_roots()[i].to_array()))
}
fn root(id: u8) -> [f64; 4] {
    roots()[usize::from(id)]
}
fn dot(a: [f64; 4], b: [f64; 4]) -> f64 {
    (0..4).map(|i| a[i] * b[i]).sum()
}
fn add(a: &mut [f64; 4], b: [f64; 4]) {
    for i in 0..4 {
        a[i] += b[i];
    }
}
fn tangent(q: [f64; 4], g: [f64; 4]) -> [f64; 4] {
    let radial = dot(q, g) / dot(q, q);
    std::array::from_fn(|i| g[i] - q[i] * radial)
}
#[cfg(test)]
fn hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}
fn hamilton_pullback(a: [f64; 4], b: [f64; 4], g: [f64; 4]) -> ([f64; 4], [f64; 4]) {
    (
        [
            g[0] * b[0] + g[1] * b[1] + g[2] * b[2] + g[3] * b[3],
            -g[0] * b[1] + g[1] * b[0] - g[2] * b[3] + g[3] * b[2],
            -g[0] * b[2] + g[1] * b[3] + g[2] * b[0] - g[3] * b[1],
            -g[0] * b[3] - g[1] * b[2] + g[2] * b[1] + g[3] * b[0],
        ],
        [
            g[0] * a[0] + g[1] * a[1] + g[2] * a[2] + g[3] * a[3],
            -g[0] * a[1] + g[1] * a[0] + g[2] * a[3] - g[3] * a[2],
            -g[0] * a[2] - g[1] * a[3] + g[2] * a[0] + g[3] * a[1],
            -g[0] * a[3] + g[1] * a[2] - g[2] * a[1] + g[3] * a[0],
        ],
    )
}
fn best(values: &[f64]) -> usize {
    let mut index = 0;
    for i in 1..values.len() {
        if values[i] > values[index] {
            index = i;
        }
    }
    index
}
fn probabilities(scores: &[f64; 120]) -> [f64; 120] {
    let maximum = scores[best(scores)];
    let mut p = scores.map(|s| (s - maximum).exp());
    let sum = p.iter().sum::<f64>();
    for v in &mut p {
        *v /= sum;
    }
    p
}
fn checked_f32(values: Vec<f64>) -> candle_core::Result<Vec<f32>> {
    let values = values.into_iter().map(|x| x as f32).collect::<Vec<_>>();
    if values.iter().any(|x| !x.is_finite()) {
        candle_core::bail!("nonfinite event controller value/gradient");
    }
    Ok(values)
}

pub struct EventWeights {
    config: GeometricEventConfig,
    parameters: BTreeMap<String, Var>,
}
impl EventWeights {
    pub fn new(vocab_size: usize, lanes: usize, seed: u64) -> Result<Self> {
        let config = GeometricEventConfig::new(vocab_size, lanes, seed)?;
        let mut random = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
        let mut draw = || {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let v = ((random >> 40) as f32 / 16_777_216. - 0.5) * 0.04;
            if v == 0. {
                0.001
            } else {
                v
            }
        };
        let mut parameters = BTreeMap::new();
        for (name, shape) in config.shapes() {
            let mut values = (0..shape.iter().product())
                .map(|_| draw())
                .collect::<Vec<f32>>();
            if name == TT {
                for row in values.chunks_exact_mut(120) {
                    row[usize::from(group_table().identity)] += 0.05;
                }
            }
            parameters.insert(name, Var::from_vec(values, shape, &Device::Cpu)?);
        }
        Ok(Self { config, parameters })
    }
    pub fn parameters(&self) -> &BTreeMap<String, Var> {
        &self.parameters
    }
    pub fn config(&self) -> &GeometricEventConfig {
        &self.config
    }
    fn validate(&self) -> Result<()> {
        self.config.validate()?;
        if self.parameters.keys().cloned().collect::<BTreeSet<_>>()
            != self.config.shapes().keys().cloned().collect()
        {
            return Err(invalid("event parameter names differ"));
        }
        for (name, shape) in self.config.shapes() {
            let t = &self.parameters[&name];
            if t.dims() != shape || t.dtype() != DType::F32 || !t.device().is_cpu() {
                return Err(invalid("event parameter shape/type/device differs"));
            }
            if t.flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|v| !v.is_finite())
            {
                return Err(invalid("nonfinite event parameter"));
            }
        }
        Ok(())
    }
    fn validate_ids(&self, ids: &[u32], batch: usize, time: usize) -> Result<()> {
        if batch == 0
            || time == 0
            || batch.checked_mul(time) != Some(ids.len())
            || ids.iter().any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid(
                "event ids need nonempty valid [batch*time] vocabulary rows",
            ));
        }
        Ok(())
    }
    fn inputs(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        detach_event_bias: bool,
    ) -> Result<(Tensor, Tensor)> {
        self.validate()?;
        self.validate_ids(ids, batch, time)?;
        let l = self.config.lanes;
        let transition = self.parameters[TT].reshape((self.config.vocab_size, l * 120))?;
        let event = if detach_event_bias {
            self.parameters[TE].detach()
        } else {
            self.parameters[TE].as_tensor().clone()
        };
        let rows = Tensor::cat(&[&transition, &event], 1)?
            .index_select(&Tensor::from_vec(ids.to_vec(), ids.len(), &Device::Cpu)?, 0)?
            .reshape((batch, time, l * 120 + 4))?;
        let own = self.parameters[ST].flatten_all()?;
        let event = self.parameters[SE].flatten_all()?;
        let basis = if l > 1 {
            Tensor::cat(&[&own, &self.parameters[NT].flatten_all()?, &event], 0)?
        } else {
            Tensor::cat(&[&own, &event], 0)?
        };
        Ok((rows.contiguous()?, basis.contiguous()?))
    }
    fn logits_impl(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        reset: bool,
        detach_event_bias: bool,
    ) -> Result<Tensor> {
        let (rows, basis) = self.inputs(ids, batch, time, detach_event_bias)?;
        Ok(rows.apply_op2(
            &basis,
            EventOp {
                batch,
                time,
                lanes: self.config.lanes,
                reset,
            },
        )?)
    }
    pub fn event_logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.logits_impl(ids, batch, time, false, false)
    }
    pub fn event_logits_reset_each_token(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        self.logits_impl(ids, batch, time, true, false)
    }
    pub fn float_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        reset_each_token: bool,
    ) -> Result<EventFloatTrace> {
        let (rows, basis) = self.inputs(ids, batch, time, false)?;
        let op = EventOp {
            batch,
            time,
            lanes: self.config.lanes,
            reset: reset_each_token,
        };
        let rows = rows.flatten_all()?.to_vec1::<f32>()?;
        let basis = basis.to_vec1::<f32>()?;
        let (steps, logits) = op.forward(&rows, &basis)?;
        let events = logits
            .chunks_exact(4)
            .map(|r| {
                let mut i = 0;
                for j in 1..4 {
                    if r[j] > r[i] {
                        i = j;
                    }
                }
                i as u8
            })
            .collect();
        Ok(EventFloatTrace {
            batch,
            time,
            lanes: self.config.lanes,
            states: steps
                .iter()
                .map(|s| s.new[..self.config.lanes].to_vec())
                .collect(),
            actions: steps
                .iter()
                .map(|s| s.action[..self.config.lanes].to_vec())
                .collect(),
            events,
            logits,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventFloatTrace {
    pub batch: usize,
    pub time: usize,
    pub lanes: usize,
    pub states: Vec<Vec<u8>>,
    pub actions: Vec<Vec<u8>>,
    pub events: Vec<u8>,
    pub logits: Vec<f32>,
}
#[derive(Clone)]
struct Step {
    old: [u8; 4],
    new: [u8; 4],
    action: [u8; 4],
}
struct EventOp {
    batch: usize,
    time: usize,
    lanes: usize,
    reset: bool,
}
impl EventOp {
    fn event_offset(&self) -> usize {
        self.lanes * 120 * 4 * if self.lanes > 1 { 2 } else { 1 }
    }
    fn validate(&self, rows: &[f32], basis: &[f32]) -> candle_core::Result<()> {
        if rows.len() != self.batch * self.time * (self.lanes * 120 + 4)
            || basis.len() != self.event_offset() + self.lanes * 16
            || rows.iter().chain(basis).any(|v| !v.is_finite())
        {
            candle_core::bail!("event packed shape or finite coefficient differs");
        }
        Ok(())
    }
    fn scores(&self, row: &[f32], basis: &[f32], old: &[u8; 4], lane: usize) -> [f64; 120] {
        std::array::from_fn(|d| {
            let at = (lane * 120 + d) * 4;
            let mut value = f64::from(row[lane * 120 + d])
                + dot(
                    root(old[lane]),
                    std::array::from_fn(|i| f64::from(basis[at + i])),
                );
            if self.lanes > 1 {
                let at = self.lanes * 480 + at;
                value += dot(
                    root(old[(lane + 1) % self.lanes]),
                    std::array::from_fn(|i| f64::from(basis[at + i])),
                );
            }
            value
        })
    }
    fn forward(&self, rows: &[f32], basis: &[f32]) -> candle_core::Result<(Vec<Step>, Vec<f32>)> {
        self.validate(rows, basis)?;
        let mut trace = Vec::with_capacity(self.batch * self.time);
        let mut output = Vec::with_capacity(self.batch * self.time * 4);
        let rowwidth = self.lanes * 120 + 4;
        let id = group_table().identity;
        for b in 0..self.batch {
            let mut state = [id; 4];
            for t in 0..self.time {
                if self.reset {
                    state.fill(id);
                }
                let row = &rows[(b * self.time + t) * rowwidth..(b * self.time + t + 1) * rowwidth];
                let old = state;
                let mut action = [id; 4];
                for lane in 0..self.lanes {
                    action[lane] = best(&self.scores(row, basis, &old, lane)) as u8;
                    state[lane] = group_table().product
                        [usize::from(old[lane]) * ROW_STRIDE + usize::from(action[lane])];
                }
                for event in 0..4 {
                    let mut value = f64::from(row[self.lanes * 120 + event]);
                    for lane in 0..self.lanes {
                        let at = self.event_offset() + (lane * 4 + event) * 4;
                        value += dot(
                            root(state[lane]),
                            std::array::from_fn(|i| f64::from(basis[at + i])),
                        );
                    }
                    output.push(value);
                }
                trace.push(Step {
                    old,
                    new: state,
                    action,
                });
            }
        }
        Ok((trace, checked_f32(output)?))
    }
    fn backward(
        &self,
        rows: &[f32],
        basis: &[f32],
        grad: &[f32],
    ) -> candle_core::Result<(Vec<f32>, Vec<f32>)> {
        let (trace, _) = self.forward(rows, basis)?;
        if grad.len() != self.batch * self.time * 4 || grad.iter().any(|v| !v.is_finite()) {
            candle_core::bail!("event upstream shape or finite gradient differs");
        }
        let mut dr = vec![0f64; rows.len()];
        let mut db = vec![0f64; basis.len()];
        let rowwidth = self.lanes * 120 + 4;
        for b in 0..self.batch {
            let mut future = [[0.; 4]; 4];
            for t in (0..self.time).rev() {
                let rowidx = b * self.time + t;
                let step = &trace[rowidx];
                let row = &rows[rowidx * rowwidth..(rowidx + 1) * rowwidth];
                if self.reset {
                    future = [[0.; 4]; 4];
                }
                let mut old_gradient = [[0.; 4]; 4];
                for event in 0..4 {
                    let g = f64::from(grad[rowidx * 4 + event]);
                    dr[rowidx * rowwidth + self.lanes * 120 + event] += g;
                    for lane in 0..self.lanes {
                        let at = self.event_offset() + (lane * 4 + event) * 4;
                        let q = root(step.new[lane]);
                        for i in 0..4 {
                            db[at + i] += g * q[i];
                            future[lane][i] += g * f64::from(basis[at + i]);
                        }
                    }
                }
                for lane in 0..self.lanes {
                    let g = tangent(root(step.new[lane]), future[lane]);
                    let (direct, action_gradient) =
                        hamilton_pullback(root(step.old[lane]), root(step.action[lane]), g);
                    add(&mut old_gradient[lane], direct);
                    let p = probabilities(&self.scores(row, basis, &step.old, lane));
                    let credit =
                        std::array::from_fn::<_, 120, _>(|d| dot(action_gradient, roots()[d]));
                    let mean = (0..120).map(|d| p[d] * credit[d]).sum::<f64>();
                    for d in 0..120 {
                        let dz = p[d] * (credit[d] - mean);
                        dr[rowidx * rowwidth + lane * 120 + d] += dz;
                        let at = (lane * 120 + d) * 4;
                        let q = root(step.old[lane]);
                        for i in 0..4 {
                            db[at + i] += dz * q[i];
                            old_gradient[lane][i] += dz * f64::from(basis[at + i]);
                        }
                        if self.lanes > 1 {
                            let neighbor = (lane + 1) % self.lanes;
                            let q = root(step.old[neighbor]);
                            let at = self.lanes * 480 + at;
                            for i in 0..4 {
                                db[at + i] += dz * q[i];
                                old_gradient[neighbor][i] += dz * f64::from(basis[at + i]);
                            }
                        }
                    }
                }
                future = old_gradient;
            }
        }
        Ok((checked_f32(dr)?, checked_f32(db)?))
    }
}
fn contiguous<'a>(s: &'a CpuStorage, l: &Layout) -> candle_core::Result<&'a [f32]> {
    let v = s.as_slice::<f32>()?;
    match l.contiguous_offsets() {
        Some((a, b)) => Ok(&v[a..b]),
        None => candle_core::bail!("event operation requires contiguous F32 CPU input"),
    }
}
impl CustomOp2 for EventOp {
    fn name(&self) -> &'static str {
        "geometric-event-hard-context"
    }
    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        Ok((
            CpuStorage::F32(self.forward(contiguous(s1, l1)?, contiguous(s2, l2)?)?.1),
            Shape::from((self.batch, self.time, 4)),
        ))
    }
    fn bwd(
        &self,
        rows: &Tensor,
        basis: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (dr, db) = self.backward(
            &rows.flatten_all()?.to_vec1::<f32>()?,
            &basis.flatten_all()?.to_vec1::<f32>()?,
            &grad.flatten_all()?.to_vec1::<f32>()?,
        )?;
        Ok((
            Some(Tensor::from_vec(dr, rows.shape(), rows.device())?),
            Some(Tensor::from_vec(db, basis.shape(), basis.device())?),
        ))
    }
}

// The source bundle is separate from the frozen base reader. Its parameter
// file contains ONLY the newly learned controller; base_* hashes retain the
// original reader, including its now-unused historical float controller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventBaseBinding {
    pub base_model_sha256: String,
    pub base_config_sha256: String,
    pub base_address_sha256: String,
    pub base_span_sha256: String,
    pub vocab_size: usize,
}
impl EventBaseBinding {
    fn read(directory: &Path, vocab_size: usize) -> Result<Self> {
        let names = [
            "model.safetensors",
            "config.json",
            GEOMETRIC_ADDRESS_RECORD,
            GEOMETRIC_SPAN_RECORD,
        ];
        let before = names
            .iter()
            .map(|name| fs::read(directory.join(name)))
            .collect::<std::io::Result<Vec<_>>>()?;
        let config: StackConfig = serde_json::from_slice(&before[1])?;
        config.validate()?;
        if config.vocab_size != vocab_size
            || StackModel::saved_geometric_address(directory)?.is_none()
            || StackModel::saved_geometric_span(directory)?.is_none()
        {
            return Err(invalid(
                "event source needs matching saved geometric span/address reader",
            ));
        }
        for (name, bytes) in names.iter().zip(&before) {
            if fs::read(directory.join(name))? != *bytes {
                return Err(invalid("event base source changed during admission"));
            }
        }
        Ok(Self {
            base_model_sha256: sha256_bytes(&before[0]),
            base_config_sha256: sha256_bytes(&before[1]),
            base_address_sha256: sha256_bytes(&before[2]),
            base_span_sha256: sha256_bytes(&before[3]),
            vocab_size,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventParameterIdentity {
    pub name: String,
    pub shape: Vec<usize>,
    pub f32_bytes_sha256: String,
}

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .collect()
}
fn parameter_values(weights: &EventWeights) -> Result<BTreeMap<String, Vec<f32>>> {
    weights.validate()?;
    weights
        .parameters
        .iter()
        .map(|(name, value)| Ok((name.clone(), value.flatten_all()?.to_vec1::<f32>()?)))
        .collect()
}
fn parameter_identities(weights: &EventWeights) -> Result<Vec<EventParameterIdentity>> {
    let values = parameter_values(weights)?;
    Ok(weights
        .config
        .shapes()
        .into_iter()
        .map(|(name, shape)| EventParameterIdentity {
            f32_bytes_sha256: sha256_bytes(&f32_bytes(&values[&name])),
            name,
            shape,
        })
        .collect())
}
fn parameter_bytes(weights: &EventWeights) -> Result<Vec<u8>> {
    let values = parameter_values(weights)?;
    let bytes = values
        .into_iter()
        .map(|(name, values)| (name, f32_bytes(&values)))
        .collect::<BTreeMap<_, _>>();
    let views = weights
        .config
        .shapes()
        .into_iter()
        .map(|(name, shape)| {
            let view = TensorView::new(SafeDtype::F32, shape, &bytes[&name])?;
            Ok((name, view))
        })
        .collect::<std::result::Result<Vec<_>, safetensors::SafeTensorError>>()?;
    Ok(safetensors::serialize(views, None)?)
}
fn restore_parameters(config: &GeometricEventConfig, bytes: &[u8]) -> Result<EventWeights> {
    config.validate()?;
    let archive = SafeTensors::deserialize(bytes)?;
    let shapes = config.shapes();
    if archive
        .names()
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>()
        != shapes.keys().cloned().collect()
    {
        return Err(invalid("event source parameter file set differs"));
    }
    let weights = EventWeights::new(config.vocab_size, config.lanes, config.seed)?;
    for (name, shape) in shapes {
        let tensor = archive.tensor(&name)?;
        if tensor.dtype() != SafeDtype::F32 || tensor.shape() != shape {
            return Err(invalid("event source parameter shape/type differs"));
        }
        let data = tensor.data();
        if data.len() != shape.iter().product::<usize>() * 4 {
            return Err(invalid("event source parameter byte count differs"));
        }
        let values = data
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        if values.iter().any(|v| !v.is_finite()) {
            return Err(invalid("nonfinite saved event parameter"));
        }
        weights.parameters[&name].set(&Tensor::from_vec(values, shape, &Device::Cpu)?)?;
    }
    weights.validate()?;
    Ok(weights)
}

fn admit_geometry(payload: &[u8]) -> Result<HistoricalH4Tables> {
    if payload != PINNED {
        return Err(invalid(
            "event algebra payload differs from pinned geometry",
        ));
    }
    let tables = HistoricalH4Tables::from_bytes(payload).map_err(|e| invalid(e.to_string()))?;
    if Some(
        mathematical_sha256(payload)
            .map_err(|e| invalid(e.to_string()))?
            .as_str(),
    ) != TRUSTED_MATHEMATICAL_SHA256
    {
        return Err(invalid("event mathematical geometry digest differs"));
    }
    let exact = historical_roots();
    let phi = (1. + 5f64.sqrt()) / 2.;
    if exact.len() != GROUP_ORDER || roots().len() != GROUP_ORDER {
        return Err(invalid("event signed root count differs"));
    }
    for i in 0..GROUP_ORDER {
        for j in 0..4 {
            for k in 0..2 {
                if exact[i][j][k] != i64::from(H4_ROOT_COEFFICIENTS[i][j][k]) {
                    return Err(invalid("event exact coefficient order differs"));
                }
            }
            let [a, b] = H4_ROOT_COEFFICIENTS[i][j];
            if (roots()[i][j] - (f64::from(a) + f64::from(b) * phi) / 2.).abs() > 1e-14 {
                return Err(invalid("event canonical root order differs"));
            }
        }
        if encode_lane(roots()[i].map(|x| x as f32))?.root != i as u8 {
            return Err(invalid("event signed root classification differs"));
        }
        let a = H4Code::try_from(i as u8).map_err(|e| invalid(e.to_string()))?;
        if tables.inverse(a).index() != group_table().inverse[i] {
            return Err(invalid("event inverse table differs"));
        }
        for j in 0..GROUP_ORDER {
            let b = H4Code::try_from(j as u8).map_err(|e| invalid(e.to_string()))?;
            if tables.compose(a, b).index() != group_table().product[i * ROW_STRIDE + j] {
                return Err(invalid("event product table differs"));
            }
        }
    }
    if tables.identity().index() != group_table().identity {
        return Err(invalid("event identity differs"));
    }
    Ok(tables)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventSourceMetadata {
    pub schema: String,
    pub config: GeometricEventConfig,
    pub base: EventBaseBinding,
    pub parameters: Vec<EventParameterIdentity>,
    pub parameter_file_sha256: String,
    pub parameter_file_bytes: usize,
    pub tokenizer_sha256: String,
    pub tokenizer_bytes: usize,
    pub algebra_payload_sha256: String,
    pub algebra_mathematical_sha256: String,
    pub training_geometry_sha256: String,
}
fn source_metadata(
    weights: &EventWeights,
    parameter_file: &[u8],
    base: EventBaseBinding,
    tokenizer: &[u8],
) -> Result<EventSourceMetadata> {
    if tokenizer.is_empty() {
        return Err(invalid(
            "event source requires explicit tokenizer identity bytes",
        ));
    }
    Ok(EventSourceMetadata {
        schema: SOURCE_SCHEMA.into(),
        config: weights.config.clone(),
        base,
        parameters: parameter_identities(weights)?,
        parameter_file_sha256: sha256_bytes(parameter_file),
        parameter_file_bytes: parameter_file.len(),
        tokenizer_sha256: sha256_bytes(tokenizer),
        tokenizer_bytes: tokenizer.len(),
        algebra_payload_sha256: sha256_bytes(PINNED),
        algebra_mathematical_sha256: mathematical_sha256(PINNED)
            .map_err(|e| invalid(e.to_string()))?,
        training_geometry_sha256: geometry_digest().into(),
    })
}
fn require_file_set(directory: &Path, files: &[&str]) -> Result<()> {
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(invalid("event artifact contains a non-regular file"));
        }
        actual.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("event artifact filename is not UTF-8"))?,
        );
    }
    if actual != files.iter().map(|name| name.to_string()).collect() {
        return Err(invalid("event artifact file set differs"));
    }
    Ok(())
}
fn write_new(directory: &Path, files: &[(&str, &[u8])]) -> Result<()> {
    fs::create_dir(directory)?;
    for (name, bytes) in files {
        fs::File::create_new(directory.join(name))?.write_all(bytes)?;
    }
    Ok(())
}
impl EventWeights {
    /// Save newly learned event parameters separately from the frozen base.
    /// The directory must not exist; no partial or prior artifact is replaced.
    pub fn save_source(
        &self,
        directory: &Path,
        base_directory: &Path,
        tokenizer: &[u8],
    ) -> Result<()> {
        let base = EventBaseBinding::read(base_directory, self.config.vocab_size)?;
        let parameters = parameter_bytes(self)?;
        admit_geometry(PINNED)?;
        let metadata =
            serde_json::to_vec_pretty(&source_metadata(self, &parameters, base, tokenizer)?)?;
        write_new(
            directory,
            &[
                (SOURCE_FILES[0], &metadata),
                (SOURCE_FILES[1], &parameters),
                (SOURCE_FILES[2], PINNED),
                (SOURCE_FILES[3], tokenizer),
            ],
        )
    }
    pub fn load_source(directory: &Path, base_directory: &Path, tokenizer: &[u8]) -> Result<Self> {
        Ok(load_source(directory, base_directory, tokenizer)?.0)
    }
}
fn load_source(
    directory: &Path,
    base_directory: &Path,
    tokenizer: &[u8],
) -> Result<(EventWeights, EventSourceMetadata, Vec<u8>)> {
    require_file_set(directory, &SOURCE_FILES)?;
    let metadata_bytes = fs::read(directory.join(SOURCE_FILES[0]))?;
    let metadata: EventSourceMetadata = serde_json::from_slice(&metadata_bytes)?;
    metadata.config.validate()?;
    // Bound parameter reads by the admitted shapes, allowing a small header.
    let maximum = metadata
        .config
        .shapes()
        .values()
        .map(|s| s.iter().product::<usize>() * 4)
        .sum::<usize>()
        + 16_384;
    if fs::metadata(directory.join(SOURCE_FILES[1]))?.len() > maximum as u64
        || fs::metadata(directory.join(SOURCE_FILES[2]))?.len() != PINNED.len() as u64
        || fs::metadata(directory.join(SOURCE_FILES[3]))?.len() != tokenizer.len() as u64
    {
        return Err(invalid("event source payload length differs"));
    }
    let parameter_file = fs::read(directory.join(SOURCE_FILES[1]))?;
    admit_geometry(&fs::read(directory.join(SOURCE_FILES[2]))?)?;
    if fs::read(directory.join(SOURCE_FILES[3]))? != tokenizer {
        return Err(invalid("event source tokenizer identity differs"));
    }
    let weights = restore_parameters(&metadata.config, &parameter_file)?;
    let expected = source_metadata(
        &weights,
        &parameter_file,
        EventBaseBinding::read(base_directory, metadata.config.vocab_size)?,
        tokenizer,
    )?;
    if metadata != expected {
        return Err(invalid(
            "event source parameter/base/tokenizer/geometry binding differs",
        ));
    }
    Ok((weights, metadata, metadata_bytes))
}

fn quantize(value: f64) -> Result<i32> {
    let scaled = (value * (1u64 << FRACTIONAL_BITS) as f64).round();
    if !scaled.is_finite() || scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
        return Err(invalid("event potential overflows fixed signed Q24"));
    }
    Ok(scaled as i32)
}
struct ExpandedEvents {
    token_transition: Vec<i32>,
    own: Vec<i32>,
    neighbor: Option<Vec<i32>>,
    token_event: Vec<i32>,
    state_event: Vec<i32>,
}
impl ExpandedEvents {
    fn compile(weights: &EventWeights) -> Result<Self> {
        let values = parameter_values(weights)?;
        let (v, l) = (weights.config.vocab_size, weights.config.lanes);
        let mut token_transition = vec![0; v * l * 128];
        let mut own = vec![0; l * 128 * 128];
        let mut neighbor = if l > 1 {
            Some(vec![0; l * 128 * 128])
        } else {
            None
        };
        let mut token_event = vec![0; v * 4];
        let mut state_event = vec![0; l * 128 * 4];
        for token in 0..v {
            for lane in 0..l {
                for action in 0..120 {
                    token_transition[(token * l + lane) * 128 + action] =
                        quantize(f64::from(values[TT][(token * l + lane) * 120 + action]))?;
                }
            }
            for e in 0..4 {
                token_event[token * 4 + e] = quantize(f64::from(values[TE][token * 4 + e]))?;
            }
        }
        for lane in 0..l {
            for state in 0..120 {
                for action in 0..120 {
                    let at = (lane * 120 + action) * 4;
                    own[(lane * 128 + state) * 128 + action] = quantize(dot(
                        roots()[state],
                        std::array::from_fn(|i| f64::from(values[ST][at + i])),
                    ))?;
                    if let Some(neighbor) = &mut neighbor {
                        neighbor[(lane * 128 + state) * 128 + action] = quantize(dot(
                            roots()[state],
                            std::array::from_fn(|i| f64::from(values[NT][at + i])),
                        ))?;
                    }
                }
                for e in 0..4 {
                    let at = (lane * 4 + e) * 4;
                    state_event[(lane * 128 + state) * 4 + e] = quantize(dot(
                        roots()[state],
                        std::array::from_fn(|i| f64::from(values[SE][at + i])),
                    ))?;
                }
            }
        }
        Ok(Self {
            token_transition,
            own,
            neighbor,
            token_event,
            state_event,
        })
    }
    fn native(&self, config: &GeometricEventConfig) -> Result<NativeEventTables> {
        NativeEventTables::new(
            config.vocab_size,
            config.lanes,
            &self.token_transition,
            &self.own,
            self.neighbor.as_deref(),
            &self.token_event,
            &self.state_event,
        )
        .map_err(|e| invalid(e.to_string()))
    }
    fn bytes(&self) -> Vec<u8> {
        self.token_transition
            .iter()
            .chain(&self.own)
            .chain(self.neighbor.iter().flatten())
            .chain(&self.token_event)
            .chain(&self.state_event)
            .flat_map(|v| v.to_le_bytes())
            .collect()
    }
    fn lengths(&self) -> Vec<usize> {
        vec![
            self.token_transition.len(),
            self.own.len(),
            self.neighbor.as_ref().map_or(0, Vec::len),
            self.token_event.len(),
            self.state_event.len(),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledEventMetadata {
    pub schema: String,
    pub source: EventSourceMetadata,
    pub source_metadata_sha256: String,
    pub coefficient_policy: String,
    pub fractional_bits: u32,
    pub table_layout: String,
    pub table_lengths: Vec<usize>,
    pub table_bytes: usize,
    pub table_sha256: String,
}
fn compiled_metadata(
    source: EventSourceMetadata,
    source_bytes: &[u8],
    tables: &ExpandedEvents,
) -> CompiledEventMetadata {
    let bytes = tables.bytes();
    CompiledEventMetadata {
        schema:COMPILED_SCHEMA.into(),source,source_metadata_sha256:sha256_bytes(source_bytes),
        coefficient_policy:COEFFICIENT_POLICY.into(),fractional_bits:FRACTIONAL_BITS,
        table_layout:"token[V,L,128];own[L,128,128];next-neighbor[L,128,128]-if-L>1;token-event[V,4];new-state-event[L,128,4];zero-padding/1".into(),
        table_lengths:tables.lengths(),table_bytes:bytes.len(),table_sha256:sha256_bytes(&bytes),
    }
}
pub struct CompiledEvents {
    metadata: CompiledEventMetadata,
    native: NativeEventTables,
    geometry: HistoricalH4Tables,
    table_bytes: Vec<u8>,
    tokenizer: Vec<u8>,
}
impl CompiledEvents {
    pub fn compile(
        weights: &EventWeights,
        source_directory: &Path,
        base_directory: &Path,
        tokenizer: &[u8],
    ) -> Result<Self> {
        let (source, metadata, metadata_bytes) =
            load_source(source_directory, base_directory, tokenizer)?;
        if weights.config != source.config || parameter_identities(weights)? != metadata.parameters
        {
            return Err(invalid("live event coefficients differ from saved source"));
        }
        let tables = ExpandedEvents::compile(&source)?;
        Ok(Self {
            native: tables.native(&source.config)?,
            geometry: admit_geometry(PINNED)?,
            table_bytes: tables.bytes(),
            metadata: compiled_metadata(metadata, &metadata_bytes, &tables),
            tokenizer: tokenizer.to_vec(),
        })
    }
    pub fn config(&self) -> &GeometricEventConfig {
        &self.metadata.source.config
    }
    pub fn vocab_size(&self) -> usize {
        self.config().vocab_size
    }
    pub fn metadata(&self) -> &CompiledEventMetadata {
        &self.metadata
    }
    pub fn validate_for(&self, weights: &EventWeights) -> Result<()> {
        if weights.config != *self.config()
            || parameter_identities(weights)? != self.metadata.source.parameters
        {
            return Err(invalid(
                "compiled event tables are stale for live coefficients/configuration",
            ));
        }
        Ok(())
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        let metadata = serde_json::to_vec_pretty(&self.metadata)?;
        write_new(
            directory,
            &[
                (NATIVE_FILES[0], &metadata),
                (NATIVE_FILES[1], &self.table_bytes),
                (NATIVE_FILES[2], PINNED),
                (NATIVE_FILES[3], &self.tokenizer),
            ],
        )
    }
    /// Admit only a byte-for-byte recompilation of the actual supplied source.
    /// Resealing modified table bytes with new hashes cannot satisfy this check.
    pub fn load(
        directory: &Path,
        source_directory: &Path,
        base_directory: &Path,
        tokenizer: &[u8],
    ) -> Result<Self> {
        require_file_set(directory, &NATIVE_FILES)?;
        let (source, source_metadata, source_bytes) =
            load_source(source_directory, base_directory, tokenizer)?;
        let tables = ExpandedEvents::compile(&source)?;
        let expected = compiled_metadata(source_metadata, &source_bytes, &tables);
        if fs::metadata(directory.join(NATIVE_FILES[1]))?.len() != expected.table_bytes as u64
            || fs::metadata(directory.join(NATIVE_FILES[2]))?.len() != PINNED.len() as u64
            || fs::metadata(directory.join(NATIVE_FILES[3]))?.len() != tokenizer.len() as u64
        {
            return Err(invalid("compiled event artifact lengths differ"));
        }
        let actual: CompiledEventMetadata =
            serde_json::from_slice(&fs::read(directory.join(NATIVE_FILES[0]))?)?;
        let table_bytes = tables.bytes();
        if actual != expected
            || fs::read(directory.join(NATIVE_FILES[1]))? != table_bytes
            || fs::read(directory.join(NATIVE_FILES[3]))? != tokenizer
        {
            return Err(invalid(
                "compiled events differ from actual source recompilation",
            ));
        }
        let geometry = admit_geometry(&fs::read(directory.join(NATIVE_FILES[2]))?)?;
        Ok(Self {
            metadata: actual,
            native: tables.native(&source.config)?,
            geometry,
            table_bytes,
            tokenizer: tokenizer.to_vec(),
        })
    }
}

/// `actions` are integer event decisions and can feed SpanRegister directly.
/// Group transition actions and NEW state codes are separate diagnostics.
#[derive(Clone, Debug)]
pub struct NativeEventTrace {
    pub batch: usize,
    pub time: usize,
    pub lanes: usize,
    pub actions: Vec<SpanAction>,
    pub transition_actions: Vec<Vec<u8>>,
    pub states: Vec<Vec<u8>>,
    pub event_scores: Vec<[i64; 4]>,
    pub coefficient_reads: usize,
}
pub fn trace_native(
    ids: &[u32],
    batch: usize,
    time: usize,
    compiled: &CompiledEvents,
    reset_each_token: bool,
) -> Result<NativeEventTrace> {
    if batch == 0
        || time == 0
        || batch.checked_mul(time) != Some(ids.len())
        || ids.iter().any(|&id| id as usize >= compiled.vocab_size())
    {
        return Err(invalid(
            "native events need nonempty valid [batch*time] token ids",
        ));
    }
    let lanes = compiled.config().lanes;
    let mut trace = NativeEventTrace {
        batch,
        time,
        lanes,
        actions: Vec::with_capacity(ids.len()),
        transition_actions: Vec::with_capacity(ids.len()),
        states: Vec::with_capacity(ids.len()),
        event_scores: Vec::with_capacity(ids.len()),
        coefficient_reads: 0,
    };
    let mut state = NativeEventState::new(lanes).map_err(|e| invalid(e.to_string()))?;
    for b in 0..batch {
        state.reset();
        for t in 0..time {
            if reset_each_token {
                state.reset();
            }
            let step = state
                .step(
                    ids[b * time + t] as usize,
                    &compiled.native,
                    &compiled.geometry,
                )
                .map_err(|e| invalid(e.to_string()))?;
            trace.actions.push(step.event);
            trace
                .transition_actions
                .push(step.actions[..lanes].iter().map(|c| c.index()).collect());
            trace
                .states
                .push(step.states[..lanes].iter().map(|c| c.index()).collect());
            trace.event_scores.push(step.event_scores);
            trace.coefficient_reads = trace
                .coefficient_reads
                .checked_add(step.coefficient_reads)
                .ok_or_else(|| invalid("event diagnostic read count overflow"))?;
        }
    }
    Ok(trace)
}

/// Diagnostic float boundary only; native event-to-span integration should pass
/// `trace.actions` directly, without a float conversion or a second argmax.
pub fn event_one_hot_for_replay(trace: &NativeEventTrace) -> Result<Tensor> {
    if trace.batch == 0
        || trace.time == 0
        || trace.batch.checked_mul(trace.time) != Some(trace.actions.len())
    {
        return Err(invalid("event trace shape differs"));
    }
    let mut values = vec![0f32; trace.actions.len() * 4];
    for (i, event) in trace.actions.iter().enumerate() {
        values[i * 4 + *event as usize] = 1.;
    }
    Ok(Tensor::from_vec(
        values,
        (trace.batch, trace.time, 4),
        &Device::Cpu,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_address::GeometricAddressConfig;
    use crate::geometric_span::GeometricSpanConfig;
    use crate::geometric_stack::{logits_cross_entropy, ReadIdentityLatch, ReadScore, StackArch};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn norm(values: &[f32]) -> f64 {
        values.iter().map(|&v| f64::from(v).abs()).sum()
    }
    fn set(weights: &EventWeights, name: &str, values: Vec<f32>) -> Result<()> {
        let variable = &weights.parameters[name];
        variable.set(&Tensor::from_vec(values, variable.shape(), &Device::Cpu)?)?;
        Ok(())
    }
    #[test]
    fn geometric_event_delayed_loss_reaches_earlier_transitions() -> Result<()> {
        let weights = EventWeights::new(4, 2, 719)?;
        let ids = [0, 1, 2];
        let logits = weights
            .logits_impl(&ids, 1, 3, false, true)?
            .reshape((3, 4))?
            .narrow(0, 2, 1)?;
        let loss = logits_cross_entropy(&logits, &[2], None)?;
        let gradients = loss.backward()?;
        let transition = gradients
            .get(weights.parameters[TT].as_tensor())
            .ok_or_else(|| invalid("missing event transition gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(
            norm(&transition[..240]) > 1e-8,
            "delayed event loss must reach first-token actions"
        );
        for name in [ST, NT, SE] {
            let gradient = gradients
                .get(weights.parameters[name].as_tensor())
                .ok_or_else(|| invalid(format!("missing {name} gradient")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(gradient.iter().all(|v| v.is_finite()));
            assert!(norm(&gradient) > 1e-8, "{name} has no credit");
        }
        assert!(
            gradients.get(weights.parameters[TE].as_tensor()).is_none(),
            "direct event bias was detached"
        );
        let reset = weights
            .logits_impl(&ids, 1, 3, true, true)?
            .reshape((3, 4))?
            .narrow(0, 2, 1)?;
        let reset_gradients = logits_cross_entropy(&reset, &[2], None)?.backward()?;
        let reset_transition = reset_gradients
            .get(weights.parameters[TT].as_tensor())
            .ok_or_else(|| invalid("missing reset transition gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(
            norm(&reset_transition[..480]),
            0.,
            "per-token reset must sever earlier context credit"
        );
        assert!(norm(&reset_transition[480..720]) > 1e-8);
        Ok(())
    }

    // Finite-difference the DECLARED local surrogate with hard branch anchors
    // held fixed. Finite-differencing the discontinuous argmax forward would
    // test a different derivative and would necessarily report zero almost
    // everywhere. Here recurrent score dependence on both old lanes is live.
    fn surrogate_scalar(
        op: &EventOp,
        base_rows: &[f32],
        base_basis: &[f32],
        rows: &[f64],
        basis: &[f64],
        upstream: &[f32],
    ) -> Result<f64> {
        let (trace, _) = op.forward(base_rows, base_basis)?;
        let rowwidth = op.lanes * 120 + 4;
        let mut output = 0.;
        for b in 0..op.batch {
            let mut states = [root(group_table().identity); 4];
            for t in 0..op.time {
                if op.reset {
                    states.fill(root(group_table().identity));
                }
                let at = b * op.time + t;
                let step = &trace[at];
                let old = states;
                for lane in 0..op.lanes {
                    let scores = std::array::from_fn(|d| {
                        let p = (lane * 120 + d) * 4;
                        let mut z = rows[at * rowwidth + lane * 120 + d]
                            + dot(old[lane], std::array::from_fn(|i| basis[p + i]));
                        if op.lanes > 1 {
                            z += dot(
                                old[(lane + 1) % op.lanes],
                                std::array::from_fn(|i| basis[op.lanes * 480 + p + i]),
                            );
                        }
                        z
                    });
                    let p = probabilities(&scores);
                    let p0 = probabilities(&op.scores(
                        &base_rows[at * rowwidth..(at + 1) * rowwidth],
                        base_basis,
                        &step.old,
                        lane,
                    ));
                    let mut action = root(step.action[lane]);
                    for d in 0..120 {
                        for i in 0..4 {
                            action[i] += (p[d] - p0[d]) * roots()[d][i];
                        }
                    }
                    let moved = hamilton(old[lane], action);
                    let anchor = hamilton(root(step.old[lane]), root(step.action[lane]));
                    let delta = tangent(
                        root(step.new[lane]),
                        std::array::from_fn(|i| moved[i] - anchor[i]),
                    );
                    states[lane] = std::array::from_fn(|i| root(step.new[lane])[i] + delta[i]);
                }
                for e in 0..4 {
                    let mut logit = rows[at * rowwidth + op.lanes * 120 + e];
                    for lane in 0..op.lanes {
                        let p = op.event_offset() + (lane * 4 + e) * 4;
                        logit += dot(states[lane], std::array::from_fn(|i| basis[p + i]));
                    }
                    output += f64::from(upstream[at * 4 + e]) * logit;
                }
            }
        }
        Ok(output)
    }
    fn largest(values: &[f32], start: usize, end: usize) -> usize {
        let mut winner = start;
        for i in start + 1..end {
            if values[i].abs() > values[winner].abs() {
                winner = i;
            }
        }
        winner
    }
    #[test]
    fn geometric_event_local_surrogate_matches_delayed_adjoint() -> Result<()> {
        let weights = EventWeights::new(4, 2, 173)?;
        let (rows, basis) = weights.inputs(&[0, 1, 2], 1, 3, false)?;
        let rows = rows.flatten_all()?.to_vec1::<f32>()?;
        let basis = basis.to_vec1::<f32>()?;
        let op = EventOp {
            batch: 1,
            time: 3,
            lanes: 2,
            reset: false,
        };
        let upstream = [0., 0., 0., 0., 0., 0., 0., 0., 0.2, -0.7, 0.3, 0.1];
        let (dr, db) = op.backward(&rows, &basis, &upstream)?;
        let r = rows.iter().map(|&v| f64::from(v)).collect::<Vec<_>>();
        let p = basis.iter().map(|&v| f64::from(v)).collect::<Vec<_>>();
        let eps = 1e-5;
        for (in_rows, index) in [
            (true, largest(&dr, 0, 240)),
            (false, largest(&db, 0, 960)),
            (false, largest(&db, 960, 1920)),
            (false, largest(&db, 1920, 1952)),
        ] {
            let mut rp = r.clone();
            let mut rm = r.clone();
            let mut pp = p.clone();
            let mut pm = p.clone();
            let analytic = if in_rows {
                rp[index] += eps;
                rm[index] -= eps;
                f64::from(dr[index])
            } else {
                pp[index] += eps;
                pm[index] -= eps;
                f64::from(db[index])
            };
            let numerical = (surrogate_scalar(&op, &rows, &basis, &rp, &pp, &upstream)?
                - surrogate_scalar(&op, &rows, &basis, &rm, &pm, &upstream)?)
                / (2. * eps);
            assert!(analytic.abs() > 1e-9, "vacuous local derivative fixture");
            assert!(
                (analytic - numerical).abs() < 1e-7 + analytic.abs() * 2e-5,
                "rows={in_rows} index={index} analytic={analytic} numerical={numerical}"
            );
        }
        Ok(())
    }

    #[test]
    fn geometric_event_causal_reset_and_synchronous_hard_trace() -> Result<()> {
        let weights = EventWeights::new(4, 2, 37)?;
        // Pin large-margin token actions, leaving context/event bases nonzero.
        let mut tt = vec![-2.; 4 * 2 * 120];
        for token in 0..4 {
            for lane in 0..2 {
                tt[(token * 2 + lane) * 120 + 2 + token + lane] = 2.;
            }
        }
        set(&weights, TT, tt)?;
        let ids = [0, 1, 2, 3, 3, 2, 1, 0];
        let whole = weights.float_trace(&ids, 2, 4, false)?;
        for b in 0..2 {
            let single = weights.float_trace(&ids[b * 4..(b + 1) * 4], 1, 4, false)?;
            assert_eq!(whole.states[b * 4..(b + 1) * 4], single.states);
            assert_eq!(whole.logits[b * 16..(b + 1) * 16], single.logits);
            for end in 1..=4 {
                let prefix = weights.float_trace(&ids[b * 4..b * 4 + end], 1, end, false)?;
                assert_eq!(prefix.states, single.states[..end]);
                assert_eq!(prefix.logits, single.logits[..end * 4]);
            }
        }
        let reset = weights.float_trace(&ids, 2, 4, true)?;
        assert_ne!(whole.states, reset.states);
        for (i, &id) in ids.iter().enumerate() {
            let single = weights.float_trace(&[id], 1, 1, false)?;
            assert_eq!(reset.states[i], single.states[0]);
            assert_eq!(reset.logits[i * 4..(i + 1) * 4], single.logits);
        }
        assert!(weights.event_logits(&[4], 1, 1).is_err());
        assert!(weights.event_logits(&[0], 0, 1).is_err());
        assert!(GeometricEventConfig::new(4097, 1, 1).is_err());
        let mut nonfinite = weights.parameters[ST].flatten_all()?.to_vec1::<f32>()?;
        nonfinite[0] = f32::NAN;
        set(&weights, ST, nonfinite)?;
        assert!(weights.event_logits(&[0], 1, 1).is_err());
        Ok(())
    }

    fn base_fixture() -> Result<PathBuf> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-event-{}-{clock}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 4,
            width: 8,
            heads: 1,
            mlp_hidden: 16,
            context: 16,
            pattern: "rra".into(),
            read: ReadScore::Lorentz,
            rotation: true,
            seed: 19,
            memory: None,
            select: None,
            pointer: None,
        };
        let mut model = StackModel::new(config, &Device::Cpu)?;
        model.set_read_identity_latch(ReadIdentityLatch::Held)?;
        model.set_geometric_address(GeometricAddressConfig::new(8, 1)?)?;
        model.set_geometric_span(GeometricSpanConfig::new(8)?)?;
        model.save(&directory.join("base"))?;
        Ok(directory)
    }
    #[test]
    fn geometric_event_source_reload_resealed_tamper_and_stale_guard() -> Result<()> {
        let directory = base_fixture()?;
        let base = directory.join("base");
        let source = directory.join("source");
        let artifact = directory.join("native");
        let registry = b"event fixture explicit token registry 0..3/v1";
        let weights = EventWeights::new(4, 2, 183)?;
        weights.save_source(&source, &base, registry)?;
        assert!(weights.save_source(&source, &base, registry).is_err());
        let loaded = EventWeights::load_source(&source, &base, registry)?;
        assert_eq!(
            parameter_identities(&weights)?,
            parameter_identities(&loaded)?
        );
        assert_eq!(
            weights
                .event_logits(&[0, 1, 2], 1, 3)?
                .flatten_all()?
                .to_vec1::<f32>()?,
            loaded
                .event_logits(&[0, 1, 2], 1, 3)?
                .flatten_all()?
                .to_vec1::<f32>()?
        );
        assert!(EventWeights::load_source(&source, &base, b"different token registry").is_err());
        let compiled = CompiledEvents::compile(&loaded, &source, &base, registry)?;
        compiled.save(&artifact)?;
        let reload = CompiledEvents::load(&artifact, &source, &base, registry)?;
        let ids = [0, 1, 2, 3, 3, 2, 1, 0];
        let expected = trace_native(&ids, 2, 4, &compiled, false)?;
        let actual = trace_native(&ids, 2, 4, &reload, false)?;
        assert_eq!(expected.actions, actual.actions);
        assert_eq!(expected.states, actual.states);
        assert_eq!(expected.transition_actions, actual.transition_actions);
        assert_eq!(expected.event_scores, actual.event_scores);
        let reference = loaded.float_trace(&ids, 2, 4, false)?;
        // This seeded, untuned small fixture has ample rounding margin; the
        // general float/native action-retention question remains measured.
        assert_eq!(reference.states, actual.states);
        assert_eq!(reference.actions, actual.transition_actions);
        assert_eq!(
            reference.events,
            actual.actions.iter().map(|&a| a as u8).collect::<Vec<_>>()
        );
        let onehot = event_one_hot_for_replay(&actual)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        for (i, event) in actual.actions.iter().enumerate() {
            assert_eq!(onehot[i * 4 + *event as usize], 1.);
        }
        let reset = trace_native(&ids, 2, 4, &reload, true)?;
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(
                reset.states[i],
                trace_native(&[id], 1, 1, &reload, false)?.states[0]
            );
        }
        assert!(trace_native(&[4], 1, 1, &reload, false).is_err());
        let binary_path = artifact.join(NATIVE_FILES[1]);
        let original = fs::read(&binary_path)?;
        let mut changed = original.clone();
        changed[0] ^= 1;
        fs::write(&binary_path, &changed)?;
        let metadata_path = artifact.join(NATIVE_FILES[0]);
        let metadata_bytes = fs::read(&metadata_path)?;
        let mut resealed: CompiledEventMetadata = serde_json::from_slice(&metadata_bytes)?;
        resealed.table_sha256 = sha256_bytes(&changed);
        fs::write(&metadata_path, serde_json::to_vec_pretty(&resealed)?)?;
        assert!(
            CompiledEvents::load(&artifact, &source, &base, registry).is_err(),
            "rehashed table tamper must fail source recompilation"
        );
        fs::write(&binary_path, &original)?;
        fs::write(&metadata_path, &metadata_bytes)?;
        fs::write(artifact.join("unbound-extra"), b"x")?;
        assert!(CompiledEvents::load(&artifact, &source, &base, registry).is_err());
        fs::remove_file(artifact.join("unbound-extra"))?;
        let mut values = loaded.parameters[TT].flatten_all()?.to_vec1::<f32>()?;
        values[0] = f32::from_bits(values[0].to_bits() ^ 1);
        set(&loaded, TT, values)?;
        assert!(
            reload.validate_for(&loaded).is_err(),
            "even an unchanged quantization bin cannot conceal stale learned bits"
        );
        assert!(CompiledEvents::compile(&loaded, &source, &base, registry).is_err());
        let config_path = base.join("config.json");
        let original_config = fs::read(&config_path)?;
        fs::write(&config_path, b"{}")?;
        assert!(EventWeights::load_source(&source, &base, registry).is_err());
        fs::write(&config_path, original_config)?;
        fs::remove_dir_all(directory)?;
        Ok(())
    }
    #[test]
    fn geometric_event_fixed_codec_padding_and_overflow() -> Result<()> {
        let half = 0.5 / (1u64 << FRACTIONAL_BITS) as f64;
        assert_eq!(quantize(half)?, 1);
        assert_eq!(quantize(-half)?, -1);
        assert_eq!(quantize(-128.)?, i32::MIN);
        assert!(quantize(128.).is_err());
        assert!(quantize(f64::NAN).is_err());
        let weights = EventWeights::new(3, 2, 97)?;
        let expanded = ExpandedEvents::compile(&weights)?;
        for lane in 0..2 {
            for state in 0..128 {
                for action in 0..128 {
                    if state >= 120 || action >= 120 {
                        assert_eq!(expanded.own[(lane * 128 + state) * 128 + action], 0);
                        assert_eq!(
                            expanded
                                .neighbor
                                .as_ref()
                                .ok_or_else(|| invalid("missing neighbor fixture"))?
                                [(lane * 128 + state) * 128 + action],
                            0
                        );
                    }
                }
            }
            for state in 120..128 {
                assert_eq!(
                    &expanded.state_event[(lane * 128 + state) * 4..(lane * 128 + state + 1) * 4],
                    &[0; 4]
                );
            }
        }
        let single = EventWeights::new(3, 1, 98)?;
        let single_tables = ExpandedEvents::compile(&single)?;
        assert!(single_tables.neighbor.is_none());
        single_tables.native(single.config())?;
        let mut values = weights.parameters[TE].flatten_all()?.to_vec1::<f32>()?;
        values[0] = 128.;
        set(&weights, TE, values)?;
        assert!(ExpandedEvents::compile(&weights).is_err());
        Ok(())
    }
}
