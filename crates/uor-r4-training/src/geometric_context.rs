//! Latent finite geometric context, with a separate observable address readout.
//!
//! Each head has up to four signed 2I registers. Token, OLD own, and OLD next
//! neighbor potentials choose right actions; all registers update synchronously.
//! Separate token/NEW own/NEW neighbor potentials emit 120 root logits and 33
//! radius/presence logits per lane. Hidden roots are NOT donor-code targets.
//! Distinct hidden histories may emit the same address code and diverge later.
//! This is a factorization, not a full-state 120^(H*L) lookup or a universal
//! donor conversion. Donor observations need not be Markov-sufficient.
//!
//! Legacy offline backward uses the hard-branch Hamilton adjoint, tangent
//! projection, and a local 120-action expected-root softmax derivative. The
//! explicitly selected finite-choice policy instead carries the full ambient
//! adjoint through both the Hamilton and action-choice paths. Both include own
//! and neighbor state-score derivatives. Hard emitted directions use the
//! tangent/120-softmax surrogate. Answer-to-radius credit uses only the hard
//! present category and its immediately adjacent physical dyadic bins. Absent
//! output stops answer credit; full 33-category CE supplies explicit presence
//! supervision. These are declared biased first-order surrogates, not argmax
//! derivatives. Labels never enter forward or native state transitions.
//!
//! Native compilation expands finite factors to fixed Q24 integer lookup scores
//! (nearest/ties-away; overflow rejected). These four-byte scores are not a
//! four-bit linear-weight codec. Tensor reconstruction and the surrounding
//! reader remain separately scoped offline/replay boundaries.

use crate::geometric_address::{geometry_digest, AddressCode, GeometricAddressConfig};
use crate::geometric_event::CompiledEvents;
use crate::geometric_potential_native::{
    artifact_file_names, CompiledGeometricPotentials, PotentialSourceBinding,
};
use crate::geometric_span_native::{CompiledSpanActions, SpanSourceBinding};
use crate::geometric_stack::{
    StackConfig, StackModel, GEOMETRIC_ADDRESS_RECORD, GEOMETRIC_SPAN_RECORD,
};
use crate::{invalid, sha256_bytes, Result};
use candle_core::{CpuStorage, CustomOp2, DType, Device, Layout, Shape, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use uor_r4_core::native_geometric::learner::{
    embedding::canonical_h4_roots,
    group_table::{group_table, ROW_STRIDE},
};
use uor_r4_integer::geometric_context::{
    ContextTableSlices, NativeContextState, NativeContextTables,
};
use uor_r4_integer::geometric_potential::AddressLane;
use uor_r4_integer::h4_tables::{
    mathematical_sha256, HistoricalH4Tables, TRUSTED_MATHEMATICAL_SHA256,
};

pub const SCHEMA: &str = "uor-r4.geometric-context/1";
const RULE:&str="old-own+old-next-neighbor-within-head;120-earliest-argmax;right-group-update;synchronous;new-state-separate-root120-and-category33-readout;identity-reset/1";
const SURROGATE:&str="latent-new-root-tangent;hard-hamilton;120-softmax-expected-action-root-T1;all-old-state-score-credit;emitted-root-tangent-softmax120;physical-radius-local-neighbor-softmax;absence-answer-stop/1";
/// Full ambient latent-state credit is a declared biased discrete-choice
/// surrogate, not a derivative of the exact argmax or a native runtime change.
pub const FINITE_CHOICE_SURROGATE:&str="latent-new-root-full-adjoint;hard-hamilton-direct-full;120-softmax-expected-action-root-T1-full;all-old-state-score-credit;emitted-root-tangent-softmax120;physical-radius-local-neighbor-softmax;absence-answer-stop/1";
const PACKED_WIDTH: usize = 157;
const LATENT_OFFSET: usize = 153;
const INITIALIZATION:&str="xorshift64-seed;uniform[-.02,.02];transition-identity+.05;category17+1;nonzero-readout-bases/1";
const CATEGORY_RULE:&str="0=absent(root1,bin0);1..32=present(bin=category-1,radius=2^(bin-16));physical-nearest-midpoint-lower-clipped[-16,15]/1";
const POLICY:&str="signed-i32-Q24;nearest-ties-away;overflow-reject;no-centering;root128/category64-zero-padding/1";
const TT: &str = "token_transition";
const ST: &str = "self_transition";
const NT: &str = "neighbor_transition";
const TR: &str = "token_root";
const SR: &str = "self_root";
const NR: &str = "neighbor_root";
const TC: &str = "token_category";
const SC: &str = "self_category";
const NC: &str = "neighbor_category";
const TOKEN: [&str; 3] = [TT, TR, TC];
const OWN: [&str; 3] = [ST, SR, SC];
const NEIGHBOR: [&str; 3] = [NT, NR, NC];
const CLASSES: [usize; 3] = [120, 120, 33];
const PINNED: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
const SOURCE_FILES: [&str; 4] = [
    "metadata.json",
    "context-parameters.safetensors",
    "h4-tables.bin",
    "tokenizer-identity.bin",
];
const NATIVE_FILES: [&str; 4] = [
    "metadata.json",
    "context-tables-i32le.bin",
    "h4-tables.bin",
    "tokenizer-identity.bin",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometricContextConfig {
    pub schema: String,
    pub vocab_size: usize,
    pub width: usize,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub seed: u64,
    pub geometry_sha256: String,
    pub rule: String,
    pub surrogate: String,
    pub initialization: String,
    pub category_rule: String,
    pub temperature: f64,
}
impl GeometricContextConfig {
    pub fn new(vocab_size: usize, width: usize, heads: usize, seed: u64) -> Result<Self> {
        if vocab_size == 0
            || vocab_size > 4096
            || heads == 0
            || heads > 2
            || width == 0
            || width % (heads * 4) != 0
            || width / (heads * 4) > 4
        {
            return Err(invalid(
                "context requires vocab1..4096,heads1..2,lanes/head1..4,width=4*heads*lanes",
            ));
        }
        Ok(Self {
            schema: SCHEMA.into(),
            vocab_size,
            width,
            heads,
            lanes_per_head: width / (heads * 4),
            seed,
            geometry_sha256: geometry_digest().into(),
            rule: RULE.into(),
            surrogate: SURROGATE.into(),
            initialization: INITIALIZATION.into(),
            category_rule: CATEGORY_RULE.into(),
            temperature: 1.,
        })
    }
    pub fn validate(&self) -> Result<()> {
        let mut expected = Self::new(self.vocab_size, self.width, self.heads, self.seed)?;
        match self.surrogate.as_str() {
            SURROGATE => {}
            FINITE_CHOICE_SURROGATE => expected.surrogate = FINITE_CHOICE_SURROGATE.into(),
            _ => return Err(invalid("unknown context backward surrogate")),
        }
        if *self != expected {
            return Err(invalid("context configuration/geometry/surrogate differs"));
        }
        Ok(())
    }
    pub fn new_finite_choice(
        vocab_size: usize,
        width: usize,
        heads: usize,
        seed: u64,
    ) -> Result<Self> {
        let mut config = Self::new(vocab_size, width, heads, seed)?;
        config.surrogate = FINITE_CHOICE_SURROGATE.into();
        Ok(config)
    }
    pub fn lanes(&self) -> usize {
        self.heads * self.lanes_per_head
    }
    fn shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let mut out = BTreeMap::new();
        for family in 0..3 {
            out.insert(
                TOKEN[family].into(),
                vec![
                    self.vocab_size,
                    self.heads,
                    self.lanes_per_head,
                    CLASSES[family],
                ],
            );
            out.insert(
                OWN[family].into(),
                vec![self.heads, self.lanes_per_head, CLASSES[family], 4],
            );
            if self.lanes_per_head > 1 {
                out.insert(
                    NEIGHBOR[family].into(),
                    vec![self.heads, self.lanes_per_head, CLASSES[family], 4],
                );
            }
        }
        out
    }
}
fn roots() -> &'static [[f64; 4]; 120] {
    static R: OnceLock<[[f64; 4]; 120]> = OnceLock::new();
    R.get_or_init(|| std::array::from_fn(|i| canonical_h4_roots()[i].to_array()))
}
fn root(code: u8) -> [f64; 4] {
    roots()[usize::from(code)]
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
    let r = dot(q, g) / dot(q, q);
    std::array::from_fn(|i| g[i] - q[i] * r)
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
    let mut winner = 0;
    for i in 1..values.len() {
        if values[i] > values[winner] {
            winner = i;
        }
    }
    winner
}
fn probabilities<const N: usize>(scores: &[f64; N]) -> [f64; N] {
    let m = scores[best(scores)];
    let mut p = scores.map(|x| (x - m).exp());
    let total = p.iter().sum::<f64>();
    for x in &mut p {
        *x /= total;
    }
    p
}
fn checked_f32(values: Vec<f64>) -> candle_core::Result<Vec<f32>> {
    let values = values.into_iter().map(|x| x as f32).collect::<Vec<_>>();
    if values.iter().any(|x| !x.is_finite()) {
        candle_core::bail!("nonfinite context value/gradient");
    }
    Ok(values)
}
fn radius(category: usize) -> f64 {
    if category == 0 {
        0.
    } else {
        2f64.powi(category as i32 - 17)
    }
}
fn code(root: u8, category: u8) -> AddressCode {
    if category == 0 {
        AddressCode {
            root: 1,
            radius_bin: 0,
            present: false,
        }
    } else {
        AddressCode {
            root,
            radius_bin: category - 1,
            present: true,
        }
    }
}
fn validate_ids(
    config: &GeometricContextConfig,
    ids: &[u32],
    batch: usize,
    time: usize,
) -> Result<()> {
    config.validate()?;
    if batch == 0
        || time == 0
        || batch.checked_mul(time) != Some(ids.len())
        || ids.iter().any(|&x| x as usize >= config.vocab_size)
    {
        return Err(invalid(
            "context needs nonempty valid [batch*time] token IDs",
        ));
    }
    Ok(())
}

