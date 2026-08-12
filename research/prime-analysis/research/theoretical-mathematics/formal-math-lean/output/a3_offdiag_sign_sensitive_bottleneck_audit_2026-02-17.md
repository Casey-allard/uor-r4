# A3 Sign-Sensitive Bottleneck Audit (2026-02-17)

## What was wrong

1. Cache fragmentation: the probe wrote/read only its own cache path, so it missed existing warm event-series caches from related A3 pipelines.
2. Python nested loops: low-lag accumulation used row-by-row and lag-by-lag Python loops, creating avoidable interpreter overhead.

## Fixes applied

1. Shared cache integration:
- Default write cache set to `research/cache/a3_offdiag_dynamic_majorant`.
- Added multi-read cache fallback via `--cache-read-dirs` including:
  - `research/cache/a3_offdiag_dynamic_majorant`
  - `research/cache/a3_offdiag_symbolic_chain`
  - `research/cache/a3_offdiag_sign_sensitive_lagbound`
2. Vectorized lag accumulation:
- Replaced nested row×lag Python loops with NumPy checkpoint-index vectorized accumulation.
- Added timing telemetry (`build_series_seconds`, `lagprep_seconds`, `eval_seconds`, `total_seconds`).

## Measured performance

### Full grid (bases 30,210,2310,30030; n up to 1e7; x-step 2500; lag-cut 32)
- Cold-ish run: `research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17.json`
  - `cache_hits=1`, `cache_writes=3`
  - `total_seconds=22.1532`
  - `build_series_seconds=16.0962`
  - `lagprep_seconds=5.9568`
  - `eval_seconds=0.0236`
- Warm run: `research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17_warm.json`
  - `cache_hits=4`, `cache_writes=0`
  - `total_seconds=6.1795`
  - `build_series_seconds=0.7610`
  - `lagprep_seconds=5.3280`
  - `eval_seconds=0.0230`

## Current dominant bottleneck

- With warm cache, >85% of runtime is lag-prep (`O(lag_cut * event_count)` product/cumsum work).
- This is now the true computational floor for this probe at current settings.

## Next optimization options

1. FFT-based lag correlations for profile-only modes (fast for large lag windows).
2. Adaptive lag schedule (dense small lags, sparse larger lags) to reduce `lag_cut` cost.
3. Optional reduced checkpoint grid during calibration, then final pass only for accepted configs.
4. Move lag kernel to numba/Cython if we need sub-second warm passes.

