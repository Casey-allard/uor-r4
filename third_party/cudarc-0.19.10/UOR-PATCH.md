# UOR prebuilt module loader patch

Upstream baseline: `cudarc` 0.19.10, copied from its Cargo registry source.
Upstream licensing and original manifests remain intact. The owning delivery
records the upstream archive/checksum separately.

`cudarc::ptx::Ptx` now owns prebuilt PTX text, module-file paths, or binary CUBIN.
`cudarc::driver::Ptx` re-exports that type, and
`CudaContext::load_module(Ptx)` is available with `driver` alone. Existing
`context.load_module(ptx.into())` calls retain their source compatibility.
Loading never calls a runtime CUDA compiler or retries through NVRTC. The CUDA
driver may translate PTX assembly for the device; that driver load is distinct
from compiling CUDA C/C++ source using NVRTC.

The historical, feature-gated NVRTC implementation remains inactive source.
Its `cudarc::nvrtc::Ptx` compatibility re-export uses the same carrier type.
A project-specific compile-time guard rejects `nvrtc`, `cublas`, `cublaslt`,
`cudnn`, every direct cuDNN version feature, `cutensor`, `cusolver`,
`cusolvermg`, and `cusparse`, including activation through
Cargo feature unification. Upstream default features therefore fail closed;
the active graph must disable them. `driver`, `curand`, `f16`, and `f8` remain
available. The upstream build script's NVRTC link directives remain
feature-gated, but a prohibited-feature build cannot produce this library.

The vendored float8 0.6.1 belongs to Candle. Cudarc independently retains the
registry float8 0.7.0 data-type dependency; the vendored packages are not the
complete transitive dependency closure. The CUDA qualification records the
resolved graph and selected features rather than inferring them from patches.

Driver context binding, module ownership/unloading, and CUDA error propagation
are preserved. `Ptx::validate` rejects invalid host metadata before binding a
CUDA context. Interior NULs in PTX or file paths, non-UTF-8 paths, and empty
binary inputs return `CUDA_ERROR_INVALID_VALUE` instead of panicking or passing
an empty buffer to the driver. The upstream `Ptx::to_src` convenience method
keeps its documented panic behavior; loading does not use that method.

Driver documentation uses prebuilt assembly, and the upstream half-kernel test
now requires both `f16` and `nvrtc` rather than importing a disabled compiler.
New carrier tests operate on metadata only, without any CUDA context or model.
Composed CUDA builds and device qualification belong to the owning delivery;
this patch alone is not a model, training, or full dependency-legality result.
