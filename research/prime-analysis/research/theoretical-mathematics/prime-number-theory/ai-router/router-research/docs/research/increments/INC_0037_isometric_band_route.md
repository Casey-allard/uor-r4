# INC-0037: Isometric-Band Routing

## Status
Completed.

## Hypothesis
`INC-0036` showed that pure rotation-only routing preserves geometry perfectly but over-compresses capacity and loses the practical frontier. The next plausible route is:
- start from a near-isometric base coordinate
- then apply shared-state widening on top of that coordinate
- avoid letting learned chart scale dominate the routing geometry

## Mathematical Basis
1. `phase4d_hopf_iso` proves the route can recover exact Poincare alignment when the scale term is removed.
2. `phase4d_hopf_fib_band` proves shared-state widening can reopen Hopf capacity without the full cost of ungated `phi^2` forcing.
3. The live question was whether their combination could preserve geometry and still keep an operational widening signal.

## Implementation
- Added `sector_mode=phase4d_hopf_fib_band_iso`.
- The new mode:
  - routes on the rotation-only chart coordinate
  - uses the existing shared-state Hopf band allocator on top of that coordinate
- Updated:
  - `hyperbolic_router_so8.py`
  - `tasks/router_proxy_eval.py`
  - CLI/tests/contracts

## Evidence
- Config: `configs/proxy_transfer_inc0037_isometric_band_screen.json`
- Analysis: `results/analysis/inc0037_isometric_band_screen.json`
- Gate: `docs/governance/gates/gate_20260306_075923.md`

## 2-Seed Screen Means
- `HOPF_PHI2_BAND`: `mse=0.003910130`, `total=30.184s`, `pair_mae=0.103762`, health pass
- `PHASE_K25_C035`: `0.003923775`, `30.686s`, `pair_mae=0.231601`, health pass
- `HOPF_PHI2_BAND_ISO`: `0.003924123`, `43.879s`, `pair_mae=0.000000`, health fail on runtime only
- `HOPF_K25_ISO`: `0.003926004`, `37.821s`, `pair_mae=0.000000`, health fail on runtime only
- `HOPF_K25_BASE`: `0.003934246`, `29.686s`, `pair_mae=0.118315`, health pass
- `R0`: `0.003946221`, `28.349s`, collapsed

## Reading
- The combination did prove the geometric point:
  - perfect Poincare alignment and widened sectors can coexist in one route family.
- It failed operationally:
  - runtime worsened further than pure `HOPF_K25_ISO`
  - the widening signal survived (`11` sectors, `16.5` buckets), but the cost exploded
- Interpretation:
  - pure scale removal is not enough
  - pure scale removal plus widening is still not enough
  - learned chart scale appears to be carrying some operationally useful structure even when it distorts geometry

## Decision
- Keep `HOPF_PHI2_BAND_ISO` as a strong mathematical result and a negative operational result.
- Keep `HOPF_PHI2_BAND` as the healthiest widened Hopf quality branch.
- Do not promote any isometric route as an operational lead yet.
- Move next to `INC-0038`: bounded-isometry band routing.

## Next Increment
- `docs/research/increments/INC_0038_bounded_isometry_band_route.md`
