# INC-0064: Coupled Complex-Field Phase Transport

## Status
Complete on corrected screen rerun. The old queued-next / inert framing is obsolete.

## Trigger
The corrected `INC-0063` rerun changed the phase reading materially:
- standalone Hopf transport is not inert once `alpha` bins are live
- transported phase now changes addresses versus `phase4d_hopf_base`
- the remaining open question is whether explicit field coupling makes that
  phase motion stronger or more useful

User clarification sharpens the intended mechanism:
- first `H^4` = routing geometry
- second `H^4` = discrete complex-value field
- the two factors are coupled
- minima routing should come from the first factor
- phase jumping should come from the coupled complex field

## Hypothesis
Phase remains a live claim only if it is induced by the coupled `H^4 x H^4` geometry.
A phase law that couples the second-factor complex field into the transported fiber phase should:
- change addresses relative to `phase4d_hopf_base`
- remain healthier than raw `phase4d_hopf`
- preserve or improve at least one primary routing/retrieval metric without unacceptable cost

## Minimal Scope
1. Keep the first factor on Hopf-base coarse routing.
2. Keep the second factor as the discrete complex-value field.
3. Implement one explicit coupling law where the complex field modulates phase transport.
4. Compare:
   - `phase4d_hopf_base`
   - `phase4d_hopf`
   - new coupled complex-phase route law
5. Measure:
   - proxy quality/runtime
   - route health
   - address-difference versus `phase4d_hopf_base`
   - coupled-field occupancy
   - phase-shift contribution from the complex field

## Acceptance
- coupled complex phase changes addresses materially versus `phase4d_hopf_base`
- route health remains acceptable
- the branch beats either the no-phase control or raw Hopf phase on at least one primary metric
- the gain is attributable to the coupled field rather than a free local heuristic

## Failure Meaning
If the coupled complex-field law still does not move addresses or improve metrics, then the near-term phase claim should narrow sharply:
- geometry routing remains live
- phase remains unproven as a necessary mechanism in this routing stack
- spectral claims should still be tested, but phase should stop driving route-law changes until a stronger mathematical transport law is identified

## Corrected Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0064_coupled_complex_phase_screen.json`
- Screen analysis:
  - `results/analysis/inc0064_coupled_complex_phase_screen_corrected.json`
- Address-diff audit:
  - `results/analysis/inc0064_coupled_complex_phase_address_diff_corrected.json`
- Gate note:
  - `docs/governance/gates/gate_20260311_101607.md`

## Result
2-seed screen means:
- `HOPF_CPX_TRANSPORT_L050_F050`
  - `mse=0.003920694`
  - `total=5.731s`
  - `phase_transport_shift_abs_mean=0.3611`
  - `phase_transport_field_shift_abs_mean=0.3045`
  - `phase_transport_alpha_bins=2.0`
  - health pass
- `HOPF_CPX_TRANSPORT_L050_F100`
  - `mse=0.003932169`
  - `total=5.532s`
  - `phase_transport_shift_abs_mean=0.6644`
  - `phase_transport_field_shift_abs_mean=0.6085`
  - `phase_transport_alpha_bins=2.0`
  - health pass
- `HOPF_CPX_TRANSPORT_L100_F100`
  - `mse=0.003934491`
  - `total=6.448s`
  - `phase_transport_shift_abs_mean=0.7202`
  - `phase_transport_field_shift_abs_mean=0.6085`
  - `phase_transport_alpha_bins=2.0`
  - health pass
- `HOPF_BASE_K25_PHI`
  - `mse=0.003900382`
  - `total=6.461s`
  - health pass
- `HOPF_K25_BASE_PHI`
  - `mse=0.003902717`
  - `total=5.888s`
  - health pass
- `R0`
  - `mse=0.003916428`
  - `total=6.820s`
  - health fail

Address-diff audit against `phase4d_hopf_base`:
- `HOPF_CPX_TRANSPORT_L050_F050`
  - `sector_diff_count=2466`
  - `sector_diff_rate=0.9864`
  - `shell_diff_count=185`
- `HOPF_CPX_TRANSPORT_L050_F100`
  - `sector_diff_count=2466`
  - `sector_diff_rate=0.9864`
  - `shell_diff_count=148`
- `HOPF_CPX_TRANSPORT_L100_F100`
  - `sector_diff_count=2466`
  - `sector_diff_rate=0.9864`
  - `shell_diff_count=148`

## Reading
- The coupled complex field is mechanically live in the corrected harness.
- `phase_transport_field_shift_abs_mean` is strongly nonzero, so the field term
  is not decorative.
- The branch changes addresses materially while staying inside the route-health
  gate.
- The screen does **not** yet make the coupled branch the routed quality lead;
  both `phase4d_hopf_base` and pure Hopf still keep better proxy MSE.

## Decision
- Replace the old queued-next framing with the corrected screen result.
- Treat explicit complex-field phase coupling as mechanism-positive and health-positive.
- Use this as the corrected bridge to the next product-phase-field branch rather
  than as proof that the current same-chart coupling law is already optimal.
