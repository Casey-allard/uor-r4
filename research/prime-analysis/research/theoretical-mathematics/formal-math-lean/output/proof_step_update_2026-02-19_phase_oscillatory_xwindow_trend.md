# Proof Step Update (2026-02-19): Phase-Oscillatory X-Window Trend

## Goal
Test whether single-mode dominance improves as the x-window is pushed farther out, which is the asymptotic behavior needed by the remaining K1 source term.

## New outputs
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_xwindow_scan.json`
- `research/output/k1_source_shape_probe_2026-02-19_phase_majorant_xwindow_scan.md`

## Findings
- Best-fit `tau` remains locked at `14.134725142` for all tested windows.
- Best-fit `beta` remains `0.60` for all tested windows.
- `tail_sup/amp` trend decreases as `x_max` grows:
  - `x_max=1e7`: `3.305`
  - `x_max=1e8`: `3.182`
  - `x_max=1e9`: `2.794`
  - `x_max=1e10`: `2.440`
- The fitted majorant exponent `eta` stays positive in each run.

## Interpretation
- This is consistent with gradual asymptotic dominance of the principal phase mode, but dominance is still weak over tested finite windows.
- Therefore the final missing K1 theorem term is still open; this run narrows the expected asymptotic direction rather than closing it.
