//! Standalone strict-q4 paired-H4 potential source and first-order answer bridge.
//!
//! The immutable StackModel parent and its original coefficients are lineage,
//! not the live coefficients. Hard scores use freshly packed current shadows,
//! exact signed relative codes and integer Q25-derived tables. Backward uses the
//! fixed coordinate/cell features with identity nat-shadow and rounding STEs;
//! it is not the derivative of the discrete coefficient quantizer. Typed input
//! codes are constants: this module does not train state placement or selectors.
use crate::geometric_address::GeometricAddressConfig;
use crate::geometric_potential_native::CoefficientIdentity;
use crate::{invalid, sha256_bytes, Result};
use candle_core::{CpuStorage, CustomOp1, DType, Device, Layout, Shape, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    sync::Arc,
};
use uor_r4_integer::{
    geometric_potential::AddressLane,
    geometric_potential_q4::{
        canonical_basis_q25, pack_coefficients, NativePotentialQ4, PotentialQ4Config,
        FAMILY_COUNTS, FAMILY_NAMES, POLICY,
    },
    h4_tables::{H4Code, HistoricalH4Tables},
};

pub const SCHEMA: &str = "uor-r4.geometric-potential-q4-offline/1";
pub const SURROGATE: &str = "nat-shadow-quarter-grid-identity-STE;fresh-current-packed-Q25-native-Q24-hard-score;fixed-typed-code-coefficient-adjoint;rounding-STE;causal-only;no-state-gradient/1";
pub const SOURCE_FILES: [&str; 2] = ["metadata.json", "potential-q4-parameters.safetensors"];
const ALGEBRA: &[u8] = include_bytes!("../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin");
const SIDECAR: &str = "geometric-address.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PotentialParentIdentity {
    pub model_sha256: String,
    pub config_sha256: String,
    pub address_sidecar_sha256: String,
    pub tokenizer_sha256: String,
    pub address: GeometricAddressConfig,
    pub coefficients: Vec<CoefficientIdentity>,
}

