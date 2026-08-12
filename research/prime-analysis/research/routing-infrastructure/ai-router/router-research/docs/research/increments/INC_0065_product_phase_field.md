# INC-0065: Product Phase-Field Routing

## Status
Confirm completed positive on 2026-03-11.

## Trigger
The corrected reruns of `INC-0062`, `INC-0063`, and `INC-0064` changed the
phase story:
- `phase4d_hopf_base` is now the canonical no-fiber-phase control
- standalone transported phase is mechanism-live once `alpha` bins are active
- same-chart coupled-field transport is also mechanism-live and field-shift
  positive

That means the next valid phase step is no longer “retest whether phase is
inert.” The next valid step is to make the intended product split explicit.

## Branch Contract
- first `H^4` = routing geometry
- second `H^4` = coupled field
- product coupling should drive phase motion without collapsing the two roles
  back into one same-chart surrogate

## Evidence
- Config:
  - `configs/proxy_transfer_inc0065_product_phase_field_screen.json`
  - `configs/proxy_transfer_inc0065_product_phase_field_confirm.json`
- Analysis:
  - `results/analysis/inc0065_product_phase_field_screen.json`
  - `results/analysis/inc0065_product_phase_field_address_diff.json`
  - `results/analysis/inc0065_product_phase_field_confirm.json`
  - `results/analysis/inc0065_product_phase_field_confirm_address_diff.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_105034.md`
  - `docs/governance/gates/gate_20260311_110024.md`

## Screen Read
- All three explicit product candidates passed the configured route-health gate.
- Field-coupled phase motion is live across the whole screen:
  - `H4XH4_FIELD_A050`: `phase_transport_field_shift_abs_mean=0.00654`
  - `H4XH4_FIELD_A100`: `phase_transport_field_shift_abs_mean=0.01339`
  - `H4XH4_FIELD_A150`: `phase_transport_field_shift_abs_mean=0.01451`
- Seed-0 address audit vs `HOPF_BASE_K25_PHI` shows:
  - `H4XH4_FIELD_A050`: `98.56%` sector difference
  - `H4XH4_FIELD_A100`: `98.56%` sector difference
  - `H4XH4_FIELD_A150`: `98.40%` sector difference
- The product branch is not inert and not shell-collapsed.
- `H4XH4_FIELD_A150` produced the best screen MSE (`0.003899`), while the
  sweep recommendation chose `H4XH4_FIELD_A100` as the stabilized carry-forward
  candidate.

## Decision
- Close the confirm stage positive.
- Treat the explicit product split as a confirmed phase-field branch.
- Keep `H4XH4_FIELD_A100` as the stabilized product reference.
- Keep `H4XH4_FIELD_A150` as the best confirmed product-MSE point.

## Confirm Read
- `H4XH4_FIELD_A100`
  - `mse=0.003904`
  - `total=5.530s`
  - `phase_transport_field_shift_abs_mean=0.01030`
  - passes the 4-seed health gate
- `H4XH4_FIELD_A150`
  - `mse=0.003900`
  - `total=5.890s`
  - `phase_transport_field_shift_abs_mean=0.01239`
  - passes the 4-seed health gate
- `HOPF_K25_BASE_PHI` remains the overall routed quality lead
  - `mse=0.003895`
  - `total=5.580s`
- Reading:
  - explicit product phase-field routing is confirmed mechanism-positive
  - it is operationally healthy
  - it does not yet displace pure Hopf as the best routed proxy-MSE family

## Next Step
- Move next to direct spectral/operator measurement (`INC-0066`) on the
  confirmed route set.
- Keep the confirmed `INC-0065` routes in that evaluation set:
  - `H4XH4_FIELD_A100`
  - `H4XH4_FIELD_A150`
  - `HOPF_BASE_K25_PHI`
  - `HOPF_K25_BASE_PHI`
  - `HOPF_PHI2_BAND_PHI`
  - `R0`

## Resume Note
Do not resume from the pre-screen or pre-confirm state. Resume from the
confirmed 2026-03-11 artifacts and the spectral measurement path.
