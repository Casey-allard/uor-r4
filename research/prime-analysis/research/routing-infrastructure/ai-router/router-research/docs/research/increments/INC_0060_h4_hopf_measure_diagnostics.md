# INC-0060: `H^4` / Hopf Measure Diagnostics

## Status
Completed.

## Branch
- `codex/RR-060-h4-hopf-measure-diagnostics`

## Trigger
The imported theory corpus and the new mechanism-first plan agree on the first missing piece:
- current routing evidence is still dominated by task metrics and runtime
- the project is missing direct geometry-specific invariants
- without those invariants, the route law keeps drifting into repair heuristics

## Hypothesis
The current routed frontier is carrying real signal, but measure mismatch is likely still a root cause of:
- shell collapse
- over-widening repairs
- unstable angular allocation

If that is true, then the first mechanism-first slice should expose:
- shell occupancy vs `H^4` shell mass
- Hopf angular occupancy vs equal-mass `chi` and periodic `theta` structure
- route entropy growth with radius
- geodesic neighborhood preservation

## Minimal Scope
1. Add reusable diagnostics helpers in the router core.
2. Emit the diagnostics from the proxy harness JSON summary.
3. Extend sweep summaries and gate notes with the new geometry metrics.
4. Add targeted unit coverage.
5. Run a focused 2-seed screen on the current routed frontier:
   - `R0`
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`

## Public Outputs
- Geometry diagnostics:
  - `shell_mass_error_l1`
  - `shell_mass_error_max`
  - `shell_mass_kl`
  - `shell_mass_corr`
  - `hopf_angular_mass_error`
  - `hopf_chi_mass_error`
  - `hopf_theta1_mass_error`
  - `hopf_theta2_mass_error`
  - `route_entropy_radius_corr`
  - `route_entropy_radius_slope`
  - `geodesic_knn_overlap_mean`
  - `geodesic_knn_jaccard_mean`

## Acceptance
- diagnostics compile and are covered by unit tests
- current frontier routes produce stable geometry metrics in a screen run
- the screen is sufficient to rank which current family is closest to the intended `H^4` / Hopf measure law

## Follow-On
- If measure mismatch is visible:
  - move next to a measure-consistent routing law
- If measure mismatch is already low:
  - move next to phase necessity or coupled-field diagnostics without reopening shell heuristics

## Result
Artifacts:
- `configs/proxy_transfer_inc0060_measure_diag_screen.json`
- `results/analysis/inc0060_measure_diag_screen.json`
- `docs/governance/gates/gate_20260310_230415.md`

2-seed screen means:
- `HOPF_K25_BASE_IT40_P2_STATIC`
  - `mse=0.0039027`
  - `total=7.504s`
  - `shell_mass_l1=1.1416`
  - `hopf_mass=1.3020`
  - `knn_overlap=0.7490`
- `HOPF_PHI2_BAND_IT40_P2_STATIC`
  - `mse=0.0039048`
  - `total=6.632s`
  - `shell_mass_l1=1.1140`
  - `hopf_mass=1.2739`
  - `knn_overlap=0.8806`
- `R0`
  - `mse=0.0039164`
  - `total=7.684s`
  - shell-collapsed
  - health fail

## Decision
- Geometry routing is real enough to keep pursuing:
  - both routed Hopf families beat the collapsed dense control on the proxy task
- The current route law is still not measure-consistent:
  - shell-mass error is high
  - angular mass remains concentrated
- Widened Hopf is the stronger geometry reference for the next correction because it preserved neighborhood structure better than pure Hopf.
- Next branch:
  - `docs/research/increments/INC_0061_measure_consistent_route_law.md`
