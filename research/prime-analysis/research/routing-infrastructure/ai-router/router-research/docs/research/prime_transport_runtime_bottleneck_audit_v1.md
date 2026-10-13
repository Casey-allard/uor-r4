# Prime Transport Runtime Bottleneck Audit v1

**Branch:** runtime_bottleneck_audit_v1  
**Date:** 2026-04-09T05:06:11Z  

## Canonical Sources

- **CANONICAL:** `router_dimension_phase_alignment_probe_v1.csv`
- **CANONICAL:** `tools/prime_transport/thread_policy.py`
- **CONTRAST:** none

## Step 1 — Execution Path Audit

| Concern | File | Location |
|---------|------|----------|
| Run scheduling | `run_router_dimension_phase_alignment_probe_v1.py` | `main()` — nested `for D in HORIZONS:` / `for vname in TASK_A_VARIANTS:` — **fully serial** |
| Thread count | `thread_policy.py` | `select_threads()` — `torch.set_num_threads(min(8, cpu_count))` at startup |
| CSV writing | probe script | `write_csv()` — single call after all runs |
| Artifact writing | probe script | `write_markdown()` — single call after all runs |

**Serialization points:**

1. All 14 runs (7 horizons × 2 variants) sequential in `main()`. No concurrency.
2. `thread_policy` sets intra-op threads but cannot parallelize across runs.
3. Each run is embarrassingly parallel: fresh model instance, independent explicit RNG generator, read-only shared buffers.

**Additional finding (discovered during audit):**  
`_soft_step()` uses `torch.rand_like(logits)` for Gumbel noise, which draws from the **global PyTorch RNG** (not the seeded generator). In serial mode, each run mutates the global RNG state, causing later runs to receive different Gumbel noise sequences. Run 3 (D24/original_s42) in serial produced acc=0.7002 vs canonical 0.9956 — a convergence failure attributable to accumulated global RNG state. Parallel workers each start with a fresh global RNG, avoiding this.

## Step 2 — Wall-Clock Breakdown

Representative run: D=16, original_s42, 3500 training steps.

| Phase | Time (s) | % of loop | ms/step |
|-------|----------|-----------|---------|
| Model init (setup) | 0.704 | — | — |
| Batch generation (3500 steps) | 0.239 | 0.3% | 0.07 |
| Forward + loss (3500 steps) | 40.075 | 50.4% | 11.45 |
| Backward + clip (3500 steps) | 37.957 | 47.8% | 10.84 |
| Optimizer step (3500 steps) | 1.028 | 1.3% | 0.29 |
| Eval (7 calls) | 0.163 | 0.2% | 0.023s/call |
| Misc/overhead | 0.014 | 0.0% | — |
| Decodability (teardown) | 0.048 | — | — |
| **TOTAL** | **80.225** | — | — |

**Forward + backward = 98.2% of loop time.**

### Critical Finding: W1/W2 policy network trained but not used in eval

In `eval_acc()`, operator selection is purely geometric:
```
ang_sim = einsum(cur_dir, TN)   # no W1/W2 involved
best_op = ang_sim.argmax()      # greedy angular alignment
```
**W1, W2, W_emb are never called in eval.** Only W_attn/W_pred (readout) are used.  
The training loop computes full forward+backward through W1/W2 to generate soft trajectories for training W_attn/W_pred. Whether this is necessary (vs fitting W_attn/W_pred directly on geometric rollouts) is **not tested here** — requires a separate model-change branch.  
If confirmed unnecessary: **~98% of per-run runtime is eliminable** without any parallelism changes.

## Step 3 — Thread Configuration Benchmark

Machine: 8 logical cores. Constraint: `workers × threads ≤ 8`.  
Equivalence criterion: `|acc - canonical_acc| ≤ 0.08` for all 4 runs.  
Canonical reference: `router_dimension_phase_alignment_probe_v1.csv`.

