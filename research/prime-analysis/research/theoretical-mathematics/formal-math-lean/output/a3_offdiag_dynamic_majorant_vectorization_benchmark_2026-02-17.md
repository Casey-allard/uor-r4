# A3 Dynamic Majorant Vectorization Benchmark (2026-02-17)

## Scope

Optimized `research/a3_offdiag_dynamic_majorant.py` for cache reuse and array execution:
- Shared cache reads across major A3 caches (`dynamic_majorant`, `symbolic_chain`, `sign_sensitive_lagbound`).
- Vectorized row construction via `numpy` (`searchsorted` + batched formulas).
- Vectorized envelope fitting/checks (removed repeated row-dict scans).
- Added timing telemetry to output JSON.

## New baseline run

Artifact: `research/output/a3_offdiag_dynamic_majorant_2026-02-17_vectorized.json`

Key timing:
- `build_series_seconds = 0.9978`
- `row_build_seconds = 0.6126`
- `fit_and_checks_seconds = 0.0414`
- `total_seconds = 1.6537`

Cache behavior:
- `cache_hits = 4`
- `cache_writes = 0`

Theorem-relevant constants (unchanged from selected stress-calibrated branch):
- `A_eta = 4.0`
- `C_eta_uplifted = 1.9531254774350515`
- `A_H = 1.1`
- `C_H_uplifted = 4.526748350185781`
- Train/valid checks hold.

## Conclusion

The dynamic majorant stage now runs in ~1.65s warm for the full standard grid while preserving the same envelope outcomes.

