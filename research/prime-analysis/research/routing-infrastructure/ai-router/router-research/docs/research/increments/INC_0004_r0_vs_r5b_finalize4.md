# INC-0004: 4-Seed Non-Fast Finalize Validation (R0 vs R5B)

## Hypothesis
R5B remains superior to R0 under stricter non-fast conditions with 4-seed finalize evaluation.

## Configuration
- Sweep config: `configs/route_sweep_inc0004_r0_vs_r5b_finalize4.json`
- Stage policy: screen(1), confirm(2), finalize(4)
- Runtime settings: `fast_dev=0`, `cache_chart=0`, `cache_routes=0`
- Common phase4 dims: `0,2,4,6`

## Evidence
- Gate note: `docs/governance/gates/gate_20260305_131933.md`
- Logs (batch window): `results/raw/sweep_R0_*_20260305_131*.log`, `results/raw/sweep_R5B_*_20260305_131*.log`

## Results (finalize mean)
- R0:
  - `test_mse_after=0.829354`
  - `total_sec=32.591`
- R5B:
  - `test_mse_after=0.825124`
  - `total_sec=29.478`

## Relative Delta (R5B vs R0)
- MSE improvement: `0.00423` absolute (`~0.51%` relative)
- Runtime improvement: `3.113s` absolute (`~9.55%` relative)

## Decision
- R5B is the current best-known route and should be used as default candidate branch.
- Keep R0 as control baseline for future regressions.

## Next Increment
- `INC-0005`: R5B time-pressure ablation (`lambda in {0.0,0.25,0.5,0.8,1.2}`) to test whether pressure can improve post-growth without collapsing routing quality.
