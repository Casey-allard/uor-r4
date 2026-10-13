# INC-0061: Measure-Consistent `H^4` / Hopf Route Law

## Status
In progress.

## Trigger
`INC-0060` established that geometry routing is real enough to keep pursuing:
- both routed Hopf families beat the collapsed `R0` control on proxy quality and runtime
- widened Hopf improved geodesic neighborhood preservation

But it also showed the current route law is still mathematically off:
- shell occupancy is far from `H^4` shell mass
- Hopf angular mass remains concentrated, especially in `theta1/theta2`

## Hypothesis
The next correction should be structural, not another controller repair:
- shell boundaries should follow hyperbolic measure more directly
- angular binning should be derived from Hopf coordinates with explicit equal-mass treatment

If that is right, then a measure-consistent route law should improve:
- shell-mass diagnostics
- Hopf angular-mass diagnostics
- geodesic-neighborhood preservation

without giving back the current routed task win.

## Minimal Scope
1. Add a measure-consistent shell law derived from `H^4` shell mass.
2. Add a measure-consistent Hopf angular law with explicit equal-mass `chi` handling.
3. Keep the current cheap static training schedule fixed.
4. Screen against:
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`
   - the new measure-consistent route variant(s)
5. Evaluate both task and geometry metrics.

## Acceptance
- materially reduces shell-mass error and/or Hopf angular-mass error
- preserves or improves geodesic neighborhood overlap
- stays within the current routed quality/runtime band

## Scope Guardrail
- Do not open the event-driven / gated-intelligence branch yet.
- Do not move to spectral claims until the measure-consistent route law is tested.

## First Screen Result
Artifacts:
- `configs/proxy_transfer_inc0061_h4_mass_shell_screen.json`
- `results/analysis/inc0061_h4_mass_shell_screen.json`
- `docs/governance/gates/gate_20260310_231241.md`

What was tested first:
- a shell-only correction:
  - `shell_mode=h4_mass`

Reading:
- naive equal-mass `H^4` shells are not sufficient by themselves
- they reduce shell-mass mismatch locally, but they over-open shell states and break route health
- the current phi-shell references remain stronger operationally

Key means:
- `HOPF_K25_BASE_PHI`
  - `mse=0.0039027`
  - `total=6.681s`
  - `shell_mass_l1=1.1416`
  - pass
- `HOPF_K25_BASE_H4M`
  - `mse=0.0039045`
  - `total=6.626s`
  - `shell_mass_l1=1.4784`
  - `eval_shells=85`
  - fail
- `HOPF_PHI2_BAND_PHI`
  - `mse=0.0039048`
  - `total=6.487s`
  - `shell_mass_l1=1.1140`
  - `knn_overlap=0.8806`
  - pass
- `HOPF_PHI2_BAND_H4M`
  - `mse=0.0039177`
  - `total=6.906s`
  - `shell_mass_l1=0.9846`
  - `eval_shells=27`
  - fail

## Interim Decision
- Keep `RR-061` open.
- Do not promote raw `h4_mass` shells.
- Next correction inside this branch should be:
  - bounded/shared-state shell mass control
  - and likely shell+angular measure correction together
  - not shell-only equal-mass expansion

## Second Screen Result
Artifacts:
- `configs/proxy_transfer_inc0061_h4_mass_phi_screen.json`
- `results/analysis/inc0061_h4_mass_phi_screen.json`
- `docs/governance/gates/gate_20260310_231855.md`

What was tested next:
- bounded `H^4` shell mass through the project’s discrete `phi` ladder:
  - `shell_mode=h4_mass_phi`

Reading:
- bounded shell mass is still not sufficient by itself
- it improves proxy MSE slightly in some cases, but still breaks route health
- shell-only work is now exhausted enough to justify shifting the correction toward the actual Hopf base law

Key means:
- `HOPF_K25_BASE_H4MPHI`
  - `mse=0.0038954`
  - `total=6.344s`
  - `shell_mass_l1=1.9054`
  - `eval_shells=12.5`
  - fail
- `HOPF_PHI2_BAND_H4MPHI`
  - `mse=0.0039005`
  - `total=6.954s`
  - `shell_mass_l1=0.9334`
  - `eval_shells=6.5`
  - fail

## Updated Decision
- Keep `RR-061` open only long enough to document that shell-only measure fixes failed twice.
- Next structural correction should target the Hopf base directly:
  - route on `(eta, delta)` or equivalent Hopf-base coordinates
  - keep `alpha` as the fiber phase, not the coarse routing variable
