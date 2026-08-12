# INC-0045: Static Routed Chart Floor

## Status
Complete.

## Trigger
`INC-0044` showed that the widened static branch was healthy and higher-quality than cheap `R0`, but still slower on absolute runtime.

## Objective
Determine whether one more chart-floor step can close the remaining runtime gap for the strict-gate routed lead without erasing its quality advantage.

## Minimal Scope
1. Compare only:
   - `HOPF_PHI2_BAND_IT48_P3_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `R0`
2. Keep `train_route_mode=final_static` fixed for routed branches.
3. Use the same larger-subset proxy workload.
4. Change only chart schedule pressure.

## Screen Result
2-seed screen means:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `0.0039027`, `5.725s`, health pass
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.0039048`, `6.488s`, health pass
- `HOPF_PHI2_BAND_IT48_P3_STATIC`: `0.0039026`, `11.874s`, runtime fail vs cheap `R0`
- `R0`: `0.0039164`, `8.152s`, shell-collapse fail

## Confirm Result
4-seed confirm means:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `0.0038951`, `6.800s`, health pass
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.0039034`, `7.176s`, health pass
- `R0`: `0.0039114`, `8.334s`, shell-collapse fail

Timing read from confirm:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `chart_opt=5.527s`, `training_ema=0.078s`
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `chart_opt=6.045s`, `training_ema=0.081s`
- `R0`: `chart_opt=5.129s`, `training_ema=1.686s`

## Decision
- Promote `HOPF_K25_BASE_IT40_P2_STATIC` as the current operational routed lead.
- Promote `HOPF_PHI2_BAND_IT40_P2_STATIC` as the current widened cheap routed lead.
- Kill `HOPF_PHI2_BAND_IT48_P3_STATIC` as the active frontier; it was overtaken by the `IT40_P2` family.
- Move next to scale robustness.

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0045_static_chart_floor_screen.json`
- Screen analysis:
  - `results/analysis/inc0045_static_chart_floor_screen.json`
- Screen gate:
  - `docs/governance/gates/gate_20260306_103538.md`
- Confirm config:
  - `configs/proxy_transfer_inc0045_static_chart_floor_confirm.json`
- Confirm analysis:
  - `results/analysis/inc0045_static_chart_floor_confirm.json`
- Confirm gate:
  - `docs/governance/gates/gate_20260306_103811.md`
- Report:
  - `docs/reports/HOPF_COST_DECOMPOSITION.md`