pub struct PotentialQ4Weights {
    config: PotentialQ4Config,
    parent: PotentialParentIdentity,
    parameters: BTreeMap<String, Var>,
    algebra: Arc<HistoricalH4Tables>,
}
pub struct PotentialQ4Output {
    /// [B,H,T,T]. Only causal entries are computed; future entries are zero and
    /// must be masked by the actual reader before normalization.
    pub scores: Tensor,
    pub scores_q24: Vec<i64>,
    /// Actual typed endpoints [B,T,H,L], retained for exact native comparison.
    pub content_codes: Vec<AddressLane>,
    pub context_codes: Vec<AddressLane>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    schema: String,
    policy: String,
    surrogate: String,
    config: PotentialQ4Config,
    parent: PotentialParentIdentity,
    parameter_bytes: usize,
    parameter_sha256: String,
    packed_sha256: String,
    basis_sha256: String,
    algebra_sha256: String,
}
pub fn basis_bytes() -> Vec<u8> {
    canonical_basis_q25()
        .iter()
        .flatten()
        .flat_map(|x| x.to_le_bytes())
        .collect()
}
fn admitted_algebra() -> Result<Arc<HistoricalH4Tables>> {
    Ok(Arc::new(
        HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?,
    ))
}
fn source_name(name: &str) -> String {
    format!("layers.02.read.address.{name}")
}
fn floats(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}
fn read_base(
    base: &Path,
    tokenizer: &[u8],
) -> Result<(PotentialParentIdentity, BTreeMap<String, Vec<f32>>)> {
    if tokenizer.is_empty() {
        return Err(invalid("q4 potential needs explicit tokenizer identity"));
    }
    let model = fs::read(base.join("model.safetensors"))?;
    let config = fs::read(base.join("config.json"))?;
    let sidecar = fs::read(base.join(SIDECAR))?;
    let address = crate::geometric_stack::StackModel::saved_geometric_address(base)?
        .ok_or_else(|| invalid("q4 potential parent has no saved geometric address"))?;
    let c = PotentialQ4Config {
        heads: address.heads,
        lanes_per_head: address.lanes_per_head,
    };
    c.validate().map_err(|e| invalid(e.to_string()))?;
    let tensor = SafeTensors::deserialize(&model).map_err(|e| invalid(e.to_string()))?;
    let mut values = BTreeMap::new();
    let mut coefficients = Vec::new();
    for (name, shape) in c.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
        let full = source_name(&name);
        let view = tensor.tensor(&full).map_err(|e| invalid(e.to_string()))?;
        if view.dtype() != SafeDtype::F32 || view.shape() != shape {
            return Err(invalid("q4 parent coefficient shape/type differs"));
        }
        let v = floats(view.data());
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite parent potential"));
        }
        coefficients.push(CoefficientIdentity {
            name: full,
            shape,
            bytes: view.data().len(),
            sha256: sha256_bytes(view.data()),
        });
        values.insert(name, v);
    }
    if model != fs::read(base.join("model.safetensors"))?
        || config != fs::read(base.join("config.json"))?
        || sidecar != fs::read(base.join(SIDECAR))?
    {
        return Err(invalid("q4 parent changed during admission"));
    }
    Ok((
        PotentialParentIdentity {
            model_sha256: sha256_bytes(&model),
            config_sha256: sha256_bytes(&config),
            address_sidecar_sha256: sha256_bytes(&sidecar),
            tokenizer_sha256: sha256_bytes(tokenizer),
            address,
            coefficients,
        },
        values,
    ))
}
impl PotentialParentIdentity {
    pub fn from_base(base: &Path, tokenizer: &[u8]) -> Result<Self> {
        Ok(read_base(base, tokenizer)?.0)
    }
    fn validate(&self, config: PotentialQ4Config) -> Result<()> {
        self.address
            .validate(config.heads * config.lanes_per_head * 4, config.heads)?;
        let hashes = [
            &self.model_sha256,
            &self.config_sha256,
            &self.address_sidecar_sha256,
            &self.tokenizer_sha256,
        ];
        if hashes
            .iter()
            .any(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(invalid("parent hash format differs"));
        }
        let shapes = config
            .coefficient_shapes()
            .map_err(|e| invalid(e.to_string()))?;
        if self.coefficients.len() != shapes.len() {
            return Err(invalid("parent coefficient inventory differs"));
        }
        for (identity, (name, shape)) in self.coefficients.iter().zip(shapes) {
            if identity.name != source_name(&name)
                || identity.shape != shape
                || identity.bytes != shape.iter().product::<usize>() * 4
                || identity.sha256.len() != 64
                || !identity.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(invalid("parent coefficient identity differs"));
            }
        }
        Ok(())
    }
}
impl PotentialQ4Weights {
    pub fn from_base(base: &Path, tokenizer: &[u8]) -> Result<Self> {
        let (parent, mut values) = read_base(base, tokenizer)?;
        for v in values.values_mut() {
            for x in v {
                *x = x.clamp(-1.75, 1.75);
            }
        }
        Self::from_values(parent, values)
    }
    fn from_values(
        parent: PotentialParentIdentity,
        mut values: BTreeMap<String, Vec<f32>>,
    ) -> Result<Self> {
        let config = PotentialQ4Config {
            heads: parent.address.heads,
            lanes_per_head: parent.address.lanes_per_head,
        };
        config.validate().map_err(|e| invalid(e.to_string()))?;
        parent.validate(config)?;
        let mut parameters = BTreeMap::new();
        for (name, shape) in config
            .coefficient_shapes()
            .map_err(|e| invalid(e.to_string()))?
        {
            let data = values
                .remove(&name)
                .ok_or_else(|| invalid("potential source family missing"))?;
            if data.len() != shape.iter().product::<usize>()
                || data.iter().any(|v| !v.is_finite() || v.abs() > 1.75)
            {
                return Err(invalid(
                    "potential shadows outside fixed source range/shape",
                ));
            }
            parameters.insert(name, Var::from_vec(data, shape, &Device::Cpu)?);
        }
        if !values.is_empty() {
            return Err(invalid("extra potential source families"));
        }
        Ok(Self {
            config,
            parent,
            parameters,
            algebra: admitted_algebra()?,
        })
    }
    pub fn config(&self) -> &PotentialQ4Config {
        &self.config
    }
    pub fn parent(&self) -> &PotentialParentIdentity {
        &self.parent
    }
    pub fn parameters(&self) -> &BTreeMap<String, Var> {
        &self.parameters
    }
    pub fn validate_parent(&self, base: &Path, tokenizer: &[u8]) -> Result<()> {
        if self.parent != PotentialParentIdentity::from_base(base, tokenizer)? {
            return Err(invalid("q4 potential immutable parent differs"));
        }
        Ok(())
    }
    fn values(&self) -> Result<Vec<f32>> {
        let mut out = Vec::new();
        let shapes = self
            .config
            .coefficient_shapes()
            .map_err(|e| invalid(e.to_string()))?;
        if self.parameters.len() != shapes.len() {
            return Err(invalid("potential family inventory differs"));
        }
        for (name, shape) in shapes {
            let v = self
                .parameters
                .get(&name)
                .ok_or_else(|| invalid("potential family missing"))?;
            if v.dtype() != DType::F32 || !v.device().is_cpu() || v.dims() != shape {
                return Err(invalid("potential source tensor shape/device differs"));
            }
            out.extend(v.flatten_all()?.to_vec1::<f32>()?);
        }
        if out.iter().any(|v| !v.is_finite() || v.abs() > 1.75) {
            return Err(invalid(
                "potential shadows must be finite within[-1.75,1.75]",
            ));
        }
        Ok(out)
    }
    fn tensor(&self) -> Result<Tensor> {
        let tensors = FAMILY_NAMES
            .iter()
            .map(|name| {
                self.parameters
                    .get(*name)
                    .ok_or_else(|| invalid("potential family absent"))?
                    .flatten_all()
                    .map_err(Into::into)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Tensor::cat(&tensors.iter().collect::<Vec<_>>(), 0)?)
    }
    pub fn packed_coefficients(&self) -> Result<Vec<u8>> {
        pack_coefficients(
            &self
                .values()?
                .into_iter()
                .map(|v| (v * 4.).round() as i8)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        let mut pending = Vec::new();
        for v in self.parameters.values() {
            let values = v.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite potential optimizer shadow"));
            }
            pending.push((
                v,
                Tensor::from_vec(
                    values
                        .into_iter()
                        .map(|x| x.clamp(-1.75, 1.75))
                        .collect::<Vec<_>>(),
                    v.shape(),
                    &Device::Cpu,
                )?,
            ));
        }
        for (v, t) in pending {
            v.set(&t)?;
        }
        Ok(())
    }
    pub fn forward_codes(
        &self,
        batch: usize,
        time: usize,
        content: &[AddressLane],
        context: &[AddressLane],
    ) -> Result<PotentialQ4Output> {
        let count = batch
            .checked_mul(time)
            .and_then(|x| x.checked_mul(self.config.heads))
            .and_then(|x| x.checked_mul(self.config.lanes_per_head))
            .ok_or_else(|| invalid("potential input shape overflow"))?;
        if batch == 0
            || batch > 32
            || time == 0
            || time > 128
            || self.config.heads > 2
            || self.config.lanes_per_head > 4
            || content.len() != count
            || context.len() != count
        {
            return Err(invalid(
                "q4 potential bridge requiresB1..32/T1..128/H1..2/L1..4 and exact typed inputs",
            ));
        }
        let values = self.values()?;
        let native = NativePotentialQ4::new(self.config, &self.packed_coefficients()?)
            .map_err(|e| invalid(e.to_string()))?;
        let (h, l) = (self.config.heads, self.config.lanes_per_head);
        let mut scores = vec![0i64; batch * h * time * time];
        for b in 0..batch {
            for head in 0..h {
                for q in 0..time {
                    for k in 0..=q {
                        let qi = ((b * time + q) * h + head) * l;
                        let ki = ((b * time + k) * h + head) * l;
                        scores[((b * h + head) * time + q) * time + k] = native
                            .score(
                                head,
                                &content[qi..qi + l],
                                &content[ki..ki + l],
                                &context[qi..qi + l],
                                &context[ki..ki + l],
                                &self.algebra,
                            )
                            .map_err(|e| invalid(e.to_string()))?;
                    }
                }
            }
        }
        let op = PotentialOp {
            config: self.config,
            batch,
            time,
            content: content.to_vec(),
            context: context.to_vec(),
            algebra: self.algebra.clone(),
            hard: scores.clone(),
            shadow: values,
        };
        Ok(PotentialQ4Output {
            scores: self.tensor()?.apply_op1(op)?,
            scores_q24: scores,
            content_codes: content.to_vec(),
            context_codes: context.to_vec(),
        })
    }
    pub(crate) fn source_bytes(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        self.values()?;
        let mut buffers = BTreeMap::new();
        for (name, var) in &self.parameters {
            buffers.insert(
                name.clone(),
                var.flatten_all()?
                    .to_vec1::<f32>()?
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect::<Vec<_>>(),
            );
        }
        let views = self
            .parameters
            .iter()
            .map(|(name, var)| {
                Ok((
                    name.as_str(),
                    TensorView::new(SafeDtype::F32, var.dims().to_vec(), &buffers[name])
                        .map_err(|e| invalid(e.to_string()))?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let parameters = safetensors::serialize(views, None).map_err(|e| invalid(e.to_string()))?;
        let metadata = Metadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            surrogate: SURROGATE.into(),
            config: self.config,
            parent: self.parent.clone(),
            parameter_bytes: parameters.len(),
            parameter_sha256: sha256_bytes(&parameters),
            packed_sha256: sha256_bytes(&self.packed_coefficients()?),
            basis_sha256: sha256_bytes(&basis_bytes()),
            algebra_sha256: sha256_bytes(ALGEBRA),
        };
        Ok((serde_json::to_vec_pretty(&metadata)?, parameters))
    }
    pub(crate) fn from_source_bytes(metadata: &[u8], parameters: &[u8]) -> Result<Self> {
        let meta: Metadata = serde_json::from_slice(metadata)?;
        meta.config.validate().map_err(|e| invalid(e.to_string()))?;
        meta.parent.validate(meta.config)?;
        if meta.schema != SCHEMA
            || meta.policy != POLICY
            || meta.surrogate != SURROGATE
            || meta.basis_sha256 != sha256_bytes(&basis_bytes())
            || meta.algebra_sha256 != sha256_bytes(ALGEBRA)
            || meta.parameter_bytes != parameters.len()
            || meta.parameter_sha256 != sha256_bytes(parameters)
        {
            return Err(invalid("q4 potential source policy/binding differs"));
        }
        let tensors = SafeTensors::deserialize(parameters).map_err(|e| invalid(e.to_string()))?;
        if tensors
            .names()
            .into_iter()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>()
            != FAMILY_NAMES.iter().map(|s| s.to_string()).collect()
        {
            return Err(invalid("q4 potential saved family inventory differs"));
        }
        let mut values = BTreeMap::new();
        for (name, shape) in meta
            .config
            .coefficient_shapes()
            .map_err(|e| invalid(e.to_string()))?
        {
            let view = tensors.tensor(&name).map_err(|e| invalid(e.to_string()))?;
            if view.dtype() != SafeDtype::F32 || view.shape() != shape {
                return Err(invalid("q4 potential saved shape/type differs"));
            }
            values.insert(name, floats(view.data()));
        }
        let weights = Self::from_values(meta.parent, values)?;
        if weights.config != meta.config
            || sha256_bytes(&weights.packed_coefficients()?) != meta.packed_sha256
        {
            return Err(invalid("q4 potential saved packed/config differs"));
        }
        Ok(weights)
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        let (metadata, parameters) = self.source_bytes()?;
        fs::create_dir(directory)?;
        fs::write(directory.join(SOURCE_FILES[0]), metadata)?;
        fs::write(directory.join(SOURCE_FILES[1]), parameters)?;
        Ok(())
    }
    pub fn load(directory: &Path) -> Result<Self> {
        let entries = fs::read_dir(directory)?
            .map(|e| {
                e.and_then(|e| {
                    if e.file_type()?.is_file() {
                        Ok(e.file_name().to_string_lossy().into_owned())
                    } else {
                        Err(std::io::Error::other("nonfile in q4 potential source"))
                    }
                })
            })
            .collect::<std::io::Result<BTreeSet<_>>>()?;
        if entries != SOURCE_FILES.into_iter().map(str::to_string).collect() {
            return Err(invalid("q4 potential source file set differs"));
        }
        Self::from_source_bytes(
            &fs::read(directory.join(SOURCE_FILES[0]))?,
            &fs::read(directory.join(SOURCE_FILES[1]))?,
        )
    }
}

struct PotentialOp {
    config: PotentialQ4Config,
    batch: usize,
    time: usize,
    content: Vec<AddressLane>,
    context: Vec<AddressLane>,
    algebra: Arc<HistoricalH4Tables>,
    hard: Vec<i64>,
    shadow: Vec<f32>,
}
impl CustomOp1 for PotentialOp {
    fn name(&self) -> &'static str {
        "geometric-potential-q4-typed-score"
    }
    fn cpu_fwd(
        &self,
        storage: &CpuStorage,
        layout: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        let (start, end) = layout.contiguous_offsets().ok_or_else(|| {
            candle_core::Error::Msg("potential parameter layout must be contiguous".into())
        })?;
        let values = &storage.as_slice::<f32>()?[start..end];
        if values.len() != self.shadow.len()
            || values
                .iter()
                .zip(&self.shadow)
                .any(|(a, b)| a.to_bits() != b.to_bits())
        {
            candle_core::bail!("potential source changed during hard forward");
        }
        Ok((
            CpuStorage::F32(
                self.hard
                    .iter()
                    .map(|&v| (v as f64 / 16777216.) as f32)
                    .collect(),
            ),
            Shape::from((self.batch, self.config.heads, self.time, self.time)),
        ))
    }
    fn bwd(
        &self,
        input: &Tensor,
        _output: &Tensor,
        gradient: &Tensor,
    ) -> candle_core::Result<Option<Tensor>> {
        let upstream = gradient.flatten_all()?.to_vec1::<f32>()?;
        if upstream.len() != self.hard.len() || upstream.iter().any(|g| !g.is_finite()) {
            candle_core::bail!("potential upstream shape/finiteness differs");
        }
        let (h, l) = (self.config.heads, self.config.lanes_per_head);
        let mut offsets = [0usize; 7];
        for i in 1..7 {
            offsets[i] = offsets[i - 1] + h * l * FAMILY_COUNTS[i - 1];
        }
        let mut dw = vec![0f64; self.shadow.len()];
        let basis = canonical_basis_q25();
        let code =
            |id: u8| H4Code::try_from(id).map_err(|e| candle_core::Error::Msg(e.to_string()));
        for b in 0..self.batch {
            for head in 0..h {
                for q in 0..self.time {
                    for k in 0..=q {
                        let g =
                            f64::from(upstream[((b * h + head) * self.time + q) * self.time + k]);
                        if g == 0. {
                            continue;
                        }
                        for lane in 0..l {
                            let qi = ((b * self.time + q) * h + head) * l + lane;
                            let ki = ((b * self.time + k) * h + head) * l + lane;
                            let (cq, ck, rq, rk) = (
                                self.content[qi],
                                self.content[ki],
                                self.context[qi],
                                self.context[ki],
                            );
                            let at = |family: usize, index: usize| {
                                offsets[family] + (head * l + lane) * FAMILY_COUNTS[family] + index
                            };
                            dw[at(5, 2 * usize::from(cq.present()) + usize::from(ck.present()))] +=
                                g;
                            dw[at(6, 2 * usize::from(rq.present()) + usize::from(rk.present()))] +=
                                g;
                            let mut dc = [0f64; 4];
                            let mut dr = [0f64; 4];
                            if cq.present() && ck.present() {
                                let code = self.algebra.compose(
                                    self.algebra.inverse(code(cq.root())?),
                                    code(ck.root())?,
                                );
                                for i in 0..4 {
                                    dc[i] =
                                        f64::from(basis[usize::from(code.index())][i]) / 33554432.;
                                    dw[at(0, i)] += g * dc[i];
                                }
                                dw[at(
                                    3,
                                    usize::from(cq.radius_bin()) * 32
                                        + usize::from(ck.radius_bin()),
                                )] += g;
                            }
                            if rq.present() && rk.present() {
                                let code = self.algebra.compose(
                                    self.algebra.inverse(code(rq.root())?),
                                    code(rk.root())?,
                                );
                                for i in 0..4 {
                                    dr[i] =
                                        f64::from(basis[usize::from(code.index())][i]) / 33554432.;
                                    dw[at(1, i)] += g * dr[i];
                                }
                                dw[at(
                                    4,
                                    usize::from(rq.radius_bin()) * 32
                                        + usize::from(rk.radius_bin()),
                                )] += g;
                            }
                            if cq.present() && ck.present() && rq.present() && rk.present() {
                                for i in 0..4 {
                                    for j in 0..4 {
                                        dw[at(2, i * 4 + j)] += g * dc[i] * dr[j];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let dw = dw.into_iter().map(|x| x as f32).collect::<Vec<_>>();
        if dw.iter().any(|x| !x.is_finite()) {
            candle_core::bail!("nonfinite q4 potential coefficient adjoint");
        }
        Ok(Some(Tensor::from_vec(dw, input.shape(), input.device())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(heads: usize, lanes: usize) -> Result<PotentialQ4Weights> {
        let config = PotentialQ4Config {
            heads,
            lanes_per_head: lanes,
        };
        let mut values = BTreeMap::new();
        let mut coefficients = Vec::new();
        for (name, shape) in config
            .coefficient_shapes()
            .map_err(|e| invalid(e.to_string()))?
        {
            let n = shape.iter().product::<usize>();
            let data = vec![0f32; n];
            coefficients.push(CoefficientIdentity {
                name: source_name(&name),
                shape,
                bytes: n * 4,
                sha256: sha256_bytes(&vec![0; n * 4]),
            });
            values.insert(name, data);
        }
        PotentialQ4Weights::from_values(
            PotentialParentIdentity {
                model_sha256: "0".repeat(64),
                config_sha256: "1".repeat(64),
                address_sidecar_sha256: "2".repeat(64),
                tokenizer_sha256: "3".repeat(64),
                address: GeometricAddressConfig::new(heads * lanes * 4, heads)?,
                coefficients,
            },
            values,
        )
    }
    fn set(weights: &PotentialQ4Weights, name: &str, values: Vec<f32>) -> Result<()> {
        let v = &weights.parameters[name];
        v.set(&Tensor::from_vec(values, v.shape(), &Device::Cpu)?)?;
        Ok(())
    }
    #[test]
    fn potential_q4_live_native_scores_signed_features_and_presence_adjoint() -> Result<()> {
        let w = fixture(1, 1)?;
        let a = |root, bin, present| {
            AddressLane::new(root, bin, present).map_err(|e| invalid(e.to_string()))
        };
        let content = vec![a(0, 3, true)?, a(1, 4, true)?, a(1, 0, false)?];
        let context = vec![a(0, 6, true)?, a(1, 7, true)?, a(1, 0, false)?];
        let out = w.forward_codes(1, 3, &content, &context)?;
        assert!(out.scores_q24.iter().all(|s| *s == 0));
        // Query1 -> source0 has both relative roots -identity. Unary derivative
        // is -1, pair(0,0) derivative +1; selected radius/presence cells get one.
        let gradient = out
            .scores
            .narrow(2, 1, 1)?
            .narrow(3, 0, 1)?
            .sum_all()?
            .backward()?;
        let g = |name: &str| -> Result<Vec<f32>> {
            Ok(gradient
                .get(w.parameters[name].as_tensor())
                .ok_or_else(|| invalid("potential coefficient gradient missing"))?
                .flatten_all()?
                .to_vec1::<f32>()?)
        };
        assert_eq!(g("content_unary")?, vec![-1., 0., 0., 0.]);
        assert_eq!(g("context_unary")?, vec![-1., 0., 0., 0.]);
        assert_eq!(g("pair")?[0], 1.);
        assert_eq!(g("content_radius")?[4 * 32 + 3], 1.);
        assert_eq!(g("context_radius")?[7 * 32 + 6], 1.);
        assert_eq!(g("content_presence")?, vec![0., 0., 0., 1.]);
        set(&w, "content_unary", vec![0.13, 0., 0., 0.])?;
        let refreshed = w.forward_codes(1, 3, &content, &context)?;
        assert_eq!(refreshed.scores_q24[3], -(1i64 << 22));
        let native = NativePotentialQ4::new(*w.config(), &w.packed_coefficients()?)
            .map_err(|e| invalid(e.to_string()))?;
        for q in 0..3 {
            for k in 0..=q {
                assert_eq!(
                    refreshed.scores_q24[q * 3 + k],
                    native
                        .score(
                            0,
                            &content[q..q + 1],
                            &content[k..k + 1],
                            &context[q..q + 1],
                            &context[k..k + 1],
                            &w.algebra
                        )
                        .map_err(|e| invalid(e.to_string()))?
                );
            }
        }
        let absent = refreshed
            .scores
            .narrow(2, 2, 1)?
            .narrow(3, 0, 1)?
            .sum_all()?
            .backward()?;
        assert_eq!(
            absent
                .get(w.parameters["content_unary"].as_tensor())
                .ok_or_else(|| invalid("absent gradient missing"))?
                .abs()?
                .sum_all()?
                .to_scalar::<f32>()?,
            0.
        );
        assert_eq!(
            absent
                .get(w.parameters["content_presence"].as_tensor())
                .ok_or_else(|| invalid("presence gradient missing"))?
                .flatten_all()?
                .to_vec1::<f32>()?,
            vec![0., 1., 0., 0.]
        );
        assert!(w.forward_codes(1, 3, &content[..2], &context).is_err());
        Ok(())
    }
    #[test]
    fn potential_q4_actual_denominator_answer_credit_and_future_causality() -> Result<()> {
        let w = fixture(1, 1)?;
        let code = AddressLane::new(1, 16, true).map_err(|e| invalid(e.to_string()))?;
        let out = w.forward_codes(1, 2, &[code; 2], &[code; 2])?;
        let row = out.scores.narrow(2, 1, 1)?.reshape((1, 2))?;
        let logits = Tensor::cat(
            &[&Tensor::zeros((1, 1), DType::F32, &Device::Cpu)?, &row],
            1,
        )?;
        let mixed = candle_nn::ops::softmax(&logits, 1)?.matmul(&Tensor::from_vec(
            vec![0f32, 2., 0.],
            (3, 1),
            &Device::Cpu,
        )?)?;
        let answer = Tensor::cat(
            &[&Tensor::zeros((1, 1), DType::F32, &Device::Cpu)?, &mixed],
            1,
        )?;
        let loss = candle_nn::loss::cross_entropy(
            &answer,
            &Tensor::from_vec(vec![1u32], 1, &Device::Cpu)?,
        )?;
        let grad = loss.backward()?;
        let g = grad
            .get(w.parameters["content_presence"].as_tensor())
            .ok_or_else(|| invalid("answer misses potential"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(
            g[3].is_finite() && g[3] < 0.,
            "shared score increases nonnull answer value relative to null"
        );
        let future = out
            .scores
            .narrow(2, 0, 1)?
            .narrow(3, 1, 1)?
            .sum_all()?
            .backward()?;
        for v in w.parameters.values() {
            assert_eq!(
                future
                    .get(v.as_tensor())
                    .ok_or_else(|| invalid("future gradient missing"))?
                    .abs()?
                    .sum_all()?
                    .to_scalar::<f32>()?,
                0.
            );
        }
        let longer = w.forward_codes(1, 3, &[code; 3], &[code; 3])?;
        for q in 0..2 {
            for k in 0..=q {
                assert_eq!(out.scores_q24[q * 2 + k], longer.scores_q24[q * 3 + k]);
            }
        }
        Ok(())
    }
    #[test]
    fn potential_q4_source_preserves_shadows_parent_and_rejects_policy_changes() -> Result<()> {
        let w = fixture(2, 4)?;
        let v = &w.parameters["pair"];
        set(&w, "pair", vec![0.13; v.elem_count()])?;
        let (meta, param) = w.source_bytes()?;
        let loaded = PotentialQ4Weights::from_source_bytes(&meta, &param)?;
        assert_eq!(w.parent(), loaded.parent());
        assert_eq!(w.values()?, loaded.values()?);
        assert_eq!(w.packed_coefficients()?, loaded.packed_coefficients()?);
        let mut changed: serde_json::Value = serde_json::from_slice(&meta)?;
        changed["surrogate"] = serde_json::json!("unknown");
        assert!(
            PotentialQ4Weights::from_source_bytes(&serde_json::to_vec(&changed)?, &param).is_err()
        );
        let mut broken = param.clone();
        let last = broken.len() - 1;
        broken[last] ^= 1;
        assert!(PotentialQ4Weights::from_source_bytes(&meta, &broken).is_err());
        set(&w, "pair", vec![2.; v.elem_count()])?;
        assert!(w.packed_coefficients().is_err());
        w.project_shadow_range()?;
        assert!(w.parameters["pair"]
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|x| *x == 1.75));
        Ok(())
    }
}