pub struct ContextWeights {
    config: GeometricContextConfig,
    parameters: BTreeMap<String, Var>,
}
pub struct ContextOutput {
    pub root_logits: Tensor,
    pub category_logits: Tensor,
    pub context: Tensor,
    /// Actual retained post-update roots [B,T,H,L,4], from the same recurrence
    /// and autograd operation as the emitted address logits.
    pub latent_roots: Tensor,
}
impl ContextWeights {
    pub fn new(vocab_size: usize, width: usize, heads: usize, seed: u64) -> Result<Self> {
        Self::from_config(GeometricContextConfig::new(vocab_size, width, heads, seed)?)
    }
    /// Preserve the exact finite forward and initialization while explicitly
    /// selecting full latent-state adjoints for new offline fits.
    pub fn new_finite_choice(
        vocab_size: usize,
        width: usize,
        heads: usize,
        seed: u64,
    ) -> Result<Self> {
        Self::from_config(GeometricContextConfig::new_finite_choice(
            vocab_size, width, heads, seed,
        )?)
    }
    /// Continue an existing offline fit with full latent-state adjoints. This
    /// consumes the weights without copying or reinitializing their variables;
    /// only the declared backward policy changes. Saving the result produces a
    /// new source configuration, rather than reinterpreting a legacy artifact.
    pub fn into_finite_choice(mut self) -> Result<Self> {
        self.validate()?;
        self.config.surrogate = FINITE_CHOICE_SURROGATE.into();
        Ok(self)
    }
    fn from_config(config: GeometricContextConfig) -> Result<Self> {
        config.validate()?;
        let seed = config.seed;
        let mut rng = if seed == 0 { 0x9e3779b97f4a7c15 } else { seed };
        let mut draw = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let v = ((rng >> 40) as f32 / 16_777_216. - 0.5) * 0.04;
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
                    row[1] += 0.05;
                }
            }
            if name == TC {
                for row in values.chunks_exact_mut(33) {
                    row[17] += 1.;
                }
            }
            parameters.insert(name, Var::from_vec(values, shape, &Device::Cpu)?);
        }
        Ok(Self { config, parameters })
    }
    pub fn config(&self) -> &GeometricContextConfig {
        &self.config
    }
    pub fn parameters(&self) -> &BTreeMap<String, Var> {
        &self.parameters
    }
    fn validate(&self) -> Result<()> {
        self.config.validate()?;
        if self.parameters.keys().cloned().collect::<BTreeSet<_>>()
            != self.config.shapes().keys().cloned().collect()
        {
            return Err(invalid("context parameter names differ"));
        }
        for (name, shape) in self.config.shapes() {
            let v = &self.parameters[&name];
            if v.dims() != shape
                || v.dtype() != DType::F32
                || !v.device().is_cpu()
                || v.flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .any(|x| !x.is_finite())
            {
                return Err(invalid(
                    "context parameter shape/type/device/finite value differs",
                ));
            }
        }
        Ok(())
    }
    fn inputs(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        detach_token_readouts: bool,
    ) -> Result<(Tensor, Tensor)> {
        self.validate()?;
        validate_ids(&self.config, ids, batch, time)?;
        let mut tokens = Vec::new();
        let mut basis = Vec::new();
        for f in 0..3 {
            let token = self.parameters[TOKEN[f]]
                .reshape((self.config.vocab_size, self.config.lanes() * CLASSES[f]))?;
            tokens.push(if detach_token_readouts && f > 0 {
                token.detach()
            } else {
                token
            });
            basis.push(self.parameters[OWN[f]].flatten_all()?);
            if self.config.lanes_per_head > 1 {
                basis.push(self.parameters[NEIGHBOR[f]].flatten_all()?);
            }
        }
        let rows = Tensor::cat(&tokens, 1)?
            .index_select(&Tensor::from_vec(ids.to_vec(), ids.len(), &Device::Cpu)?, 0)?
            .reshape((batch, time, self.config.lanes() * 273))?;
        Ok((rows.contiguous()?, Tensor::cat(&basis, 0)?.contiguous()?))
    }
    fn packed(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        reset: bool,
        detach_token_readouts: bool,
    ) -> Result<Tensor> {
        let (rows, basis) = self.inputs(ids, batch, time, detach_token_readouts)?;
        Ok(rows.apply_op2(
            &basis,
            ContextOp {
                batch,
                time,
                heads: self.config.heads,
                lanes_per_head: self.config.lanes_per_head,
                reset,
                finite_choice: self.config.surrogate == FINITE_CHOICE_SURROGATE,
            },
        )?)
    }
    pub fn forward(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        reset_each_token: bool,
    ) -> Result<ContextOutput> {
        let packed = self.packed(ids, batch, time, reset_each_token, false)?;
        let roots = packed.narrow(3, 0, 120)?.contiguous()?;
        let categories = packed.narrow(3, 120, 33)?.contiguous()?;
        let context = roots.apply_op2(
            &categories,
            EmitOp {
                batch,
                time,
                lanes: self.config.lanes(),
            },
        )?;
        Ok(ContextOutput {
            root_logits: roots.reshape(vec![
                batch,
                time,
                self.config.heads,
                self.config.lanes_per_head,
                120,
            ])?,
            category_logits: categories.reshape(vec![
                batch,
                time,
                self.config.heads,
                self.config.lanes_per_head,
                33,
            ])?,
            context,
            latent_roots: packed.narrow(3, LATENT_OFFSET, 4)?.contiguous()?.reshape((
                batch,
                time,
                self.config.heads,
                self.config.lanes_per_head,
                4,
            ))?,
        })
    }
    pub fn float_trace(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        reset_each_token: bool,
    ) -> Result<ContextFloatTrace> {
        let (rows, basis) = self.inputs(ids, batch, time, false)?;
        let op = ContextOp {
            batch,
            time,
            heads: self.config.heads,
            lanes_per_head: self.config.lanes_per_head,
            reset: reset_each_token,
            finite_choice: self.config.surrogate == FINITE_CHOICE_SURROGATE,
        };
        let (steps, output) = op.forward(
            &rows.flatten_all()?.to_vec1::<f32>()?,
            &basis.to_vec1::<f32>()?,
        )?;
        let mut emitted_roots = Vec::new();
        let mut categories = Vec::new();
        let mut codes = Vec::new();
        for row in output.chunks_exact(PACKED_WIDTH) {
            let r = best(&row[..120].iter().map(|&x| f64::from(x)).collect::<Vec<_>>()) as u8;
            let c = best(
                &row[120..LATENT_OFFSET]
                    .iter()
                    .map(|&x| f64::from(x))
                    .collect::<Vec<_>>(),
            ) as u8;
            emitted_roots.push(r);
            categories.push(c);
            codes.push(code(r, c));
        }
        Ok(ContextFloatTrace {
            batch,
            time,
            heads: self.config.heads,
            lanes_per_head: self.config.lanes_per_head,
            states: steps.iter().map(|s| s.new[..op.lanes()].to_vec()).collect(),
            actions: steps
                .iter()
                .map(|s| s.actions[..op.lanes()].to_vec())
                .collect(),
            emitted_roots,
            categories,
            codes,
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextFloatTrace {
    pub batch: usize,
    pub time: usize,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub states: Vec<Vec<u8>>,
    pub actions: Vec<Vec<u8>>,
    pub emitted_roots: Vec<u8>,
    pub categories: Vec<u8>,
    pub codes: Vec<AddressCode>,
}
impl ContextFloatTrace {
    pub fn address_codes(&self) -> Result<Vec<AddressLane>> {
        self.codes
            .iter()
            .map(|c| {
                AddressLane::new(c.root, c.radius_bin, c.present)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect()
    }
}
struct Step {
    old: [u8; 8],
    new: [u8; 8],
    actions: [u8; 8],
}
struct ContextOp {
    batch: usize,
    time: usize,
    heads: usize,
    lanes_per_head: usize,
    reset: bool,
    finite_choice: bool,
}
impl ContextOp {
    fn lanes(&self) -> usize {
        self.heads * self.lanes_per_head
    }
    fn neighbor(&self, lane: usize) -> usize {
        let head = lane / self.lanes_per_head;
        head * self.lanes_per_head + (lane + 1) % self.lanes_per_head
    }
    fn token_offset(&self, f: usize) -> usize {
        CLASSES[..f].iter().sum::<usize>() * self.lanes()
    }
    fn basis_offset(&self, f: usize, neighbor: bool) -> usize {
        self.token_offset(f) * 4 * if self.lanes_per_head > 1 { 2 } else { 1 }
            + if neighbor {
                self.lanes() * CLASSES[f] * 4
            } else {
                0
            }
    }
    fn scores<const N: usize>(
        &self,
        row: &[f32],
        basis: &[f32],
        state: &[u8; 8],
        lane: usize,
        f: usize,
    ) -> [f64; N] {
        std::array::from_fn(|d| {
            let at = (lane * N + d) * 4;
            let mut z = f64::from(row[self.token_offset(f) + lane * N + d])
                + dot(
                    root(state[lane]),
                    std::array::from_fn(|i| f64::from(basis[self.basis_offset(f, false) + at + i])),
                );
            if self.lanes_per_head > 1 {
                z += dot(
                    root(state[self.neighbor(lane)]),
                    std::array::from_fn(|i| f64::from(basis[self.basis_offset(f, true) + at + i])),
                );
            }
            z
        })
    }
    fn validate(&self, rows: &[f32], basis: &[f32]) -> candle_core::Result<()> {
        if rows.len() != self.batch * self.time * self.lanes() * 273
            || basis.len() != self.lanes() * 273 * 4 * if self.lanes_per_head > 1 { 2 } else { 1 }
            || rows.iter().chain(basis).any(|x| !x.is_finite())
        {
            candle_core::bail!("context packed shape or finite coefficients differ");
        }
        Ok(())
    }
    fn forward(&self, rows: &[f32], basis: &[f32]) -> candle_core::Result<(Vec<Step>, Vec<f32>)> {
        self.validate(rows, basis)?;
        let n = self.lanes();
        let mut trace = Vec::with_capacity(self.batch * self.time);
        let mut output = Vec::with_capacity(self.batch * self.time * n * PACKED_WIDTH);
        for b in 0..self.batch {
            let mut state = [1; 8];
            for t in 0..self.time {
                if self.reset {
                    state.fill(1);
                }
                let at = b * self.time + t;
                let row = &rows[at * n * 273..(at + 1) * n * 273];
                let old = state;
                let mut actions = [1; 8];
                for lane in 0..n {
                    actions[lane] = best(&self.scores::<120>(row, basis, &old, lane, 0)) as u8;
                    state[lane] = group_table().product
                        [usize::from(old[lane]) * ROW_STRIDE + usize::from(actions[lane])];
                }
                for lane in 0..n {
                    output.extend(self.scores::<120>(row, basis, &state, lane, 1));
                    output.extend(self.scores::<33>(row, basis, &state, lane, 2));
                    output.extend(root(state[lane]));
                }
                trace.push(Step {
                    old,
                    new: state,
                    actions,
                });
            }
        }
        Ok((trace, checked_f32(output)?))
    }
    #[allow(clippy::too_many_arguments)]
    fn score_pullback(
        &self,
        rowidx: usize,
        f: usize,
        lane: usize,
        class: usize,
        g: f64,
        state: &[u8; 8],
        basis: &[f32],
        dr: &mut [f64],
        db: &mut [f64],
        state_gradient: &mut [[f64; 4]; 8],
    ) {
        let n = CLASSES[f];
        dr[rowidx * self.lanes() * 273 + self.token_offset(f) + lane * n + class] += g;
        for neighbor in [false, true] {
            if neighbor && self.lanes_per_head == 1 {
                continue;
            }
            let target = if neighbor { self.neighbor(lane) } else { lane };
            let at = self.basis_offset(f, neighbor) + (lane * n + class) * 4;
            let q = root(state[target]);
            for i in 0..4 {
                db[at + i] += g * q[i];
                state_gradient[target][i] += g * f64::from(basis[at + i]);
            }
        }
    }
    fn backward(
        &self,
        rows: &[f32],
        basis: &[f32],
        upstream: &[f32],
    ) -> candle_core::Result<(Vec<f32>, Vec<f32>)> {
        let (trace, _) = self.forward(rows, basis)?;
        let n = self.lanes();
        if upstream.len() != self.batch * self.time * n * PACKED_WIDTH
            || upstream.iter().any(|x| !x.is_finite())
        {
            candle_core::bail!("context upstream shape/nonfinite value");
        }
        let mut dr = vec![0.; rows.len()];
        let mut db = vec![0.; basis.len()];
        for b in 0..self.batch {
            let mut future = [[0.; 4]; 8];
            for t in (0..self.time).rev() {
                if self.reset {
                    future = [[0.; 4]; 8];
                }
                let at = b * self.time + t;
                let step = &trace[at];
                let row = &rows[at * n * 273..(at + 1) * n * 273];
                for lane in 0..n {
                    add(
                        &mut future[lane],
                        std::array::from_fn(|coordinate| {
                            f64::from(
                                upstream
                                    [(at * n + lane) * PACKED_WIDTH + LATENT_OFFSET + coordinate],
                            )
                        }),
                    );
                    for f in 1..3 {
                        for class in 0..CLASSES[f] {
                            let offset = if f == 1 { 0 } else { 120 };
                            let g = f64::from(
                                upstream[(at * n + lane) * PACKED_WIDTH + offset + class],
                            );
                            self.score_pullback(
                                at,
                                f,
                                lane,
                                class,
                                g,
                                &step.new,
                                basis,
                                &mut dr,
                                &mut db,
                                &mut future,
                            );
                        }
                    }
                }
                let mut old_gradient = [[0.; 4]; 8];
                for lane in 0..n {
                    let g = if self.finite_choice {
                        future[lane]
                    } else {
                        tangent(root(step.new[lane]), future[lane])
                    };
                    let (direct, action_g) =
                        hamilton_pullback(root(step.old[lane]), root(step.actions[lane]), g);
                    add(&mut old_gradient[lane], direct);
                    let p = probabilities(&self.scores::<120>(row, basis, &step.old, lane, 0));
                    let credit = std::array::from_fn::<_, 120, _>(|a| dot(action_g, roots()[a]));
                    let mean = (0..120).map(|a| p[a] * credit[a]).sum::<f64>();
                    for a in 0..120 {
                        self.score_pullback(
                            at,
                            0,
                            lane,
                            a,
                            p[a] * (credit[a] - mean),
                            &step.old,
                            basis,
                            &mut dr,
                            &mut db,
                            &mut old_gradient,
                        );
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
        None => candle_core::bail!("context requires contiguous CPU F32 input"),
    }
}
impl CustomOp2 for ContextOp {
    fn name(&self) -> &'static str {
        "latent-geometric-context"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        Ok((
            CpuStorage::F32(self.forward(contiguous(a, la)?, contiguous(b, lb)?)?.1),
            Shape::from((self.batch, self.time, self.lanes(), PACKED_WIDTH)),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (da, db) = self.backward(
            &a.flatten_all()?.to_vec1::<f32>()?,
            &b.flatten_all()?.to_vec1::<f32>()?,
            &grad.flatten_all()?.to_vec1::<f32>()?,
        )?;
        Ok((
            Some(Tensor::from_vec(da, a.shape(), a.device())?),
            Some(Tensor::from_vec(db, b.shape(), b.device())?),
        ))
    }
}
struct EmitOp {
    batch: usize,
    time: usize,
    lanes: usize,
}
impl EmitOp {
    fn validate(&self, r: &[f32], c: &[f32]) -> candle_core::Result<()> {
        let count = self.batch * self.time * self.lanes;
        if r.len() != count * 120
            || c.len() != count * 33
            || r.iter().chain(c).any(|v| !v.is_finite())
        {
            candle_core::bail!("context emitted logits shape/nonfinite value");
        }
        Ok(())
    }
    fn forward(&self, r: &[f32], c: &[f32]) -> candle_core::Result<Vec<f32>> {
        self.validate(r, c)?;
        let mut out = Vec::with_capacity(self.batch * self.time * self.lanes * 4);
        for lane in 0..self.batch * self.time * self.lanes {
            let rootscore = std::array::from_fn::<_, 120, _>(|i| f64::from(r[lane * 120 + i]));
            let catscore = std::array::from_fn::<_, 33, _>(|i| f64::from(c[lane * 33 + i]));
            let q = roots()[best(&rootscore)];
            let rad = radius(best(&catscore));
            out.extend(q.map(|x| x * rad));
        }
        checked_f32(out)
    }
    fn backward(
        &self,
        r: &[f32],
        c: &[f32],
        grad: &[f32],
    ) -> candle_core::Result<(Vec<f32>, Vec<f32>)> {
        self.validate(r, c)?;
        let count = self.batch * self.time * self.lanes;
        if grad.len() != count * 4 || grad.iter().any(|v| !v.is_finite()) {
            candle_core::bail!("context emitted gradient shape/nonfinite value");
        }
        let mut dr = vec![0.; r.len()];
        let mut dc = vec![0.; c.len()];
        for lane in 0..count {
            let rs = std::array::from_fn::<_, 120, _>(|i| f64::from(r[lane * 120 + i]));
            let cs = std::array::from_fn::<_, 33, _>(|i| f64::from(c[lane * 33 + i]));
            let selected = best(&cs);
            if selected == 0 {
                continue;
            }
            let q = roots()[best(&rs)];
            let g = std::array::from_fn(|i| f64::from(grad[lane * 4 + i]));
            let angular = tangent(q, g).map(|x| x * radius(selected));
            let p = probabilities(&rs);
            let mean = (0..120)
                .map(|i| p[i] * dot(angular, roots()[i]))
                .sum::<f64>();
            for i in 0..120 {
                dr[lane * 120 + i] = p[i] * (dot(angular, roots()[i]) - mean);
            }
            let lo = selected.saturating_sub(1).max(1);
            let hi = (selected + 1).min(32);
            let max = cs[lo..=hi]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let sum = (lo..=hi).map(|i| (cs[i] - max).exp()).sum::<f64>();
            let mean = (lo..=hi)
                .map(|i| (cs[i] - max).exp() / sum * radius(i))
                .sum::<f64>();
            let radial = dot(g, q);
            for i in lo..=hi {
                dc[lane * 33 + i] = radial * (cs[i] - max).exp() / sum * (radius(i) - mean);
            }
        }
        Ok((checked_f32(dr)?, checked_f32(dc)?))
    }
}
impl CustomOp2 for EmitOp {
    fn name(&self) -> &'static str {
        "geometric-context-hard-emission"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        Ok((
            CpuStorage::F32(self.forward(contiguous(a, la)?, contiguous(b, lb)?)?),
            Shape::from((self.batch, self.time, self.lanes * 4)),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        _out: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let (da, db) = self.backward(
            &a.flatten_all()?.to_vec1::<f32>()?,
            &b.flatten_all()?.to_vec1::<f32>()?,
            &grad.flatten_all()?.to_vec1::<f32>()?,
        )?;
        Ok((
            Some(Tensor::from_vec(da, a.shape(), a.device())?),
            Some(Tensor::from_vec(db, b.shape(), b.device())?),
        ))
    }
}

/// Actual immutable dependencies, not caller-asserted hashes. Event source and
/// native directories are distinct: both are independently admitted on load.
#[derive(Clone, Copy)]
pub struct ContextSourcePaths<'a> {
    pub base: &'a Path,
    pub event_source: &'a Path,
    pub event_native: &'a Path,
    pub span_native: &'a Path,
    pub potential_native: &'a Path,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundFile {
    pub bytes: usize,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenContextBinding {
    pub files: BTreeMap<String, BoundFile>,
    pub address: GeometricAddressConfig,
}
fn dependency_files(paths: ContextSourcePaths<'_>) -> Result<BTreeMap<String, BoundFile>> {
    let mut files = BTreeMap::new();
    for (group, dir, names) in [
        (
            "base",
            paths.base,
            vec![
                "model.safetensors",
                "config.json",
                GEOMETRIC_ADDRESS_RECORD,
                GEOMETRIC_SPAN_RECORD,
            ],
        ),
        (
            "event_source",
            paths.event_source,
            vec![
                "metadata.json",
                "event-parameters.safetensors",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
        (
            "event_native",
            paths.event_native,
            vec![
                "metadata.json",
                "event-tables-i32le.bin",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
        (
            "span_native",
            paths.span_native,
            vec![
                "metadata.json",
                "token-actions.bin",
                "h4-tables.bin",
                "tokenizer-identity.bin",
            ],
        ),
        (
            "potential_native",
            paths.potential_native,
            // Strict potential /2 embeds the independent coefficient source;
            // bind its packed and shadow bytes without adding a context cycle.
            // Legacy /1 retains exactly its original four-file dependency map.
            artifact_file_names(paths.potential_native)?,
        ),
    ] {
        for name in names {
            let bytes = fs::read(dir.join(name))?;
            files.insert(
                format!("{group}/{name}"),
                BoundFile {
                    bytes: bytes.len(),
                    sha256: sha256_bytes(&bytes),
                },
            );
        }
    }
    Ok(files)
}
impl FrozenContextBinding {
    fn read(
        paths: ContextSourcePaths<'_>,
        config: &GeometricContextConfig,
        tokenizer: &[u8],
    ) -> Result<Self> {
        config.validate()?;
        if tokenizer.is_empty() {
            return Err(invalid(
                "context requires explicit tokenizer identity bytes",
            ));
        }
        let before = dependency_files(paths)?;
        let base: StackConfig = serde_json::from_slice(&fs::read(paths.base.join("config.json"))?)?;
        base.validate()?;
        if base.vocab_size != config.vocab_size
            || base.width != config.width
            || base.heads != config.heads
        {
            return Err(invalid(
                "context source dimensions differ from frozen reader",
            ));
        }
        let address = StackModel::saved_geometric_address(paths.base)?
            .ok_or_else(|| invalid("context source geometric address absent"))?;
        let span = StackModel::saved_geometric_span(paths.base)?
            .ok_or_else(|| invalid("context source geometric span absent"))?;
        let event = CompiledEvents::load(
            paths.event_native,
            paths.event_source,
            paths.base,
            tokenizer,
        )?;
        if event.vocab_size() != config.vocab_size {
            return Err(invalid("context source event vocabulary differs"));
        }
        let span_source = SpanSourceBinding::from_files(
            &paths.base.join("model.safetensors"),
            &paths.base.join("config.json"),
            tokenizer,
        )?;
        // This admission also compares ALL signed roots, products, inverse,
        // identity and canonical coefficients with the pinned training algebra.
        CompiledSpanActions::load(paths.span_native, &span_source, &span)?;
        let potential_source = PotentialSourceBinding::from_directory(paths.base, tokenizer)?;
        CompiledGeometricPotentials::load(paths.potential_native, &potential_source)?;
        if before != dependency_files(paths)? {
            return Err(invalid("context frozen source changed during admission"));
        }
        Ok(Self {
            files: before,
            address,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextParameterIdentity {
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
fn parameter_values(weights: &ContextWeights) -> Result<BTreeMap<String, Vec<f32>>> {
    weights.validate()?;
    weights
        .parameters
        .iter()
        .map(|(name, v)| Ok((name.clone(), v.flatten_all()?.to_vec1::<f32>()?)))
        .collect()
}
fn parameter_identities(weights: &ContextWeights) -> Result<Vec<ContextParameterIdentity>> {
    let values = parameter_values(weights)?;
    Ok(weights
        .config
        .shapes()
        .into_iter()
        .map(|(name, shape)| ContextParameterIdentity {
            f32_bytes_sha256: sha256_bytes(&f32_bytes(&values[&name])),
            name,
            shape,
        })
        .collect())
}
fn parameter_bytes(weights: &ContextWeights) -> Result<Vec<u8>> {
    let bytes = parameter_values(weights)?
        .into_iter()
        .map(|(name, v)| (name, f32_bytes(&v)))
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
fn restore_parameters(config: &GeometricContextConfig, bytes: &[u8]) -> Result<ContextWeights> {
    config.validate()?;
    let archive = SafeTensors::deserialize(bytes)?;
    let shapes = config.shapes();
    if archive
        .names()
        .iter()
        .map(|s| s.to_string())
        .collect::<BTreeSet<_>>()
        != shapes.keys().cloned().collect()
    {
        return Err(invalid("context source parameter names differ"));
    }
    let weights = ContextWeights::from_config(config.clone())?;
    for (name, shape) in shapes {
        let view = archive.tensor(&name)?;
        if view.dtype() != SafeDtype::F32
            || view.shape() != shape
            || view.data().len() != shape.iter().product::<usize>() * 4
        {
            return Err(invalid("context source parameter shape/type/size differs"));
        }
        let values = view
            .data()
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        if values.iter().any(|v| !v.is_finite()) {
            return Err(invalid("nonfinite saved context coefficient"));
        }
        weights.parameters[&name].set(&Tensor::from_vec(values, shape, &Device::Cpu)?)?;
    }
    weights.validate()?;
    Ok(weights)
}
fn admit_geometry(payload: &[u8]) -> Result<HistoricalH4Tables> {
    if payload != PINNED
        || Some(
            mathematical_sha256(payload)
                .map_err(|e| invalid(e.to_string()))?
                .as_str(),
        ) != TRUSTED_MATHEMATICAL_SHA256
    {
        return Err(invalid("context pinned mathematical geometry differs"));
    }
    HistoricalH4Tables::from_bytes(payload).map_err(|e| invalid(e.to_string()))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextSourceMetadata {
    pub schema: String,
    pub config: GeometricContextConfig,
    pub frozen: FrozenContextBinding,
    pub parameters: Vec<ContextParameterIdentity>,
    pub parameter_file_bytes: usize,
    pub parameter_file_sha256: String,
    pub tokenizer_bytes: usize,
    pub tokenizer_sha256: String,
    pub algebra_payload_sha256: String,
    pub algebra_mathematical_sha256: String,
}
fn source_metadata(
    weights: &ContextWeights,
    parameters: &[u8],
    frozen: FrozenContextBinding,
    tokenizer: &[u8],
) -> Result<ContextSourceMetadata> {
    Ok(ContextSourceMetadata {
        schema: "uor-r4.geometric-context-source/1".into(),
        config: weights.config.clone(),
        frozen,
        parameters: parameter_identities(weights)?,
        parameter_file_bytes: parameters.len(),
        parameter_file_sha256: sha256_bytes(parameters),
        tokenizer_bytes: tokenizer.len(),
        tokenizer_sha256: sha256_bytes(tokenizer),
        algebra_payload_sha256: sha256_bytes(PINNED),
        algebra_mathematical_sha256: mathematical_sha256(PINNED)
            .map_err(|e| invalid(e.to_string()))?,
    })
}
fn require_files(directory: &Path, files: &[&str]) -> Result<()> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(invalid("context artifact contains non-regular file"));
        }
        names.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("context artifact filename is not UTF-8"))?,
        );
    }
    if names != files.iter().map(|s| s.to_string()).collect() {
        return Err(invalid("context artifact file set differs"));
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
impl ContextWeights {
    pub fn save_source(
        &self,
        directory: &Path,
        paths: ContextSourcePaths<'_>,
        tokenizer: &[u8],
    ) -> Result<()> {
        let frozen = FrozenContextBinding::read(paths, &self.config, tokenizer)?;
        let parameters = parameter_bytes(self)?;
        admit_geometry(PINNED)?;
        let metadata =
            serde_json::to_vec_pretty(&source_metadata(self, &parameters, frozen, tokenizer)?)?;
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
    pub fn load_source(
        directory: &Path,
        paths: ContextSourcePaths<'_>,
        tokenizer: &[u8],
    ) -> Result<Self> {
        Ok(load_source(directory, paths, tokenizer)?.0)
    }
}
fn load_source(
    directory: &Path,
    paths: ContextSourcePaths<'_>,
    tokenizer: &[u8],
) -> Result<(ContextWeights, ContextSourceMetadata, Vec<u8>)> {
    require_files(directory, &SOURCE_FILES)?;
    let metadata_bytes = fs::read(directory.join(SOURCE_FILES[0]))?;
    let saved: ContextSourceMetadata = serde_json::from_slice(&metadata_bytes)?;
    saved.config.validate()?;
    let maximum = saved
        .config
        .shapes()
        .values()
        .map(|s| s.iter().product::<usize>() * 4)
        .sum::<usize>()
        + 32768;
    if fs::metadata(directory.join(SOURCE_FILES[1]))?.len() > maximum as u64
        || fs::metadata(directory.join(SOURCE_FILES[2]))?.len() != PINNED.len() as u64
        || fs::metadata(directory.join(SOURCE_FILES[3]))?.len() != tokenizer.len() as u64
    {
        return Err(invalid("context source payload lengths differ"));
    }
    let parameters = fs::read(directory.join(SOURCE_FILES[1]))?;
    admit_geometry(&fs::read(directory.join(SOURCE_FILES[2]))?)?;
    if fs::read(directory.join(SOURCE_FILES[3]))? != tokenizer {
        return Err(invalid("context source tokenizer differs"));
    }
    let weights = restore_parameters(&saved.config, &parameters)?;
    let expected = source_metadata(
        &weights,
        &parameters,
        FrozenContextBinding::read(paths, &saved.config, tokenizer)?,
        tokenizer,
    )?;
    if expected != saved {
        return Err(invalid(
            "context source coefficients/dependencies/tokenizer binding differs",
        ));
    }
    Ok((weights, saved, metadata_bytes))
}
fn quantize(value: f64) -> Result<i32> {
    let value = (value * 16_777_216.).round();
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(invalid("context potential overflows fixed signed Q24"));
    }
    Ok(value as i32)
}
struct ExpandedContext {
    token: [Vec<i32>; 3],
    own: [Vec<i32>; 3],
    neighbor: [Option<Vec<i32>>; 3],
}
impl ExpandedContext {
    fn compile(weights: &ContextWeights) -> Result<Self> {
        let values = parameter_values(weights)?;
        let c = &weights.config;
        let n = c.lanes();
        let stride = [128, 128, 64];
        let mut token = std::array::from_fn(|f| vec![0; c.vocab_size * n * stride[f]]);
        let mut own = std::array::from_fn(|f| vec![0; n * 128 * stride[f]]);
        let mut neighbor = std::array::from_fn(|f| {
            if c.lanes_per_head > 1 {
                Some(vec![0; n * 128 * stride[f]])
            } else {
                None
            }
        });
        for f in 0..3 {
            for x in 0..c.vocab_size {
                for lane in 0..n {
                    for a in 0..CLASSES[f] {
                        token[f][(x * n + lane) * stride[f] + a] =
                            quantize(f64::from(values[TOKEN[f]][(x * n + lane) * CLASSES[f] + a]))?;
                    }
                }
            }
            for lane in 0..n {
                for s in 0..120 {
                    for a in 0..CLASSES[f] {
                        let at = (lane * CLASSES[f] + a) * 4;
                        own[f][(lane * 128 + s) * stride[f] + a] = quantize(dot(
                            roots()[s],
                            std::array::from_fn(|i| f64::from(values[OWN[f]][at + i])),
                        ))?;
                        if let Some(table) = &mut neighbor[f] {
                            table[(lane * 128 + s) * stride[f] + a] = quantize(dot(
                                roots()[s],
                                std::array::from_fn(|i| f64::from(values[NEIGHBOR[f]][at + i])),
                            ))?;
                        }
                    }
                }
            }
        }
        Ok(Self {
            token,
            own,
            neighbor,
        })
    }
    fn native(&self, c: &GeometricContextConfig) -> Result<NativeContextTables> {
        NativeContextTables::new(
            c.vocab_size,
            c.heads,
            c.lanes_per_head,
            ContextTableSlices {
                token_transition: &self.token[0],
                self_transition: &self.own[0],
                neighbor_transition: self.neighbor[0].as_deref(),
                token_root: &self.token[1],
                self_root: &self.own[1],
                neighbor_root: self.neighbor[1].as_deref(),
                token_category: &self.token[2],
                self_category: &self.own[2],
                neighbor_category: self.neighbor[2].as_deref(),
            },
        )
        .map_err(|e| invalid(e.to_string()))
    }
    fn slices(&self) -> Vec<&[i32]> {
        let mut out = Vec::new();
        for f in 0..3 {
            out.push(self.token[f].as_slice());
            out.push(self.own[f].as_slice());
            out.push(self.neighbor[f].as_deref().unwrap_or(&[]));
        }
        out
    }
    fn bytes(&self) -> Vec<u8> {
        self.slices()
            .into_iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes())
            .collect()
    }
    fn lengths(&self) -> Vec<usize> {
        self.slices().iter().map(|s| s.len()).collect()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledContextMetadata {
    pub schema: String,
    pub source: ContextSourceMetadata,
    pub source_metadata_sha256: String,
    pub coefficient_policy: String,
    pub table_layout: String,
    pub table_lengths: Vec<usize>,
    pub table_bytes: usize,
    pub table_sha256: String,
    pub coefficient_reads_per_token: usize,
}
fn compiled_metadata(
    source: ContextSourceMetadata,
    source_bytes: &[u8],
    expanded: &ExpandedContext,
) -> Result<CompiledContextMetadata> {
    let bytes = expanded.bytes();
    let stats = expanded.native(&source.config)?.stats();
    Ok(CompiledContextMetadata{schema:"uor-r4.geometric-context-native/1".into(),source,source_metadata_sha256:sha256_bytes(source_bytes),coefficient_policy:POLICY.into(),table_layout:"transition,root,category;each(token[V,H,L,C],self[H,L,128,C],next-neighbor[H,L,128,C]-if-L>1);C128,128,64;zero-state>=120/action>=120/category>=33/1".into(),table_lengths:expanded.lengths(),table_bytes:bytes.len(),table_sha256:sha256_bytes(&bytes),coefficient_reads_per_token:stats.coefficient_reads})
}
pub struct CompiledContext {
    metadata: CompiledContextMetadata,
    native: NativeContextTables,
    geometry: HistoricalH4Tables,
    table_bytes: Vec<u8>,
    tokenizer: Vec<u8>,
}
impl CompiledContext {
    pub fn compile(
        weights: &ContextWeights,
        source_directory: &Path,
        paths: ContextSourcePaths<'_>,
        tokenizer: &[u8],
    ) -> Result<Self> {
        let (source, metadata, metadata_bytes) = load_source(source_directory, paths, tokenizer)?;
        if weights.config != source.config || parameter_identities(weights)? != metadata.parameters
        {
            return Err(invalid(
                "live context coefficients differ from saved source",
            ));
        }
        let expanded = ExpandedContext::compile(&source)?;
        Ok(Self {
            native: expanded.native(&source.config)?,
            geometry: admit_geometry(PINNED)?,
            table_bytes: expanded.bytes(),
            metadata: compiled_metadata(metadata, &metadata_bytes, &expanded)?,
            tokenizer: tokenizer.to_vec(),
        })
    }
    pub fn config(&self) -> &GeometricContextConfig {
        &self.metadata.source.config
    }
    pub fn metadata(&self) -> &CompiledContextMetadata {
        &self.metadata
    }
    pub fn vocab_size(&self) -> usize {
        self.config().vocab_size
    }
    pub fn validate_for(&self, weights: &ContextWeights) -> Result<()> {
        if weights.config != *self.config()
            || parameter_identities(weights)? != self.metadata.source.parameters
        {
            return Err(invalid(
                "compiled context stale for live coefficient bits/config",
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
    pub fn load(
        directory: &Path,
        source_directory: &Path,
        paths: ContextSourcePaths<'_>,
        tokenizer: &[u8],
    ) -> Result<Self> {
        require_files(directory, &NATIVE_FILES)?;
        let (source, metadata, metadata_bytes) = load_source(source_directory, paths, tokenizer)?;
        let expanded = ExpandedContext::compile(&source)?;
        let expected = compiled_metadata(metadata, &metadata_bytes, &expanded)?;
        if fs::metadata(directory.join(NATIVE_FILES[1]))?.len() != expected.table_bytes as u64
            || fs::metadata(directory.join(NATIVE_FILES[2]))?.len() != PINNED.len() as u64
            || fs::metadata(directory.join(NATIVE_FILES[3]))?.len() != tokenizer.len() as u64
        {
            return Err(invalid("compiled context payload lengths differ"));
        }
        let saved: CompiledContextMetadata =
            serde_json::from_slice(&fs::read(directory.join(NATIVE_FILES[0]))?)?;
        let table_bytes = expanded.bytes();
        if saved != expected
            || fs::read(directory.join(NATIVE_FILES[1]))? != table_bytes
            || fs::read(directory.join(NATIVE_FILES[3]))? != tokenizer
        {
            return Err(invalid(
                "compiled context differs from actual source recompilation",
            ));
        }
        let geometry = admit_geometry(&fs::read(directory.join(NATIVE_FILES[2]))?)?;
        Ok(Self {
            metadata: saved,
            native: expanded.native(&source.config)?,
            geometry,
            table_bytes,
            tokenizer: tokenizer.to_vec(),
        })
    }
}
#[derive(Clone, Debug)]
pub struct NativeContextTrace {
    pub batch: usize,
    pub time: usize,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub states: Vec<Vec<u8>>,
    pub actions: Vec<Vec<u8>>,
    pub emitted_roots: Vec<u8>,
    pub categories: Vec<u8>,
    pub codes: Vec<AddressLane>,
    pub coefficient_reads: usize,
}
pub fn trace_native(
    ids: &[u32],
    batch: usize,
    time: usize,
    compiled: &CompiledContext,
    reset_each_token: bool,
) -> Result<NativeContextTrace> {
    validate_ids(compiled.config(), ids, batch, time)?;
    let c = compiled.config();
    let n = c.lanes();
    let mut state =
        NativeContextState::new(c.heads, c.lanes_per_head).map_err(|e| invalid(e.to_string()))?;
    let mut trace = NativeContextTrace {
        batch,
        time,
        heads: c.heads,
        lanes_per_head: c.lanes_per_head,
        states: Vec::with_capacity(ids.len()),
        actions: Vec::with_capacity(ids.len()),
        emitted_roots: Vec::with_capacity(ids.len() * n),
        categories: Vec::with_capacity(ids.len() * n),
        codes: Vec::with_capacity(ids.len() * n),
        coefficient_reads: 0,
    };
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
            trace
                .states
                .push(step.states[..n].iter().map(|q| q.index()).collect());
            trace
                .actions
                .push(step.actions[..n].iter().map(|q| q.index()).collect());
            trace
                .emitted_roots
                .extend(step.readout_roots[..n].iter().map(|q| q.index()));
            trace.categories.extend_from_slice(&step.categories[..n]);
            trace.codes.extend_from_slice(&step.output[..n]);
            trace.coefficient_reads = trace
                .coefficient_reads
                .checked_add(step.coefficient_reads)
                .ok_or_else(|| invalid("context diagnostic count overflow"))?;
        }
    }
    Ok(trace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_event::EventWeights;
    use crate::geometric_span::GeometricSpanConfig;
    use crate::geometric_stack::{logits_cross_entropy, ReadIdentityLatch, ReadScore, StackArch};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    fn norm(values: &[f32]) -> f64 {
        values.iter().map(|&x| f64::from(x).abs()).sum()
    }
    fn set(weights: &ContextWeights, name: &str, values: Vec<f32>) -> Result<()> {
        let v = &weights.parameters[name];
        v.set(&Tensor::from_vec(values, v.shape(), &Device::Cpu)?)?;
        Ok(())
    }
    fn sign_answer_loss(
        weights: &ContextWeights,
        ids: &[u32],
        lane: usize,
        reset: bool,
    ) -> Result<Tensor> {
        let output = weights.forward(ids, 1, ids.len(), reset)?;
        let final_roots = output
            .latent_roots
            .narrow(1, ids.len() - 1, 1)?
            .reshape((weights.config.lanes(), 4))?;
        let real = final_roots.narrow(0, lane, 1)?.narrow(1, 0, 1)?;
        let positive = real.affine(2., 0.)?;
        let negative = real.affine(-2., 0.)?;
        logits_cross_entropy(&Tensor::cat(&[&positive, &negative], 1)?, &[1], None)
    }
    fn tensor_bits(tensor: &Tensor) -> Result<Vec<u32>> {
        Ok(tensor
            .flatten_all()?
            .to_vec1::<f32>()?
            .into_iter()
            .map(f32::to_bits)
            .collect())
    }

    #[test]
    fn geometric_context_finite_choice_sign_credit_reaches_current_and_earlier_actions(
    ) -> Result<()> {
        let legacy = ContextWeights::new(2, 4, 1, 73)?;
        let finite = ContextWeights::new_finite_choice(2, 4, 1, 73)?;
        for weights in [&legacy, &finite] {
            for (name, variable) in weights.parameters() {
                set(weights, name, vec![0.; variable.elem_count()])?;
            }
            let mut transitions = vec![0.; 2 * 120];
            transitions[1] = 2.;
            transitions[120 + 1] = 2.;
            set(weights, TT, transitions)?;
        }
        let ids = [0, 1];
        let old = legacy.forward(&ids, 1, 2, false)?;
        let new = finite.forward(&ids, 1, 2, false)?;
        assert_eq!(new.latent_roots.dims(), [1, 2, 1, 1, 4]);
        assert_eq!(
            tensor_bits(&old.latent_roots)?,
            tensor_bits(&new.latent_roots)?
        );
        assert_eq!(
            tensor_bits(&old.root_logits)?,
            tensor_bits(&new.root_logits)?
        );
        assert_eq!(
            tensor_bits(&old.category_logits)?,
            tensor_bits(&new.category_logits)?
        );
        assert_eq!(tensor_bits(&old.context)?, tensor_bits(&new.context)?);
        let trace = finite.float_trace(&ids, 1, 2, false)?;
        assert_eq!(trace.states, vec![vec![1], vec![1]]);
        assert_eq!(trace.actions, vec![vec![1], vec![1]]);
        let expected: Vec<f32> = trace
            .states
            .iter()
            .flat_map(|row| row.iter().flat_map(|&code| root(code).map(|x| x as f32)))
            .collect();
        assert_eq!(new.latent_roots.flatten_all()?.to_vec1::<f32>()?, expected);

        for (weights, should_have_credit) in [(&legacy, false), (&finite, true)] {
            let gradients = sign_answer_loss(weights, &ids, 0, false)?.backward()?;
            let gradient = gradients
                .get(weights.parameters[TT].as_tensor())
                .ok_or_else(|| invalid("missing sign-answer transition gradient"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(gradient.iter().all(|x| x.is_finite()));
            for token in 0..2 {
                if should_have_credit {
                    assert!(
                        gradient[token * 120] < 0.,
                        "minus-identity choice needs descent credit at token {token}"
                    );
                    assert!(
                        gradient[token * 120 + 1] > 0.,
                        "identity choice needs ascent credit at token {token}"
                    );
                } else {
                    assert_eq!(norm(&gradient[token * 120..(token + 1) * 120]), 0.);
                }
            }
            for name in [TR, SR, TC, SC] {
                let gradient = gradients
                    .get(weights.parameters[name].as_tensor())
                    .ok_or_else(|| invalid("missing packed zero readout gradient"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                assert_eq!(norm(&gradient), 0., "sign loss must use latent roots alone");
            }
        }
        let gradients = sign_answer_loss(&finite, &ids, 0, true)?.backward()?;
        let gradient = gradients
            .get(finite.parameters[TT].as_tensor())
            .ok_or_else(|| invalid("missing reset sign-answer transition gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(norm(&gradient[..120]), 0.);
        assert!(gradient[120] < 0. && gradient[121] > 0.);
        Ok(())
    }

    #[test]
    fn geometric_context_finite_choice_neighbor_credit_uses_old_head_local_state() -> Result<()> {
        let weights = ContextWeights::new_finite_choice(2, 16, 2, 97)?;
        for (name, variable) in weights.parameters() {
            set(&weights, name, vec![0.; variable.elem_count()])?;
        }
        let n = weights.config.lanes();
        let mut transitions = vec![0.; 2 * n * 120];
        for (token, actions) in [[3usize, 1, 5, 7], [5, 1, 1, 1]].iter().enumerate() {
            for (lane, &action) in actions.iter().enumerate() {
                transitions[(token * n + lane) * 120 + action] = 2.;
            }
        }
        set(&weights, TT, transitions)?;
        let mut neighbor = vec![0.; n * 120 * 4];
        // Lane1 wraps to OLD lane0 inside head0. Its minus-identity score
        // observes the j coordinate. The selected actions remain strict.
        neighbor[(120 * 4) + 2] = 0.25;
        set(&weights, NT, neighbor)?;
        let trace = weights.float_trace(&[0, 1], 1, 2, false)?;
        assert_eq!(trace.actions, vec![vec![3, 1, 5, 7], vec![5, 1, 1, 1]]);
        assert_eq!(trace.states, vec![vec![3, 1, 5, 7], vec![7, 1, 5, 7]]);
        let gradients = sign_answer_loss(&weights, &[0, 1], 1, false)?.backward()?;
        let transition = gradients
            .get(weights.parameters[TT].as_tensor())
            .ok_or_else(|| invalid("missing neighbor transition gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(
            norm(&transition[..120]) > 0.,
            "previous lane0 receives neighbor credit"
        );
        assert_eq!(
            norm(&transition[n * 120..(n + 1) * 120]),
            0.,
            "current lane0 update is not the OLD neighbor"
        );
        for token in 0..2 {
            for lane in 2..4 {
                let at = (token * n + lane) * 120;
                assert_eq!(
                    norm(&transition[at..at + 120]),
                    0.,
                    "credit cannot cross heads"
                );
            }
        }
        let neighbor = gradients
            .get(weights.parameters[NT].as_tensor())
            .ok_or_else(|| invalid("missing neighbor basis gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let at = 120 * 4;
        assert!(neighbor[at + 1] < 0., "basis sees OLD +i neighbor");
        assert_eq!(
            neighbor[at + 3],
            0.,
            "basis must not see updated +k neighbor"
        );
        Ok(())
    }
    #[test]
    fn geometric_context_delayed_readout_credit_reaches_earlier_transition() -> Result<()> {
        let weights = ContextWeights::new(4, 8, 1, 219)?;
        let ids = [0, 1, 2];
        let n = weights.config.lanes();
        let loss = |reset| -> Result<Tensor> {
            let last = weights
                .packed(&ids, 1, 3, reset, true)?
                .narrow(1, 2, 1)?
                .reshape((n, PACKED_WIDTH))?;
            let roots = logits_cross_entropy(&last.narrow(1, 0, 120)?, &vec![3; n], None)?;
            let radius = logits_cross_entropy(&last.narrow(1, 120, 33)?, &vec![18; n], None)?;
            Ok(roots.add(&radius)?)
        };
        let gradients = loss(false)?.backward()?;
        let transition = gradients
            .get(weights.parameters[TT].as_tensor())
            .ok_or_else(|| invalid("missing delayed transition credit"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(norm(&transition[..n * 120]) > 1e-8);
        for name in [ST, NT, SR, NR, SC, NC] {
            let gradient = gradients
                .get(weights.parameters[name].as_tensor())
                .ok_or_else(|| invalid(format!("missing {name} credit")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(gradient.iter().all(|v| v.is_finite()));
            assert!(norm(&gradient) > 1e-8, "{name} delayed credit is vacuous");
        }
        for name in [TR, TC] {
            assert!(
                gradients
                    .get(weights.parameters[name].as_tensor())
                    .is_none(),
                "direct lexical readout was detached"
            );
        }
        let reset_gradients = loss(true)?.backward()?;
        let reset = reset_gradients
            .get(weights.parameters[TT].as_tensor())
            .ok_or_else(|| invalid("missing reset transition credit"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(norm(&reset[..2 * n * 120]), 0.);
        assert!(norm(&reset[2 * n * 120..3 * n * 120]) > 1e-8);
        Ok(())
    }
    fn noninjective() -> Result<ContextWeights> {
        let weights = ContextWeights::new(3, 4, 1, 27)?;
        for (name, v) in weights.parameters() {
            set(&weights, name, vec![0.; v.elem_count()])?;
        }
        let mut transitions = vec![-16.; 3 * 120];
        transitions[1] = 2.;
        transitions[120 + 3] = 2.;
        transitions[240 + 1] = 2.;
        set(&weights, TT, transitions)?;
        let mut root_bias = vec![-16.; 3 * 120];
        for x in 0..3 {
            root_bias[x * 120 + 1] = if x < 2 { 8. } else { 0. };
            root_bias[x * 120 + 3] = 0.;
        }
        set(&weights, TR, root_bias)?;
        let mut root_basis = vec![0.; 120 * 4];
        root_basis[4] = 2.;
        root_basis[5] = -2.;
        root_basis[12] = -2.;
        root_basis[13] = 2.;
        set(&weights, SR, root_basis)?;
        let mut category = vec![-8.; 3 * 33];
        for row in category.chunks_exact_mut(33) {
            row[17] = 8.;
        }
        set(&weights, TC, category)?;
        Ok(weights)
    }
    #[test]
    fn geometric_context_noninjective_observation_retains_hidden_history() -> Result<()> {
        let weights = noninjective()?;
        let trace = weights.float_trace(&[0, 2, 1, 2], 2, 2, false)?;
        assert_eq!(trace.states, vec![vec![1], vec![1], vec![3], vec![3]]);
        assert_eq!(
            trace.codes[0], trace.codes[2],
            "distinct hidden roots must share current emitted code"
        );
        assert_ne!(
            trace.codes[1], trace.codes[3],
            "same next token must be able to reveal retained distinction"
        );
        assert_eq!(trace.emitted_roots, vec![1, 1, 1, 3]);
        let reset = weights.float_trace(&[0, 2, 1, 2], 2, 2, true)?;
        assert_eq!(reset.codes[1], reset.codes[3]);
        for ids in [[0, 2], [1, 2]] {
            let single = weights.float_trace(&ids, 1, 2, false)?;
            let prefix = weights.float_trace(&ids[..1], 1, 1, false)?;
            assert_eq!(single.codes[..1], prefix.codes);
            assert_eq!(single.states[..1], prefix.states);
        }
        let output = weights.forward(&[0, 2, 1, 2], 2, 2, false)?;
        let values = output.context.flatten_all()?.to_vec1::<f32>()?;
        for (i, lane) in values.chunks_exact(4).enumerate() {
            assert_eq!(
                crate::geometric_address::encode_lane([lane[0], lane[1], lane[2], lane[3]])?,
                trace.codes[i]
            );
        }
        let headring = ContextOp {
            batch: 1,
            time: 1,
            heads: 2,
            lanes_per_head: 4,
            reset: false,
            finite_choice: false,
        };
        assert_eq!(headring.neighbor(3), 0);
        assert_eq!(headring.neighbor(7), 4);
        assert!(weights.forward(&[3], 1, 1, false).is_err());
        assert!(GeometricContextConfig::new(4, 36, 2, 1).is_err());
        Ok(())
    }
    #[test]
    fn geometric_context_emission_local_physical_surrogate_and_absence() -> Result<()> {
        let op = EmitOp {
            batch: 1,
            time: 1,
            lanes: 1,
        };
        let mut rs = vec![-0.3; 120];
        rs[1] = 0.8;
        let mut cs = vec![-0.4; 33];
        cs[17] = 1.;
        cs[16] = 0.1;
        cs[18] = 0.2;
        let g = [0.7, 0.3, -0.2, 0.1];
        let (dr, dc) = op.backward(&rs, &cs, &g)?;
        assert!(norm(&dr) > 0.);
        assert!(norm(&dc) > 0.);
        for (i, &value) in dc.iter().enumerate() {
            if !(16..=18).contains(&i) {
                assert_eq!(value, 0.);
            }
        }
        // Differentiate the declared local smooth emission, fixing the hard
        // selected direction and neighbor set. It is not argmax finite-diff.
        let scalar = |r: &[f64], c: &[f64]| -> f64 {
            let p = probabilities(&std::array::from_fn::<_, 120, _>(|i| r[i]));
            let angular = tangent(root(1), g.map(f64::from));
            let direction = (0..120)
                .map(|i| p[i] * dot(angular, roots()[i]))
                .sum::<f64>();
            let max = c[16..=18].iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let sum = (16..=18).map(|i| (c[i] - max).exp()).sum::<f64>();
            direction
                + (16..=18)
                    .map(|i| (c[i] - max).exp() / sum * radius(i))
                    .sum::<f64>()
                    * f64::from(g[0])
        };
        let r = rs.iter().map(|&x| f64::from(x)).collect::<Vec<_>>();
        let c = cs.iter().map(|&x| f64::from(x)).collect::<Vec<_>>();
        let eps = 1e-5;
        for (in_root, index) in [(true, 3), (false, 16), (false, 17), (false, 18)] {
            let mut rp = r.clone();
            let mut rm = r.clone();
            let mut cp = c.clone();
            let mut cm = c.clone();
            let analytic = if in_root {
                rp[index] += eps;
                rm[index] -= eps;
                f64::from(dr[index])
            } else {
                cp[index] += eps;
                cm[index] -= eps;
                f64::from(dc[index])
            };
            let fd = (scalar(&rp, &cp) - scalar(&rm, &cm)) / (2. * eps);
            assert!((analytic - fd).abs() < 1e-7 + analytic.abs() * 1e-5);
        }
        cs[0] = 4.;
        let (dr, dc) = op.backward(&rs, &cs, &g)?;
        assert_eq!(norm(&dr), 0.);
        assert_eq!(norm(&dc), 0.);
        assert_eq!(op.forward(&rs, &cs)?, vec![0.; 4]);
        for cat in [1, 17, 32] {
            for r in [0, 1, 3, 24, 119] {
                let mut rs = vec![-1.; 120];
                rs[r] = 1.;
                let mut cs = vec![-1.; 33];
                cs[cat] = 1.;
                let actual = op.forward(&rs, &cs)?;
                assert_eq!(
                    crate::geometric_address::encode_lane([
                        actual[0], actual[1], actual[2], actual[3]
                    ])?,
                    code(r as u8, cat as u8)
                );
            }
        }
        Ok(())
    }
    struct Fixture {
        root: PathBuf,
        base: PathBuf,
        event_source: PathBuf,
        event_native: PathBuf,
        span_native: PathBuf,
        potential_native: PathBuf,
    }
    impl Fixture {
        fn paths(&self) -> ContextSourcePaths<'_> {
            ContextSourcePaths {
                base: &self.base,
                event_source: &self.event_source,
                event_native: &self.event_native,
                span_native: &self.span_native,
                potential_native: &self.potential_native,
            }
        }
    }
    const REGISTRY: &[u8] = b"context fixture token registry 0..3/v1";
    fn fixture() -> Result<Fixture> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-context-{}-{clock}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let fixture = Fixture {
            base: root.join("base"),
            event_source: root.join("event-source"),
            event_native: root.join("event-native"),
            span_native: root.join("span-native"),
            potential_native: root.join("potential-native"),
            root,
        };
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
        let address = GeometricAddressConfig::new(8, 1)?;
        model.set_geometric_address(address.clone())?;
        let span = GeometricSpanConfig::new(8)?;
        model.set_geometric_span(span.clone())?;
        model.save(&fixture.base)?;
        let event = EventWeights::new(4, 2, 173)?;
        event.save_source(&fixture.event_source, &fixture.base, REGISTRY)?;
        CompiledEvents::compile(&event, &fixture.event_source, &fixture.base, REGISTRY)?
            .save(&fixture.event_native)?;
        let source = SpanSourceBinding::from_files(
            &fixture.base.join("model.safetensors"),
            &fixture.base.join("config.json"),
            REGISTRY,
        )?;
        CompiledSpanActions::compile(
            model.variables()["embedding.weight"].as_tensor(),
            &span,
            &source,
        )?
        .save(&fixture.span_native)?;
        let potential = PotentialSourceBinding::from_directory(&fixture.base, REGISTRY)?;
        CompiledGeometricPotentials::compile(
            &model.geometric_address_potentials()?,
            &address,
            &potential,
        )?
        .save(&fixture.potential_native)?;
        Ok(fixture)
    }
    #[test]
    fn geometric_context_q4_potential_rebind_preserves_source_and_tables() -> Result<()> {
        let fixture = fixture()?;
        let old_source = fixture.root.join("old-context-source");
        let weights = ContextWeights::new_finite_choice(4, 8, 1, 173)?;
        weights.save_source(&old_source, fixture.paths(), REGISTRY)?;
        let before = CompiledContext::compile(&weights, &old_source, fixture.paths(), REGISTRY)?;
        let potential_source = fixture.root.join("q4-potential-source");
        let potential_native = fixture.root.join("q4-potential-native");
        let strict =
            crate::geometric_potential_q4::PotentialQ4Weights::from_base(&fixture.base, REGISTRY)?;
        strict.save(&potential_source)?;
        let base = PotentialSourceBinding::from_directory(&fixture.base, REGISTRY)?;
        CompiledGeometricPotentials::compile_q4(&strict, &potential_source, &base)?
            .save(&potential_native)?;
        let new_paths = ContextSourcePaths {
            potential_native: &potential_native,
            ..fixture.paths()
        };
        // Old source binding must fail with the new dependency; only an
        // explicit re-save of the already admitted weights creates the envelope.
        assert!(ContextWeights::load_source(&old_source, new_paths, REGISTRY).is_err());
        let new_source = fixture.root.join("rebound-context-source");
        let new_native = fixture.root.join("rebound-context-native");
        weights.save_source(&new_source, new_paths, REGISTRY)?;
        let after = CompiledContext::compile(&weights, &new_source, new_paths, REGISTRY)?;
        after.save(&new_native)?;
        let reload = CompiledContext::load(&new_native, &new_source, new_paths, REGISTRY)?;
        assert_eq!(before.table_bytes, reload.table_bytes);
        assert_eq!(
            before.metadata.source.parameters,
            reload.metadata.source.parameters
        );
        assert_eq!(
            fs::read(old_source.join(SOURCE_FILES[1]))?,
            fs::read(new_source.join(SOURCE_FILES[1]))?
        );
        assert_eq!(
            reload
                .metadata
                .source
                .frozen
                .files
                .keys()
                .filter(|k| k.starts_with("potential_native/"))
                .count(),
            7
        );
        let ids = [0, 1, 2, 3, 3, 2, 1, 0];
        let old_trace = trace_native(&ids, 2, 4, &before, false)?;
        let new_trace = trace_native(&ids, 2, 4, &reload, false)?;
        assert_eq!(old_trace.states, new_trace.states);
        assert_eq!(old_trace.actions, new_trace.actions);
        assert_eq!(old_trace.codes, new_trace.codes);
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn geometric_context_source_reload_stale_and_resealed_tamper() -> Result<()> {
        let fixture = fixture()?;
        let source = fixture.root.join("context-source");
        let native = fixture.root.join("context-native");
        let weights = ContextWeights::new(4, 8, 1, 173)?;
        weights.save_source(&source, fixture.paths(), REGISTRY)?;
        assert!(weights
            .save_source(&source, fixture.paths(), REGISTRY)
            .is_err());
        let loaded = ContextWeights::load_source(&source, fixture.paths(), REGISTRY)?;
        assert_eq!(
            parameter_identities(&weights)?,
            parameter_identities(&loaded)?
        );
        let compiled = CompiledContext::compile(&weights, &source, fixture.paths(), REGISTRY)?;
        compiled.save(&native)?;
        let reload = CompiledContext::load(&native, &source, fixture.paths(), REGISTRY)?;
        let ids = [0, 1, 2, 3, 3, 2, 1, 0];
        let before = trace_native(&ids, 2, 4, &compiled, false)?;
        let after = trace_native(&ids, 2, 4, &reload, false)?;
        assert_eq!(before.states, after.states);
        assert_eq!(before.actions, after.actions);
        assert_eq!(before.codes, after.codes);
        assert_eq!(before.emitted_roots, after.emitted_roots);
        assert_eq!(before.categories, after.categories);
        let reference = loaded.float_trace(&ids, 2, 4, false)?;
        assert_eq!(reference.states, after.states);
        assert_eq!(reference.actions, after.actions);
        assert_eq!(reference.address_codes()?, after.codes);
        assert!(
            ContextWeights::load_source(&source, fixture.paths(), b"different registry").is_err()
        );
        let payload = native.join(NATIVE_FILES[1]);
        let original = fs::read(&payload)?;
        let mut changed = original.clone();
        changed[0] ^= 1;
        fs::write(&payload, &changed)?;
        let meta_path = native.join(NATIVE_FILES[0]);
        let meta_bytes = fs::read(&meta_path)?;
        let mut meta: CompiledContextMetadata = serde_json::from_slice(&meta_bytes)?;
        meta.table_sha256 = sha256_bytes(&changed);
        fs::write(&meta_path, serde_json::to_vec_pretty(&meta)?)?;
        assert!(CompiledContext::load(&native, &source, fixture.paths(), REGISTRY).is_err());
        fs::write(&payload, &original)?;
        fs::write(&meta_path, &meta_bytes)?;
        let span = fixture.span_native.join("token-actions.bin");
        let span_bytes = fs::read(&span)?;
        let mut changed = span_bytes.clone();
        changed[0] = (changed[0] + 1) % 120;
        fs::write(&span, &changed)?;
        assert!(ContextWeights::load_source(&source, fixture.paths(), REGISTRY).is_err());
        fs::write(&span, &span_bytes)?;
        let mut values = loaded.parameters[TR].flatten_all()?.to_vec1::<f32>()?;
        values[0] = f32::from_bits(values[0].to_bits() ^ 1);
        set(&loaded, TR, values)?;
        assert!(reload.validate_for(&loaded).is_err());
        assert!(CompiledContext::compile(&loaded, &source, fixture.paths(), REGISTRY).is_err());
        fs::write(native.join("unknown"), b"x")?;
        assert!(CompiledContext::load(&native, &source, fixture.paths(), REGISTRY).is_err());
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }
    #[test]
    fn geometric_context_finite_choice_policy_reload_preserves_forward_and_identity() -> Result<()>
    {
        let fixture = fixture()?;
        let legacy = ContextWeights::new(4, 8, 1, 173)?;
        // A retained fit differs from its initialization: the conversion must
        // preserve those learned values and the loaded variables themselves.
        let mut transition = legacy.parameters[TT].flatten_all()?.to_vec1::<f32>()?;
        transition[0] += 0.125;
        set(&legacy, TT, transition)?;
        let retained_source = fixture.root.join("retained-parent-source");
        legacy.save_source(&retained_source, fixture.paths(), REGISTRY)?;
        let retained = ContextWeights::load_source(&retained_source, fixture.paths(), REGISTRY)?;
        let variable_ids: Vec<_> = retained
            .parameters
            .iter()
            .map(|(name, value)| (name.clone(), value.as_tensor().id()))
            .collect();
        let original_config = retained.config.clone();
        let original_parameters = parameter_identities(&retained)?;
        let finite = retained.into_finite_choice()?;
        assert_eq!(parameter_identities(&finite)?, original_parameters);
        assert_eq!(
            finite
                .parameters
                .iter()
                .map(|(name, value)| (name.clone(), value.as_tensor().id()))
                .collect::<Vec<_>>(),
            variable_ids
        );
        let mut expected_config = original_config;
        expected_config.surrogate = FINITE_CHOICE_SURROGATE.into();
        assert_eq!(finite.config, expected_config);
        assert_eq!(legacy.config.surrogate, SURROGATE);
        assert_eq!(finite.config.surrogate, FINITE_CHOICE_SURROGATE);
        assert_eq!(parameter_bytes(&legacy)?, parameter_bytes(&finite)?);
        assert_eq!(
            ExpandedContext::compile(&legacy)?.bytes(),
            ExpandedContext::compile(&finite)?.bytes()
        );
        let ids = [0, 1, 2, 3];
        assert_eq!(
            tensor_bits(&legacy.forward(&ids, 1, 4, false)?.latent_roots)?,
            tensor_bits(&finite.forward(&ids, 1, 4, false)?.latent_roots)?
        );
        let mut artifacts = Vec::new();
        for (name, weights) in [("legacy", &legacy), ("finite", &finite)] {
            let source = fixture.root.join(format!("{name}-source"));
            let native = fixture.root.join(format!("{name}-native"));
            weights.save_source(&source, fixture.paths(), REGISTRY)?;
            let loaded = ContextWeights::load_source(&source, fixture.paths(), REGISTRY)?;
            assert_eq!(loaded.config(), weights.config());
            assert_eq!(parameter_bytes(&loaded)?, parameter_bytes(weights)?);
            assert_eq!(
                tensor_bits(&loaded.forward(&ids, 1, 4, false)?.latent_roots)?,
                tensor_bits(&weights.forward(&ids, 1, 4, false)?.latent_roots)?
            );
            let compiled = CompiledContext::compile(&loaded, &source, fixture.paths(), REGISTRY)?;
            compiled.save(&native)?;
            let reload = CompiledContext::load(&native, &source, fixture.paths(), REGISTRY)?;
            assert_eq!(reload.config(), weights.config());
            reload.validate_for(weights)?;
            artifacts.push(reload);
        }
        assert!(artifacts[0].validate_for(&finite).is_err());
        assert!(artifacts[1].validate_for(&legacy).is_err());
        let old = trace_native(&ids, 1, 4, &artifacts[0], false)?;
        let new = trace_native(&ids, 1, 4, &artifacts[1], false)?;
        assert_eq!(old.states, new.states);
        assert_eq!(old.actions, new.actions);
        assert_eq!(old.codes, new.codes);
        assert_eq!(old.categories, new.categories);
        let float = finite.float_trace(&ids, 1, 4, false)?;
        assert_eq!(float.states, new.states);
        let expected: Vec<f32> = float
            .states
            .iter()
            .flat_map(|row| row.iter().flat_map(|&code| root(code).map(|x| x as f32)))
            .collect();
        assert_eq!(
            finite
                .forward(&ids, 1, 4, false)?
                .latent_roots
                .flatten_all()?
                .to_vec1::<f32>()?,
            expected
        );
        let source = fixture.root.join("finite-source");
        let metadata_path = source.join(SOURCE_FILES[0]);
        let mut metadata: ContextSourceMetadata =
            serde_json::from_slice(&fs::read(&metadata_path)?)?;
        metadata.config.surrogate = "unrecognized-choice-policy".into();
        fs::write(metadata_path, serde_json::to_vec_pretty(&metadata)?)?;
        assert!(ContextWeights::load_source(&source, fixture.paths(), REGISTRY).is_err());
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }
    #[test]
    fn geometric_context_fixed_codec_padding_and_typed_absence() -> Result<()> {
        assert_eq!(quantize(0.5 / 16_777_216.)?, 1);
        assert_eq!(quantize(-0.5 / 16_777_216.)?, -1);
        assert_eq!(quantize(-128.)?, i32::MIN);
        assert!(quantize(128.).is_err());
        assert!(quantize(f64::NAN).is_err());
        let weights = ContextWeights::new(3, 32, 2, 77)?;
        let expanded = ExpandedContext::compile(&weights)?;
        let native = expanded.native(weights.config())?;
        assert_eq!(native.stats().coefficient_reads, 6552);
        for f in 0..3 {
            let stride = if f == 2 { 64 } else { 128 };
            for lane in 0..8 {
                for s in 0..128 {
                    for choice in 0..stride {
                        if s >= 120 || choice >= CLASSES[f] {
                            assert_eq!(expanded.own[f][(lane * 128 + s) * stride + choice], 0);
                            assert_eq!(
                                expanded.neighbor[f]
                                    .as_ref()
                                    .ok_or_else(|| invalid("missing neighbor fixture"))?
                                    [(lane * 128 + s) * stride + choice],
                                0
                            );
                        }
                    }
                }
            }
        }
        let single = ContextWeights::new(3, 4, 1, 78)?;
        let single = ExpandedContext::compile(&single)?;
        assert!(single.neighbor.iter().all(Option::is_none));
        assert_eq!(
            code(77, 0),
            AddressCode {
                root: 1,
                radius_bin: 0,
                present: false
            }
        );
        assert_ne!(code(1, 17), code(1, 0));
        let mut high = weights.parameters[TC].flatten_all()?.to_vec1::<f32>()?;
        high[0] = 128.;
        set(&weights, TC, high)?;
        assert!(ExpandedContext::compile(&weights).is_err());
        Ok(())
    }
}
