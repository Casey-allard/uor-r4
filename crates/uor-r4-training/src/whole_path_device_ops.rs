//! Linear-time detached CDF and segmented alias reductions. No dense one-hot
//! or triangular matrix, no host numerical fallback on CUDA.
use crate::{invalid, Result};
use candle_core::{CpuStorage, CustomOp1, DType, Layout, Shape, Tensor};

struct Cdf;
impl CustomOp1 for Cdf {
    fn name(&self) -> &'static str {
        "whole-path-linear-cdf"
    }
    fn cpu_fwd(&self, s: &CpuStorage, l: &Layout) -> candle_core::Result<(CpuStorage, Shape)> {
        let CpuStorage::F32(v) = s else {
            candle_core::bail!("CDF expects F32");
        };
        let Some((start, end)) = l.contiguous_offsets() else {
            candle_core::bail!("CDF requires contiguous scores");
        };
        let mut sum = 0f64;
        let mut out = Vec::with_capacity(end - start);
        for &p in &v[start..end] {
            sum += f64::from(p);
            out.push(sum);
        }
        if let Some(last) = out.last_mut() {
            *last = 1.;
        }
        Ok((CpuStorage::F64(out), l.shape().clone()))
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s: &candle_core::CudaStorage,
        l: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        use crate::native_geometric_cuda_kernels::{launch, Arg};
        let Some((start, end)) = l.contiguous_offsets() else {
            candle_core::bail!("CDF requires contiguous scores");
        };
        let d = &s.device;
        let input = s.as_cuda_slice::<f32>()?;
        let input = input
            .try_slice(start..end)
            .ok_or_else(|| candle_core::Error::Msg("CDF storage bounds".into()))?;
        let output = d.alloc_zeros::<f64>(end - start)?;
        launch(
            d,
            "whole_path_cdf",
            1,
            &[
                Arg::F(input),
                Arg::D(output.as_view()),
                Arg::N((end - start) as u32),
            ],
        )?;
        Ok((
            candle_core::CudaStorage::wrap_cuda_slice(output, d.clone()),
            l.shape().clone(),
        ))
    }
}
pub(super) fn cdf(probability: &Tensor) -> Result<Tensor> {
    if probability.dtype() != DType::F32
        || probability.dims().len() != 1
        || !(1..=4096).contains(&probability.elem_count())
    {
        return Err(invalid("linear CDF admission invalid"));
    }
    Ok(probability.detach().contiguous()?.apply_op1_no_bwd(&Cdf)?)
}

