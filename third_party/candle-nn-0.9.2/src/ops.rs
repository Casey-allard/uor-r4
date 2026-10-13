//! UOR D29/D30 boundary: historical normalized/dense operations are refusals.
//! Original source is retained in ../historical/src/ops.rs, never included.
use candle::{CpuStorage, DType, Layout, Module, Result, Shape, Tensor, D};

pub fn softmax<D: candle::shape::Dim>(_: &Tensor, _: D) -> Result<Tensor> {
    candle::bail!("D29: normalized exponential operation is inactive")
}
pub fn log_softmax<D: candle::shape::Dim>(_: &Tensor, _: D) -> Result<Tensor> {
    candle::bail!("D29: logarithmic normalized exponential operation is inactive")
}
pub fn softmax_last_dim(_: &Tensor) -> Result<Tensor> {
    candle::bail!("D29: normalized exponential custom operation is inactive")
}
pub fn sigmoid(_: &Tensor) -> Result<Tensor> {
    candle::bail!("D29: historical squashing operation is inactive")
}
pub fn hard_sigmoid(_: &Tensor) -> Result<Tensor> {
    candle::bail!("D29: historical named squashing operation is inactive")
}
pub fn silu(_: &Tensor) -> Result<Tensor> {
    candle::bail!("D29: historical squashing activation is inactive")
}
pub fn swiglu(_: &Tensor) -> Result<Tensor> {
    candle::bail!("D29: historical squashing gate is inactive")
}
pub fn mish(_: &Tensor) -> Result<Tensor> {
    candle::bail!("D29: historical exponential activation is inactive")
}
pub fn selu(_: &Tensor, _: f32, _: f32) -> Result<Tensor> {
    candle::bail!("D29: historical exponential activation is inactive")
}
pub fn leaky_relu(xs: &Tensor, negative_slope: f64) -> Result<Tensor> {
    if !xs.device().is_cuda() {
        candle::bail!("D24: CPU model activation is inactive")
    }
    let zeros = xs.zeros_like()?;
    xs.maximum(&zeros)? + xs.minimum(&zeros)? * negative_slope
}

pub fn dropout(xs: &Tensor, drop_p: f32) -> Result<Tensor> {
    if !xs.device().is_cuda() {
        candle::bail!("D24: CPU model dropout is inactive")
    }
    // This implementation is inefficient as it stores the full mask for the backward pass.
    // Instead we could just store the seed and have a specialized kernel that would both
    // generate the random mask and apply it.
    // Another easier optimization would be to be able to generate boolean mask using just a bit of
    // entropy per element rather than generating a full float per element.
    if !(0. ..1.).contains(&drop_p) {
        candle::bail!("dropout probability has to be in [0, 1), got {drop_p}")
    }
    let rand = Tensor::rand(0f32, 1f32, xs.shape(), xs.device())?;
    let scale = 1.0 / (1.0 - drop_p as f64);
    let drop_p = Tensor::new(drop_p, xs.device())?.broadcast_as(xs.shape())?;
    let mask = (rand.ge(&drop_p)?.to_dtype(xs.dtype())? * scale)?;
    xs * mask
}

#[derive(Clone, Debug)]
pub struct Dropout {
    drop_p: f32,
}

impl Dropout {
    pub fn new(drop_p: f32) -> Dropout {
        Self { drop_p }
    }

    pub fn forward(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        if train {
            dropout(xs, self.drop_p)
        } else {
            Ok(xs.clone())
        }
    }
}

impl candle::ModuleT for Dropout {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        self.forward(xs, train)
    }
}

#[derive(Debug, Clone)]
struct RmsNorm {
    eps: f32,
}

