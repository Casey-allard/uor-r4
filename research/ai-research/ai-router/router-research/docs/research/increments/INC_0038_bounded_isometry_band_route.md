# INC-0038: Bounded-Isometry Band Routing

## Status
Completed.

## Hypothesis
If pure isometry is too expensive and full chart scale is too distorted, a bounded route coordinate should recover part of the Poincare alignment signal while staying operationally closer to the widened Hopf band route.

## Mathematical Basis
1. `INC-0036` proved pure isometry restores Poincare alignment exactly.
2. `INC-0037` proved exact alignment and widened capacity can coexist, but at unacceptable cost.
3. The remaining live degree of freedom was partial scale:
   - keep routing closer to the ball-aligned geometry than `HOPF_PHI2_BAND`
   - retain enough learned chart scale to avoid the full isometric runtime penalty

## Implementation
- Added `sector_mode=phase4d_hopf_fib_band_bound`.
- Added `apply_chart_route_blend(v, chart, route_scale_lambda)`.
- Added `--route_scale_lambda` to the router and proxy harness.
- Added tests for:
  - route-scale interpolation
  - `lambda=0` matching the isometric route
  - bounded alignment sitting between pure band and pure isometry

## Evidence
- Config: `configs/proxy_transfer_inc0038_bounded_band_screen.json`
- Analysis: `results/analysis/inc0038_bounded_band_screen.json`
- Gate: `docs/governance/gates/gate_20260306_082106.md`

## 2-Seed Screen Means
- `HOPF_PHI2_BAND`: `mse=0.003910130`, `total=35.837s`, `pair_mae=0.103762`, health pass
- `HOPF_PHI2_BAND_B075`: `0.003914296`, `46.314s`, `pair_mae=0.077111`, health fail on runtime only
- `HOPF_PHI2_BAND_B025`: `0.003918418`, `56.210s`, `pair_mae=0.004458`, health fail on runtime only
- `HOPF_PHI2_BAND_ISO`: `0.003924123`, `42.842s`, `pair_mae=0.000000`, health fail on runtime only
- `HOPF_PHI2_BAND_B050`: `0.003925862`, `46.515s`, `pair_mae=0.055238`, health fail on runtime only
- `HOPF_K25_BASE`: `0.003934246`, `33.363s`, `pair_mae=0.118315`, health pass
- `R0`: `0.003946221`, `32.968s`, collapsed

## Reading
- The bounded ladder behaved exactly like a geometry/runtime interpolation:
  - lower `route_scale_lambda` improved alignment strongly
  - lower `route_scale_lambda` also inherited the isometric runtime penalty
- No bounded point cleared the configured route-health gate.
- `B050` was the intended middle candidate, but it did not reproduce a usable middle frontier across seeds.
- The widened Hopf band route remains the healthiest quality branch in this family.

## Decision
- Kill bounded-isometry band routing as an active promotion branch.
- Keep `phase4d_hopf_fib_band_bound` as a diagnostic family showing the alignment/runtime continuum is real.
- Keep `HOPF_PHI2_BAND` as the widened Hopf geometry candidate.
- Move next to explicit route/memory coordinate separation:
  - address with a geometry-aligned route coordinate
  - store and compare with the full learned chart coordinate

## Next Increment
- `docs/research/increments/INC_0039_route_memory_separation.md`
