# INC-0043: Static Training-Route Reuse

## Status
Complete.

## Trigger
`INC-0042` showed that large-subset `training_ema` cost is almost entirely `training_route`, not bucket prediction or EMA writes.

## Objective
Test whether reusing final `tau=1.0` training routes during EMA updates can materially narrow the runtime gap to `R0` without breaking the current routed quality ordering.

## Minimal Scope
1. Compare only the current reduced-schedule large-subset leads and `R0`.
2. Keep chart schedule fixed at the `IT60_P4` setting.
3. Compare `train_route_mode=dynamic` against `train_route_mode=final_static` within the same batch.
4. Keep geometry fixed.

## Screen Result
2-seed screen means:
- `HOPF_K25_BASE_IT60_P4`: `0.0038965`, `26.597s`, runtime fail
- `HOPF_K25_BASE_IT60_P4_STATIC`: `0.0038974`, `17.783s`, health pass
- `HOPF_PHI2_BAND_IT60_P4`: `0.0039068`, `34.232s`, runtime fail
- `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.0038970`, `18.263s`, health pass
- `R0`: `0.0039128`, `19.094s`, shell-collapse fail

## Confirm Result
4-seed confirm means:
- `HOPF_K25_BASE_IT60_P4`: `0.0038957`, `32.034s`, runtime fail vs `R0`
- `HOPF_K25_BASE_IT60_P4_STATIC`: `0.0038995`, `19.798s`, health pass
- `HOPF_PHI2_BAND_IT60_P4_STATIC`: `0.0039023`, `19.602s`, health pass
- `R0`: `0.0039137`, `22.520s`, shell-collapse fail

Timing read from confirm:
- `HOPF_K25_BASE_IT60_P4`: `training_route=12.800s`, `training_update=0.149s`
- `HOPF_K25_BASE_IT60_P4_STATIC`: `training_route=0.003s`, `training_update=0.094s`
- `HOPF_PHI2_BAND_IT60_P4_STATIC`: `training_route=0.003s`, `training_update=0.067s`

## Decision
- Promote `HOPF_PHI2_BAND_IT60_P4_STATIC` as the current large-subset operational routed lead.
- Promote `HOPF_K25_BASE_IT60_P4_STATIC` as the current large-subset quality-balanced routed lead.
- Demote the dynamic `IT60_P4` routes to quality references only.
- Move next to chart-pressure reduction on the static frontier.

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0043_train_route_static_screen.json`
- Screen analysis:
  - `results/analysis/inc0043_train_route_static_screen.json`
- Screen gate:
  - `docs/governance/gates/gate_20260306_095825.md`
- Confirm config:
  - `configs/proxy_transfer_inc0043_train_route_static_confirm.json`
- Confirm analysis:
  - `results/analysis/inc0043_train_route_static_confirm.json`
- Confirm gate:
  - `docs/governance/gates/gate_20260306_100530.md`
- Report:
  - `docs/reports/HOPF_COST_DECOMPOSITION.md`
