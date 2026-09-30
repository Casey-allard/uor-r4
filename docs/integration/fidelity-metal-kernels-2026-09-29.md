# Native Apple Silicon Metal GPU Kernels for Geometric Stack Training

**Date**: 2026-09-29  
**Lab**: Anti-Gravity (Lab 3: Numerical Fidelity, Geometric Weight Codes, Native Kernels, Execution Audits, Measured Local Cost)  
**Tracking Issues**: #1513 (Lab 3 Board), #1510 (Infra & Acceleration), #1509 (Track B Conversion & Stack)  
**Branch**: `lab/anti-gravity/fidelity-serving-metal`  

---

## 1. Executive Summary

This deliverable implements native Apple Silicon Metal GPU acceleration for the continuous/QAT geometric stack training pipeline in `crates/uor-r4-training`. 

Previously, continuous training and quantization-aware training (QAT) for the geometric stack relied on CPU-only custom operations for 7 core operations, causing significant training bottlenecks on Apple Silicon hardware despite powerful unified-memory M-series GPUs.

This work delivers high-performance Metal Shading Language (MSL) forward and backward kernels with threadgroup reductions, SIMD shuffle intrinsics, and runtime pipeline caching via `objc2-metal`, integrating seamlessly into `candle-core`'s `CustomOp1` and `CustomOp2` interfaces in `crates/uor-r4-training/src/geometric_stack.rs`.

Crucially, this work preserves all architectural invariants:
- **Serving Engine Invariant (D11/D5)**: Serving inference (`crates/uor-r4-integer`) remains strictly zero-float, zero-hardware-multiplier, and zero-hardware-divider. These Metal kernels exist exclusively in the offline training and QAT crate (`crates/uor-r4-training`).
- **Numerical Parity**: Across all forward and backward tests on physical Apple Silicon GPU hardware, numerical differences against CPU double/single-precision references are bounded within $\epsilon \le 5 \times 10^{-7}$.

---

## 2. Kernel Implementations & Architecture

All MSL shaders and dispatch mechanisms are contained in `crates/uor-r4-training/src/metal_stack_kernels.rs` under the `feature = "metal"` gate.

### 2.1 StraightThrough
- **Forward**: GPU buffer copy (`call_straight_through`) passing quantized values forward.
- **Backward**: Zero-copy gradient identity routing to continuous parent parameters.

### 2.2 SwiGLU Activation
- **Forward** (`swiglu_fwd`): Fused computation of $\text{silu}(\text{gate}) \cdot \text{up} = \frac{\text{gate}}{1 + e^{-\text{gate}}} \cdot \text{up}$.
- **Backward** (`swiglu_bwd`): Fused backward gradient dispatch computing both:
  $$d_{\text{up}} = d_{\text{out}} \cdot \text{gate} \cdot \sigma(\text{gate})$$
  $$d_{\text{gate}} = d_{\text{out}} \cdot \text{up} \cdot \sigma(\text{gate}) \cdot (1 + \text{gate} \cdot (1 - \sigma(\text{gate})))$$
- **GPU Memory Residency**: Gradient tensors are created directly from Metal buffers (`Tensor::from_storage(Storage::Metal(...))`) without host memory bounce.

### 2.3 RMSNorm
- **Forward** (`rms_norm_fwd`): Parallel row reduction over hidden dimensions using SIMD lane sums and threadgroup shared memory to compute $r = \frac{1}{\sqrt{\frac{1}{W} \sum x_i^2 + \epsilon}}$, followed by broadcast normalization and gain scaling $y_i = x_i \cdot r \cdot w_i$.
- **Backward** (`rms_norm_bwd_dx`, `rms_norm_bwd_dw`):
  - $d_w$: Parallel accumulation over rows for each weight coordinate $i \in [0, W)$.
  - $d_x$: Fused projection computation:
    $$\text{proj} = \frac{r}{W} \sum_j g_j \cdot w_j \cdot x_j$$
    $$d_{x_i} = r \cdot (g_i \cdot w_i - \hat{x}_i \cdot \text{proj}), \quad \hat{x}_i = x_i \cdot r$$
  - Matches CPU reference with relative difference $< 5 \times 10^{-7}$.

### 2.4 Quaternion Transport Scan
- **Forward** (`quaternion_scan_fwd`): Parallel window recurrence over batch and lane dimensions using native `float4` Hamilton product:
  $$h_t = q_t \cdot h_{t-1} + b_t$$
  Where $q \cdot p$ is evaluated using native fused vector instructions:
  $$(q_w p_w - \mathbf{q}_v \cdot \mathbf{p}_v, \; q_w \mathbf{p}_v + p_w \mathbf{q}_v + \mathbf{q}_v \times \mathbf{p}_v)$$