impl candle::CustomOp2 for RmsNorm {
    fn name(&self) -> &'static str {
        "rms-norm"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
    ) -> Result<(CpuStorage, Shape)> {
        candle::bail!("D24: CPU model normalization is inactive")
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &candle::CudaStorage,
        l1: &Layout,
        s2: &candle::CudaStorage,
        l2: &Layout,
    ) -> Result<(candle::CudaStorage, Shape)> {
        use candle::cuda_backend::cudarc::driver::{
            CudaSlice, DeviceRepr, LaunchConfig, PushKernelArg,
        };
        use candle::cuda_backend::{kernel_name, kernels, Map2, WrapErr};
        use candle::{CudaDevice, WithDType};

        struct S {
            eps: f32,
        }
        impl Map2 for S {
            fn f<T: DeviceRepr + WithDType>(
                &self,
                src: &CudaSlice<T>,
                layout: &Layout,
                alpha: &CudaSlice<T>,
                alpha_layout: &Layout,
                dev: &CudaDevice,
            ) -> Result<CudaSlice<T>> {
                let src = match layout.contiguous_offsets() {
                    None => candle::bail!("input has to be contiguous"),
                    Some((o1, o2)) => src.slice(o1..o2),
                };
                let alpha = match alpha_layout.contiguous_offsets() {
                    None => candle::bail!("alpha has to be contiguous"),
                    Some((o1, o2)) => alpha.slice(o1..o2),
                };
                let el = layout.shape().elem_count();
                let dims = layout.shape().dims();
                let dim_m1 = dims[dims.len() - 1];
                let (n_rows, n_cols) = (el / dim_m1, dim_m1);

                let block_size = if n_cols < 1024 { 32 } else { 1024 };
                let cfg = LaunchConfig {
                    grid_dim: (n_rows as u32, 1, 1),
                    block_dim: (block_size, 1, 1),
                    shared_mem_bytes: 0,
                };
                let func = dev.get_or_load_func(&kernel_name::<T>("rmsnorm"), &kernels::REDUCE)?;
                // SAFETY: Set later by running the kernel.
                let dst = unsafe { dev.alloc::<T>(el)? };
                let mut builder = func.builder();
                builder.arg(&src);
                builder.arg(&dst);
                builder.arg(&alpha);
                candle::builder_arg!(builder, n_cols as i32, block_size as i32, self.eps);
                // SAFETY: ffi.
                unsafe { builder.launch(cfg) }.w()?;
                Ok(dst)
            }
        }

        use candle::backend::BackendStorage;
        let dev = s1.device();
        let slice = S { eps: self.eps }.map(&s1.slice, l1, &s2.slice, l2, dev)?;
        let dst = candle::cuda_backend::CudaStorage {
            slice,
            device: dev.clone(),
        };
        Ok((dst, l1.shape().clone()))
    }

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &candle::MetalStorage,
        l1: &Layout,
        s2: &candle::MetalStorage,
        l2: &Layout,
    ) -> Result<(candle::MetalStorage, Shape)> {
        use candle::backend::BackendStorage;
        let device = s1.device();
        let encoder = device.command_encoder()?;
        encoder.set_label("rmsnorm");
        let kernels = device.kernels();
        let name = match (s1.dtype(), s2.dtype()) {
            (DType::F32, DType::F32) => "rmsnorm_f32",
            (DType::F16, DType::F16) => "rmsnorm_f16",
            (DType::BF16, DType::BF16) => "rmsnorm_bf16",
            (dt1, dt2) => candle::bail!("rmsnorm is not implemented for {dt1:?} {dt2:?}"),
        };

        if !(l1.is_contiguous() && l2.is_contiguous()) {
            candle::bail!("Non contiguous rmsnorm is not implemented");
        }

        let last_dim = l1.dims()[l1.shape().rank() - 1];
        let elem_count = l1.shape().elem_count();
        let output = device.new_buffer(elem_count, s1.dtype(), "rmsnorm")?;
        candle_metal_kernels::call_rms_norm(
            device.metal_device(),
            &encoder,
            kernels,
            name,
            elem_count,
            last_dim,
            self.eps,
            s1.buffer(),
            l1.start_offset() * s1.dtype().size_in_bytes(),
            s2.buffer(),
            l2.start_offset() * s2.dtype().size_in_bytes(),
            &output,
        )
        .map_err(candle::Error::wrap)?;
        let newstorage = candle::MetalStorage::new(output, device.clone(), elem_count, s1.dtype());
        Ok((newstorage, l1.shape().clone()))
    }
}

