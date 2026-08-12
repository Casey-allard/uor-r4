# INC-0047: Near-Full Proxy Scale

## Status
Complete.

## Trigger
`INC-0046` showed that the cheap routed frontier survived the next larger-subset step.

## Objective
Test whether the cheap routed win survives near-full-proxy scale before shifting effort toward integration or new task translation.

## Minimal Scope
1. Compare only:
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`
   - `R0`
2. Keep the cheap `IT40_P2` chart schedule fixed.
3. Raise the proxy subset again toward the available dataset ceiling.
4. Keep geometry fixed.

## Screen Result
2-seed screen means:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003876735`, `15.255s`, health pass
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003884715`, `13.888s`, health pass
- `R0`: `0.003886670`, `22.461s`, shell-collapse fail

## Confirm Result
4-seed confirm means:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `0.003880503`, `15.904s`, health pass
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `0.003881127`, `16.079s`, health pass
- `R0`: `0.003884838`, `31.826s`, shell-collapse fail

Timing read from confirm:
- `HOPF_K25_BASE_IT40_P2_STATIC`: `chart_opt=13.057s`, `training_ema=0.194s`
- `HOPF_PHI2_BAND_IT40_P2_STATIC`: `chart_opt=12.973s`, `training_ema=0.209s`
- `R0`: `chart_opt=11.659s`, `training_ema=15.907s`

## Decision
- Keep `HOPF_K25_BASE_IT40_P2_STATIC` as the operational routed lead.
- Keep `HOPF_PHI2_BAND_IT40_P2_STATIC` as the hardware-efficiency routed lead.
- Move next to integration/translation beyond the proxy harness.

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0047_near_full_proxy_scale_screen.json`
- Screen analysis:
  - `results/analysis/inc0047_near_full_proxy_scale_screen.json`
- Screen gate:
  - `docs/governance/gates/gate_20260306_105627.md`
- Confirm config:
  - `configs/proxy_transfer_inc0047_near_full_proxy_scale_confirm.json`
- Confirm analysis:
  - `results/analysis/inc0047_near_full_proxy_scale_confirm.json`
- Confirm gate:
  - `docs/governance/gates/gate_20260306_110140.md`
- Report:
  - `docs/reports/HOPF_COST_DECOMPOSITION.md`
