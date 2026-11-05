//! The served quantizers as one kernel a direction (float-kernel-fusion, claude-285).
//!
//! Trained as served, the trunk rounds its activations and its residual stream as the integer
//! session serves them, through [`crate::hamilton_sheet`]'s `activation_quantize16` (about 49
//! stock tensor ops a call with its straight-through bridge) and `q16_residual` (13). claude-283
//! measured the two chains at 7.5% of a training step at (S). The ops here compute the same
//! values and the same straight-through gradients in one launch a direction, from
//! `src/fused_quantize.cu`, built ahead of time as its own PTX module. They must equal the chains
//! bit for bit: the GPU gates below and `in_space_long`'s one-step gate check that, and
//! `hamilton_sheet`'s flag chooses them only for contiguous f32 CUDA tensors.
//!
//! The ops refuse the CPU, as the project's candle does for every model op (D24/D29): no CPU
//! chain exists to compare with, and a host path here would be the one CPU arithmetic the
//! trunk has. The GPU gate checks each kernel against its chain on the device and against a
//! host twin of the kernel, in the kernel's order (`mul_add` for `__fmaf_rn`, `f32::max` and
//! `f32::min` for `fmaxf` and `fminf`), which says which side moved when the two disagree.

use candle_core::{CpuStorage, CustomOp1, CustomOp2, Layout, Shape, Tensor};

use crate::hamilton_sheet::{ACTIVATION_EXP, ACTIVATION_SATURATED, Q16_SCALE};

type CResult<T> = candle_core::Result<T>;

/// The chains' scale constants as the f32 each one becomes. Candle casts an `affine` multiplier
/// and a `clamp` bound from the f64 the chain passes to the tensor's dtype, so each is cast here
/// from that same f64.
#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
#[derive(Clone, Copy)]
struct Constants {
    /// `2^-ACTIVATION_EXP`, the activation's integer scale before its shift.
    unit: f32,
    /// The 2^62 bound on a row's integer magnitude.
    saturated: f32,
    /// `Q16_SCALE`, the residual's 2^16.
    scale: f32,
    /// `1 / Q16_SCALE`.
    inverse: f32,
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn constants() -> Constants {
    Constants {
        unit: 2f64.powi(-ACTIVATION_EXP) as f32,
        saturated: ACTIVATION_SATURATED as f32,
        scale: f64::from(Q16_SCALE) as f32,
        inverse: (1.0 / f64::from(Q16_SCALE)) as f32,
    }
}

/// The fused `activation_quantize16` with its bridge, for a contiguous f32 `x`: the served
/// wrapper's form, `free + (quantized − free).detach()` with `free = x · inside`.
pub(crate) fn activation_quantize16(x: &Tensor) -> CResult<Tensor> {
    x.apply_op1(ActivationQuantize16)
}

/// The fused `q16_residual`, for a contiguous f32 `x`. With `bridge` it is the straight-through
/// form. Without it, it is the rounded values, untracked, as the chain's are: the chain's only
/// path to `x` runs through `floor`, which carries no gradient.
pub(crate) fn q16_residual(x: &Tensor, bridge: bool) -> CResult<Tensor> {
    if bridge {
        x.apply_op1(Q16Residual { bridge })
    } else {
        x.apply_op1_no_bwd(&Q16Residual { bridge })
    }
}

/// Every op's CPU path: candle's own refusal of CPU model compute (D24/D29), word for word.
fn refuse_cpu<T>() -> CResult<T> {
    candle_core::bail!(
        "D24/D29: CPU model compute is unavailable; use the admitted CUDA geometric path"
    )
}

/// The width of the last dimension, the activation quantizer's row.
#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn width_of(shape: &Shape) -> CResult<usize> {
    match shape.dims().last() {
        Some(&width) if width > 0 => Ok(width),
        _ => candle_core::bail!("the fused activation quantizer needs rows of width 1 or more"),
    }
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
fn same_shape(l1: &Layout, l2: &Layout) -> CResult<()> {
    if l1.shape() != l2.shape() {
        candle_core::bail!("a fused quantizer's gradient must match its input's shape");
    }
    Ok(())
}

struct ActivationQuantize16;

impl CustomOp1 for ActivationQuantize16 {
    fn name(&self) -> &'static str {
        "fused-activation-quantize16"
    }
    fn cpu_fwd(&self, _: &CpuStorage, _: &Layout) -> CResult<(CpuStorage, Shape)> {
        refuse_cpu()
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s: &candle_core::CudaStorage,
        l: &Layout,
    ) -> CResult<(candle_core::CudaStorage, Shape)> {
        cuda::activation_quantize16_fwd(s, l)
    }
    fn bwd(&self, x: &Tensor, _: &Tensor, grad: &Tensor) -> CResult<Option<Tensor>> {
        Ok(Some(x.apply_op2_no_bwd(
            &grad.contiguous()?,
            &ActivationQuantize16Grad,
        )?))
    }
}