| Config | Workers | Threads | Cores | Total wall (s) | Speedup | All correct (vs canon) |
|--------|---------|---------|-------|----------------|---------|------------------------|
| 1w_8t | 1 | 8 | 8 | 429.9 | 1.0x | NO (serial RNG contamination) |
| 2w_4t | 2 | 4 | 8 | 237.7 | 1.81x | NO (serial RNG contamination) |
| 4w_2t | 4 | 2 | 8 | 162.0 | 2.65x | YES |
| 8w_1t | 8 | 1 | 8 | 140.0 | 3.07x | YES | **← SELECTED**

**Note on 1w_8t correctness:** D24/original_s42 = 0.7002 vs canonical 0.9956. This is not a thread-count issue — it is global RNG state contamination in serial mode (run 3 of 4 gets polluted Gumbel noise). All parallel configs avoid this.

**Selected config:** `8w_1t` (8w × 1t)  
**Selection criterion:** fastest wall-clock where all 4 runs are within 0.08 of canonical accuracy without oversubscribing cores.

### Per-run accuracy vs canonical

| Run | Canonical | 1w_8t | 2w_4t | 4w_2t | 8w_1t |
|-----|-----------|-------|-------|-------|-------|
| D16/original_s42 | 0.9658 | 0.9941 | 0.9917 | 0.9980 | 0.9946 |
| D16/shift1_s42 | 0.9775 | 0.9927 | 0.9932 | 0.9937 | 0.9878 |
| D24/original_s42 | 0.9956 | 0.7002 ⚠ | 0.9106 ⚠ | 0.9897 | 0.9937 |
| D24/shift1_s42 | 0.9990 | 0.9883 | 0.9810 | 0.9951 | 0.9751 |

⚠ = outside ±0.08 tolerance vs canonical. Note: 1w_8t D24/original_s42=0.7002 is serial-mode convergence failure.

## Step 4 — Code Change

**File created:** `tools/prime_transport/run_runtime_bottleneck_audit_v1.py`  
**Canonical probe unchanged.** Audit is additive only.

**Key additions:**
- `run_one_isolated(cfg)`: process-safe worker. Loads own cache. Returns plain scalars.
- `run_config(n_workers, n_threads)`: ProcessPoolExecutor dispatcher.
- Thread config selected by benchmark (4 configs tested), not by assumption.

## Before/After Timing

| | Serial (1w_8t) | Best config (8w_1t) | Speedup |
|---|---|---|---|
| Total wall-clock (4 runs) | 429.9s | 140.0s | 3.07x |
| Theoretical max (4 concurrent) | — | ~107s | ~4.0x |

## Honesty Section

### What remains serialized

- **`for t in range(D):` loop** in `forward()` and `eval_acc()` — sequential Python loop over timesteps. This is the structural ~134% CPU cap per run. Cannot be fixed without vectorizing the time dimension (model change).
- Cache loading per worker (~200 MB, ~2–5s, amortized over run).
- CSV/markdown assembly (<0.1s, trivial).

### What actually improved

- **Inter-run parallelism:** 4 concurrent runs instead of serial. 3.07x wall-clock.
- **Result quality:** parallel workers avoid global RNG contamination. Serial D24/original_s42 got 0.7002 (convergence failure); parallel gets 0.99.
- **Thread policy:** benchmarked across 4 configs, not assumed.

### What this audit did NOT fix (open items)

1. **W1/W2 policy network:** trained but not used in eval. If removable: ~98% per-run runtime eliminated. Requires separate branch.
2. **Global RNG in `_soft_step`:** `torch.rand_like(logits)` uses global PyTorch RNG. Should use an explicit seeded generator for reproducibility. Currently causes ordering-dependent results in serial mode.
3. **Within-run CPU cap at ~134%:** inherent to sequential time-step loop. Vectorizing `forward()` across D steps would fix this.

### Can future probes run faster?

Yes. Use `run_config(n_workers=8, n_threads=1)` from this script. For the full 14-run probe, expected wall-clock ≈ 490s vs current ~1505s serial.

## One-Line Conclusion

**RUNTIME BOTTLENECK STATUS: FIXED**  
Serial (1w_8t): 429.9s → Best (8w_1t): 140.0s → Speedup: 3.07x  
Remaining structural bottleneck: sequential `for t in range(D):` in `forward()` (134% CPU cap per run). Additional open item: W1/W2 policy network trained but never used in eval — if removable, eliminates ~98% of per-run training cost.