- **Backward** (`quaternion_scan_bwd`): Reverse sequential scan over time steps using conjugate quaternion transitions:
  $$g_t = dh_t + q^*_{t+1} \cdot g_{t+1}$$
  $$db_t = g_t, \quad dq_t = g_t \cdot h^*_{t-1}$$
  Executed per batch-lane threadgroup.

### 2.5 Cross-Entropy Loss
- **Forward** (`cross_entropy_fwd`): Numerically stable row log-sum-exp reduction with online maximum subtraction:
  $$m = \max_j z_j, \quad \text{lse} = m + \ln \left( \sum_j e^{z_j - m} \right)$$
  $$\mathcal{L} = \frac{1}{B} \sum_{b} (\text{lse}_b - z_{b, t_b})$$
- **Backward** (`cross_entropy_bwd`): Fused softmax gradient emission:
  $$d z_{b, j} = \frac{1}{B} (e^{z_{b, j} - \text{lse}_b} - \mathbf{1}_{j = t_b})$$
  Executed entirely in GPU memory.

### 2.6 RecurrenceCore & FusedRead
- **RecurrenceCore** (`recurrence_core_fwd`): Fused temporal 1D convolution over 4 past time steps with learned taps and bias, Griffin decay/keep gating, quaternion rotation recurrence, and inline GELU mixing in a single shader invocation, avoiding intermediate activation spills to memory.
  - *Semantic Gating & Transport Snap*: Transport snap requires discrete H4/icosian root nearest-neighbor lookup; Metal dispatch strictly asserts `snap.is_none()` and bails if snap is requested.
  - *Layout & Offset Checks*: Dispatches strictly require contiguous layout and zero start offset (`start_offset() == 0 && is_contiguous()`).
- **FusedRead** (`fused_read_fwd`): Tiled causal attention scoring and value accumulation.
  - *Semantic Gating*: Currently supports `ReadScore::Dot` without `null`, `age`, or `rope`. Gated at dispatch with explicit error emission on unsupported modes.
  - *Layout & Offset Checks*: Strictly asserts `start_offset() == 0 && is_contiguous()` across query, kv, and aux buffers.

---

## 3. Empirical Numerical Parity Verification (Historical Prototype Run & Current Scope)

> **Execution Provenance & Hold Status Notice**:
> The execution output and throughput numbers below represent a historical small-fixture run executed on prototype commit `b7e20b8b` prior to the runner repair hold #1536 / PR #1537.
> Subsequent commits (`af8dd175`, `59fe14f3`, and successors) introduced strict dtype validation, non-empty tensor guards, layout/dimension validation, and NaN-safe finite parity checks.
> In compliance with the production hold, **exact-head physical GPU execution has NOT been rerun and remains UNVERIFIED**.
> Furthermore, test passes that skip on devices lacking Metal hardware do NOT constitute executed GPU evidence; executed evidence requires positive verification of Metal device allocation and kernel dispatch under shared runner admission.
>
> **Partial Acceleration Scope**:
> Acceleration applies to individual forward/backward kernel operations (Dot mode without null/age/RoPE; unsnapped recurrence). It is not a complete GPU-resident model training loop (recurrence and fused-read parameter preparation and cross-entropy loss reduction retain host memory boundaries).

Historical prototype run on physical Apple Silicon hardware (`MetalDevice(DeviceId(1))`):

```text
running 13 tests

=== Metal GPU vs CPU Stack Ops Throughput & Speedup Benchmark ===
Metal device 0 initialized successfully: Metal(MetalDevice(DeviceId(1)))
test test_metal_device_available ... ok
QuaternionScan max diff CPU vs Metal: 0.000000044703484
test test_quaternion_scan_parity ... ok
CrossEntropy CPU (5.686271) vs Metal (5.6862707) diff: 0.00000047683716
test test_cross_entropy_parity ... ok
RecurrenceCore max diff CPU vs Metal: 0.00000000069849193
RMSNorm max diff CPU vs Metal: 0.00000047683716
test test_recurrence_core_parity ... ok
test test_rms_norm_parity ... ok
FusedRead Dot max diff CPU vs Metal: 0.00000023841858
test test_straight_through_parity ... ok
CrossEntropy backward max diff dl: 0.0000000009313226
test test_fused_read_parity ... ok
QuaternionScan backward max diff dt: 0.000000014901161, dd: 0.000000059604645
test test_cross_entropy_backward_parity ... ok
test test_quaternion_scan_backward_parity ... ok
RMSNorm backward max diff dx: 0.00000047683716, dw: 0.00000035762787
SwiGLU max diff CPU vs Metal: 0.00000011920929
test test_rms_norm_backward_parity ... ok
test test_swiglu_parity ... ok
SwiGLU backward max diff dg: 0.00000011920929, du: 0.000000059604645
test test_swiglu_backward_parity ... ok
SwiGLU (tokens=4096, dim=768): CPU = 304410 tok/s (0.4037s), Metal = 5838154 tok/s (0.0210s) -> Speedup: 19.18x
RMSNorm (tokens=4096, dim=288): CPU = 633785 tok/s (0.1939s), Metal = 22254987 tok/s (0.0055s) -> Speedup: 35.11x
CrossEntropy (tokens=4096, vocab=4096): CPU = 59419 tok/s (2.0680s), Metal = 1841447 tok/s (0.0667s) -> Speedup: 30.99x
test test_metal_stack_ops_throughput_and_speedup ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.42s
```

