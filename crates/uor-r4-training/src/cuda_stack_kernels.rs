//! Native CUDA forward and backward kernels for the geometric stack ops.
//!
//! The CUDA counterpart of [`crate::metal_stack_kernels`]: one CUDA C source
//! compiled at runtime with NVRTC (no `nvcc` at build time), loaded once per
//! device as a Candle custom module, and launched on the device's stream.
//! The kernels port the Metal kernels' math one-to-one, with three
//! deliberate differences that bring them closer to the exact CPU reference:
//!
//! - where the CPU computes in `f64` (RMSNorm statistics, cross-entropy
//!   log-sum-exp, the recurrence's parameter partials and `log a`, the
//!   read's Lorentz/L2 lifts, excesses and distances, the read backward's
//!   softmax gradient and per-head reductions, and the pointer mixture's
//!   attention, copy mass, branch shares and gradients) the kernels use
//!   `double` too; every other quantity is `float`, as on the CPU;
//! - the U(1) recurrence control and the L2 read control have kernels (Metal
//!   runs them on the host);
//! - the AdamW update uses non-contracting `__fmul_rn`/`__fadd_rn`, so its
//!   f32 operations are the CPU's operations in the CPU's order.
//!
//! The GELU gate keeps the Metal numerics fix: `tanhf` of an argument clamped
//! to `[-20, 20]` (exactly `+-1` in f32 there), never a fast approximation.
//! The source is compiled without `--use_fast_math`, so `expf`, `logf`,
//! `tanhf`, division and `sqrtf` are the accurate CUDA functions.
//!
//! This module contains the crate's only `unsafe` code (kernel launches and a
//! `DeviceRepr`-free argument list); it exists only with the `cuda` feature.

#[cfg(feature = "cuda")]
#[allow(unsafe_code)]
pub mod cuda {
    use std::sync::OnceLock;

    use candle_core::{CudaDevice, Error, Result};
    use cudarc::driver::{CudaSlice, CudaView, LaunchConfig, PushKernelArg};
    use half::bf16;

    /// The Candle custom-module name the f32 stack kernels are loaded under.
    const MODULE: &str = "uor_r4_geometric_stack";
    /// The Candle custom-module name the bf16 activation-storage kernels (the
    /// same source compiled with UOR_STORAGE_BF16) are loaded under.
    const MODULE_BF16: &str = "uor_r4_geometric_stack_bf16";

    /// Kernels D29 removed from every compiled module (13 October 2026): they
    /// carry softmax, sigmoid, softplus, tanh-GELU, exp/log normalisation or
    /// tensor-core contractions, or belong to the softmax read and pointer
    /// mixture families D29 names. Their source is evidence only, in
    /// docs/history/d29-excluded-kernels-2026-10-13/ (never compiled); a launch is refused here,
    /// loudly, rather than failing on a missing symbol.
    pub const HISTORICAL_KERNELS: [&str; 33] = [
        "cross_entropy_grad",
        "cross_entropy_grad_act",
        "cross_entropy_rows",
        "cross_entropy_rows_act",
        "pointer_lift",
        "pointer_rows",
        "pointer_side_grad",
        "pointer_sum",
        "read_dage",
        "read_dbeta",
        "read_dkv",
        "read_dq",
        "read_flash_bwd_key",
        "read_flash_bwd_query",
        "read_flash_bwd_reduce",
        "read_flash_fwd",
        "read_key_self",
        "read_lift",
        "read_mix",
        "read_row_grad_warp",
        "read_softmax_warp",
        "read_tile_inner",
        "recurrence_bwd_direct",
        "recurrence_bwd_post",
        "recurrence_core_bwd",
        "recurrence_core_fwd",
        "recurrence_log_a",
        "recurrence_out",
        "recurrence_param_reduce",
        "recurrence_prep",
        "recurrence_scan_bwd",
        "swiglu_bwd",
        "swiglu_fwd",
    ];

