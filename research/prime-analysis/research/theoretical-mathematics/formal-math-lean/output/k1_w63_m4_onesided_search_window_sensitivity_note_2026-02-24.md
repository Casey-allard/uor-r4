# K1 W63 M4 One-Sided Search-Window Sensitivity (2026-02-24)

## Goal
Check whether the one-sided M4 tail-split margin is stable across constructive neighborhood sizes \(W\).

## Setup
- Gate: tau14 anchor phase, \(|\cos_1|\ge a_1=0.98\), \(\cos_2\ge0\), phase-only selection.
- Split: \(R_2 = T_4 + E_4\), one-sided metric via negative parts.
- Windows: \(x\in\{10^7,2\cdot10^7,3\cdot10^7,4\cdot10^7,5\cdot10^7\}\).
- Tested \(W\in\{100,200,500,1000\}\).

## Aggregated five-window results
- `W=100`:
  - `delta_split_neg_min ≈ 0.549699`
  - `q_split_neg_max ≈ 0.430301`
- `W=200`:
  - `delta_split_neg_min ≈ 0.532328`
  - `q_split_neg_max ≈ 0.447672`
- `W=500`:
  - `delta_split_neg_min ≈ 0.514013`
  - `q_split_neg_max ≈ 0.465987`
- `W=1000`:
  - `delta_split_neg_min ≈ 0.190588`
  - `q_split_neg_max ≈ 0.789412`
  - absolute split margin (`delta_split_min`) becomes negative.

## Conclusion
`W=100` remains the strongest and most stable choice in this sweep for the one-sided M4 route.

Use this as the current constructive parameter in theorem-target constants until an analytic neighborhood-control lemma is proved.

Artifacts:
- `research/output/k1_w63_tail_split_a098_W100_five_window_summary_M4.md`
- `research/output/k1_w63_tail_split_a098_W200_five_window_summary_M4.md`
- `research/output/k1_w63_tail_split_a098_W500_five_window_summary_M4.md`
- `research/output/k1_w63_tail_split_a098_W1000_five_window_summary_M4.md`
