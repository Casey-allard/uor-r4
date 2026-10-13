# INC-0044: Static-Frontier Chart Pressure

## Status
Complete.

## Trigger
`INC-0043` solved the large-subset training-route bottleneck. The dominant remaining routed cost term was `chart_opt`.

## Objective
Test whether the new static routed frontier can keep its quality/runtime win over `R0` while reducing chart optimization cost further.

## Minimal Scope
1. Compare only:
   - `HOPF_K25_BASE_IT60_P4_STATIC`
   - `HOPF_K25_BASE_IT48_P3_STATIC`
   - `HOPF_PHI2_BAND_IT60_P4_STATIC`
   - `HOPF_PHI2_BAND_IT48_P3_STATIC`
   - `R0`
2. Keep `train_route_mode=final_static` fixed for routed branches.
3. Change only chart schedule pressure (`chart_iters`, `so8_candidates`, `scale_candidates`).
4. Use seed-major ordering.

## Screen Result
2-seed screen means:
- `HOPF_PHI2_BAND_IT48_P3_STATIC`: `0.0039026`, `10.298s`, health pass
- `HOPF_K25_BASE_IT48_P3_STATIC`: `0.0039087`, `13.028s`, health pass
- `HOPF_K25_BASE_IT60_P4_STATIC`: `0.0038974`, `16.934s`, health pass
- `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.0038970`, `20.816s`, runtime fail vs cheap `R0`
- `R0`: `0.0039201`, `15.524s`, shell-collapse fail

## Confirm Result
4-seed confirm means:
- `HOPF_PHI2_BAND_IT48_P3_STATIC`: `0.0039013`, `17.217s`, health pass
- `HOPF_K25_BASE_IT60_P4_STATIC`: `0.0038995`, `24.155s`, runtime fail vs cheap `R0`
- `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.0039023`, `20.963s`, runtime fail vs cheap `R0`
- `R0`: `0.0039228`, `16.183s`, shell-collapse fail

Timing read from confirm:
- `HOPF_PHI2_BAND_IT48_P3_STATIC`: `chart_opt=15.728s`, `training_ema=0.090s`
- `HOPF_PHI2_BAND_IT60_P4_STATIC`: `chart_opt=19.690s`, `training_ema=0.080s`
- `R0`: `chart_opt=11.373s`, `training_ema=3.234s`

## Decision
- Promote `HOPF_PHI2_BAND_IT48_P3_STATIC` as the current strict-gate routed lead under the cheaper common schedule.
- Kill `HOPF_K25_BASE_IT48_P3_STATIC` as a promotion candidate; it is dominated by the widened cheap lead.
- Demote both `IT60_P4_STATIC` variants behind the cheaper widened branch for the next systems step.
- Do not claim an absolute runtime win vs cheap `R0`; the routed lead is healthier and higher-quality, but still slightly slower.
- Move next to chart-floor testing.

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0044_static_chart_pressure_screen.json`
- Screen analysis:
  - `results/analysis/inc0044_static_chart_pressure_screen.json`
- Screen gate:
  - `docs/governance/gates/gate_20260306_101427.md`
- Confirm config:
  - `configs/proxy_transfer_inc0044_static_chart_pressure_confirm.json`
- Confirm analysis:
  - `results/analysis/inc0044_static_chart_pressure_confirm.json`
- Confirm gate:
  - `docs/governance/gates/gate_20260306_102058.md`
- Report:
  - `docs/reports/HOPF_COST_DECOMPOSITION.md`
