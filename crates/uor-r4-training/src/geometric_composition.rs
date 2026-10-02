//! Offline learner for a fixed two-term Spin(4) payload composition bank.
//! Hard forward uses the native i64-Q16 operator. Backward is a declared
//! conditional 120-way finite-choice surrogate, never an argmax derivative.
//! The remaining float residual/trunk/vocabulary path is outside this module.
use crate::geometric_value_native::{ValuePacketRecord, ValuePacketStatus};
use crate::geometric_value_producer::ValueProducerTrace;
use crate::{invalid, sha256_bytes, Result};
use candle_core::{CpuStorage, CustomOp1, CustomOp3, DType, Device, Layout, Shape, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype, SafeTensors};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    sync::Arc,
};
use uor_r4_integer::geometric_composition::{pack_gains, NativeGeometricComposition};
use uor_r4_integer::geometric_value::{NativeGeometricValues, ValuePacket, ValueState};
use uor_r4_integer::h4_tables::{H4Code, HistoricalH4Tables};

pub const SCHEMA: &str = "uor-r4.geometric-composition-offline/1";
pub const SOURCE_FILES: [&str; 2] = ["metadata.json", "composition-parameters.safetensors"];
pub const FROZEN_VALUE_SURROGATE: &str = "matched-values-Tensor/native-packet-hard-forward;frozen-selected-left-right-H4-adjoint-with-q4-gain;coordinate-and-quarter-rounding-STE;invalid-stop;no-bank-gradient/1";
pub const TERMS: usize = 128;
pub const ROOTS: usize = 120;
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
const RULE:&str="H2;head-output8-input4-term2;left*root*inverse(right);signed-q4[-7,7]/4-dimensionless;canonical-Q16-atoms;i64-sum-all-then-one-quarter-round-ties-away;no-presence-inference/1";
const SURROGATE:&str="hard-native-i64Q16-forward;conditional120-softmax-T1-full-transformed-K2-value-credit;other-selector-hard;gain-included;dimensionless-gain-shadow-hard-quarter-grid-identity-STE;post-update-project[-1.75,1.75];frozen-input-packets/1";
const INITIALIZATION:&str="fixed-v1;term0-left-right-identity-gain1/4;term1-left2-right3-gain-minus1/4;selector-selected-logit1-other0;no-data-selection/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionConfig {
    pub schema: String,
    pub heads: usize,
    pub input_lanes: usize,
    pub output_lanes: usize,
    pub terms_per_block: usize,
    pub geometry_sha256: String,
    pub rule: String,
    pub surrogate: String,
    pub initialization: String,
}
impl CompositionConfig {
    pub fn new() -> Self {
        Self {
            schema: SCHEMA.into(),
            heads: 2,
            input_lanes: 4,
            output_lanes: 8,
            terms_per_block: 2,
            geometry_sha256: sha256_bytes(ALGEBRA),
            rule: RULE.into(),
            surrogate: SURROGATE.into(),
            initialization: INITIALIZATION.into(),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if *self != Self::new() {
            return Err(invalid("composition config/policy differs"));
        }
        Ok(())
    }
}
impl Default for CompositionConfig {
    fn default() -> Self {
        Self::new()
    }
}
pub struct CompositionWeights {
    config: CompositionConfig,
    parameters: BTreeMap<String, Var>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    config: CompositionConfig,
    parameter_sha256: String,
    parameter_bytes: usize,
    left_sha256: String,
    right_sha256: String,
    packed_gains_sha256: String,
}

fn values(t: &Tensor, shape: &[usize]) -> Result<Vec<f32>> {
    if t.dtype() != DType::F32 || !t.device().is_cpu() || t.dims() != shape {
        return Err(invalid("composition requires declared CPU F32 shape"));
    }
    let out = t.flatten_all()?.to_vec1::<f32>()?;
    if out.iter().any(|v| !v.is_finite()) {
        return Err(invalid("nonfinite composition parameter"));
    }
    Ok(out)
}
fn choices(logits: &[f32]) -> Vec<u8> {
    logits
        .chunks_exact(ROOTS)
        .map(|row| {
            let mut best = 0;
            for i in 1..ROOTS {
                if row[i] > row[best] {
                    best = i;
                }
            }
            best as u8
        })
        .collect()
}
fn codes(gains: &[f32]) -> Result<Vec<i8>> {
    if gains.len() != TERMS || gains.iter().any(|g| !g.is_finite() || g.abs() > 1.75) {
        return Err(invalid(
            "composition gains require128 finite dimensionless shadows within[-1.75,1.75]",
        ));
    }
    Ok(gains.iter().map(|g| (g * 4.).round() as i8).collect())
}
impl CompositionWeights {
    pub fn new() -> Result<Self> {
        let mut left = vec![0f32; TERMS * ROOTS];
        let mut right = left.clone();
        let mut gains = vec![0f32; TERMS];
        for term in 0..TERMS {
            left[term * ROOTS + if term % 2 == 0 { 1 } else { 2 }] = 1.;
            right[term * ROOTS + if term % 2 == 0 { 1 } else { 3 }] = 1.;
            gains[term] = if term % 2 == 0 { 0.25 } else { -0.25 };
        }
        Ok(Self {
            config: CompositionConfig::new(),
            parameters: BTreeMap::from([
                (
                    "left_logits".into(),
                    Var::from_vec(left, (TERMS, ROOTS), &Device::Cpu)?,
                ),
                (
                    "right_logits".into(),
                    Var::from_vec(right, (TERMS, ROOTS), &Device::Cpu)?,
                ),
                ("gains".into(), Var::from_vec(gains, TERMS, &Device::Cpu)?),
            ]),
        })
    }
    pub fn config(&self) -> &CompositionConfig {
        &self.config
    }
    pub fn parameters(&self) -> &BTreeMap<String, Var> {
        &self.parameters
    }
    fn parameter(&self, name: &str) -> Result<&Tensor> {
        self.parameters
            .get(name)
            .map(Var::as_tensor)
            .ok_or_else(|| invalid("composition parameter missing"))
    }
    pub fn selected_roots(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        self.config.validate()?;
        Ok((
            choices(&values(self.parameter("left_logits")?, &[TERMS, ROOTS])?),
            choices(&values(self.parameter("right_logits")?, &[TERMS, ROOTS])?),
        ))
    }
    pub fn packed_gains(&self) -> Result<Vec<u8>> {
        pack_gains(&codes(&values(self.parameter("gains")?, &[TERMS])?)?)
            .map_err(|e| invalid(e.to_string()))
    }
    pub fn hard_bank(&self) -> Result<NativeGeometricComposition> {
        let (left, right) = self.selected_roots()?;
        NativeGeometricComposition::new(&left, &right, &self.packed_gains()?, ALGEBRA)
            .map_err(|e| invalid(e.to_string()))
    }
    pub fn project_gain_range(&self) -> Result<()> {
        let gains = values(self.parameter("gains")?, &[TERMS])?
            .into_iter()
            .map(|x| x.clamp(-1.75, 1.75))
            .collect::<Vec<_>>();
        self.parameters
            .get("gains")
            .ok_or_else(|| invalid("composition gains missing"))?
            .set(&Tensor::from_vec(gains, TERMS, &Device::Cpu)?)?;
        Ok(())
    }
    pub fn compose_trace(&self, trace: &ValueProducerTrace) -> Result<Vec<i64>> {
        let op = CompositionOp::new(trace)?;
        op.forward(
            &values(self.parameter("left_logits")?, &[TERMS, ROOTS])?,
            &values(self.parameter("right_logits")?, &[TERMS, ROOTS])?,
            &values(self.parameter("gains")?, &[TERMS])?,
        )
        .map_err(Into::into)
    }
    pub fn forward(&self, trace: &ValueProducerTrace) -> Result<Tensor> {
        self.config.validate()?;
        self.selected_roots()?;
        self.packed_gains()?;
        let op = CompositionOp::new(trace)?;
        Ok(self.parameter("left_logits")?.contiguous()?.apply_op3(
            &self.parameter("right_logits")?.contiguous()?,
            &self.parameter("gains")?.contiguous()?,
            op,
        )?)
    }
    /// Actual value graph input, with hard native packet composition forward.
    /// The selected bank is captured as immutable data, never a graph input.
    /// The declared adjoint ignores coordinate and quarter rounding; it is not
    /// a derivative of the discrete codec. Trace integers remain authoritative.
    pub fn forward_frozen_values(
        &self,
        input: &Tensor,
        trace: &ValueProducerTrace,
    ) -> Result<Tensor> {
        let expected = trace
            .values_q16
            .iter()
            .map(|&v| (f64::from(v) / 65536.) as f32)
            .collect::<Vec<_>>();
        let actual = values(input, &[trace.batch, trace.heads, trace.time, 16])?;
        if actual
            .iter()
            .map(|v| v.to_bits())
            .ne(expected.iter().map(|v| v.to_bits()))
        {
            return Err(invalid(
                "frozen composition value Tensor differs from packet trace reconstruction",
            ));
        }
        // compose_trace validates structural validity and exact decoded-pair bits.
        let output = self
            .compose_trace(trace)?
            .into_iter()
            .map(|v| (v as f64 / 65536.) as f32)
            .collect::<Vec<_>>();
        let (left, right) = self.selected_roots()?;
        let gains = codes(&values(self.parameter("gains")?, &[TERMS])?)?;
        let roots = uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots();
        let mut maps = vec![[[0f64; 4]; 4]; 2 * 8 * 4];
        for h in 0..2 {
            for out in 0..8 {
                for lane in 0..4 {
                    for term in 0..2 {
                        let at = ((h * 8 + out) * 4 + lane) * 2 + term;
                        let a = roots[usize::from(left[at])].to_array();
                        let mut b = roots[usize::from(right[at])].to_array();
                        for value in &mut b[1..] {
                            *value = -*value;
                        }
                        for column in 0..4 {
                            let mut axis = [0.; 4];
                            axis[column] = 1.;
                            let image = compose_hamilton(compose_hamilton(a, axis), b);
                            for row in 0..4 {
                                maps[(h * 8 + out) * 4 + lane][row][column] +=
                                    f64::from(gains[at]) * 0.25 * image[row];
                            }
                        }
                    }
                }
            }
        }
        Ok(input.contiguous()?.apply_op1(FrozenValueComposition {
            batch: trace.batch,
            time: trace.time,
            input: expected,
            output,
            occurrence_valid: trace.occurrence_valid.clone(),
            maps,
        })?)
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        self.config.validate()?;
        let (left, right) = self.selected_roots()?;
        let packed = self.packed_gains()?;
        let mut owned = BTreeMap::new();
        for (name, shape) in [
            ("gains", vec![TERMS]),
            ("left_logits", vec![TERMS, ROOTS]),
            ("right_logits", vec![TERMS, ROOTS]),
        ] {
            owned.insert(
                name,
                (
                    shape,
                    values(
                        self.parameter(name)?,
                        if name == "gains" {
                            &[TERMS]
                        } else {
                            &[TERMS, ROOTS]
                        },
                    )?
                    .iter()
                    .flat_map(|v| v.to_bits().to_le_bytes())
                    .collect::<Vec<_>>(),
                ),
            );
        }
        let views = owned
            .iter()
            .map(|(n, (s, b))| {
                Ok((
                    *n,
                    TensorView::new(Dtype::F32, s.clone(), b)
                        .map_err(|e| invalid(e.to_string()))?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let bytes = safetensors::serialize(views, None).map_err(|e| invalid(e.to_string()))?;
        let metadata = Metadata {
            config: self.config.clone(),
            parameter_sha256: sha256_bytes(&bytes),
            parameter_bytes: bytes.len(),
            left_sha256: sha256_bytes(&left),
            right_sha256: sha256_bytes(&right),
            packed_gains_sha256: sha256_bytes(&packed),
        };
        fs::create_dir(directory)?;
        fs::write(directory.join(SOURCE_FILES[1]), bytes)?;
        fs::write(
            directory.join(SOURCE_FILES[0]),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        Ok(())
    }
    pub fn load(directory: &Path) -> Result<Self> {
        let names = fs::read_dir(directory)?
            .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect::<std::io::Result<BTreeSet<_>>>()?;
        if names != SOURCE_FILES.into_iter().map(str::to_owned).collect() {
            return Err(invalid("composition source file set differs"));
        }
        let meta: Metadata = serde_json::from_slice(&fs::read(directory.join(SOURCE_FILES[0]))?)?;
        meta.config.validate()?;
        let bytes = fs::read(directory.join(SOURCE_FILES[1]))?;
        if bytes.len() != meta.parameter_bytes || sha256_bytes(&bytes) != meta.parameter_sha256 {
            return Err(invalid("composition parameter binding differs"));
        }
        let tensors = SafeTensors::deserialize(&bytes).map_err(|e| invalid(e.to_string()))?;
        if tensors
            .names()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>()
            != ["gains", "left_logits", "right_logits"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        {
            return Err(invalid("composition inventory differs"));
        }
        let model = Self::new()?;
        for (name, var) in &model.parameters {
            let view = tensors.tensor(name).map_err(|e| invalid(e.to_string()))?;
            if view.dtype() != Dtype::F32 || view.shape() != var.dims() {
                return Err(invalid("composition saved shape differs"));
            }
            let values = view
                .data()
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect::<Vec<_>>();
            var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        }
        let (left, right) = model.selected_roots()?;
        if sha256_bytes(&left) != meta.left_sha256
            || sha256_bytes(&right) != meta.right_sha256
            || sha256_bytes(&model.packed_gains()?) != meta.packed_gains_sha256
        {
            return Err(invalid("composition hard bank binding differs"));
        }
        Ok(model)
    }
}

fn packet(record: ValuePacketRecord) -> Result<ValuePacket> {
    let state = match record.status {
        ValuePacketStatus::Absent => ValueState::Absent,
        ValuePacketStatus::PresentZero => ValueState::PresentZero,
        ValuePacketStatus::PresentNonzero => ValueState::PresentNonzero,
    };
    ValuePacket::new(state, record.root, record.radius_bin).map_err(|e| invalid(e.to_string()))
}
struct CompositionOp {
    batch: usize,
    time: usize,
    packets: Vec<[[ValuePacket; 2]; 4]>,
    tables: Arc<HistoricalH4Tables>,
    codec: Arc<NativeGeometricValues>,
}
impl CompositionOp {
    fn new(trace: &ValueProducerTrace) -> Result<Self> {
        let rows = trace
            .batch
            .checked_mul(2)
            .and_then(|v| v.checked_mul(trace.time))
            .ok_or_else(|| invalid("composition trace size overflow"))?;
        if trace.batch == 0
            || trace.heads != 2
            || trace.time == 0
            || trace.time > 128
            || trace.packets.len() != rows * 4
            || trace.values_q16.len() != rows * 16
            || trace.occurrence_valid.len() != trace.batch * trace.time
        {
            return Err(invalid("composition trace layout differs"));
        }
        let tables =
            Arc::new(HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?);
        let codec =
            Arc::new(NativeGeometricValues::canonical().map_err(|e| invalid(e.to_string()))?);
        let mut packets = Vec::with_capacity(rows);
        for row in 0..rows {
            let b = row / (2 * trace.time);
            let t = row % trace.time;
            let valid = trace.occurrence_valid[b * trace.time + t];
            let mut lanes = [[ValuePacket::absent(); 2]; 4];
            for lane in 0..4 {
                for atom in 0..2 {
                    lanes[lane][atom] = packet(trace.packets[row * 4 + lane][atom].clone())?;
                }
                if !valid && lanes[lane].iter().any(|v| v.state() != ValueState::Absent) {
                    return Err(invalid(
                        "invalid occurrence contains a present composition packet",
                    ));
                }
                let decoded = codec
                    .decode_pair(lanes[lane][0], lanes[lane][1])
                    .map_err(|e| invalid(e.to_string()))?;
                if decoded.as_slice()
                    != &trace.values_q16[row * 16 + lane * 4..row * 16 + lane * 4 + 4]
                {
                    return Err(invalid(
                        "composition packets and decoded source values differ",
                    ));
                }
            }
            packets.push(lanes);
        }
        Ok(Self {
            batch: trace.batch,
            time: trace.time,
            packets,
            tables,
            codec,
        })
    }
    fn forward(&self, left: &[f32], right: &[f32], gains: &[f32]) -> candle_core::Result<Vec<i64>> {
        let bank = NativeGeometricComposition::new(
            &choices(left),
            &choices(right),
            &pack_gains(&codes(gains).map_err(|e| candle_core::Error::Msg(e.to_string()))?)
                .map_err(|e| candle_core::Error::Msg(e.to_string()))?,
            ALGEBRA,
        )
        .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        let mut output = Vec::with_capacity(self.packets.len() * 32);
        for (row, packets) in self.packets.iter().enumerate() {
            output.extend(
                bank.compose((row / self.time) % 2, packets)
                    .map_err(|e| candle_core::Error::Msg(e.to_string()))?,
            );
        }
        Ok(output)
    }
    fn transformed(
        &self,
        packets: &[ValuePacket; 2],
        a: u8,
        b: u8,
    ) -> candle_core::Result<[f64; 4]> {
        let a = H4Code::try_from(a).map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        let b = self
            .tables
            .inverse(H4Code::try_from(b).map_err(|e| candle_core::Error::Msg(e.to_string()))?);
        let mut out = [0f64; 4];
        for packet in packets {
            if packet.state() != ValueState::PresentNonzero {
                continue;
            }
            let root = self
                .tables
                .compose(self.tables.compose(a, packet.root()), b);
            let decoded = self.codec.decode(
                ValuePacket::present_nonzero(root.index(), packet.radius_bin())
                    .map_err(|e| candle_core::Error::Msg(e.to_string()))?,
            );
            for c in 0..4 {
                out[c] += f64::from(decoded[c]) / 65536.;
            }
        }
        Ok(out)
    }
    fn backward(
        &self,
        left: &[f32],
        right: &[f32],
        gains: &[f32],
        gradient: &[f32],
    ) -> candle_core::Result<(Vec<f32>, Vec<f32>, Vec<f32>)> {
        let a = choices(left);
        let b = choices(right);
        let q = codes(gains).map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        let mut da = vec![0f64; left.len()];
        let mut db = vec![0f64; right.len()];
        let mut dg = vec![0f64; TERMS];
        if gradient.len() != self.packets.len() * 32 {
            return Err(candle_core::Error::Msg("composition gradient shape".into()));
        }
        for (row, packets) in self.packets.iter().enumerate() {
            let head = (row / self.time) % 2;
            for output in 0..8 {
                for input in 0..4 {
                    for term in 0..2 {
                        let index = ((head * 8 + output) * 4 + input) * 2 + term;
                        let g = &gradient[row * 32 + output * 4..row * 32 + output * 4 + 4];
                        let dot = |v: [f64; 4]| (0..4).map(|c| v[c] * f64::from(g[c])).sum::<f64>();
                        dg[index] += dot(self.transformed(&packets[input], a[index], b[index])?);
                        for selector in 0..2 {
                            let logits = if selector == 0 {
                                &left[index * ROOTS..(index + 1) * ROOTS]
                            } else {
                                &right[index * ROOTS..(index + 1) * ROOTS]
                            };
                            let p = probabilities(logits);
                            let mut score = [0f64; ROOTS];
                            for r in 0..ROOTS {
                                let v = if selector == 0 {
                                    self.transformed(&packets[input], r as u8, b[index])?
                                } else {
                                    self.transformed(&packets[input], a[index], r as u8)?
                                };
                                score[r] = dot(v) * f64::from(q[index]) / 4.;
                            }
                            let mean = p.iter().zip(score).map(|(p, s)| p * s).sum::<f64>();
                            let target = if selector == 0 { &mut da } else { &mut db };
                            for r in 0..ROOTS {
                                target[index * ROOTS + r] += p[r] * (score[r] - mean);
                            }
                        }
                    }
                }
            }
        }
        fn cast(v: Vec<f64>) -> candle_core::Result<Vec<f32>> {
            let out = v.into_iter().map(|v| v as f32).collect::<Vec<_>>();
            if out.iter().any(|x| !x.is_finite()) {
                return Err(candle_core::Error::Msg(
                    "nonfinite composition gradient".into(),
                ));
            }
            Ok(out)
        }
        Ok((cast(da)?, cast(db)?, cast(dg)?))
    }
}
// Ordinary Hamilton multiplication in the same scalar-first root basis.
fn compose_hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}
struct FrozenValueComposition {
    batch: usize,
    time: usize,
    input: Vec<f32>,
    output: Vec<f32>,
    occurrence_valid: Vec<bool>,
    // Fixed selected geometric maps, offline adjoint only; no learned dense map.
    maps: Vec<[[f64; 4]; 4]>,
}
impl CustomOp1 for FrozenValueComposition {
    fn name(&self) -> &'static str {
        "frozen-geometric-composition-value-credit"
    }
    fn cpu_fwd(
        &self,
        storage: &CpuStorage,
        layout: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let input = contiguous(storage, layout)?;
        if input.len() != self.input.len()
            || input
                .iter()
                .map(|x| x.to_bits())
                .ne(self.input.iter().map(|x| x.to_bits()))
        {
            candle_core::bail!("frozen composition input changed after trace validation");
        }
        Ok((
            CpuStorage::F32(self.output.clone()),
            Shape::from((self.batch, 2, self.time, 32)),
        ))
    }
    fn bwd(
        &self,
        input: &Tensor,
        _out: &Tensor,
        gradient: &Tensor,
    ) -> candle_core::Result<Option<Tensor>> {
        let g = gradient.flatten_all()?.to_vec1::<f32>()?;
        if g.len() != self.batch * 2 * self.time * 32 || g.iter().any(|x| !x.is_finite()) {
            candle_core::bail!("frozen composition gradient shape/finiteness differs");
        }
        let mut dx = vec![0f64; self.input.len()];
        for b in 0..self.batch {
            for h in 0..2 {
                for t in 0..self.time {
                    if !self.occurrence_valid[b * self.time + t] {
                        continue;
                    }
                    let row = (b * 2 + h) * self.time + t;
                    for lane in 0..4 {
                        for out in 0..8 {
                            for column in 0..4 {
                                for axis in 0..4 {
                                    dx[row * 16 + lane * 4 + column] += self.maps
                                        [(h * 8 + out) * 4 + lane][axis][column]
                                        * f64::from(g[row * 32 + out * 4 + axis]);
                                }
                            }
                        }
                    }
                }
            }
        }
        let dx = dx.into_iter().map(|x| x as f32).collect::<Vec<_>>();
        if dx.iter().any(|x| !x.is_finite()) {
            candle_core::bail!("nonfinite frozen composition input adjoint");
        }
        Ok(Some(Tensor::from_vec(dx, input.shape(), input.device())?))
    }
}
fn probabilities(logits: &[f32]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut p = logits
        .iter()
        .map(|x| (f64::from(*x) - f64::from(max)).exp())
        .collect::<Vec<_>>();
    let sum = p.iter().sum::<f64>();
    for value in &mut p {
        *value /= sum;
    }
    p
}
fn contiguous<'a>(storage: &'a CpuStorage, layout: &Layout) -> candle_core::Result<&'a [f32]> {
    let (start, end) = layout
        .contiguous_offsets()
        .ok_or_else(|| candle_core::Error::Msg("composition contiguous input required".into()))?;
    Ok(&storage.as_slice::<f32>()?[start..end])
}
impl CustomOp3 for CompositionOp {
    fn name(&self) -> &'static str {
        "geometric-composition-hard-native-conditional-full-credit"
    }
    fn cpu_fwd(
        &self,
        a: &CpuStorage,
        la: &Layout,
        b: &CpuStorage,
        lb: &Layout,
        c: &CpuStorage,
        lc: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let result = self.forward(contiguous(a, la)?, contiguous(b, lb)?, contiguous(c, lc)?)?;
        Ok((
            CpuStorage::F32(
                result
                    .into_iter()
                    .map(|v| (v as f64 / 65536.) as f32)
                    .collect(),
            ),
            Shape::from((self.batch, 2, self.time, 32)),
        ))
    }
    fn bwd(
        &self,
        a: &Tensor,
        b: &Tensor,
        c: &Tensor,
        _out: &Tensor,
        gradient: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>, Option<Tensor>)> {
        let (da, db, dc) = self.backward(
            &a.flatten_all()?.to_vec1::<f32>()?,
            &b.flatten_all()?.to_vec1::<f32>()?,
            &c.flatten_all()?.to_vec1::<f32>()?,
            &gradient.flatten_all()?.to_vec1::<f32>()?,
        )?;
        Ok((
            Some(Tensor::from_vec(da, a.shape(), a.device())?),
            Some(Tensor::from_vec(db, b.shape(), b.device())?),
            Some(Tensor::from_vec(dc, c.shape(), c.device())?),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace() -> Result<ValueProducerTrace> {
        let codec = NativeGeometricValues::canonical().map_err(|e| invalid(e.to_string()))?;
        let zero = ValuePacketRecord {
            status: ValuePacketStatus::PresentZero,
            root: 1,
            radius_bin: 0,
        };
        let mut trace = ValueProducerTrace {
            batch: 1,
            heads: 2,
            time: 1,
            occurrence_valid: vec![true],
            packets: vec![[zero.clone(), zero]; 8],
            values_q16: vec![0; 32],
        };
        trace.packets[0] = [
            ValuePacketRecord {
                status: ValuePacketStatus::PresentNonzero,
                root: 1,
                radius_bin: 16,
            },
            ValuePacketRecord {
                status: ValuePacketStatus::PresentNonzero,
                root: 1,
                radius_bin: 12,
            },
        ];
        let decoded = codec
            .decode_pair(
                packet(trace.packets[0][0].clone())?,
                packet(trace.packets[0][1].clone())?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        trace.values_q16[..4].copy_from_slice(&decoded);
        Ok(trace)
    }

    #[test]
    fn composition_hard_forward_is_native_and_preserves_both_atoms() -> Result<()> {
        let weights = CompositionWeights::new()?;
        let trace = trace()?;
        let op = CompositionOp::new(&trace)?;
        let bank = weights.hard_bank()?;
        let mut expected = Vec::new();
        for (head, packets) in op.packets.iter().enumerate() {
            expected.extend(
                bank.compose(head, packets)
                    .map_err(|e| invalid(e.to_string()))?,
            );
        }
        assert_eq!(weights.compose_trace(&trace)?, expected);
        assert_eq!(
            weights.forward(&trace)?.flatten_all()?.to_vec1::<f32>()?,
            expected
                .iter()
                .map(|&v| (v as f64 / 65536.) as f32)
                .collect::<Vec<_>>()
        );
        let mut one_atom = trace.clone();
        one_atom.packets[0][1] = ValuePacketRecord {
            status: ValuePacketStatus::PresentZero,
            root: 1,
            radius_bin: 0,
        };
        one_atom.values_q16[..4]
            .copy_from_slice(&op.codec.decode(packet(one_atom.packets[0][0].clone())?));
        assert_ne!(
            weights.compose_trace(&trace)?,
            weights.compose_trace(&one_atom)?
        );
        let mut invalid = trace.clone();
        invalid.occurrence_valid[0] = false;
        assert!(weights.forward(&invalid).is_err());
        invalid = trace;
        invalid.values_q16[0] += 1;
        assert!(weights.forward(&invalid).is_err());
        Ok(())
    }

    #[test]
    fn composition_conditional_full_value_credit_includes_gain_and_antipodes() -> Result<()> {
        let weights = CompositionWeights::new()?;
        let trace = trace()?;
        let op = CompositionOp::new(&trace)?;
        let out = weights.forward(&trace)?;
        let gradient = out.flatten_all()?.narrow(0, 0, 1)?.sum_all()?.backward()?;
        for name in ["left_logits", "right_logits", "gains"] {
            let g = gradient
                .get(weights.parameters[name].as_tensor())
                .ok_or_else(|| invalid("composition coefficient gradient absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(g.iter().all(|v| v.is_finite()));
            assert!(g.iter().any(|v| *v != 0.), "{name}");
        }
        let left = gradient
            .get(weights.parameters["left_logits"].as_tensor())
            .ok_or_else(|| invalid("left gradient missing"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let right = gradient
            .get(weights.parameters["right_logits"].as_tensor())
            .ok_or_else(|| invalid("right gradient missing"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(left[0] < 0. && left[1] > 0.);
        assert!(right[0] < 0. && right[1] > 0.);
        let gain = gradient
            .get(weights.parameters["gains"].as_tensor())
            .ok_or_else(|| invalid("gain gradient missing"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(gain[0], 1.0625);
        let mut negative_gains = weights.parameters["gains"].to_vec1::<f32>()?;
        negative_gains[0] = -0.25;
        weights.parameters["gains"].set(&Tensor::from_vec(negative_gains, TERMS, &Device::Cpu)?)?;
        let negative_gradient = weights
            .forward(&trace)?
            .flatten_all()?
            .narrow(0, 0, 1)?
            .sum_all()?
            .backward()?;
        for (name, positive) in [("left_logits", &left), ("right_logits", &right)] {
            let negative = negative_gradient
                .get(weights.parameters[name].as_tensor())
                .ok_or_else(|| invalid("negative-gain selector gradient missing"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            for r in 0..ROOTS {
                assert_eq!(negative[r], -positive[r]);
            }
        }
        // Finite difference of the DECLARED conditional expected-value
        // surrogate, not of hard argmax or final quarter-rounding forward.
        let logits = weights
            .parameter("left_logits")?
            .narrow(0, 0, 1)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let objective = |v: &[f32]| -> Result<f64> {
            let p = probabilities(v);
            let mut sum = 0.;
            for r in 0..ROOTS {
                sum += p[r] * op.transformed(&op.packets[0][0], r as u8, 1)?[0] * 0.25;
            }
            Ok(sum)
        };
        for root in [0, 1, 17] {
            let mut plus = logits.clone();
            let mut minus = logits.clone();
            plus[root] += 0.001;
            minus[root] -= 0.001;
            let fd = (objective(&plus)? - objective(&minus)?) / f64::from(plus[root] - minus[root]);
            assert!(
                (fd - f64::from(left[root])).abs() < 2e-7,
                "root {root}: {fd} {}",
                left[root]
            );
        }
        Ok(())
    }

    #[test]
    fn composition_source_reload_binds_policy_and_exact_parameters() -> Result<()> {
        let model = CompositionWeights::new()?;
        let trace = trace()?;
        let root = std::env::temp_dir().join(format!(
            "uor-composition-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        model.save(&root)?;
        let loaded = CompositionWeights::load(&root)?;
        for (name, var) in model.parameters() {
            assert_eq!(
                var.flatten_all()?.to_vec1::<f32>()?,
                loaded.parameters()[name].flatten_all()?.to_vec1::<f32>()?
            );
        }
        assert_eq!(model.compose_trace(&trace)?, loaded.compose_trace(&trace)?);
        let metadata_path = root.join(SOURCE_FILES[0]);
        let original = fs::read(&metadata_path)?;
        let mut metadata: Metadata = serde_json::from_slice(&original)?;
        metadata.config.surrogate.push_str("-changed");
        fs::write(&metadata_path, serde_json::to_vec(&metadata)?)?;
        assert!(CompositionWeights::load(&root).is_err());
        fs::write(&metadata_path, original)?;
        let parameter_path = root.join(SOURCE_FILES[1]);
        let mut bytes = fs::read(&parameter_path)?;
        let at = bytes.len() - 1;
        bytes[at] ^= 1;
        fs::write(&parameter_path, bytes)?;
        assert!(CompositionWeights::load(&root).is_err());
        model.parameters()["gains"].set(&Tensor::full(2f32, TERMS, &Device::Cpu)?)?;
        assert!(model.hard_bank().is_err());
        model.project_gain_range()?;
        assert!(model.parameters()["gains"]
            .to_vec1::<f32>()?
            .iter()
            .all(|v| *v == 1.75));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn composition_frozen_values_are_connected_with_signed_geometric_adjoint() -> Result<()> {
        let weights = CompositionWeights::new()?;
        let trace = trace()?;
        let mut left = vec![0f32; TERMS * ROOTS];
        let mut right = left.clone();
        for row in left.chunks_exact_mut(ROOTS) {
            row[3] = 1.;
        }
        for row in right.chunks_exact_mut(ROOTS) {
            row[5] = 1.;
        }
        weights.parameters()["left_logits"].set(&Tensor::from_vec(
            left,
            (TERMS, ROOTS),
            &Device::Cpu,
        )?)?;
        weights.parameters()["right_logits"].set(&Tensor::from_vec(
            right,
            (TERMS, ROOTS),
            &Device::Cpu,
        )?)?;
        let mut gains = vec![0f32; TERMS];
        gains[0] = -0.5;
        weights.parameters()["gains"].set(&Tensor::from_vec(gains, TERMS, &Device::Cpu)?)?;
        let raw = trace
            .values_q16
            .iter()
            .map(|&v| (f64::from(v) / 65536.) as f32)
            .collect::<Vec<_>>();
        let input = Var::from_vec(raw.clone(), (1, 2, 1, 16), &Device::Cpu)?;
        let out = weights.forward_frozen_values(&input, &trace)?;
        assert_eq!(
            out.flatten_all()?.to_vec1::<f32>()?,
            weights.forward(&trace)?.flatten_all()?.to_vec1::<f32>()?
        );
        let g = Tensor::from_vec(vec![1f32, 2., 3., 4.], 4, &Device::Cpu)?;
        let gradient = (out.flatten_all()?.narrow(0, 0, 4)? * &g)?
            .sum_all()?
            .backward()?;
        let dx = gradient
            .get(input.as_tensor())
            .ok_or_else(|| invalid("frozen value input disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        // For -1/2 * i*x*(-j), transpose is -1/2 * (-i)*g*j.
        let expected = compose_hamilton(
            compose_hamilton([0., -1., 0., 0.], [1., 2., 3., 4.]),
            [0., 0., 1., 0.],
        )
        .map(|x| (x * -0.5) as f32);
        assert_eq!(&dx[..4], &expected);
        assert!(dx[4..].iter().all(|x| *x == 0.));
        for var in weights.parameters().values() {
            assert!(gradient.get(var.as_tensor()).is_none());
        }
        let mut bad = raw;
        bad[0] += 0.25;
        assert!(weights
            .forward_frozen_values(&Tensor::from_vec(bad, (1, 2, 1, 16), &Device::Cpu)?, &trace)
            .is_err());
        Ok(())
    }
}
