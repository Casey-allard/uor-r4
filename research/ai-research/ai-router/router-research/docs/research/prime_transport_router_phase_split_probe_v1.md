# Phase-Split Runtime Probe — v1

## Purpose

Decompose runtime into BFS/pre-cache and steady-state training phases,
then re-evaluate thread policy on uncontaminated steady-state measurements.

## Concern Being Tested

The previous thread-policy sweep might have been confounded if BFS or
pre-cache work leaked into training timing, artificially favoring fewer threads.

## Locked Configuration

| Item | Value |
|------|-------|
| device | cpu |
| D_HIDDEN | 32 |
| batch_size | 256 |
| D | 24 |
| representation | hybrid angular+radial (D_TAU=12) |
| torch version | 2.11.0 |

## Phase 1: Pre-Cache Breakdown

| Sub-phase | Wall Time | CPU User | CPU Sys | CPU Eff | Nature |
|-----------|-----------|---------|---------|---------|--------|
| build_warmup_pool | 1.63s | 1.61s | 0.01s | 1.00 | POOL_SIZE=4000 |
| bfs_warm_up | 87.24s | 85.87s | 1.11s | 1.00 | 343665 states discovered |
| build_state_tables | 2.95s | 2.88s | 0.06s | 1.00 | TN=[343665, 6, 21], TR=[343665, 6] |
| angular_conversion | 0.24s | 0.20s | 0.04s | 1.00 | TN_ang=[343665, 6, 8] |
| **TOTAL** | **92.1s** | | | | |

### Pre-cache interpretation

BFS dominates pre-cache at 87.2s (95% of 92.1s total). It is pure single-threaded Python — a BFS loop
over a deque, populating dictionaries. There are **no background threads** and
**no persistent processes** — once BFS returns, all computation is complete.
The `build_state_tables` step converts dictionaries to numpy/torch tensors.

**There is no persistent BFS/pre-cache process running during training.**
The previous thread-policy sweep already excluded BFS from training timing
(training timer starts after BFS completes). However, this probe also tests
for cold-start effects (page faults, cache misses on first access to 173MB TN tensor).

## Phase 2: Cold vs Steady-State Training

| Threads | Phase | SPS (run 0) | SPS (run 1) | Mean SPS | CPU Eff |
|---------|-------|-------------|-------------|----------|---------|
| 1 | cold | 101.0 | 92.2 | 96.6 | 0.99 |
| 1 | steady | 93.0 | 99.8 | 96.4 | 0.99 |
| 2 | cold | 90.7 | 92.0 | 91.3 | 1.04 |
| 2 | steady | 89.9 | 95.0 | 92.5 | 1.04 |
| 4 | cold | 91.4 | 90.2 | 90.8 | 1.12 |
| 4 | steady | 91.4 | 90.2 | 90.8 | 1.12 |
| 8 | cold | 79.4 | 68.6 | 74.0 | 1.25 |
| 8 | steady | 72.3 | 76.7 | 74.5 | 1.26 |

### Cold-Start Penalty

| Threads | Cold SPS | Steady SPS | Δ SPS | Δ % |
|---------|----------|------------|-------|-----|
| 1 | 96.6 | 96.4 | -0.2 | -0.2% |
| 2 | 91.3 | 92.5 | +1.1 | +1.2% |
| 4 | 90.8 | 90.8 | +0.0 | +0.0% |
| 8 | 74.0 | 74.5 | +0.5 | +0.7% |

## Corrected Thread-Policy (Steady-State Only)

| Threads | Mean Steady SPS | Std | vs 1-thread |
|---------|-----------------|-----|-------------|
| 1 | 96.4 | 4.8 | 1.000× |
| 2 | 92.5 | 3.6 | 0.959× |
| 4 | 90.8 | 0.8 | 0.942× |
| 8 | 74.5 | 3.1 | 0.773× |

## CPU Efficiency by Thread Count

CPU efficiency = (cpu_user + cpu_sys) / wall_time. Values > 1.0 indicate
multiple cores being used; values near 1.0 indicate single-core saturation.

| Threads | Steady Eff | Cold Eff | Warmup Eff | First-Step Eff |
|---------|-----------|---------|------------|----------------|
| 1 | 0.99 | 0.99 | 0.99 | 0.92 |
| 2 | 1.04 | 1.04 | 1.04 | 1.04 |
| 4 | 1.12 | 1.12 | 1.12 | 1.13 |
| 8 | 1.26 | 1.25 | 1.26 | 1.28 |

## Explicit Answers

### 1. Is there a persistent BFS / pre-cache phase consuming ~1 core?

**No.** BFS is pure single-threaded Python. It runs to completion (~84s),
then returns. There is no background BFS process during training.
`build_state_tables` adds ~3.0s, `angular_conversion` ~0.2s.
All are sequential and fully complete before training starts.

### 2. How much wall time does pre-cache account for?

**92.1s total** (BFS=87.2s dominant). This is a fixed
one-time cost, independent of thread count, and was already excluded from
training SPS measurements in the previous recheck.

### 3. Does `num_threads=1` remain optimal for steady-state training?

**Yes.** Even on uncontaminated steady-state measurements with warmed caches,
`num_threads=1` (96.4 sps) remains fastest.

### 4. Is BFS itself now the main systems bottleneck?

BFS takes 87.2s. A full 3000-step training run takes ~31s
at 96 sps. BFS is 74% of total end-to-end wall time for a single run —
**the dominant fixed cost**. However, for multi-run sweeps (e.g., 9-variant
M3 probe), BFS is shared once and amortized: 87s / (87 + 9×31) = 24%.

### 5. Should future comparisons exclude BFS time?

**Yes**, and they already do. All previous SPS measurements start timing
after BFS/table-build completes. This probe confirms no contamination.

## Was the Previous Thread-Policy Conclusion Confounded?

**No.** The previous recheck (v1) already:

1. Ran BFS once with `threads=1` before any training
2. Started training timers with `time.perf_counter()` inside `train_run()`
3. Reported SPS = BENCH_BUDGET / training_runtime (excluding BFS)

This probe additionally verifies:

- No cold-start penalty large enough to change the ranking
- No persistent background process from BFS
- CPU efficiency confirms single-core saturation at `threads=1`
  and wasteful multi-core overhead at higher thread counts

The previous conclusion — `num_threads=1` is optimal — is **clean and correct**.

## Honesty Section

### What Is Proven

- BFS is a one-shot pure-Python phase with no background persistence
- Pre-cache wall time is ~84s (BFS dominant), fully complete before training
- Cold→steady SPS difference exists but does not change thread-count ranking
- CPU efficiency at threads=1 is ~1.0 (single-core saturated)
- CPU efficiency increases with more threads but SPS decreases (overhead > parallelism)
- Previous thread-policy measurements were not confounded by BFS

### What Remains Uncertain

- Whether OS-level memory pressure from the 173MB TN tensor causes
  intermittent slowdowns under memory contention (not observed here)
- Whether the ~100% Activity Monitor reading comes from the training
  loop alone or from background macOS services triggered by memory usage
- Whether BFS time could be reduced via Cython, multiprocessing, or caching to disk