pub fn rms_norm_slow(x: &Tensor, alpha: &Tensor, eps: f32) -> Result<Tensor> {
    if !x.device().is_cuda() {
        candle::bail!("D24: CPU model normalization is inactive")
    }
    let x_dtype = x.dtype();
    let internal_dtype = match x_dtype {
        DType::F16 | DType::BF16 => DType::F32,
        d => d,
    };
    let hidden_size = x.dim(D::Minus1)?;
    let x = x.to_dtype(internal_dtype)?;
    let norm_x = (x.sqr()?.sum_keepdim(D::Minus1)? / hidden_size as f64)?;
    let x_normed = x.broadcast_div(&(norm_x + eps as f64)?.sqrt()?)?;
    x_normed.to_dtype(x_dtype)?.broadcast_mul(alpha)
}

pub fn rms_norm(xs: &Tensor, alpha: &Tensor, eps: f32) -> Result<Tensor> {
    if !xs.device().is_cuda() {
        candle::bail!("D24: CPU model normalization is inactive")
    }
    let hidden_size_xs = xs.dim(D::Minus1)?;
    let hidden_size_alpha = alpha.dims1()?;
    if hidden_size_xs != hidden_size_alpha {
        candle::bail!(
            "shape mismatch in rms-norm {:?} {:?}",
            xs.shape(),
            alpha.shape()
        )
    }
    xs.apply_op2_no_bwd(alpha, &RmsNorm { eps })
}

#[derive(Debug, Clone)]
struct LayerNorm {
    eps: f32,
}