#[derive(Clone)]
struct Aliases {
    vocab: usize,
    ids: Vec<u32>,
    offsets: Vec<u32>,
    indices: Vec<u32>,
    sum: bool,
}
impl Aliases {
    fn new(vocab: usize, copy_ids: &[u32], sum: bool) -> Result<Self> {
        if !(1..=4096).contains(&vocab)
            || copy_ids.len() > 128
            || copy_ids.iter().any(|&id| id as usize >= vocab)
        {
            return Err(invalid("alias descriptor admission invalid"));
        }
        let ids = (0..vocab as u32)
            .chain(copy_ids.iter().copied())
            .collect::<Vec<_>>();
        let mut groups = vec![Vec::new(); vocab];
        for (i, &id) in ids.iter().enumerate() {
            groups[id as usize].push(i as u32);
        }
        let mut offsets = vec![0];
        let mut indices = Vec::new();
        for group in groups {
            indices.extend(group);
            offsets.push(indices.len() as u32);
        }
        Ok(Self {
            vocab,
            ids,
            offsets,
            indices,
            sum,
        })
    }
}
impl CustomOp1 for Aliases {
    fn name(&self) -> &'static str {
        "whole-path-linear-alias-pool"
    }
    fn cpu_fwd(&self, s: &CpuStorage, l: &Layout) -> candle_core::Result<(CpuStorage, Shape)> {
        let Some((start, end)) = l.contiguous_offsets() else {
            candle_core::bail!("aliases require contiguous scores");
        };
        if end - start != self.ids.len() {
            candle_core::bail!("alias descriptor/score size differs");
        }
        if self.sum {
            let CpuStorage::F64(v) = s else {
                candle_core::bail!("native alias sum expects F64 integers");
            };
            let mut out = Vec::new();
            for pair in self.offsets.windows(2) {
                let mut sum = 0.;
                for &i in &self.indices[pair[0] as usize..pair[1] as usize] {
                    sum += v[start + i as usize];
                }
                out.push(sum);
            }
            Ok((CpuStorage::F64(out), Shape::from(self.vocab)))
        } else {
            let CpuStorage::F32(v) = s else {
                candle_core::bail!("alias logsumexp expects F32");
            };
            let mut out = Vec::new();
            for pair in self.offsets.windows(2) {
                let indices = &self.indices[pair[0] as usize..pair[1] as usize];
                let max = indices
                    .iter()
                    .map(|&i| f64::from(v[start + i as usize]))
                    .fold(f64::NEG_INFINITY, f64::max);
                let mut sum = 0.;
                for &i in indices {
                    sum += (f64::from(v[start + i as usize]) - max).exp();
                }
                out.push((max + sum.ln()) as f32);
            }
            Ok((CpuStorage::F32(out), Shape::from(self.vocab)))
        }
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s: &candle_core::CudaStorage,
        l: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        use crate::native_geometric_cuda_kernels::{launch, Arg};
        let Some((start, end)) = l.contiguous_offsets() else {
            candle_core::bail!("aliases require contiguous scores");
        };
        if end - start != self.ids.len() {
            candle_core::bail!("alias descriptor/score size differs");
        }
        let d = &s.device;
        let offsets = d
            .cuda_stream()
            .clone_htod(&self.offsets)
            .map_err(|e| candle_core::Error::Cuda(Box::new(e)))?;
        let indices = d
            .cuda_stream()
            .clone_htod(&self.indices)
            .map_err(|e| candle_core::Error::Cuda(Box::new(e)))?;
        let result = if self.sum {
            let input = s.as_cuda_slice::<f64>()?;
            let input = input
                .try_slice(start..end)
                .ok_or_else(|| candle_core::Error::Msg("alias storage bounds".into()))?;
            let output = d.alloc_zeros::<f64>(self.vocab)?;
            launch(
                d,
                "whole_path_alias_sum",
                self.vocab,
                &[
                    Arg::D(input),
                    Arg::U(offsets.as_view()),
                    Arg::U(indices.as_view()),
                    Arg::D(output.as_view()),
                    Arg::N(self.vocab as u32),
                ],
            )?;
            candle_core::CudaStorage::wrap_cuda_slice(output, d.clone())
        } else {
            let input = s.as_cuda_slice::<f32>()?;
            let input = input
                .try_slice(start..end)
                .ok_or_else(|| candle_core::Error::Msg("alias storage bounds".into()))?;
            let output = d.alloc_zeros::<f32>(self.vocab)?;
            launch(
                d,
                "whole_path_alias_log",
                self.vocab,
                &[
                    Arg::F(input),
                    Arg::U(offsets.as_view()),
                    Arg::U(indices.as_view()),
                    Arg::F(output.as_view()),
                    Arg::N(self.vocab as u32),
                ],
            )?;
            candle_core::CudaStorage::wrap_cuda_slice(output, d.clone())
        };
        Ok((result, Shape::from(self.vocab)))
    }
    fn bwd(&self, x: &Tensor, y: &Tensor, dy: &Tensor) -> candle_core::Result<Option<Tensor>> {
        if self.sum {
            return Ok(None);
        }
        let ids = Tensor::from_vec(self.ids.clone(), self.ids.len(), x.device())?;
        let p = (x - y.index_select(&ids, 0)?)?.exp()?;
        Ok(Some((&p * dy.index_select(&ids, 0)?)?))
    }
}
pub(super) fn log_masses(scores: &Tensor, vocab: usize, copy_ids: &[u32]) -> Result<Tensor> {
    let op = Aliases::new(vocab, copy_ids, false)?;
    Ok(scores.contiguous()?.apply_op1(op)?)
}
pub(super) fn native_masses(weights: &Tensor, vocab: usize, copy_ids: &[u32]) -> Result<Tensor> {
    let op = Aliases::new(vocab, copy_ids, true)?;
    Ok(weights.detach().contiguous()?.apply_op1_no_bwd(&op)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Var};
    #[test]
    fn whole_path_linear_pool_preserves_every_alias_and_gradient() -> Result<()> {
        let x = Var::from_vec(vec![0f32, -1000., 3., 3., -1000.], 5, &Device::Cpu)?;
        let pooled = log_masses(x.as_tensor(), 3, &[2, 1])?;
        let values = pooled.to_vec1::<f32>()?;
        assert_eq!(values[0], 0.);
        assert!((values[1] - (-1000. + 2f32.ln())).abs() < 1e-4);
        assert!((values[2] - (3. + 2f32.ln())).abs() < 1e-6);
        let gradient = pooled.sum_all()?.backward()?;
        let got = gradient
            .get(x.as_tensor())
            .ok_or_else(|| invalid("alias gradient missing"))?
            .to_vec1::<f32>()?;
        for (actual, expected) in got.iter().zip([1., 0.5, 0.5, 0.5, 0.5]) {
            assert!((actual - expected).abs() < 2e-5);
        }
        let weights = Tensor::from_vec(vec![1f64, 7., 11., 13., 17.], 5, &Device::Cpu)?;
        assert_eq!(
            native_masses(&weights, 3, &[2, 1])?.to_vec1::<f64>()?,
            vec![1., 24., 24.]
        );
        assert!(native_masses(&weights, 3, &[3, 1]).is_err());
        Ok(())
    }
    #[test]
    fn whole_path_linear_cdf_handles_singleton_and_rounding_tail() -> Result<()> {
        assert_eq!(
            cdf(&Tensor::new(&[1f32], &Device::Cpu)?)?.to_vec1::<f64>()?,
            vec![1.]
        );
        let result =
            cdf(&Tensor::new(&[0.25f32, 0.25, 0.49999997], &Device::Cpu)?)?.to_vec1::<f64>()?;
        assert_eq!(result, vec![0.25, 0.5, 1.]);
        Ok(())
    }
}