/// `(x, grad) → grad · inside`, untracked.
struct ActivationQuantize16Grad;

impl CustomOp2 for ActivationQuantize16Grad {
    fn name(&self) -> &'static str {
        "fused-activation-quantize16-grad"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> CResult<(CpuStorage, Shape)> {
        refuse_cpu()
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &candle_core::CudaStorage,
        l1: &Layout,
        s2: &candle_core::CudaStorage,
        l2: &Layout,
    ) -> CResult<(candle_core::CudaStorage, Shape)> {
        same_shape(l1, l2)?;
        cuda::activation_quantize16_bwd(s1, l1, s2, l2)
    }
}

struct Q16Residual {
    bridge: bool,
}

impl CustomOp1 for Q16Residual {
    fn name(&self) -> &'static str {
        "fused-q16-residual"
    }
    fn cpu_fwd(&self, _: &CpuStorage, _: &Layout) -> CResult<(CpuStorage, Shape)> {
        refuse_cpu()
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s: &candle_core::CudaStorage,
        l: &Layout,
    ) -> CResult<(candle_core::CudaStorage, Shape)> {
        cuda::q16_residual_fwd(s, l, self.bridge)
    }
    fn bwd(&self, x: &Tensor, _: &Tensor, grad: &Tensor) -> CResult<Option<Tensor>> {
        if !self.bridge {
            candle_core::bail!("the bridge-off fused residual is untracked and has no backward");
        }
        Ok(Some(
            x.apply_op2_no_bwd(&grad.contiguous()?, &Q16ResidualGrad)?,
        ))
    }
}

/// `(x, grad) → grad · inside`, untracked.
struct Q16ResidualGrad;

impl CustomOp2 for Q16ResidualGrad {
    fn name(&self) -> &'static str {
        "fused-q16-residual-grad"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> CResult<(CpuStorage, Shape)> {
        refuse_cpu()
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &candle_core::CudaStorage,
        l1: &Layout,
        s2: &candle_core::CudaStorage,
        l2: &Layout,
    ) -> CResult<(candle_core::CudaStorage, Shape)> {
        same_shape(l1, l2)?;
        cuda::q16_residual_bwd(s1, l1, s2, l2)
    }
}

#[cfg(feature = "cuda")]
mod cuda {
    use candle_core::backend::BackendStorage;
    use candle_core::{CudaStorage, DType, Layout, Shape};
    use cudarc::driver::CudaView;

    use super::{constants, width_of, CResult};
    use crate::cuda_stack_kernels::cuda::{launch_groups_prebuilt, launch_prebuilt, uninit, Arg};
    use crate::cuda_stack_kernels::FUSED_QUANTIZE_MODULE;

    /// `src/fused_quantize.cu`, compiled ahead of time by nvcc (`build.rs`).
    pub(super) const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/fused_quantize_f32.ptx"));

    /// Threads per row of the activation kernels: a whole number of warps, as their full-mask
    /// shuffle needs.
    pub(super) const ROW_THREADS: usize = 256;
    const _: () = assert!(ROW_THREADS % 32 == 0 && ROW_THREADS <= 1024);

