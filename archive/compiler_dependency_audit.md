# CPU-Only Compiler Dependency and Feature Audit Specification

> **Preserved graph-compiler contract.** This audit applies to the named
> historical compiler toolchain. It does not establish the dependencies or
> serving purity of the unfinished route-native engine. See the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

**Version:** 0.1.0  
**Status:** Normative Specification  
**Issue:** #174 (Feeds #175; related to #160, #157 §6)

---

## 1. Executive Summary

This document specifies the normative CPU-only dependency and feature audit policy for the R⁴ holographic graph compiler toolchain (`uor-r4-graph-compiler`, `uor-r4-graph-certify`, `uor-r4-graph-cli`, and `uor-r4-model-source`).

While Issue #160 enforces zero GPU/accelerator dependencies in the production *runtime* graph (`crates/uor-r4-graph-format`, `crates/uor-r4-core`), this specification protects the *compiler* graph from GPU, tensor, or BLAS accelerator crate contamination across all dependency paths (direct, transitive, build scripts, target-specific, and optional features).

---

## 2. Normative Invariants

1. **CPU-Native Compilation Spine**: The normative compiler pipeline must execute completely on CPU hardware without requiring CUDA, ROCm, Metal compute, OpenCL, Vulkan compute, WebGPU, DirectML, SYCL, or proprietary vendor BLAS accelerator runtimes.
2. **Zero Default GPU Features**: No crate in the compiler tree may declare default workspace features that enable GPU hardware drivers, accelerator runtimes, or vendor BLAS libraries.
3. **Teacher-Backend Isolation**: Optional teacher probe or model extraction integrations that support GPU acceleration must be isolated behind non-default feature flags or external interfaces. The compiler's supported operation, artifact compatibility, and correctness must remain 100% functional without any teacher GPU backend present.
4. **Machine-Checked CI Enforcement**: Any forbidden crate appearing in `Cargo.lock` or workspace `Cargo.toml` manifests triggers an immediate CI build failure.

---

## 3. Forbidden Crate Categories

The compiler dependency graph is audited against the following denylist patterns:

- **CUDA**: `cust`, `cuda-sys`, `cudnn`, `nvml`, `chainer-cuda`
- **ROCm / HIP**: `hip-sys`, `rocm`
- **Metal Compute**: `metal`, `metal-sys`
- **OpenCL / Vulkan**: `opencl3`, `cl-sys`, `vulkano`, `ash`
- **WebGPU / DirectML / SYCL**: `wgpu`, `directml-sys`, `sycl`
- **Tensor Accelerator Frameworks**: `tch`, `torch`, `candle-core`, `ort`, `onnxruntime`
- **Vendor BLAS Accelerators**: `openblas-sys`, `intel-mkl-sys`, `accelerate-src`

---

## 4. Formal Claim Classification

| Claim | Type | Status | Verification |
|:---|:---|:---|:---|
| "The normative compiler builds, tests, and operates without any GPU dependency" | Guarantee | Witnessed | Machine-checked via `CompilerDependencyAuditor` and `scripts/check_compiler_dependencies.py` in CI per commit. |
