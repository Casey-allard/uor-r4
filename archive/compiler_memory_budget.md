# Normative Specification: Compiler Memory-Budget & Backpressure Model for Multicore Compilation

> **Preserved graph-compiler contract.** These limits describe the historical
> TLA/R4G1 compiler. They are reusable research input, not authorization for a
> long route-native run and not the resource contract for #961 or #952. Current
> run discipline is in the
> [Geometric Intelligence Evaluation Policy](geometric_intelligence_evaluation.md).

**Version:** 0.1.0  
**Status:** Normative Specification  
**Issue:** #169  
**Parent:** #122  
**Depends On:** #165 (Executor), #166 (Stage DAG), #168 (Jobs Config)  
**Normative Invariant Source:** Plan §4.1, `crates/uor-r4-graph-compiler/src/induction.rs:140-207`, #59 (shard spill/resume)  

---

## 1. Concurrency-Aware Memory Budget Formula

Multicore parallel execution multiplies peak memory requirements by the active worker count $T$ ($T \ge 1$, derived from Issue #168's `CompilerJobsConfig`). The peak resident set size (RSS) during graph compilation is bounded by:

$$\text{PeakRSS} \approx M_{\text{weights}} + T \times (S_{\text{shard}} + M_{\text{worker\_scratch}}) + M_{\text{stage\_buffer}} \le \text{MemoryBudget}$$

Where:
- $M_{\text{weights}}$: Resident baseline model weights (or mmap footprint).
- $T$: Worker thread count resolved by `CompilerJobsConfig` (Issue #168).
- $S_{\text{shard}}$: Bounded in-flight shard data buffer size.
- $M_{\text{worker\_scratch}}$: Worker-local reusable scratch memory buffer.
- $M_{\text{stage\_buffer}}$: Intermediate stage streaming queue boundary buffer.

---

## 2. In-Flight Backpressure Limiter & Worker Scratch Reuse

1. **Worker-Local Scratch Buffer Reuse**:
   - Each worker thread maintains pre-allocated, reusable scratch buffers. Task execution reuses worker scratch rather than allocating per-task transient memory arrays.

2. **In-Flight Backpressure Limiter**:
   - Stage boundaries employ bounded queues and semaphores (`InFlightBackpressureLimiter`) to cap total in-flight candidate batches and shard processing tasks.
   - Unbounded queue growth is strictly forbidden at every stage transition.

---

## 3. Deterministic Behavior Under Constrained Memory

1. **Output Invariance Above Minimum Budget**:
   - Compiling under any memory budget $B \ge B_{\min}$ yields 100% bit-identical canonical graph artifacts, CIDs, and certificates compared to unconstrained reference compiles (verified via #167's harness).

2. **Typed Error Rejection Below Minimum Budget**:
   - Attempting to compile with $B < B_{\min}$ returns a typed `MemoryBudgetError::BudgetTooSmall` or `MemoryBudgetError::BudgetExceeded` error.
   - Silent quality degradation, non-deterministic OOM crashes, and non-deterministic retry semantics are strictly prohibited.

---

## 4. Verification Harness & Compliance

The compiler crate (`uor-r4-graph-compiler::memory_budget`) provides `CompilerMemoryBudget` and `InFlightBackpressureLimiter`.

The proof model (`uor-r4-proof-model::structural_guarantees`) verifies this guarantee via `verify_compiler_memory_budget_compliance`.

---

## 5. Traceability & No-GPU Statement

- **Formal Claim Classification**: Memory budget model is a **Guarantee** with **Structural** proof status (harness-backed); peak RSS reporting is an **Empirical Criterion**.
- **CPU Portability**: Memory model governs CPU host memory (RSS) only. Zero GPU device memory, CUDA, OpenCL, Metal, Vulkan compute, WebGPU, or tensor accelerator memory assumptions exist in the model.
