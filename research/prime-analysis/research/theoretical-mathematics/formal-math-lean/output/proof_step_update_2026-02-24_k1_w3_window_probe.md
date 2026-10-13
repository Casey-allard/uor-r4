# Proof Step Update (2026-02-24): K1-W3 Finite-Mode Window Probe

## Completed in this step

1. Added and compiled finite-mode formal reduction path (`finite decaying modes -> asymptotically linear -> RH chain`).
2. Added final-lock bridge theorem from nonempty finite-mode provider to RH.
3. Ran deterministic cached window scan to test practical behavior of the new finite-mode contract.

## Deterministic probe run

Command pattern used (cached):

- `python3 research/k1_source_shape_probe.py --max-zeta-zeros 20000 --beta-grid 0.6 --x-min 1e7 --x-max {1e12..1e20} --grid-size 4096 --tail-frac 0.20 --tau-candidate-count 64 ...`

Outputs:

- `/Users/adminamn/Documents/New project/research/output/k1_source_shape_probe_2026-02-24_finite_mode_window_xmax_1e12.json`
- `/Users/adminamn/Documents/New project/research/output/k1_source_shape_probe_2026-02-24_finite_mode_window_xmax_1e14.json`
- `/Users/adminamn/Documents/New project/research/output/k1_source_shape_probe_2026-02-24_finite_mode_window_xmax_1e16.json`
- `/Users/adminamn/Documents/New project/research/output/k1_source_shape_probe_2026-02-24_finite_mode_window_xmax_1e18.json`
- `/Users/adminamn/Documents/New project/research/output/k1_source_shape_probe_2026-02-24_finite_mode_window_xmax_1e20.json`
- Summary: `/Users/adminamn/Documents/New project/research/output/k1_finite_mode_window_trend_2026-02-24.md`

Observed:

- Best mode frequency stayed stable at `tau = 14.134725142` (first zeta zero frequency proxy).
- Tail sup/amplitude ratio ranged from `~2.8746` down to `~1.6138`, with `1e20` at `~1.7792`.
- Tail RMSE/amplitude ratio stayed in `~0.626 .. 0.863`.
- All windows remained tagged `weak_single_mode_dominance_finite_range`.

Interpretation:

- Current evidence supports a persistent dominant mode but does not yet give a strong, monotone residual suppression needed to promote to an unconditional witness.
- This is compatible with finite-mode contracts but still leaves explicit assumption obligations open.

## Current proof status

- Formal closure chain is intact and green if a concrete non-circular provider is supplied.
- Unconditional closure is still blocked by missing non-circular instantiation of at least one of:
  - `ConcreteFiniteDecayingPhaseCorrectionsProvider`
  - `ConcreteAsymptoticallyLinearPhaseProvider`
  - `ConcreteLinearPhaseWitnessProvider`

## Next step

K1-W4: write explicit obligation table for finite-mode contract fields and classify each as:

- already imported from published source,
- derivable in-repo from existing lemmas,
- still open and needing new mathematics.
