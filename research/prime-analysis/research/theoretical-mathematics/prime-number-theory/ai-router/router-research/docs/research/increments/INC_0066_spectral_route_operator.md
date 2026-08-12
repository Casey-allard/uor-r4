# INC-0066: Spectral Route-Operator Measurement

## Status
Confirm completed positive on 2026-03-11.

## Trigger
`INC-0065` is now confirmed positive as a phase-field branch, but it does not
displace pure Hopf as the routed quality lead. The kill ladder in
`docs/PROJECT_CONTEXT.md` therefore points to the next unresolved rung:
spectral emergence measurement.

## Branch Contract
- use the confirmed route set, not speculative phase variants
- define a sparse geometric operator directly from route coordinates
- measure spectral structure instead of inferring it from proxy MSE

## Evidence
- Config:
  - `configs/spectral_route_inc0066_screen.json`
  - `configs/spectral_route_inc0066_confirm.json`
- Tools:
  - `tools/spectral_route_audit.py`
  - `tools/spectral_route_sweep.py`
- Analysis:
  - `results/analysis/inc0065_product_phase_field_confirm_spectral_seed0.json`
  - `results/analysis/inc0066_spectral_route_operator_screen.json`
  - `results/analysis/inc0066_spectral_route_operator_confirm.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_112010.md`
  - `docs/governance/gates/gate_20260311_112215.md`

## Minimal Scope
1. carry these confirmed routes:
   - `HOPF_BASE_K25_PHI`
   - `HOPF_K25_BASE_PHI`
   - `HOPF_PHI2_BAND_PHI`
   - `H4XH4_FIELD_A100`
   - `H4XH4_FIELD_A150`
   - `R0`
2. build a sparse geometric graph from route coordinates
3. compute normalized-Laplacian spectrum and low-frequency signal diagnostics
4. compare whether shell/sector structure lives in the low-frequency band

## Working Hypothesis
If the confirmed product branch really carries a richer coupled-field geometry,
that should become visible in operator structure, not only in routing metrics.

## Screen Read
- All audited route graphs stayed connected on the 2-seed screen.
- Product routes already separated from the control set on the operator
  diagnostics:
  - participation gap: `+0.0479`
  - sector lowfreq gap: `+0.0849`
- `H4XH4_FIELD_A150` led low-mode participation on the screen:
  - `participation_mean=0.2949`
  - `sector_lowfreq=0.0296`
  - `shell_lowfreq=0.1199`

## Confirm Read
- All audited route graphs stayed connected on the 4-seed confirm.
- The spectral distinction survived confirm:
  - product minus full-control participation gap: `+0.0222`
  - full-control minus product sector lowfreq gap: `+0.0805`
  - product minus Hopf-control participation gap: `+0.0208`
  - product minus Hopf-control shell lowfreq gap: `+0.0479`
- The confirmed product routes therefore keep a distinct operator signature:
  - more delocalized low modes
  - more shell-focused low-frequency energy
  - less sector-concentrated low-frequency energy than the control set
- `H4XH4_FIELD_A150` stayed the strongest product route on participation:
  - `participation_mean=0.2844`
  - `sector_lowfreq=0.0366`
  - `shell_lowfreq=0.1255`
- `HOPF_K25_BASE_PHI` remained the routed sector-concentration reference:
  - `participation_mean=0.2522`
  - `sector_lowfreq=0.0956`

## Decision
- Close `INC-0066` confirm positive.
- Treat direct geometry-induced spectral structure as evidence-positive on the
  confirmed route set.
- Do not treat this as a route-quality promotion over pure Hopf.

## Next Step
- Move next to spectral signal attribution (`INC-0067`) on the fixed confirmed
  route set.
- Test whether shell, sector, and task-relevant signals project onto the low
  modes in a useful way rather than merely showing a different operator shape.

## Resume Note
Do not reopen `INC-0065` or `INC-0066` as unresolved. The next question is
whether the measured modes carry useful structure.
