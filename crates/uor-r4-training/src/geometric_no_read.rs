//! Offline hard-q4 NoRead learner. Shadow variables are NAT coefficients on
//! the fixed quarter-nat grid; the chosen STE has slope one, not one quarter.
//! Features are frozen actual native token/state/category/old-held codes.
//! No donor hidden vectors or source/answer labels enter this operator.
use crate::{invalid, sha256_bytes, Result};
use candle_core::{DType, Device, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype, SafeTensors};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use uor_r4_integer::geometric_no_read::{
    canonical_basis_q25, pack_coefficients, NativeGeometricNoRead, NoReadConfig, POLICY,
};
use uor_r4_integer::{geometric_potential::AddressLane, h4_tables::H4Code};
pub const SCHEMA: &str = "uor-r4.geometric-no-read-offline/1";
pub const SURROGATE:&str="nat-shadow;hard-round-ties-away-quarter-grid;identity-first-order-STE;post-update-project[-1.75,1.75];frozen-typed-inputs/1";
pub const SOURCE_FILES: [&str; 2] = ["metadata.json", "no-read-parameters.safetensors"];
#[derive(Clone, Copy)]
pub struct NoReadBatch<'a> {
    pub ids: &'a [u32],
    pub batch: usize,
    pub time: usize,
    pub latent: &'a [H4Code],
    pub observed: &'a [AddressLane],
    pub held: &'a [H4Code],
    pub span_valid: &'a [bool],
}
/// One row per output head; flatten bias, token[V], latent[M,4], category[M,33],
/// held[M,4], valid[M]. ALL M=H*L retained lanes feed every output head.
pub struct NoReadWeights {
    config: NoReadConfig,
    parameters: BTreeMap<String, Var>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    schema: String,
    config: NoReadConfig,
    policy: String,
    surrogate: String,
    parameter_bytes: usize,
    parameter_sha256: String,
    packed_sha256: String,
    basis_sha256: String,
}
pub fn basis_bytes() -> Vec<u8> {
    canonical_basis_q25()
        .into_iter()
        .flatten()
        .flat_map(i32::to_le_bytes)
        .collect()
}
impl NoReadWeights {
    pub fn new(vocabulary: usize, heads: usize, latent_lanes_per_head: usize) -> Result<Self> {
        let config = NoReadConfig {
            vocabulary,
            heads,
            latent_lanes_per_head,
        };
        config.validate().map_err(|e| invalid(e.to_string()))?;
        let mut parameters = BTreeMap::new();
        parameters.insert(
            "coefficients".into(),
            Var::zeros(
                (heads, config.coefficients_per_head()),
                DType::F32,
                &Device::Cpu,
            )?,
        );
        Ok(Self { config, parameters })
    }
    pub fn config(&self) -> &NoReadConfig {
        &self.config
    }
    pub fn parameters(&self) -> &BTreeMap<String, Var> {
        &self.parameters
    }
    fn coefficients(&self) -> Result<&Tensor> {
        self.parameters
            .get("coefficients")
            .map(Var::as_tensor)
            .ok_or_else(|| invalid("NoRead coefficients missing"))
    }
    fn values(&self) -> Result<Vec<f32>> {
        let p = self.coefficients()?;
        if p.dims() != [self.config.heads, self.config.coefficients_per_head()]
            || p.dtype() != DType::F32
            || !p.device().is_cpu()
        {
            return Err(invalid("NoRead coefficient shape/device differs"));
        }
        let v = p.flatten_all()?.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite() || x.abs() > 1.75) {
            return Err(invalid("NoRead shadows must be finite nat coefficients within [-1.75,1.75]; project after updates"));
        }
        Ok(v)
    }
    pub fn packed_coefficients(&self) -> Result<Vec<u8>> {
        let q = self
            .values()?
            .into_iter()
            .map(|x| (x * 4.).round() as i8)
            .collect::<Vec<_>>();
        pack_coefficients(&q).map_err(|e| invalid(e.to_string()))
    }
    pub fn native(&self) -> Result<NativeGeometricNoRead> {
        NativeGeometricNoRead::new(self.config, &self.packed_coefficients()?)
            .map_err(|e| invalid(e.to_string()))
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        let p = self.coefficients()?;
        let values = p.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite NoRead optimizer shadow"));
        }
        let values = values
            .into_iter()
            .map(|x| x.clamp(-1.75, 1.75))
            .collect::<Vec<_>>();
        self.parameters["coefficients"].set(&Tensor::from_vec(values, p.shape(), &Device::Cpu)?)?;
        Ok(())
    }
    pub fn forward(&self, input: NoReadBatch<'_>) -> Result<Tensor> {
        let values = self.values()?;
        let rows = input
            .batch
            .checked_mul(input.time)
            .ok_or_else(|| invalid("NoRead batch overflow"))?;
        let lanes = self.config.lanes();
        let count = rows
            .checked_mul(lanes)
            .ok_or_else(|| invalid("NoRead feature overflow"))?;
        if rows == 0
            || input.ids.len() != rows
            || input.latent.len() != count
            || input.observed.len() != count
            || input.held.len() != count
            || input.span_valid.len() != rows
            || input
                .ids
                .iter()
                .any(|&id| id as usize >= self.config.vocabulary)
        {
            return Err(invalid("NoRead actual feature shape/token differs"));
        }
        let width = self.config.coefficients_per_head();
        let feature_count = rows
            .checked_mul(width)
            .ok_or_else(|| invalid("NoRead feature matrix overflow"))?;
        let mut features = vec![0f32; feature_count];
        let basis = canonical_basis_q25();
        for row in 0..rows {
            let f = &mut features[row * width..(row + 1) * width];
            f[0] = 1.;
            f[1 + input.ids[row] as usize] = 1.;
            let mut at = 1 + self.config.vocabulary;
            for root in &input.latent[row * lanes..(row + 1) * lanes] {
                for &coordinate in &basis[usize::from(root.index())] {
                    f[at] = coordinate as f32 / 33554432.;
                    at += 1;
                }
            }
            for address in &input.observed[row * lanes..(row + 1) * lanes] {
                let category = if address.present() {
                    usize::from(address.radius_bin()) + 1
                } else {
                    0
                };
                f[at + category] = 1.;
                at += 33;
            }
            for root in &input.held[row * lanes..(row + 1) * lanes] {
                for &coordinate in &basis[usize::from(root.index())] {
                    f[at] = if input.span_valid[row] {
                        coordinate as f32 / 33554432.
                    } else {
                        0.
                    };
                    at += 1;
                }
            }
            for item in &mut f[at..] {
                *item = if input.span_valid[row] { 1. } else { 0. };
            }
        }
        let shadow = self.coefficients()?;
        let hard = Tensor::from_vec(
            values
                .into_iter()
                .map(|x| (x * 4.).round() * 0.25)
                .collect::<Vec<_>>(),
            shadow.shape(),
            &Device::Cpu,
        )?;
        // Forward is exactly the chosen q4 coefficient; derivative is explicitly the
        // identity shadow bridge. A hard-forward finite difference is not its test.
        let coefficient = (&hard + (shadow - shadow.detach())?)?;
        Ok(Tensor::from_vec(features, (rows, width), &Device::Cpu)?
            .matmul(&coefficient.t()?)?
            .reshape((input.batch, input.time, self.config.heads))?
            .transpose(1, 2)?
            .contiguous()?)
    }
    pub fn save(&self, directory: &Path) -> Result<()> {
        let values = self.values()?;
        let bytes = values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let shape = vec![self.config.heads, self.config.coefficients_per_head()];
        let view =
            TensorView::new(Dtype::F32, shape, &bytes).map_err(|e| invalid(e.to_string()))?;
        let parameter = safetensors::serialize([("coefficients", view)], None)
            .map_err(|e| invalid(e.to_string()))?;
        let metadata = Metadata {
            schema: SCHEMA.into(),
            config: self.config,
            policy: POLICY.into(),
            surrogate: SURROGATE.into(),
            parameter_bytes: parameter.len(),
            parameter_sha256: sha256_bytes(&parameter),
            packed_sha256: sha256_bytes(&self.packed_coefficients()?),
            basis_sha256: sha256_bytes(&basis_bytes()),
        };
        fs::create_dir(directory)?;
        fs::write(directory.join(SOURCE_FILES[1]), parameter)?;
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
            return Err(invalid("NoRead source file set differs"));
        }
        let metadata: Metadata =
            serde_json::from_slice(&fs::read(directory.join(SOURCE_FILES[0]))?)?;
        if metadata.schema != SCHEMA
            || metadata.policy != POLICY
            || metadata.surrogate != SURROGATE
            || metadata.basis_sha256 != sha256_bytes(&basis_bytes())
        {
            return Err(invalid("NoRead source policy/basis differs"));
        }
        metadata
            .config
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        let parameter = fs::read(directory.join(SOURCE_FILES[1]))?;
        if parameter.len() != metadata.parameter_bytes
            || sha256_bytes(&parameter) != metadata.parameter_sha256
        {
            return Err(invalid("NoRead source parameters differ"));
        }
        let tensors = SafeTensors::deserialize(&parameter).map_err(|e| invalid(e.to_string()))?;
        if tensors.names() != ["coefficients"] {
            return Err(invalid("NoRead source parameter inventory differs"));
        }
        let view = tensors
            .tensor("coefficients")
            .map_err(|e| invalid(e.to_string()))?;
        if view.dtype() != Dtype::F32
            || view.shape()
                != [
                    metadata.config.heads,
                    metadata.config.coefficients_per_head(),
                ]
        {
            return Err(invalid("NoRead source parameter shape differs"));
        }
        let weights = Self::new(
            metadata.config.vocabulary,
            metadata.config.heads,
            metadata.config.latent_lanes_per_head,
        )?;
        let values = view
            .data()
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        weights.parameters["coefficients"].set(&Tensor::from_vec(
            values,
            view.shape(),
            &Device::Cpu,
        )?)?;
        if sha256_bytes(&weights.packed_coefficients()?) != metadata.packed_sha256 {
            return Err(invalid("NoRead source packed coefficient identity differs"));
        }
        Ok(weights)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    #[test]
    fn no_read_basis_matches_existing_canonical_f32_bits() {
        let actual = uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots();
        for (root, row) in canonical_basis_q25().iter().enumerate() {
            for (i, &q) in row.iter().enumerate() {
                assert_eq!(
                    (q as f32 / 33554432.).to_bits(),
                    (actual[root].to_array()[i] as f32).to_bits()
                );
            }
        }
    }
    #[test]
    fn no_read_hard_forward_and_answer_gradient_reach_coefficients() -> Result<()> {
        let weights = NoReadWeights::new(3, 1, 1)?;
        let mut shadow = vec![0f32; weights.config.coefficient_count()];
        shadow[0] = 0.12;
        weights.parameters["coefficients"].set(&Tensor::from_vec(
            shadow,
            (1, weights.config.coefficients_per_head()),
            &Device::Cpu,
        )?)?;
        let ids = [2];
        let roots = [H4Code::try_from(1).unwrap()];
        let codes = [AddressLane::new(1, 0, false).unwrap()];
        let null = weights.forward(NoReadBatch {
            ids: &ids,
            batch: 1,
            time: 1,
            latent: &roots,
            observed: &codes,
            held: &roots,
            span_valid: &[false],
        })?;
        assert_eq!(null.to_vec3::<f32>()?, vec![vec![vec![0.]]]);
        let null = null.reshape((1, 1))?;
        let scores = Tensor::cat(
            &[&null, &Tensor::zeros((1, 1), DType::F32, &Device::Cpu)?],
            1,
        )?;
        let mixed = candle_nn::ops::softmax(&scores, 1)?.matmul(&Tensor::from_vec(
            vec![0f32, 2.],
            (2, 1),
            &Device::Cpu,
        )?)?;
        let logits = Tensor::cat(
            &[&Tensor::zeros((1, 1), DType::F32, &Device::Cpu)?, &mixed],
            1,
        )?;
        let target = Tensor::from_vec(vec![1u32], 1, &Device::Cpu)?;
        let loss = candle_nn::loss::cross_entropy(&logits, &target)?;
        let gradient = loss.backward()?;
        let g = gradient
            .get(weights.parameters["coefficients"].as_tensor())
            .ok_or_else(|| invalid("NoRead answer gradient missing"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(g[0] > 0. && g[3] > 0. && g[4] > 0.);
        assert_eq!(g[41], 0.);
        Ok(())
    }
    #[test]
    fn no_read_saved_policy_preserves_shadows_and_packed_bits() -> Result<()> {
        static ID: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "no-read-source-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        let weights = NoReadWeights::new(3, 2, 1)?;
        let values = (0..weights.config.coefficient_count())
            .map(|i| ((i % 15) as f32 - 7.) * 0.24)
            .collect::<Vec<_>>();
        weights.parameters["coefficients"].set(&Tensor::from_vec(
            values.clone(),
            (2, weights.config.coefficients_per_head()),
            &Device::Cpu,
        )?)?;
        weights.save(&dir)?;
        let loaded = NoReadWeights::load(&dir)?;
        assert_eq!(loaded.values()?, values);
        assert_eq!(
            loaded.packed_coefficients()?,
            weights.packed_coefficients()?
        );
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("metadata.json"))?)?;
        metadata["surrogate"] = "unbound-scale".into();
        fs::write(dir.join("metadata.json"), serde_json::to_vec(&metadata)?)?;
        assert!(NoReadWeights::load(&dir).is_err());
        fs::remove_dir_all(dir)?;
        Ok(())
    }
}