impl candle::CustomOp3 for LayerNorm {
    fn name(&self) -> &'static str {
        "layer-norm"
    }

    fn cpu_fwd(
        &self,
        s1: &CpuStorage,
        l1: &Layout,
        s2: &CpuStorage,
        l2: &Layout,
        s3: &CpuStorage,
        l3: &Layout,
    ) -> Result<(CpuStorage, Shape)> {
        candle::bail!("D24: CPU model normalization is inactive")
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        s1: &candle::CudaStorage,
        l1: &Layout,
        s2: &candle::CudaStorage,
        l2: &Layout,
        s3: &candle::CudaStorage,
        l3: &Layout,
    ) -> Result<(candle::CudaStorage, Shape)> {
        use candle::cuda_backend::cudarc::driver::{
            CudaSlice, DeviceRepr, LaunchConfig, PushKernelArg,
        };
        use candle::cuda_backend::{kernel_name, kernels, Map3, WrapErr};
        use candle::{CudaDevice, WithDType};

        struct S {
            eps: f32,
        }
        impl Map3 for S {
            fn f<T: DeviceRepr + WithDType>(
                &self,
                src: &CudaSlice<T>,
                layout: &Layout,
                alpha: &CudaSlice<T>,
                alpha_layout: &Layout,
                beta: &CudaSlice<T>,
                beta_layout: &Layout,
                dev: &CudaDevice,
            ) -> Result<CudaSlice<T>> {
                let src = match layout.contiguous_offsets() {
                    None => candle::bail!("input has to be contiguous"),
                    Some((o1, o2)) => src.slice(o1..o2),
                };
                let alpha = match alpha_layout.contiguous_offsets() {
                    None => candle::bail!("alpha has to be contiguous"),
                    Some((o1, o2)) => alpha.slice(o1..o2),
                };
                let beta = match beta_layout.contiguous_offsets() {
                    None => candle::bail!("beta has to be contiguous"),
                    Some((o1, o2)) => beta.slice(o1..o2),
                };
                let el = layout.shape().elem_count();
                let dims = layout.shape().dims();
                let dim_m1 = dims[dims.len() - 1];
                let (n_rows, n_cols) = (el / dim_m1, dim_m1);

                let block_size = if n_cols < 1024 { 32 } else { 1024 };
                let cfg = LaunchConfig {
                    grid_dim: (n_rows as u32, 1, 1),
                    block_dim: (block_size, 1, 1),
                    shared_mem_bytes: 0,
                };
                let func =
                    dev.get_or_load_func(&kernel_name::<T>("layernorm"), &kernels::REDUCE)?;
                // SAFETY: Set later by running the kernel.
                let dst = unsafe { dev.alloc::<T>(el)? };
                let mut builder = func.builder();
                builder.arg(&src);
                builder.arg(&dst);
                builder.arg(&alpha);
                builder.arg(&beta);
                candle::builder_arg!(builder, n_cols as i32, block_size as i32, self.eps);
                // SAFETY: ffi.
                unsafe { builder.launch(cfg) }.w()?;
                Ok(dst)
            }
        }

        use candle::backend::BackendStorage;
        let dev = s1.device();
        let slice = S { eps: self.eps }.map(&s1.slice, l1, &s2.slice, l2, &s3.slice, l3, dev)?;
        let dst = candle::cuda_backend::CudaStorage {
            slice,
            device: dev.clone(),
        };
        Ok((dst, l1.shape().clone()))
    }

    #[cfg(feature = "metal")]
    fn metal_fwd(
        &self,
        s1: &candle::MetalStorage,
        l1: &Layout,
        s2: &candle::MetalStorage,
        l2: &Layout,
        s3: &candle::MetalStorage,
        l3: &Layout,
    ) -> Result<(candle::MetalStorage, Shape)> {
        use candle::backend::BackendStorage;
        let device = s1.device();
        let encoder = device.command_encoder()?;
        encoder.set_label("layernorm");
        let kernels = device.kernels();
        let name = match (s1.dtype(), s2.dtype(), s3.dtype()) {
            (DType::F32, DType::F32, DType::F32) => "layernorm_f32",
            (DType::F16, DType::F16, DType::F16) => "layernorm_f16",
            (DType::BF16, DType::BF16, DType::BF16) => "layernorm_bf16",
            (dt1, dt2, dt3) => {
                candle::bail!("layernorm is not implemented for {dt1:?} {dt2:?} {dt3:?}")
            }
        };

        if !(l1.is_contiguous() && l2.is_contiguous() && l3.is_contiguous()) {
            candle::bail!("Non contiguous layernorm is not implemented");
        }

        let last_dim = l1.dims()[l1.shape().rank() - 1];
        let elem_count = l1.shape().elem_count();
        let output = device.new_buffer(elem_count, s1.dtype(), "layernorm")?;
        candle_metal_kernels::call_layer_norm(
            device.metal_device(),
            &encoder,
            kernels,
            name,
            elem_count,
            last_dim,
            self.eps,
            s1.buffer(),
            l1.start_offset() * s1.dtype().size_in_bytes(),
            s2.buffer(),
            l2.start_offset() * s2.dtype().size_in_bytes(),
            s3.buffer(),
            l3.start_offset() * s3.dtype().size_in_bytes(),
            &output,
        )
        .map_err(candle::Error::wrap)?;
        let newstorage = candle::MetalStorage::new(output, device.clone(), elem_count, s1.dtype());
        Ok((newstorage, l1.shape().clone()))
    }
}