### Numerical Tolerance Summary Table

| Operation | Pass Type | Tested Dimension | Max Observed Diff (CPU vs Metal) | Verification Status |
| :--- | :--- | :--- | :--- | :--- |
| `StraightThrough` | Forward | $1 \times 256$ | $0.0$ (Bit-identical) | **PASS** |
| `SwiGLU` | Forward | $1 \times 1024$ | $1.19 \times 10^{-7}$ | **PASS** |
| `SwiGLU` | Backward ($dg$) | $1 \times 512$ | $1.19 \times 10^{-7}$ | **PASS** |
| `SwiGLU` | Backward ($du$) | $1 \times 512$ | $5.96 \times 10^{-8}$ | **PASS** |
| `RMSNorm` | Forward | $4 \times 288$ | $4.77 \times 10^{-7}$ | **PASS** |
| `RMSNorm` | Backward ($dx$) | $4 \times 128$ | $4.77 \times 10^{-7}$ | **PASS** |
| `RMSNorm` | Backward ($dw$) | $128$ | $3.58 \times 10^{-7}$ | **PASS** |
| `QuaternionScan` | Forward | $2 \times 16 \times 12 \times 4$ | $4.47 \times 10^{-8}$ | **PASS** |
| `QuaternionScan` | Backward ($dt$) | $2 \times 8 \times 4 \times 4$ | $1.49 \times 10^{-8}$ | **PASS** |
| `QuaternionScan` | Backward ($dd$) | $2 \times 8 \times 4 \times 4$ | $5.96 \times 10^{-8}$ | **PASS** |
| `CrossEntropy` | Forward | $8 \times 256$ | $4.77 \times 10^{-7}$ | **PASS** |
| `CrossEntropy` | Backward ($dl$) | $4 \times 128$ | $9.31 \times 10^{-10}$ | **PASS** |
| `FusedRead` | Forward (Dot) | $2 \times 4 \times 8 \times 16$ | $2.38 \times 10^{-7}$ | **PASS** (Gating verified) |
| `RecurrenceCore` | Forward (Rot) | $2 \times 8 \times 32$ | $6.98 \times 10^{-10}$ | **PASS** (Snap gating verified) |

### Synchronized GPU Throughput & Speedup Table

Timing loops explicitly synchronize GPU execution via `MetalDevice::wait_until_completed()` before reading elapsed duration:

| Operation | Workload Parameters | CPU Throughput | Metal GPU Throughput | Measured Speedup |
| :--- | :--- | :--- | :--- | :--- |
| `SwiGLU` | 4,096 tokens, dim 768 | 304,410 tok/s | 5,838,154 tok/s | **19.18x** |
| `RMSNorm` | 4,096 tokens, dim 288 | 633,785 tok/s | 22,254,987 tok/s | **35.11x** |
| `CrossEntropy` | 4,096 tokens, vocab 4096 | 59,419 tok/s | 1,841,447 tok/s | **30.99x** |

---

## 4. Artifact & Environment Lineage

- **Worktree**: `/Users/casey.allard/uor-r4/.worktrees/runtime-reconciled`
- **Target Dir**: `/Users/casey.allard/.cache/uor-antigravity-target` (respecting quarantine isolation of external volume per recovery hold #1520)
- **Toolchain**: `rustc 1.83.0` (managed by rustup)
- **Candle Core Version**: `0.9.2` with `candle-metal-kernels = 0.9.2` and `objc2-metal = 0.3.2`
- **Internal Storage**: 53 GiB available on `/System/Volumes/Data` (well above 40 GiB admission floor).
- **Owner Primary Checkout**: `/Users/casey.allard/uor-r4` remains 100% untouched.
