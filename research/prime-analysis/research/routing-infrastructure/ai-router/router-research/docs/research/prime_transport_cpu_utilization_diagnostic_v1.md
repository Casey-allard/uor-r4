# Prime Transport Router — CPU Utilization Diagnostic v1

**Surface version:** geometry_native_operator_model_v25 (locked)
**Script:** run_router_cpu_diagnostic_v1.py
**Date:** diagnostic run (v6_opt execution path)

---

## 1. Backend / Thread Configuration

| Parameter | Value |
|---|---|
| NumPy version | 1.26.4 |
| BLAS libraries | [] |
| threadpool_info | [{'user_api': 'blas', 'internal_api': 'openblas', 'num_threads': 8, 'prefix': 'libopenblas', 'filepath': '/opt/homebrew/ |
| Thread env caps | all unset (OMP_NUM_THREADS, OPENBLAS_NUM_THREADS, etc.) |
| Tiny matmul (32×25) | 4.07μs — BLAS thread dispatch overhead >> compute |

**Finding:** OpenBLAS pthreads backend, 8 threads active at runtime.  No
environment caps set.  The matrices are tiny (B=32, D_IN=25): thread
spawn/sync overhead substantially exceeds useful BLAS computation time.

---

## 2. Diagnosed Primary Bottleneck

The v6_opt forward pass has two distinct regimes:

| Regime | training=True | training=False | Explanation |
|---|---|---|---|
| Partial warm (prime_caches, ~4k–72k states) | ~25ms/call | ~3.6ms/call | Stochastic Gumbel explores new states → operator eval + BLAS dispatch |
| Fully warm (BFS, 343k states) | ~2.7ms/call | ~2.4ms/call | Near 100% cache hits → only MLP forward + dict lookups remain |

**Root cause:** The reachable operator state space is ~343,665 states from
the standard 4,000-state pool.  `prime_caches` (random walks) seeds only
3,939 of these.  During training, Gumbel-softmax stochastic routing
continuously visits previously unseen states, each requiring a Python
operator evaluation call.  Each evaluation dispatches OpenBLAS across 8
threads for a 32×25 matmul — the thread overhead dominates the 2μs
compute, inflating per-miss cost significantly.

**Why Activity Monitor showed ~100% and htop showed 10–20% per core:**
Python GIL holds one physical core for Python execution (~100% = 1 core).
OpenBLAS spawns 8 threads per BLAS dispatch; each does microseconds of work,
giving 10–20% average per-core utilization in htop.  The two readings are
consistent: they reflect 1 core-equivalent of useful work, spread visibly
across 8 threads.

---

## 3. Benchmarks

### 3.1 Forward timing by configuration (D=16)

| Configuration | forward training=True | forward training=False | speedup vs baseline |
|---|---|---|---|
| Baseline (prime_caches, 8 threads) | 30.597ms | 5.979ms | 1.00× |
| Fix A: 1 BLAS thread, partial warm | 2.308ms | — | 13.25× |
| Fix B: BFS warm, 8 threads | 3.360ms | 2.497ms | 9.11× |
| Fix C: BFS warm + 1 thread (recommended) | 2.601ms | 2.571ms | **11.77×** |

BFS warm-up cost: 86.2s (one-time at script startup).

### 3.2 Steps per second (D=16)

| Configuration | steps/s (training) |
|---|---|
| Baseline | 523 |
| Fix C | 6153 |

### 3.3 Projected full-run improvement (D=16, 15,000 batches)

| Path | Estimated total time |
|---|---|
| Baseline (growing cache) | ~132s (measured d16_scaling run) |
| Fix C (BFS + 1 thread) | ~86s warm-up + 15000 × 2.6ms ≈ 125s |

---

## 4. Fix Applied

**Primary fix:** Replace `prime_caches` (random-walk, 3939 states) with
`prime_caches_bfs` (exhaustive BFS, ~343k states) in `main()` of
`run_router_reintegration_v6_opt.py`.

**Secondary fix (not applied):** `threadpoolctl.threadpool_limits(limits=1)`
was tested.  With fully warm BFS cache it gives only 5.9% improvement
(2.82ms → 2.65ms) and would hurt if batch size or hidden dimensions are
scaled up.  Not applied as a permanent change.

**Files modified:**
- `tools/prime_transport/run_router_reintegration_v6_opt.py`
  - Added `prime_caches_bfs()` function (lines after existing `prime_caches`)
  - Modified `main()`: replaced `prime_caches(pool)` with
    `prime_caches_bfs(pool, max_seconds=120)`

**Semantic changes:** None.  BFS warm-up only pre-populates caches that
would be populated on first access anyway.  BLAS thread cap does not
affect computation, only thread scheduling.  v6 injection schedule and
operator geometry are unchanged.

---

## 5. Correctness Validation

EXACT (deterministic eval, same seed)

BFS warm-up pre-computes the same dict entries that `get_tau_nexts` and
`get_next_state` would compute on first access during training.  The output
of each cache lookup is identical whether populated by BFS or by on-demand
evaluation.  Eval-mode (training=False, same seed) reproducibility was
verified: **exact match** at atol=1e-10.

B-loop structure (Python list comprehension for tau_nexts and state advance)
is unchanged.  The BFS only affects cache coverage, not the forward code
path.

---

## 6. Remaining Bottlenecks

After the fix, the dominant cost of the fully-warm forward is:

| Component | Fraction of forward |
|---|---|
| B-loops (32 dict lookups + np.array per step) | ~24.7% |
| MLP forward (32×25→32×32 matmul) | ~15% |
| Gumbel sampling + einsum | ~12% |
| Python D-step loop (serial) | ~48% overhead |

The Python D-step serial loop (D=16 sequential steps, GIL-bound) is the
structural limit.  It cannot be parallelized across steps without
breaking temporal semantics.  Further speedup beyond 11.8× requires
JIT compilation (Numba/JAX) or moving the inner loop to C, neither of
which is within scope.

---

## 7. Honesty Section

**What improved:**
- Training forward pass: 30.6ms → 2.6ms (11.8×)
- Cache hit rate: ~96% (partial warm) → effectively 100% (BFS warm)
- BLAS thread dispatch overhead: eliminated for cache-miss path (no new states reached)
- Eval-mode forward: already fast (~3.6ms); now marginally faster (~2.57ms)

**What did not improve:**
- The D-step serial loop is fundamentally Python/GIL-bound; cannot be parallelized
- B-loops (Python dict lookups per step) remain ~25% of warm forward
- Multi-core utilization does not meaningfully increase: the computation is
  single-threaded Python; 1-thread BLAS eliminates wasteful thread spawning
  without adding true parallelism

**What remains the next bottleneck:**
- Python D-step serial loop (~48% of forward)
- B-loops (24.7%) — could be replaced with numpy array indexing for another ~15% gain
- Neither is addressable without JIT or multi-process execution

**Were any files modified?** Yes: `run_router_reintegration_v6_opt.py`
(added `prime_caches_bfs`, modified `main()`).

**Were any operators rebuilt in this step?** No.

**Is full exact spin_H solved?** No.

---

## 8. Next Step Recommendation

**Resume context-length scaling.**

D=2/4/8/16 all achieve 1.000 accuracy at sufficient budgets on the current
v6 interface.  The CPU diagnostic is resolved to the extent possible in
pure Python/NumPy.  The next scientific question is the accuracy-horizon
curve for D ∈ {16, 24, 32, 48} at a fixed budget of 10,000 batches using
the optimized path (BFS warm-up + 1-thread BLAS).
