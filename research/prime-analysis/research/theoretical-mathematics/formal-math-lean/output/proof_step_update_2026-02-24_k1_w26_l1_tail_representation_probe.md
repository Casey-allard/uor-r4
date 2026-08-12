# Proof Step Update — 2026-02-24 (K1-W26 L1 Tail Representation Probe)

## What was done
Advanced the remaining L1 gate (`tail_eq_omitted`) with two concrete actions:

1. Added a direct Lean kernel contract for the exact open theorem shape:
- `R6DualBandTailRepresentationKernelTerm`
- bridge theorem: `r6_dual_band_witness_with_window_and_finite_omitted_modes_of_tail_representation_kernel`
- provider endpoint: `rh_from_r6_dual_band_tail_representation_kernel_provider`

2. Added and ran a focused numerical probe for L1 omitted-mode representation:
- script: `/Users/adminamn/Documents/New project/research/r6_tail_representation_probe.py`
- fits `R(x)/x^beta` tail as finite decaying modes
  `sum_j kappa_j * x^(-eta) * sin(omega_j * log x + theta_j)`
  with optional ridge stabilization.

## Lean changes
File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

Build verification:
- `lake build PrimeRiemannBridgeSpinningTopFrontier` ✅
- `lake build PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` ✅

## New research artifacts
Primary probe outputs:
- `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_probe_2026-02-24_x1e22_selected12.json`
- `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_probe_2026-02-24_x1e22_selectedplus24.json`
- `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_probe_2026-02-24_x1e22_top64.json`
- `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_probe_2026-02-24_x1e22_top128_r1e-2.json`
- `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_probe_2026-02-24_x1e22_top256_eta001_r1e-1.json`
- `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_probe_2026-02-24_x1e22_top256_eta001_x1e21_r1e-1.json`

## Key numerical findings
Baseline tail window (`x1` from multimode tail start, `~1.008e19`):
- best stable run so far (top-256 omegas, `eta=0.01`, ridge `1e-1`):
  - `tail_sup_ratio_after_over_before = 0.3528161703608778`
  - `tail_rmse_ratio_after_over_before = 0.19079441050623044`
  - `all_sup_ratio_after_over_before = 0.9972122912521425`

Far-tail window (`x1=1e21`):
- top-256 omegas, `eta=0.01`, ridge `1e-1`:
  - `tail_sup_ratio_after_over_before = 0.052312069138122266`
  - `tail_rmse_ratio_after_over_before = 0.06139848718318631`
  - `all_sup_ratio_after_over_before = 1.0906093171234856`

Interpretation:
- On deep tail windows, finite decaying-mode representation captures most of the residual energy.
- This is strong evidence for L1 plausibility, but still not a formal theorem term.

## Status impact
- L2-L4 remain formally closed.
- L1 is still open, but now pinned to one explicit contract (`R6DualBandTailRepresentationKernelTerm`) and supported by stronger tail-focused evidence.

## Next step
- Formal L1 sublemma: derive a theorem term for `R6DualBandTailRepresentationKernelTerm` from an explicit-formula omitted-mode decomposition hypothesis (non-circular), then instantiate `R6DualBandTailRepresentationKernelProvider`.
