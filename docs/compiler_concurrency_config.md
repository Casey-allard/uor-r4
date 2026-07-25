# Normative Specification: Compiler Thread-Pool, Jobs Configuration & Oversubscription Policy

**Version:** 0.1.0  
**Status:** Normative Specification  
**Issue:** #168  
**Parent:** #122  
**Depends On:** #165 (Deterministic Compiler Executor)  
**Normative Invariant Source:** Plan §4.1, `crates/uor-r4-graph-cli`, AGENTS.md command conventions  

---

## 1. Concurrency Controls & Precedence Resolution

The R4 graph compiler provides explicit thread concurrency configuration across all compilation stages. Thread count resolution follows a strict three-tier precedence hierarchy:

```
CLI Argument (--jobs N)  >  Environment Variable (R4_COMPILER_THREADS)  >  Default Policy
```

### 1.1 Resolution Rules

1. **CLI Argument (`--jobs N`)**:
   - Explicitly provided `--jobs N` flag on the CLI surface overrides all environment variables and defaults.
   - `N` must be a positive integer ($N \ge 1$). Passing `N = 0` or invalid string formats returns a typed `JobsConfigError::ZeroJobsForbidden` or `JobsConfigError::InvalidJobCount`.

2. **Environment Variable (`R4_COMPILER_THREADS`)**:
   - Evaluated if `--jobs` is omitted.
   - Parsed as a positive decimal integer string. Invalid formats return a typed `JobsConfigError::InvalidJobCount`.

3. **Default Thread-Count Policy (`DefaultPolicy`)**:
   - Evaluated when neither `--jobs` nor `R4_COMPILER_THREADS` is specified.
   - Calculated as:
     $$\text{DefaultJobs} = \min(\text{AvailableLogicalCPUs}(), 8)$$
   - Capped at 8 worker threads by default to ensure laptop-friendly CPU utilization and avoid runaway resource contention during parallel builds.

---

## 2. Dedicated Rayon Thread-Pool Ownership

To avoid global thread pool coupling, diagnostic interference, or unexpected worker pool contention:

- The compiler executor (`uor-r4-graph-compiler::executor`) instantiates **dedicated custom thread pools** owned by the compilation context.
- Dedicated threads are explicitly named with the prefix `r4-compile-<n>` (e.g. `r4-compile-0`, `r4-compile-1`).
- The compiler **must not rely on Rayon's global thread pool** for pipeline execution.

---

## 3. Oversubscription Handling & Client Isolation

When the compiler operates in multicore parallel mode alongside external backends (e.g., teacher oracle backends, safetensors loaders, or `blake3` parallel hashing):

1. **Teacher Backend Isolation**:
   - Compiler worker thread concurrency is strictly bounded by `CompilerJobsConfig`. External teacher backend requests are dispatched over reader handles without spawning nested compiler worker pools.
2. **Worker Memory Scratch Capping**:
   - The thread count knob `jobs` directly feeds the memory-budget model (Issue #169) to bound in-flight worker scratch allocations ($\text{Workers} \times \text{PerWorkerScratch} \le \text{MemoryBudget}$).

---

## 4. Verification Harness & Compliance

The compiler crate (`uor-r4-graph-compiler::jobs_config`) provides `CompilerJobsConfig`.

The proof model (`uor-r4-proof-model::structural_guarantees`) verifies this guarantee via `verify_compiler_jobs_config_compliance`.

---

## 5. Traceability & Claim Classification

- **Formal Claim Classification**: Concurrency configuration surface is a **Definition**; "Thread count affects time only, never canonical artifact bytes" is a **Guarantee** with **Structural** proof status (verified via #167's harness).
- **CPU Portability**: Concurrency controls are strictly CPU-only (zero GPU device-count, CUDA, OpenCL, Metal, or GPU driver scheduling logic).
