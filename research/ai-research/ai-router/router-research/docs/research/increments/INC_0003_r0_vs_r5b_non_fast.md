# INC-0003: Non-Fast Validation of R0 vs R5B

## Hypothesis
The strongest phase4d variant (`phase4_dims=0,2,4,6`) remains superior to kmeans baseline under non-fast-dev settings with cache disabled.

## Configuration
- Sweep config: `configs/route_sweep_inc0003_r0_vs_r5b.json`
- Stage policy: screen(1), confirm(2), finalize(2)
- Runtime settings: `fast_dev=0`, `cache_chart=0`, `cache_routes=0`

## Evidence
- Gate note: `docs/governance/gates/gate_20260305_130833.md`
- Logs:
  - R0: `results/raw/sweep_R0_*_20260305_1303-1306.log`
  - R5B: `results/raw/sweep_R5B_*_20260305_1306-1308.log`

## Results
- R0 finalize mean:
  - `test_mse_after=0.840253`
  - `total_sec=29.663`
- R5B finalize mean:
  - `test_mse_after=0.819587`
  - `total_sec=24.350`

## Relative Delta (R5B vs R0)
- Quality: `~2.46%` lower MSE (better)
- Runtime: `~17.9%` faster total runtime

## Decision
- Promote `R5B` to **provisional leader** for continued testing.
- Keep branch promotion cautious until a 4-seed non-fast finalize confirms stability.

## Next Increment
- `INC-0004`: 4-seed non-fast finalize (`R0` vs `R5B`) with optional time-pressure ablation on `R5B`.
