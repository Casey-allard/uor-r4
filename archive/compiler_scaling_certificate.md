# Compiler Parallelism Benchmarks and Scaling Certificate Specification

> **Preserved graph-compiler certificate design.** It does not prove that the
> active route-native path uses four workers or justify running a large compile.
> That path must measure its own bottleneck and useful parallel work under #963.
> See the [documentation map](README.md).

**Version:** 0.1.0  
**Status:** Normative Specification  
**Issue:** #175 (Extends #27, #77, #161; measures #165–#174)

---

## 1. Executive Summary

This document specifies the empirical compiler parallelism benchmark harness and scaling certificate format (`CompilerScalingReport`) for the R⁴ holographic graph compiler toolchain.

While Issues #165–#173 provide the structural implementation of parallel compiler stages and Issue #167 guarantees thread-count invariance (exact byte-identical outputs across all $T \ge 1$), this specification establishes the empirical measurement framework that certifies wall-clock speedup, CPU utilization, peak RSS, and throughput across multicore thread sweeps ($T=1, 2, 4, N$) on CPU-only environments.

---

## 2. Normative Invariants & Empirical Criteria

1. **Byte-Equality Premise**: Every parallel benchmark run must verify that the compiled graph artifact is 100% byte-identical to the sequential reference output ($T=1$), confirming Issue #167 compliance before recording empirical timing values.
2. **Multi-Thread Sweep Matrix**: Parallel benchmark runs evaluate thread sweeps ($T=1, 2, 4, N$) across representative small, medium, and large dataset fixtures, plus a memory-constrained configuration (Issue #169).
3. **5-Way Bottleneck Classification Taxonomy**: Every compiler stage is classified into exactly one of 5 categories:
   - `Scaling`: Parallel speedup $S(T) \ge 0.7 \times T$ with high efficiency.
   - `BandwidthLimited`: Memory bandwidth saturation caps parallel speedup.
   - `TeacherLimited`: Teacher probe batch latency dominates runtime.
   - `SequentialFinalization`: Inherent sequential spine (Issue #166 canonical packing/spill).
   - `OversubscriptionPenalty`: Thread count exceeds physical cores, degrading efficiency.
4. **CPU-Only Benchmarking Environment**: Benchmarking executes strictly on CPU-only runners without CUDA, Metal, OpenCL, Vulkan, WebGPU, SYCL, or GPU accelerator drivers (Issue #174).

---

## 3. Metric Definitions

- **Speedup $S(T)$**: $S(T) = \frac{T_{\text{wall}}(1)}{T_{\text{wall}}(T)}$
- **Efficiency $E(T)$**: $E(T) = \frac{S(T)}{T}$
- **Throughput $\Phi(T)$**: Items processed per second ($\text{items/sec}$).
- **Peak RSS $M_{\text{peak}}$**: Maximum resident set size in bytes during compilation.

---

## 4. Formal Claim Classification

| Claim | Type | Status | Verification |
|:---|:---|:---|:---|
| "Parallel compilation yields speedup $S(T)$ over sequential baseline" | Empirical Criterion | Empirical | Measured via `CompilerScalingReport` on pinned CPU-only benchmark runs. |
| "Parallel compiler outputs are 100% byte-identical to sequential" | Guarantee | Witnessed | Verified via `reproducibility.rs` harness attached to every scaling report. |