    fn view<'a>(storage: &'a CudaStorage, layout: &Layout) -> CResult<CudaView<'a, f32>> {
        if storage.dtype() != DType::F32 {
            candle_core::bail!("the fused quantizers take f32 tensors only");
        }
        let Some((start, end)) = layout.contiguous_offsets() else {
            candle_core::bail!("the fused quantizers take contiguous tensors only");
        };
        match storage.as_cuda_slice::<f32>()?.try_slice(start..end) {
            Some(view) => Ok(view),
            None => candle_core::bail!("a fused quantizer's layout exceeds its buffer"),
        }
    }

    fn u32_of(value: usize, what: &str) -> CResult<u32> {
        u32::try_from(value)
            .map_err(|_| candle_core::Error::Msg(format!("fused quantizer {what} too large")))
    }

    /// One block a row; the kernel writes every element of `out`.
    pub(super) fn activation_quantize16_fwd(
        s: &CudaStorage,
        l: &Layout,
    ) -> CResult<(CudaStorage, Shape)> {
        let x = view(s, l)?;
        let width = width_of(l.shape())?;
        let total = l.shape().elem_count();
        let c = constants();
        let device = s.device();
        let out = uninit::<f32>(device, total)?;
        launch_groups_prebuilt(
            device,
            FUSED_QUANTIZE_MODULE,
            PTX,
            "activation_quantize16_fwd",
            (total / width, 1, 1),
            (ROW_THREADS, 1, 1),
            &[
                Arg::F(x),
                Arg::f(&out),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(c.unit.to_bits()),
                Arg::U32(c.saturated.to_bits()),
            ],
        )?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            l.shape().clone(),
        ))
    }

    /// One block a row; the kernel writes every element of `dx`.
    pub(super) fn activation_quantize16_bwd(
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> CResult<(CudaStorage, Shape)> {
        let (x, grad) = (view(s1, l1)?, view(s2, l2)?);
        let width = width_of(l1.shape())?;
        let total = l1.shape().elem_count();
        let c = constants();
        let device = s1.device();
        let dx = uninit::<f32>(device, total)?;
        launch_groups_prebuilt(
            device,
            FUSED_QUANTIZE_MODULE,
            PTX,
            "activation_quantize16_bwd",
            (total / width, 1, 1),
            (ROW_THREADS, 1, 1),
            &[
                Arg::F(x),
                Arg::F(grad),
                Arg::f(&dx),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(c.unit.to_bits()),
                Arg::U32(c.saturated.to_bits()),
            ],
        )?;
        Ok((
            CudaStorage::wrap_cuda_slice(dx, device.clone()),
            l1.shape().clone(),
        ))
    }

    /// One thread an element; the kernel writes every element of `out`.
    pub(super) fn q16_residual_fwd(
        s: &CudaStorage,
        l: &Layout,
        bridge: bool,
    ) -> CResult<(CudaStorage, Shape)> {
        let x = view(s, l)?;
        let total = l.shape().elem_count();
        let c = constants();
        let device = s.device();
        let out = uninit::<f32>(device, total)?;
        launch_prebuilt(
            device,
            FUSED_QUANTIZE_MODULE,
            PTX,
            "q16_residual_fwd",
            total,
            &[
                Arg::F(x),
                Arg::f(&out),
                Arg::U32(u32_of(total, "elements")?),
                Arg::U32(u32::from(bridge)),
                Arg::U32(c.scale.to_bits()),
                Arg::U32(c.inverse.to_bits()),
            ],
        )?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            l.shape().clone(),
        ))
    }

    /// One thread an element; the kernel writes every element of `dx`.
    pub(super) fn q16_residual_bwd(
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
    ) -> CResult<(CudaStorage, Shape)> {
        let (x, grad) = (view(s1, l1)?, view(s2, l2)?);
        let total = l1.shape().elem_count();
        let c = constants();
        let device = s1.device();
        let dx = uninit::<f32>(device, total)?;
        launch_prebuilt(
            device,
            FUSED_QUANTIZE_MODULE,
            PTX,
            "q16_residual_bwd",
            total,
            &[
                Arg::F(x),
                Arg::F(grad),
                Arg::f(&dx),
                Arg::U32(u32_of(total, "elements")?),
                Arg::U32(c.scale.to_bits()),
            ],
        )?;
        Ok((
            CudaStorage::wrap_cuda_slice(dx, device.clone()),
            l1.shape().clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamilton_sheet::{self, FusedQuantizeCounts};
    use candle_core::Device;

    const REFUSED: &str = "D24/D29: CPU model compute is unavailable";

    #[test]
    fn on_the_cpu_the_flag_still_takes_each_chain_and_the_ops_refuse_as_candle_does() {
        let x = Tensor::from_vec(vec![0.5f32, -1.25, 3.0, -0.0], (2, 2), &Device::Cpu).unwrap();
        let ((), counts) = hamilton_sheet::with_fused_quantize(true, || {
            hamilton_sheet::with_served_surrogate_training(1, 0, || {
                // Each chain stops at its first op, past the path choice the counts record.
                let activation = hamilton_sheet::activation_quantize16(&x).unwrap_err();
                let residual = hamilton_sheet::q16_residual(x.clone()).unwrap_err();
                for error in [activation, residual] {
                    assert!(error.to_string().contains(REFUSED), "{error}");
                }
                Ok(())
            })
        })
        .unwrap();
        assert_eq!(
            counts,
            FusedQuantizeCounts {
                activation_fused: 0,
                activation_chain: 1,
                residual_fused: 0,
                residual_chain: 1,
            }
        );
        for refused in [
            activation_quantize16(&x),
            q16_residual(&x, true),
            q16_residual(&x, false),
        ] {
            let error = refused.unwrap_err().to_string();
            assert!(error.contains(REFUSED), "{error}");
        }
        // The counts start at zero for each scope, whatever an earlier scope counted.
        let ((), counts) = hamilton_sheet::with_fused_quantize(false, || Ok(())).unwrap();
        assert_eq!(counts, FusedQuantizeCounts::default());
    }

    /// The GPU gates (claude-285's ORDER): G1 checks each kernel against its chain on the device,
    /// forward and gradient, and against the host twin of the kernel; G3 reads the PTX.
    #[cfg(feature = "cuda")]
    mod gpu {
        use super::super::*;
        use crate::hamilton_sheet::{self, FusedQuantizeCounts};
        use candle_core::{Device, Var};

        /// The edge rows' width, and the random rows'.
        const EDGE_WIDTH: usize = 8;
        const RANDOM_WIDTH: usize = 64;

        /// `2^e` as an f32, for `-126 <= e <= 127`: the kernels' `pow2f`.
        fn pow2(e: i32) -> f32 {
            f32::from_bits(((127 + e) as u32) << 23)
        }

        /// The twin of the kernels' `row_scale`.
        fn row_scale(max: f32, c: Constants) -> f32 {
            let mut integer = max.mul_add(c.unit, 0.0).max(0.0).min(c.saturated);
            let mut scale = c.unit + 0.0;
            for bits in [32, 16, 8, 4, 2, 1] {
                let factor = if integer >= pow2(14 + bits) {
                    pow2(-bits) + 0.0
                } else {
                    1.0
                };
                integer *= factor;
                scale *= factor;
            }
            scale
        }

        fn indicator(holds: bool) -> f32 {
            if holds {
                1.0
            } else {
                0.0
            }
        }

        /// `inside` of the activation bridge: `ge(scaled, -32767) * le(scaled, 32767)`.
        fn activation_inside(scaled: f32) -> f32 {
            indicator(scaled >= -32767.0) * indicator(scaled <= 32767.0)
        }

        /// `inside` of the residual bridge: `ge(scaled, -2^31) * lt(scaled, 2^31)`. The chain's
        /// upper bound is `i32::MAX`, which rounds to 2^31 as an f32, so the bound is exclusive.
        fn residual_inside(scaled: f32) -> f32 {
            indicator(scaled >= -pow2(31)) * indicator(scaled < pow2(31))
        }

        fn row_scales(x: &[f32], width: usize) -> Vec<f32> {
            x.chunks(width)
                .map(|row| {
                    let max = row
                        .iter()
                        .fold(f32::NEG_INFINITY, |max, value| max.max(value.abs()));
                    row_scale(max, constants())
                })
                .collect()
        }

        /// The twin of `activation_quantize16_fwd`.
        fn activation_forward(x: &[f32], width: usize) -> Vec<f32> {
            let scales = row_scales(x, width);
            x.chunks(width)
                .zip(scales)
                .flat_map(|(row, scale)| {
                    row.iter().map(move |&value| {
                        let scaled = value * scale;
                        let quantized = (scaled + 0.5).floor().max(-32767.0).min(32767.0) / scale;
                        let kept = value * activation_inside(scaled);
                        kept + (quantized - kept)
                    })
                })
                .collect()
        }

        /// The twin of `activation_quantize16_bwd`: `grad · inside`.
        fn activation_backward(x: &[f32], grad: &[f32], width: usize) -> Vec<f32> {
            let scales = row_scales(x, width);
            let scale_of = |i: usize| scales[i / width];
            x.iter()
                .zip(grad)
                .enumerate()
                .map(|(i, (&value, &grad))| grad * activation_inside(value * scale_of(i)))
                .collect()
        }

        /// The twin of `q16_residual_fwd`.
        fn residual_forward(x: &[f32], bridge: bool) -> Vec<f32> {
            let c = constants();
            x.iter()
                .map(|&value| {
                    let scaled = value.mul_add(c.scale, 0.0);
                    let rounded = (scaled + 0.5)
                        .floor()
                        .max(-pow2(31))
                        .min(pow2(31))
                        .mul_add(c.inverse, 0.0);
                    if !bridge {
                        return rounded;
                    }
                    let kept = value * residual_inside(scaled);
                    kept + (rounded - kept)
                })
                .collect()
        }

        /// The twin of `q16_residual_bwd`: `grad · inside`.
        fn residual_backward(x: &[f32], grad: &[f32]) -> Vec<f32> {
            let c = constants();
            x.iter()
                .zip(grad)
                .map(|(&value, &grad)| grad * residual_inside(value.mul_add(c.scale, 0.0)))
                .collect()
        }

        /// A seeded LCG's draws, each a value over 2^-30 to 2^9 with either sign.
        fn lcg_values(seed: u64, n: usize) -> Vec<f32> {
            let mut state = seed;
            (0..n)
                .map(|_| {
                    state = state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    let mantissa = ((state >> 40) as u32) & 0x7f_ffff;
                    let exponent = ((state >> 33) % 40) as i32 - 30;
                    let sign = if state >> 63 == 1 { -1.0 } else { 1.0 };
                    sign * f32::from_bits(0x3f80_0000 | mantissa) * pow2(exponent)
                })
                .collect()
        }

        /// Weights for `loss = Σ out · w`, with exact zeros of both signs among them.
        fn weights(n: usize) -> Vec<f32> {
            lcg_values(7, n)
                .iter()
                .enumerate()
                .map(|(i, value)| match i % 5 {
                    0 => 0.0,
                    1 => -0.0,
                    _ => value.clamp(-2.0, 2.0),
                })
                .collect()
        }

        /// Activation rows of [`EDGE_WIDTH`]: zeros of both signs, subnormals, exact .5 ties on
        /// an unshifted row (scale `2^-ACTIVATION_EXP`) and on a row shifted by 6 bits, the
        /// ±32767 bound of the bridge, a saturated row and rows with an infinity.
        fn activation_edge_rows() -> Vec<f32> {
            let unit = pow2(ACTIVATION_EXP);
            let tie = |k: f32, e: i32| (k + 0.5) * pow2(ACTIVATION_EXP + e);
            let tiny = f32::from_bits(1);
            let rows: [[f32; EDGE_WIDTH]; 9] = [
                [0.0; EDGE_WIDTH],
                [-0.0, 0.0, -0.0, -0.0, 0.0, -0.0, 0.0, -0.0],
                [
                    tiny,
                    -tiny,
                    f32::from_bits(0x7f_ffff),
                    -f32::from_bits(0x40_0000),
                    0.0,
                    -0.0,
                    tiny,
                    f32::MIN_POSITIVE,
                ],
                [
                    tie(0.0, 0),
                    tie(1.0, 0),
                    tie(2.0, 0),
                    tie(-1.0, 0),
                    tie(-3.0, 0),
                    tie(100.0, 0),
                    tie(1000.0, 0),
                    tie(-16000.0, 0),
                ],
                [
                    pow2(20) * unit,
                    tie(0.0, 6),
                    tie(-1.0, 6),
                    tie(5.0, 6),
                    tie(-6.0, 6),
                    tie(16382.0, 6),
                    tie(-16383.0, 6),
                    tie(77.0, 6),
                ],
                [
                    32767.5 * unit,
                    -32767.5 * unit,
                    32767.0 * unit,
                    -32767.0 * unit,
                    32766.5 * unit,
                    -32766.5 * unit,
                    32767.25 * unit,
                    -0.0,
                ],
                [1e20, -1e20, 1.0, 0.5, -0.25, 3e19, -7e18, 0.0],
                [f32::INFINITY, 1.0, -2.0, 0.0, -0.0, 1e-30, 65504.0, -1.5],
                [-1.0, f32::NEG_INFINITY, 2.5, tiny, 0.0, 1e10, -1e10, 3.0],
            ];
            rows.concat()
        }

        /// Random rows of `width`, with every fourth value well under its row's largest moved
        /// to an exact .5 tie of the row's scale, so the scale stays as it was. With `wide`, each
        /// row's largest magnitude is first moved past index [`cuda::ROW_THREADS`], so a thread's
        /// strided loop must reach it, and the first row's saturates the 2^62 clamp.
        fn activation_random_rows(rows: usize, width: usize, wide: bool) -> Vec<f32> {
            let mut values = lcg_values(285 + width as u64, rows * width);
            for (r, row) in values.chunks_mut(width).enumerate() {
                if wide {
                    let max = row.iter().fold(0f32, |max, value| max.max(value.abs()));
                    let sign = if r % 2 == 0 { 1.0 } else { -1.0 };
                    row[width - 1 - 3 * r] = if r == 0 { 1e20 } else { sign * 4.0 * max };
                }
                let max = row.iter().fold(0f32, |max, value| max.max(value.abs()));
                let scale = row_scale(max, constants());
                for value in row.iter_mut().step_by(4) {
                    if value.abs() < max * 0.5 {
                        *value = ((*value * scale).floor() + 0.5) / scale;
                    }
                }
            }
            values
        }

        /// Residual values: zeros of both signs, subnormals, exact .5 ties of 2^-16, the 2^31
        /// bounds of the scaled value at ±32768 and past them at ±40000, and infinities.
        fn residual_edge_values() -> Vec<f32> {
            let tie = |k: f32| (k + 0.5) * pow2(-16);
            let tiny = f32::from_bits(1);
            vec![
                0.0,
                -0.0,
                tiny,
                -tiny,
                f32::MIN_POSITIVE,
                -f32::from_bits(0x7f_ffff),
                tie(0.0),
                tie(-1.0),
                tie(1.0),
                tie(-2.0),
                tie(12345.0),
                tie(-99999.0),
                32768.0,
                -32768.0,
                40000.0,
                -40000.0,
                32767.99,
                -32767.99,
                f32::INFINITY,
                f32::NEG_INFINITY,
                1e-30,
                -1e30,
                0.5,
                -1.5,
            ]
        }

        /// Random residual values, every third one moved to an exact .5 tie of 2^-16.
        fn residual_random_values(n: usize) -> Vec<f32> {
            let mut values = lcg_values(2285, n);
            for value in values.iter_mut().step_by(3) {
                *value = ((*value * 65536.0).floor() + 0.5) / 65536.0;
            }
            values
        }

        fn host(t: &Tensor) -> Vec<f32> {
            t.flatten_all().unwrap().to_vec1::<f32>().unwrap()
        }

        fn bits(values: &[f32]) -> Vec<u32> {
            values.iter().map(|value| value.to_bits()).collect()
        }

        /// A call's output and `x`'s gradient for `loss = Σ out · w`, if it has one, and the
        /// paths its quantizer calls took, with the fused flag at `fused`.
        fn run(
            values: &[f32],
            shape: &[usize],
            fused: bool,
            quantize: impl Fn(&Tensor) -> crate::Result<Tensor>,
        ) -> (Vec<f32>, Option<Vec<f32>>, FusedQuantizeCounts) {
            let device = Device::new_cuda(0).unwrap();
            let x = Var::from_vec(values.to_vec(), shape, &device).unwrap();
            let w = Tensor::from_vec(weights(values.len()), shape, &device).unwrap();
            let ((out, grad), counts) = hamilton_sheet::with_fused_quantize(fused, || {
                let out = quantize(x.as_tensor())?;
                let grads = (&out * &w)?.sum_all()?.backward()?;
                Ok((host(&out), grads.get(x.as_tensor()).map(host)))
            })
            .unwrap();
            (out, grad, counts)
        }

        /// The twin's values against the device's: equal bits, except that a NaN matches any
        /// NaN (the host's default NaN is not CUDA's canonical one).
        fn twin_matches(twin: &[f32], device: &[f32]) -> bool {
            twin.len() == device.len()
                && twin
                    .iter()
                    .zip(device)
                    .all(|(t, d)| (t.is_nan() && d.is_nan()) || t.to_bits() == d.to_bits())
        }

        /// The gradient candle hands a quantizer for `loss = Σ out · w`: `1 · w`, which is `w`,
        /// −0 included. The patched candle stores a tensor's first gradient as it is
        /// (`backprop.rs` `store_directly`), where upstream added it to zeros, which turns −0
        /// into +0.
        fn out_grad(n: usize) -> Vec<f32> {
            weights(n)
        }

        fn activation_chain(x: &Tensor) -> crate::Result<Tensor> {
            hamilton_sheet::with_served_surrogate_training(1, 0, || {
                hamilton_sheet::activation_quantize16(x)
            })
        }

        /// The chain outside the served wrapper, where its bridge is off.
        fn activation_chain_unbridged(x: &Tensor) -> crate::Result<Tensor> {
            Ok(hamilton_sheet::activation_quantize16(x)?)
        }

        fn residual_chain(x: &Tensor) -> crate::Result<Tensor> {
            hamilton_sheet::with_served_surrogate_training(1, 0, || {
                hamilton_sheet::q16_residual(x.clone())
            })
        }

        fn residual_chain_unbridged(x: &Tensor) -> crate::Result<Tensor> {
            hamilton_sheet::with_q16_residual(|| hamilton_sheet::q16_residual(x.clone()))
        }

        #[test]
        #[ignore = "needs a CUDA device; run on the pod"]
        fn g1_the_activation_kernels_equal_the_chain_and_the_twin_bit_for_bit() {
            let edge = activation_edge_rows();
            let random = activation_random_rows(16, RANDOM_WIDTH, false);
            let (wide, wider) = (
                activation_random_rows(6, 1000, true),
                activation_random_rows(2, 2048, true),
            );
            assert!(1000 - 1 - 3 * 5 > cuda::ROW_THREADS);
            let cases: [(&[f32], Vec<usize>); 5] = [
                (&edge, vec![edge.len() / EDGE_WIDTH, EDGE_WIDTH]),
                (&edge, vec![3, 3, EDGE_WIDTH]),
                (&random, vec![16, RANDOM_WIDTH]),
                (&wide, vec![6, 1000]),
                (&wider, vec![2, 2048]),
            ];
            for (values, shape) in cases {
                let width = *shape.last().unwrap();
                let chain = run(values, &shape, false, activation_chain);
                let fused = run(values, &shape, true, activation_chain);
                assert_eq!((chain.2.activation_chain, chain.2.activation_fused), (1, 0));
                assert_eq!((fused.2.activation_chain, fused.2.activation_fused), (0, 1));
                let (chain_grad, fused_grad) = (chain.1.unwrap(), fused.1.unwrap());
                assert_eq!(bits(&fused.0), bits(&chain.0), "{shape:?}: forward");
                assert_eq!(bits(&fused_grad), bits(&chain_grad), "{shape:?}: gradient");
                let twin = activation_forward(values, width);
                assert!(
                    twin_matches(&twin, &chain.0),
                    "{shape:?}: the twin's forward"
                );
                let twin = activation_backward(values, &out_grad(values.len()), width);
                assert_eq!(
                    bits(&twin),
                    bits(&chain_grad),
                    "{shape:?}: the twin's gradient"
                );
                // The kernel computes the bridged form only, so with the bridge off the flag
                // leaves the call on the chain.
                let chain = run(values, &shape, false, activation_chain_unbridged);
                let fused = run(values, &shape, true, activation_chain_unbridged);
                assert_eq!((fused.2.activation_chain, fused.2.activation_fused), (1, 0));
                assert_eq!(
                    bits(&fused.0),
                    bits(&chain.0),
                    "{shape:?}: unbridged forward"
                );
                assert_eq!(
                    fused.1.as_deref().map(bits),
                    chain.1.as_deref().map(bits),
                    "{shape:?}: unbridged gradient"
                );
            }
        }

        #[test]
        #[ignore = "needs a CUDA device; run on the pod"]
        fn g1_the_residual_kernels_equal_the_chain_and_the_twin_bit_for_bit() {
            let edge = residual_edge_values();
            let random = residual_random_values(512);
            let cases: [(&[f32], Vec<usize>); 2] =
                [(&edge, vec![4, edge.len() / 4]), (&random, vec![2, 16, 16])];
            for (values, shape) in cases {
                let chain = run(values, &shape, false, residual_chain);
                let fused = run(values, &shape, true, residual_chain);
                assert_eq!((chain.2.residual_chain, chain.2.residual_fused), (1, 0));
                assert_eq!((fused.2.residual_chain, fused.2.residual_fused), (0, 1));
                let (chain_grad, fused_grad) = (chain.1.unwrap(), fused.1.unwrap());
                assert_eq!(bits(&fused.0), bits(&chain.0), "{shape:?}: bridged forward");
                assert_eq!(
                    bits(&fused_grad),
                    bits(&chain_grad),
                    "{shape:?}: bridged gradient"
                );
                assert!(twin_matches(&residual_forward(values, true), &chain.0));
                let twin = residual_backward(values, &out_grad(values.len()));
                assert_eq!(
                    bits(&twin),
                    bits(&chain_grad),
                    "{shape:?}: the twin's gradient"
                );
                // Without its bridge the chain's one path to `x` runs through `floor`.
                let chain = run(values, &shape, false, residual_chain_unbridged);
                let fused = run(values, &shape, true, residual_chain_unbridged);
                assert_eq!((fused.2.residual_chain, fused.2.residual_fused), (0, 1));
                assert_eq!(
                    bits(&fused.0),
                    bits(&chain.0),
                    "{shape:?}: unbridged forward"
                );
                assert_eq!((fused.1, chain.1), (None, None), "{shape:?}: no gradient");
                assert!(twin_matches(&residual_forward(values, false), &chain.0));
            }
        }

        /// G3: every f32 add, subtract, multiply, divide and fused multiply-add rounds to
        /// nearest by name (`.rn`), which ptxas never contracts or reorders, and nothing
        /// flushes subnormals or approximates.
        #[test]
        #[ignore = "reads the PTX the real nvcc built; run on the pod"]
        fn g3_the_ptx_rounds_every_float_op_to_nearest_and_flushes_nothing() {
            let ptx = cuda::PTX;
            assert!(!ptx.contains(".ftz"), "a flush to zero");
            assert!(!ptx.contains(".approx"), "an approximate op");
            let mut checked = 0;
            let mut fma = 0;
            for line in ptx.lines() {
                let Some(op) = line
                    .split_whitespace()
                    .find(|token| !token.starts_with('@'))
                else {
                    continue;
                };
                let arithmetic = ["add.", "sub.", "mul.", "div.", "fma."]
                    .iter()
                    .any(|prefix| op.starts_with(prefix));
                if arithmetic && op.ends_with(".f32") {
                    assert!(
                        op.contains(".rn."),
                        "{line}: not rounded to nearest by name"
                    );
                    checked += 1;
                    fma += usize::from(op.starts_with("fma."));
                }
            }
            assert!(
                checked > 0 && fma > 0,
                "{checked} float ops, {fma} of them fma"
            );
        }
    }
}
