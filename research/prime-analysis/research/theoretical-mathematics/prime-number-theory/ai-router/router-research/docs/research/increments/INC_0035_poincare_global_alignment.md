# INC-0035: Poincare-Global Alignment

## Status
Active.

### Slice A
- Completed:
  - Poincare-ball global-alignment diagnostics are now live in both router entrypoints.
  - Evidence:
    - `results/analysis/inc0035_alignment_diag_screen.json`
    - `docs/governance/gates/gate_20260306_030909.md`
    - `configs/proxy_transfer_inc0035_alignment_diag_screen.json`
- Result:
  - `HOPF_PHI2_BAND`: `mse=0.0039101`, `total=37.623s`, `pair_mae=0.103762`, `pair_corr=0.840958`, `chi_bin_pmax=0.9429`
  - `PHASE_K25_C035`: `mse=0.0039238`, `total=35.415s`, `pair_mae=0.231601`, `pair_corr=0.678286`, `chi_bin_pmax=0.8200`
  - `HOPF_K25_BASE`: `mse=0.0039342`, `total=36.150s`, `pair_mae=0.118315`, `pair_corr=0.833911`, `chi_bin_pmax=0.7767`
  - `R0`: `mse=0.0039462`, `total=36.584s`, `pair_mae=0.147991`, `pair_corr=0.799078`
- Reading:
  - the alignment metric is live and not redundant with proxy loss
  - the phase-coupled family is clearly the most globally distorted route in this screen
  - the banded widened-Hopf family preserved global alignment best on mean while still carrying severe `chi` concentration
  - pure Hopf remains the routed-quality lead from the full confirm; this fast screen is diagnostic only and does not replace that lead
- Next slice:
  - pilot a route law that anchors shell assignment to original Poincare-ball geodesic radius before any chart-induced widening

### Slice B
- Completed:
  - implemented `sector_mode=phase4d_hopf_ball`
  - evidence:
    - `results/analysis/inc0035_shell_anchor_screen.json`
    - `docs/governance/gates/gate_20260306_032618.md`
    - `configs/proxy_transfer_inc0035_shell_anchor_screen.json`
- Result:
  - `HOPF_PHI2_BAND`: `mse=0.0039101`, `total=39.743s`, `shell_pmax=0.5583`, `pair_mae=0.103762`
  - `HOPF_K25_BALL`: `0.0039235`, `39.581s`, `shell_pmax=0.7988`, `pair_mae=0.157295`
  - `PHASE_K25_C035`: `0.0039238`, `40.472s`, `shell_pmax=0.5754`, `pair_mae=0.231601`
  - `HOPF_K25_BASE`: `0.0039342`, `37.032s`, `shell_pmax=0.5275`, `pair_mae=0.118315`
  - `R0`: `0.0039462`, `30.975s`, collapsed
- Reading:
  - naive shell anchoring improved MSE versus pure Hopf, but worsened shell concentration and global alignment
  - the radial fix alone is too blunt when sectors are still chart-driven
  - the stronger widened-Hopf candidate remains better than the shell-anchor pilot on alignment and route health
  - this points away from shell-only repair and toward a coordinated chart-isometry / global-routing branch

## Hypothesis
The project is likely losing too much useful structure when it maps from the Poincare ball into tangent/chart space and then rebuilds routing with local shell/sector control laws.
The next serious branch should preserve more of the original global `B^4` / `H^4` alignment directly in the route law instead of layering more local widening corrections on top.

## Mathematical Basis
1. One `H^4` object in the `B^4` Poincare-ball model remains the right global manifold.
2. The angular part should still be treated through `S^3` / Hopf coordinates:
   - `chi`
   - `theta1`
   - `theta2`
3. Shells should be driven by geodesic radius, not only local post-chart heuristics.
4. Discrete `phi` / Fibonacci structure should remain a secondary lattice on top of the global geometry, not the primary allocator.

## Minimal Scope
1. Add diagnostics that measure how much the learned chart deviates from global Poincare-ball alignment.
2. Add a pilot route law that derives shell and angular capacity from geodesic `H^4` quantities before any local widening.
3. Keep the route law low-rank:
   - no per-point local gating first
   - no new complex local branch first
4. Compare against:
   - `HOPF_K25_BASE`
   - `HOPF_PHI2_BAND`
   - `PHASE_K25_C035`
   - `R0`

## Acceptance Signal
Keep the branch only if it does all of the following:
- preserves or improves pure Hopf quality
- widens capacity without reproducing severe `chi` concentration
- materially improves runtime relative to `HOPF_K25_BASE`
- keeps a clear mathematical interpretation tied to Poincare-ball global alignment

## Current Direction Inside INC-0035
1. Treat Slice A as proof that the alignment metric is worth keeping.
2. Do not reinterpret the fast diagnostic screen as a lead promotion batch.
3. Treat Slice B as a negative result for shell-only repair.
4. Use the next slice to test a coordinated global-routing change:
   - routing from a more isometric chart or rotation-only route coordinates
   - shells and sectors should share the same global source
   - no new local widening controller yet

## Fallback
If the global-alignment branch still fails, move to:
- `INC_0027_chart_cost_decomposition.md`
or a more explicit chart-isometry simplification branch.
