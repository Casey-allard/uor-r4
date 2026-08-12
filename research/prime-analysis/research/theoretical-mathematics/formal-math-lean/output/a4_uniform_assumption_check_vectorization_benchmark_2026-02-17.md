# A4 Uniform Assumption Vectorization Benchmark (2026-02-17)

## Command grid
- bases: `30,210,2310,30030`
- n-values: `300000,1000000,2000000,5000000`
- x-step: `2500`
- zeros: `m_zero=128`, `m_ref=512`
- kernel: `none`

## Runtime comparison

### Pre-optimization baseline
- Artifact: `research/output/a4_uniform_assumption_check_2026-02-17_preopt.json`
- Wall time (`/usr/bin/time -p`): `79.06s`

### Post-optimization, cold path (`--no-cache`)
- Artifact: `research/output/a4_uniform_assumption_check_2026-02-17_postopt_nocache.json`
- Wall time (`/usr/bin/time -p`): `48.45s`
- Speedup vs baseline: `1.63x`

### Post-optimization, warm cache
- Artifact: `research/output/a4_uniform_assumption_check_2026-02-17_postopt.json`
- Wall time (`/usr/bin/time -p`): `0.57s`
- In-artifact timing: `total_seconds=0.1642`
- Cache stats: `hits=8`, `writes=0`, `misses=0`

## Numeric parity
- Uniform constants: exact match vs baseline.
- Theorem RHS check: exact match vs baseline (`holds_on_grid=true`, same ratio/gap metrics).

## Implemented optimizations
- Shared cache reads via `--cache-read-dirs`.
- Vectorized event-to-grid projection (`searchsorted` + cumulative sums).
- Vectorized residual/delta/theorem computations.
- Added timing telemetry to output JSON.

