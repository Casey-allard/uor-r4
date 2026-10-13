# K1 W61 Anchor-Existence Note (2026-02-24)

## Purpose
Support Lemma C1/C2 feasibility with a phase-only diagnostic (no residual-based selection).

## Method
- Script:
  - `research/fixed_error_psi_tau12_anchor_existence_probe.py`
- Fit two-mode phases \((\phi_1,\phi_2)\) on each window.
- Construct tau1 anchor centers from phase equation.
- Check:
  1. center sign density: \(\cos(\tau_2\log x+\phi_2)\ge0\),
  2. neighborhood success: \(\exists n\) in \([center-W,center+W]\) with
     \(|\cos_1|\ge a_1=0.98\) and \(\cos_2\ge0\).

## Key finite-window findings (`x=1e7..5e7`)
- For all tested \(W\in\{0,20,50,100,200,500,1000\}\):
  - center nonnegative fraction \(\approx 0.5\),
  - constructive success fraction \(\approx 0.5\),
  - success fraction equals center fraction (window width had no effect in this range).
- At \(W=0\), on successful anchors:
  - \(\min |\cos_1| > 0.9999998\) in all windows,
  - \(\min \cos_2\) remained strictly positive (about \(0.68\) to \(0.79\)).

Artifacts:
- `research/output/k1_w61_tau12_anchor_existence_2026-02-24_x1e7.md`
- `research/output/k1_w61_tau12_anchor_existence_2026-02-24_x2e7.md`
- `research/output/k1_w61_tau12_anchor_existence_2026-02-24_x3e7.md`
- `research/output/k1_w61_tau12_anchor_existence_2026-02-24_x4e7.md`
- `research/output/k1_w61_tau12_anchor_existence_2026-02-24_x5e7.md`

## Interpretation
This strongly supports a constructive C1/C2 route:
- tau1 anchors already provide \(|\cos_1|\approx1\),
- roughly half of those anchors satisfy \(\cos_2\ge0\) directly at center.

Still finite-window evidence only; theorem closure needs an analytic recurrence/equidistribution argument.
