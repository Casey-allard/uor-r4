# Hybrid Thread-Policy Recheck — v1

## Purpose

Re-evaluate whether `torch.set_num_threads(1)` is still optimal on the
current hybrid S¹ × R⁺ representation. The original thread policy was
established on an earlier one-hot variant with different parameter count
and forward-pass shape.

## Locked Configuration

| Item | Value |
|------|-------|
| device | cpu |
| D_HIDDEN | 32 |
| batch_size | 256 |
| D | 24 |
| representation | hybrid angular+radial (D_TAU=12) |
| LR | 0.02 |
| budget | 3000 steps |
| runs per thread count | 3 |
| torch version | 2.11.0 |

## Thread Counts Tested

  [1, 2, 4, 8]

## Summary by Thread Count

| Threads | Mean SPS | Std SPS | Mean Runtime | Mean Solve Wall | All Solved |
|---------|----------|---------|--------------|-----------------|------------|
| 1 | 108.8 | 0.2 | 27.6s | 19.9s | ✓ |
| 2 | 94.7 | 0.7 | 31.7s | 22.8s | ✓ |
| 4 | 89.2 | 1.0 | 33.6s | 24.1s | ✓ |
| 8 | 76.0 | 0.6 | 39.5s | 28.5s | ✓ |

## Relative Performance (vs 1 thread)

| Threads | SPS Ratio | Runtime Ratio | Solve Wall Ratio |
|---------|-----------|---------------|------------------|
| 1 | 1.000 | 1.000 | 1.000 |
| 2 | 0.870 | 1.149 | 1.147 |
| 4 | 0.820 | 1.220 | 1.214 |
| 8 | 0.698 | 1.433 | 1.435 |

## Per-Run Detail

| Threads | Run | Runtime(s) | SPS | Solve Wall(s) | Final Acc | α₀ | T-Frac |
|---------|-----|------------|-----|---------------|-----------|-----|--------|
| 1 | 0 | 27.6 | 108.8 | 18.3 | 1.0000 | 0.8617 | 0.373 |
| 1 | 1 | 27.6 | 108.6 | 23.0 | 1.0000 | 0.8521 | 0.629 |
| 1 | 2 | 27.5 | 109.1 | 18.3 | 1.0000 | 0.8628 | 0.828 |
| 2 | 0 | 31.5 | 95.2 | 21.0 | 1.0000 | 0.8617 | 0.373 |
| 2 | 1 | 32.0 | 93.9 | 26.4 | 1.0000 | 0.8521 | 0.629 |
| 2 | 2 | 31.5 | 95.1 | 21.1 | 1.0000 | 0.8628 | 0.828 |
| 4 | 0 | 33.2 | 90.4 | 21.9 | 1.0000 | 0.8617 | 0.373 |
| 4 | 1 | 33.8 | 88.8 | 28.1 | 1.0000 | 0.8521 | 0.629 |
| 4 | 2 | 33.9 | 88.5 | 22.4 | 1.0000 | 0.8628 | 0.828 |
| 8 | 0 | 39.2 | 76.6 | 25.9 | 1.0000 | 0.8617 | 0.373 |
| 8 | 1 | 39.7 | 75.5 | 33.1 | 1.0000 | 0.8521 | 0.629 |
| 8 | 2 | 39.6 | 75.8 | 26.6 | 1.0000 | 0.8628 | 0.828 |

## Explicit Answers

### 1. Is `num_threads=1` still optimal on the current hybrid path?

**Yes.** `num_threads=1` remains the fastest configuration on the current
hybrid S¹ × R⁺ representation. The original policy still holds.

### 2. What is the new best CPU thread policy?

`torch.set_num_threads(1)` — 108.8 sps

### 3. Does the flat ~100% CPU behavior materially change?

With `num_threads=1`, the process uses one hardware thread at ~100% of one core
(reported as ~100% in Activity Monitor on macOS, which uses single-core scaling).
With more threads, total CPU% may increase but throughput decreases due to
synchronization overhead on tiny operations — the workload is too small to
benefit from parallelism.

## Interpretation

The hybrid representation has **smaller parameter matrices** than one-hot (971 vs
~1700 params), making each operation even tinier. The D=24 sequential recurrence
with (16,32) matmuls, (32,6) matmuls, and (256,6)×(256,6,8) einsums produces
operations too small to benefit from multi-threaded BLAS/LAPACK dispatch.

Thread overhead (ReadyQueue wake/sleep, condition_variable signaling, cache
coherence traffic) dominates over any parallelism gain at this operation size.

## Honesty Section

### What Is Proven

- Thread sweep covers {1, 2, 4, 8} with 3 runs each
- Same seed per run_idx across thread counts ensures identical computation
- SPS and solve-wall measurements are direct wall-clock measurements

### What Is Still Uncertain

- Whether the thread policy changes at larger batch sizes (B≥1024)
- Whether the policy changes with larger D_HIDDEN or deeper models
- Whether future representation changes (larger D_TAU) would shift the crossover
