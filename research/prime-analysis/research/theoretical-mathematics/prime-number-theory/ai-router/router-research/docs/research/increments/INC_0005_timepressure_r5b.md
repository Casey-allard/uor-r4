# INC-0005: Time-Pressure Ablation on R5B

## Hypothesis
Adding positive time pressure (`time_pressure_lambda > 0`) may improve post-growth routing quality for the current best route R5B.

## Configuration
- Sweep config: `configs/route_sweep_inc0005_r5b_timepressure.json`
- Route family: `R5B` (`sector_mode=phase4d`, `phase4_dims=0,2,4,6`)
- Lambda set: `{0.0, 0.25, 0.5, 0.8, 1.2}`
- Seeds: finalize `{0,1}`
- Runtime settings: `fast_dev=0`, cache disabled

## Evidence
- Gate note: `docs/governance/gates/gate_20260305_133115.md`
- Logs: `results/raw/sweep_R5B_L*_finalize_seed*_{20260305_1324..1330}.log`

## Results (mean MSE, lower is better)
- `lambda=0.00`: `0.819587`
- `lambda=0.25`: `0.859502`
- `lambda=0.50`: `0.877061`
- `lambda=0.80`: `0.877221`
- `lambda=1.20`: `0.883790`

## Decision
- Keep `time_pressure_lambda=0.0` for R5B.
- Treat positive time pressure as regressive in current regime.

## Next Increment
- `INC-0006`: R5B robustness check with 4 finalize seeds on larger `N` and optional SO(8) rotation off/on ablation.
