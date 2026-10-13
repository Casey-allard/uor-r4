# INC-0042: Large-Subset EMA Pressure

## Status
Complete.

## Trigger
`INC-0041` showed that the reduced-schedule cost rescue survived on quality but not on runtime under larger subset load.

## Objective
Separate the remaining large-subset runtime problem into:
- chart optimization growth with larger sample count
- training EMA growth with larger slot pressure

## Minimal Scope
1. Compare only:
   - `HOPF_K25_BASE_IT60_P4`
   - `HOPF_PHI2_BAND_IT60_P4`
   - `R0`
2. Instrument the training path so large-subset cost is separated into:
   - per-step rerouting cost
   - bucket prediction / EMA write cost
3. Run a focused large-subset timing diagnostic on the current leads.
4. Keep geometry fixed.

## Result
- `training_update` was negligible for every route.
- `training_ema` was almost entirely `training_route`.
- Mean timing read:
  - `HOPF_K25_BASE_IT60_P4`: `chart_opt=20.011s`, `training_route=14.913s`, `training_update=0.120s`, `total=36.269s`
  - `HOPF_PHI2_BAND_IT60_P4`: `chart_opt=20.573s`, `training_route=11.798s`, `training_update=0.117s`, `total=33.742s`
  - `R0`: `chart_opt=28.073s`, `training_route=1.768s`, `training_update=0.086s`, `total=31.326s`

## Decision
- Kill the original `INC-0042` intuition that post-growth knobs (`max_slots_per_bucket`, `extra_budget`, `split_rounds`) were the main next lever.
- Promote static training-route reuse as the next live systems branch.

## Artifacts
- Config:
  - `configs/proxy_transfer_inc0042_timing_diag.json`
- Analysis:
  - `results/analysis/inc0042_timing_diag.json`
- Gate:
  - `docs/governance/gates/gate_20260306_094708.md`
- Report:
  - `docs/reports/HOPF_COST_DECOMPOSITION.md`
