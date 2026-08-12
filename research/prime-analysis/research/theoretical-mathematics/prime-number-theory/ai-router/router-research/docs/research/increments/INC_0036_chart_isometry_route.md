# INC-0036: Chart-Isometry Routing Pilot

## Status
Completed.

## Hypothesis
`INC-0035` Slice B showed that shell anchoring alone is not enough. The route likely needs shells and sectors to come from the same more-isometric global coordinate source instead of mixing original-ball shells with chart-distorted angular sectors.

## Mathematical Basis
1. The Poincare-ball alignment diagnostics are live and separate route families meaningfully.
2. `phase4d_hopf_ball` improved proxy MSE relative to pure Hopf, but degraded shell concentration and pairwise alignment.
3. That pattern is consistent with a coordinate mismatch:
   - shells from original-ball geometry
   - sectors from post-chart geometry
4. The minimal fix is a more isometric route coordinate:
   - use the learned rotation
   - ignore learned scale for routing
   - let shells and sectors share that same route coordinate

## Implementation
- Added `apply_chart_isometric(v, chart)` in `hyperbolic_router_so8.py`.
- Added `sector_mode=phase4d_hopf_iso` to:
  - `hyperbolic_router_so8.py`
  - `tasks/router_proxy_eval.py`
  - CLI/tests/contracts
- `phase4d_hopf_iso` routes on the rotation-only chart coordinate while preserving the rest of the Hopf allocator logic.

## Evidence
- Config: `configs/proxy_transfer_inc0036_chart_iso_screen.json`
- Analysis: `results/analysis/inc0036_chart_iso_screen.json`
- Gate: `docs/governance/gates/gate_20260306_074531.md`

## 2-Seed Screen Means
- `HOPF_PHI2_BAND`: `mse=0.003910130`, `total=30.702s`, `pair_mae=0.103762`, `shell_pmax=0.5583`, health pass
- `PHASE_K25_C035`: `0.003923775`, `29.103s`, `pair_mae=0.231601`, `shell_pmax=0.5754`, health pass
- `HOPF_K25_ISO`: `0.003926004`, `42.004s`, `pair_mae=0.000000`, `shell_pmax=0.5813`, health fail on runtime only
- `HOPF_K25_BASE`: `0.003934246`, `33.220s`, `pair_mae=0.118315`, `shell_pmax=0.5275`, health pass
- `R0`: `0.003946221`, `31.958s`, collapsed

## Reading
- The route-law hypothesis was real:
  - `phase4d_hopf_iso` preserved Poincare alignment perfectly on the screen.
- The branch is not promotable:
  - it stayed compressed at `4` sectors / `8` buckets
  - it ran far slower than `R0`
  - it did not beat `HOPF_PHI2_BAND` or `PHASE_K25_C035` on the practical frontier
- Interpretation:
  - learned scale is part of the current operational routing signal
  - pure isometry preserves geometry but removes too much useful widening/compression behavior
  - the next route should combine a near-isometric base coordinate with an explicit widening law, not replace widening with pure isometry

## Decision
- Keep `phase4d_hopf_iso` as a positive mathematical diagnostic and a negative operational result.
- Do not promote it as a route lead.
- Keep `HOPF_K25_BASE` as the routed-quality lead.
- Keep `HOPF_PHI2_BAND` as the widened Hopf geometry candidate.
- Move next to `INC-0037`: isometric-band routing.

## Next Increment
- `docs/research/increments/INC_0037_isometric_band_route.md`