    /// The ahead-of-time PTX of `cuda_stack.cu` + `spiralcore_fbs.cu`
    /// (build.rs: nvcc, `-ftz=false -prec-div=true -prec-sqrt=true`, the
    /// options the NVRTC build used). NVRTC is banned (owner, 13 October 2026).
    const PTX_F32: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda_stack_f32.ptx"));
    const PTX_BF16: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda_stack_bf16.ptx"));

    /// The storage of the activation buffers a kernel launch reads and writes.
    /// Parameters, statistics, scores, partials and carried states are f32 or
    /// f64 in both storages.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Storage {
        /// Every buffer is f32: the source compiled without UOR_STORAGE_BF16.
        F32,
        /// Activation buffers are bf16 (`act_t` is 16 bits wide): the source
        /// compiled with UOR_STORAGE_BF16.
        Bf16,
    }

    impl Storage {
        /// The Candle custom module the storage's PTX is loaded as.
        pub fn module(self) -> &'static str {
            match self {
                Storage::F32 => MODULE,
                Storage::Bf16 => MODULE_BF16,
            }
        }
    }

    /// The stack kernels' PTX for `storage`, compiled ahead of time by nvcc
    /// (the bf16 module with UOR_STORAGE_BF16); no runtime compilation.
    fn ptx(storage: Storage) -> Result<&'static str> {
        Ok(match storage {
            Storage::F32 => PTX_F32,
            Storage::Bf16 => PTX_BF16,
        })
    }

    /// Both ahead-of-time modules are present and carry PTX (no device or
    /// toolkit needed at run time; nvcc ran at build time).
    pub fn compile_check() -> Result<()> {
        for storage in [Storage::F32, Storage::Bf16] {
            if !ptx(storage)?.contains(".entry") {
                return Err(Error::Msg(format!(
                    "the ahead-of-time {storage:?} stack module has no kernel entry"
                )));
            }
        }
        Ok(())
    }

    /// One kernel argument, in parameter order.
    pub enum Arg<'a> {
        /// An f32 device buffer (read, written or both).
        F(CudaView<'a, f32>),
        /// A bf16 activation buffer (read, written or both): the same device
        /// pointer a kernel's `act_t*` parameter takes in the bf16 module.
        B(CudaView<'a, bf16>),
        /// An f64 device buffer.
        D(CudaView<'a, f64>),
        /// A u32 device buffer.
        U(CudaView<'a, u32>),
        U32(u32),
        /// `ReadDims` by value.
        Dims([u32; 8]),
        /// `Geom` by value.
        Geom([u32; 7]),
        /// `AdamConstants` (ten f32 bit patterns) by value.
        Adam([u32; 10]),
    }

    impl<'a> Arg<'a> {
        pub fn f(slice: &'a CudaSlice<f32>) -> Self {
            Arg::F(slice.as_view())
        }

        pub fn b(slice: &'a CudaSlice<bf16>) -> Self {
            Arg::B(slice.as_view())
        }

        pub fn d(slice: &'a CudaSlice<f64>) -> Self {
            Arg::D(slice.as_view())
        }

        pub fn u(slice: &'a CudaSlice<u32>) -> Self {
            Arg::U(slice.as_view())
        }
    }

    /// A zero-filled device buffer of `len` elements (at least one, so a
    /// placeholder argument is always a valid pointer).
    pub fn zeros<T>(device: &CudaDevice, len: usize) -> Result<CudaSlice<T>>
    where
        T: cudarc::driver::DeviceRepr + cudarc::driver::ValidAsZeroBits,
    {
        device.alloc_zeros::<T>(len.max(1))
    }

    /// A device buffer of `len` elements (at least one) whose contents are
    /// unspecified, for an output the following kernels write in full before
    /// anything reads it: it skips [`zeros`]'s fill. Each call site names the
    /// kernel that covers the buffer; a buffer a kernel writes only in part
    /// (or not at all in some configuration) and later reads must stay
    /// [`zeros`].
    ///
    /// `UOR_CUDA_POISON_UNINIT=1` (diagnostic) fills each such buffer with
    /// NaN words first, so a kernel that leaves an element unwritten and a
    /// later read of it surface as nonfinite values in the parity tests.
    pub fn uninit<T>(device: &CudaDevice, len: usize) -> Result<CudaSlice<T>>
    where
        T: cudarc::driver::DeviceRepr + cudarc::driver::ValidAsZeroBits,
    {
        // SAFETY: `alloc` returns device memory that is not initialized.
        // The element types used here (f32, f64, u32) have no invalid bit
        // patterns, so unspecified device bytes are never undefined
        // behaviour on the host; callers pass only buffers whose every
        // element their kernels write before any read.
        let slice = unsafe { device.alloc::<T>(len.max(1)) }?;
        if poison_uninit() {
            let words = slice.len() * std::mem::size_of::<T>() / 4;
            // SAFETY: the view covers `words` whole 4-byte words inside the
            // allocation (f32, f64 and u32 sizes are multiples of 4), and
            // any bit pattern is a valid u32.
            let view = unsafe { slice.transmute::<u32>(words) }
                .ok_or_else(|| Error::Msg("poison view exceeds its buffer".into()))?;
            let size = u32::try_from(words)
                .map_err(|_| Error::Msg("poisoned buffer exceeds u32 words".into()))?;
            launch(
                device,
                "poison_words",
                words,
                &[Arg::U(view), Arg::U32(size)],
            )?;
        }
        Ok(slice)
    }

    /// `UOR_CUDA_POISON_UNINIT=1`: [`uninit`] buffers start as NaN words.
    fn poison_uninit() -> bool {
        static POISON: OnceLock<bool> = OnceLock::new();
        *POISON.get_or_init(|| std::env::var("UOR_CUDA_POISON_UNINIT").is_ok_and(|v| v == "1"))
    }

    /// Threads per block of a 1-D launch.
    const BLOCK: usize = 256;

    /// Launches `name` over `threads` threads (a 1-D grid of 256-thread blocks).
    pub fn launch(device: &CudaDevice, name: &str, threads: usize, args: &[Arg<'_>]) -> Result<()> {
        launch_storage(device, Storage::F32, name, threads, args)
    }

    /// [`launch`] from the bf16 module.
    pub fn launch_bf16(
        device: &CudaDevice,
        name: &str,
        threads: usize,
        args: &[Arg<'_>],
    ) -> Result<()> {
        launch_storage(device, Storage::Bf16, name, threads, args)
    }

    fn launch_storage(
        device: &CudaDevice,
        storage: Storage,
        name: &str,
        threads: usize,
        args: &[Arg<'_>],
    ) -> Result<()> {
        if threads == 0 {
            return Ok(());
        }
        let blocks = threads.div_ceil(BLOCK);
        dispatch(device, storage, name, (blocks, 1, 1), (BLOCK, 1, 1), args)
    }

    /// Launches `name` over `groups` blocks of `group` threads each.
    pub fn launch_groups(
        device: &CudaDevice,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        launch_groups_storage(device, Storage::F32, name, groups, group, args)
    }

    /// [`launch_groups`] from the bf16 module.
    pub fn launch_groups_bf16(
        device: &CudaDevice,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        launch_groups_storage(device, Storage::Bf16, name, groups, group, args)
    }

    fn launch_groups_storage(
        device: &CudaDevice,
        storage: Storage,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        if groups.0 * groups.1 * groups.2 == 0 {
            return Ok(());
        }
        dispatch(device, storage, name, groups, group, args)
    }

    fn dim(value: usize, limit: usize, what: &str) -> Result<u32> {
        if value == 0 || value > limit {
            return Err(Error::Msg(format!(
                "CUDA launch {what} {value} outside 1..={limit}"
            )));
        }
        Ok(value as u32)
    }

    fn dispatch(
        device: &CudaDevice,
        storage: Storage,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        dispatch_module(
            device,
            storage.module(),
            ptx(storage)?,
            name,
            groups,
            group,
            args,
        )
    }

    /// One thread per element of `threads` from an **ahead-of-time** PTX
    /// module (nvcc at build time, `build.rs`), loaded by the driver under its
    /// own module name: no runtime compiler (owner, 13 October 2026: NVRTC is
    /// banned project-wide) and no kernel shared with the stack module.
    pub fn launch_prebuilt(
        device: &CudaDevice,
        module: &str,
        ptx: &str,
        name: &str,
        threads: usize,
        args: &[Arg<'_>],
    ) -> Result<()> {
        if threads == 0 {
            return Ok(());
        }
        let blocks = threads.div_ceil(BLOCK);
        dispatch_module(
            device,
            module,
            ptx,
            name,
            (blocks, 1, 1),
            (BLOCK, 1, 1),
            args,
        )
    }

    fn dispatch_module(
        device: &CudaDevice,
        module: &str,
        ptx: &str,
        name: &str,
        groups: (usize, usize, usize),
        group: (usize, usize, usize),
        args: &[Arg<'_>],
    ) -> Result<()> {
        if HISTORICAL_KERNELS.contains(&name) {
            return Err(Error::Msg(format!(
                "D29: `{name}` is a historical kernel excluded from every compiled module \
                 (softmax/sigmoid/exp-family or tensor-core content); its configuration is not \
                 admitted for new runs"
            )));
        }
        let config = LaunchConfig {
            grid_dim: (
                dim(groups.0, i32::MAX as usize, "grid x")?,
                dim(groups.1, 65_535, "grid y")?,
                dim(groups.2, 65_535, "grid z")?,
            ),
            block_dim: (
                dim(group.0, 1024, "block x")?,
                dim(group.1, 1024, "block y")?,
                dim(group.2, 64, "block z")?,
            ),
            shared_mem_bytes: 0,
        };
        let function = device.get_or_load_custom_func(name, module, ptx)?;
        let started = profiling().then(std::time::Instant::now);
        let mut builder = function.builder();
        for arg in args {
            match arg {
                Arg::F(view) => {
                    builder.arg(view);
                }
                Arg::B(view) => {
                    builder.arg(view);
                }
                Arg::D(view) => {
                    builder.arg(view);
                }
                Arg::U(view) => {
                    builder.arg(view);
                }
                Arg::U32(value) => {
                    builder.arg(value);
                }
                Arg::Dims(words) => {
                    builder.arg(words);
                }
                Arg::Geom(words) => {
                    builder.arg(words);
                }
                Arg::Adam(words) => {
                    builder.arg(words);
                }
            }
        }
        // SAFETY: every kernel in the ahead-of-time stack module and in each prebuilt
        // module takes exactly the arguments its Rust caller passes, in order (device pointers for
        // buffers, `uint`s, and `ReadDims`/`Geom`/`AdamConstants` structs of
        // 4-byte words matching the `[u32; N]` arrays); callers size every
        // buffer for the kernel's index range and bounds-check the grid.
        unsafe { builder.launch(config) }.map_err(|error| Error::Cuda(Box::new(error)))?;
        if let Some(started) = started {
            device
                .cuda_stream()
                .synchronize()
                .map_err(|error| Error::Cuda(Box::new(error)))?;
            eprintln!("cuda-kernel {name} {} us", started.elapsed().as_micros());
        }
        Ok(())
    }

    /// `UOR_CUDA_PROFILE=1` synchronizes after each launched kernel and
    /// reports its wall time on stderr (diagnostic only; it serializes the GPU).
    fn profiling() -> bool {
        static PROFILE: OnceLock<bool> = OnceLock::new();
        *PROFILE.get_or_init(|| std::env::var_os("UOR_CUDA_PROFILE").is_some())
    }
}
