# INC-0046: Static Routed Scale Robustness

## Status
Complete.

## Trigger
`INC-0045` produced the first cheap-schedule 4-seed routed win over cheap `R0` in this branch.

## Objective
Test whether the cheap routed win survives a larger data subset without giving back runtime or route health.

## Minimal Scope
1. Compare only:
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`
   - `R0`
2. Keep the `IT40_P2` chart schedule fixed.
3. Increase the proxy workload beyond the current larger-subset setting.
4. Keep geometry fixed.

## Screen Result
2-seed screen means:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003886145`, `12.201s`, health pass
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003905894`, `11.685s`, health pass
- `R0`: `0.003891917`, `15.917s`, shell-collapse fail

## Confirm Result
4-seed confirm means:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003884370`, `11.035s`, health pass
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003900404`, `10.186s`, health pass
- `R0`: `0.003892404`, `18.872s`, shell-collapse fail

Timing read from confirm:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `chart_opt=9.170s`, `training_ema=0.138s`
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `chart_opt=8.629s`, `training_ema=0.110s`
- `R0`: `chart_opt=10.716s`, `training_ema=5.628s`

## Decision
- Keep `HOPF_K25_BASE_IT40_P2_STATIC` as the operational routed lead.
- Keep `HOPF_PHI2_BAND_IT40_P2_STATIC` as the hardware-efficiency routed lead.
- Promote the frontier to a near-full-proxy scale check.

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0046_static_scale_robustness_screen.json`
- Screen analysis:
  - `results/analysis/inc0046_static_scale_robustness_screen.json`
- Screen gate:
  - `docs/governance/gates/gate_20260306_104728.md`
- Confirm config:
  - `configs/proxy_transfer_inc0046_static_scale_robustness_confirm.json`
- Confirm analysis:
  - `results/analysis/inc0046_static_scale_robustness_confirm.json`
- Confirm gate:
  - `docs/governance/gates/gate_20260306_105119.md`
- Report:
  - `docs/reports/HOPF_COST_DECOMPOSITION.md`