pub fn layer_norm_slow(x: &Tensor, alpha: &Tensor, beta: &Tensor, eps: f32) -> Result<Tensor> {
    if !x.device().is_cuda() {
        candle::bail!("D24: CPU model normalization is inactive")
    }
    let x_dtype = x.dtype();
    let internal_dtype = match x_dtype {
        DType::F16 | DType::BF16 => DType::F32,
        d => d,
    };
    let hidden_size = x.dim(D::Minus1)?;
    let x = x.to_dtype(internal_dtype)?;
    let x = {
        let mean_x = (x.sum_keepdim(D::Minus1)? / hidden_size as f64)?;
        x.broadcast_sub(&mean_x)?
    };
    let norm_x = (x.sqr()?.sum_keepdim(D::Minus1)? / hidden_size as f64)?;
    let x_normed = x.broadcast_div(&(norm_x + eps as f64)?.sqrt()?)?;
    x_normed
        .to_dtype(x_dtype)?
        .broadcast_mul(alpha)?
        .broadcast_add(beta)
}

pub fn layer_norm(xs: &Tensor, alpha: &Tensor, beta: &Tensor, eps: f32) -> Result<Tensor> {
    if !xs.device().is_cuda() {
        candle::bail!("D24: CPU model normalization is inactive")
    }
    let hidden_size_xs = xs.dim(D::Minus1)?;
    let hidden_size_alpha = alpha.dims1()?;
    let hidden_size_beta = beta.dims1()?;
    if hidden_size_xs != hidden_size_alpha || hidden_size_xs != hidden_size_beta {
        candle::bail!(
            "shape mismatch in layer-norm src: {:?} alpha: {:?} beta: {:?}",
            xs.shape(),
            alpha.shape(),
            beta.shape()
        )
    }
    xs.apply_op3_no_bwd(alpha, beta, &LayerNorm { eps })
}

// https://pytorch.org/docs/stable/generated/torch.nn.PixelShuffle.html
pub fn pixel_shuffle(xs: &Tensor, upscale_factor: usize) -> Result<Tensor> {
    let (b_size, c, h, w) = xs.dims4()?;
    let out_c = c / upscale_factor / upscale_factor;
    xs.reshape((b_size, out_c, upscale_factor, upscale_factor, h, w))?
        .permute((0, 1, 4, 2, 5, 3))?
        .reshape((b_size, out_c, h * upscale_factor, w * upscale_factor))
}

pub fn pixel_unshuffle(xs: &Tensor, downscale_factor: usize) -> Result<Tensor> {
    let (b_size, c, h, w) = xs.dims4()?;
    let out_c = c * downscale_factor * downscale_factor;
    xs.reshape((
        b_size,
        c,
        h / downscale_factor,
        downscale_factor,
        w / downscale_factor,
        downscale_factor,
    ))?
    .permute((0, 1, 3, 5, 2, 4))?
    .reshape((b_size, out_c, h / downscale_factor, w / downscale_factor))
}

// https://pytorch.org/docs/stable/generated/torch.nn.ReplicationPad2d.html
pub fn replication_pad2d(xs: &Tensor, pad: usize) -> Result<Tensor> {
    match pad {
        0 => Ok(xs.clone()),
        1 => {
            let (_b_size, _c, h, w) = xs.dims4()?;
            let (first, last) = (xs.narrow(3, 0, 1)?, xs.narrow(3, w - 1, 1)?);
            let xs = Tensor::cat(&[&first, xs, &last], 3)?;
            let (first, last) = (xs.narrow(2, 0, 1)?, xs.narrow(2, h - 1, 1)?);
            Tensor::cat(&[&first, &xs, &last], 2)
        }
        n => candle::bail!("replication-pad with a size of {n} is not supported"),
    }
}

#[derive(Clone, Debug)]
pub struct Identity;

impl Identity {
    pub fn new() -> Identity {
        Self
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self
    }
}

impl Module for Identity {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        Ok(xs.clone())
    }
}

pub fn sdpa(
    _: &Tensor,
    _: &Tensor,
    _: &Tensor,
    _: Option<&Tensor>,
    _: bool,
    _: f32,
    _: f32,
) -> Result<Tensor> {
    candle::bail!("D29: historical dense normalized attention is inactive")
}
