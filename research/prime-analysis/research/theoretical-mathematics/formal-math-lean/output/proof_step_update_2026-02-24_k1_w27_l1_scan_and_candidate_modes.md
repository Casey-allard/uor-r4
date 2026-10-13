# Proof Step Update — 2026-02-24 (K1-W27 L1 Scan + Candidate Modes)

## What was done
Pushed the remaining L1 gate with two concrete moves:

1. Formal lock (Lean): proved bidirectional equivalence between the new L1 kernel term and the existing finite-omitted witness term.
2. Numerical lock (research): ran a stability scan over `x1/eta/omega_count/ridge/policy` and extracted best candidate mode sets for theorem-facing work.

## Lean update
File:
- `/Users/adminamn/Documents/New project/research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean`

Added:
- `r6_dual_band_tail_representation_kernel_of_witness_with_window_and_finite_omitted_modes`
- `r6_dual_band_tail_representation_kernel_iff_witness_with_window_and_finite_omitted_modes`

Build:
- `lake build PrimeRiemannBridgeSpinningTopFrontier` ✅
- `lake build PrimeRiemannBridgeSchlagePuchta2019ImportedInstance` ✅

## New artifacts
- Scan summary:
  - `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_scan_2026-02-24.json`
  - `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_scan_2026-02-24.md`
- Candidate mode export:
  - `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_candidate_modes_2026-02-24_topk.json`
  - `/Users/adminamn/Documents/New project/research/output/r6_tail_representation_candidate_modes_2026-02-24_topk.md`

## Best stable L1 candidate (from scan)
- `x1 = 1e21`
- `eta = 0.01`
- `omega_policy = top`
- `omega_count = 256`
- `ridge_alpha = 0.1`

Metrics:
- `tail_sup_ratio_after_over_before = 0.052312069138122266`
- `tail_rmse_ratio_after_over_before = 0.06139848718318631`
- `all_sup_ratio_after_over_before = 1.0906093171234856`
- `eps_majorant_C_tail_with_eta = 0.004548531233749219`
- `eps_majorant_C_all_with_eta = 0.2981776550583154`

## Interpretation
- Deep-tail representation fit is now strong and stable enough to treat as a concrete theorem-target candidate shape.
- Formal closure remains unchanged in type: we still need a non-circular theorem term for
  `R6DualBandTailRepresentationKernelTerm`.

## Next step
- Attempt an explicit-formula-to-kernel derivation lemma using the candidate omitted-mode shape (`eta=0.01`, top modes) as the primary hypothesis contract, then instantiate `R6DualBandTailRepresentationKernelProvider`.
