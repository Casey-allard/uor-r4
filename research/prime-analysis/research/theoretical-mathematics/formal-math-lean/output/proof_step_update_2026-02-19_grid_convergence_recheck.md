# Proof Step Update (2026-02-19): K1 Near-Strict Grid-Convergence Recheck

## Objective
Recheck whether the observed finite-range near-strict tail crossing (`tail_ratio_sup_to_amp < 1`) is robust under increased discretization density, while keeping the strongest previously observed window fixed.

## Work performed
1. Build/audit hardening for the near-strict bridge:
   - added `PrimeRiemannBridgeNearStrictTailToPintz` to `research/formal/lean/lakefile.toml`
   - expanded `research/formal_axiom_audit.py` default Lean scan list to include the frontier/final modules and near-strict bridge
2. Lean verification:
   - `lake build PrimeRiemannBridgeNearStrictTailToPintz PrimeRiemannBridgeFinalTargetEquivalence` ✅
   - refreshed axiom audit: `axiom_count = 0` (`research/output/formal_axiom_audit_2026-02-19.json`)
3. Numerical recheck (fixed strongest regime):
   - zeros: 100000
   - beta: 0.74
   - tau candidate count: 1 (first zeta zero)
   - x-window: `[1e7, 1e20]`
   - tail fraction: `0.05`
   - grid sizes rechecked: 1536, 3072, 6144, 7168, 8192

## New artifacts
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g1536_recheck.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g3072_recheck.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g6144_recheck.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g7168_recheck.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_beta074_z100000_xmin1e7_xmax1e20_tail005_g8192_recheck.json`
- `research/output/k1_phase_majorant_grid_convergence_2026-02-19.json`
- `research/output/k1_phase_majorant_grid_convergence_2026-02-19.md`

## Result summary
- `g=1536`: ratio `0.996728` (near-strict)
- `g=3072`: ratio `0.998374` (near-strict)
- `g=6144`: ratio `0.999392` (near-strict)
- `g=7168`: ratio `1.001150` (borderline)
- `g=8192`: ratio `1.001140` (borderline)

## Interpretation
- The finite-range near-strict crossing is real for moderate grids in this fixed regime, but currently sits in a narrow near-threshold band and flips slightly above `1` on denser grids.
- This does **not** close K1; it refines the remaining source obligation into a sharper target:
  derive a **grid/discretization-stable** asymptotic strict-tail theorem term rather than relying on finite-grid crossing alone.

## Status impact
- Remaining open kernel is unchanged: `K1-SOURCE`.
- Proof state remains: one non-circular source theorem term still missing.
