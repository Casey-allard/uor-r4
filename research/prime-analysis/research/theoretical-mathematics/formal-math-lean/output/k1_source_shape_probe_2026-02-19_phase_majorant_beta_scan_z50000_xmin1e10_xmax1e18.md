# K1 Source Shape Probe

Generated: 2026-02-19T08:16:52.866647+00:00

- zeros used: 50000
- tau candidates: 1
- beta grid: [0.74, 0.75, 0.76, 0.77, 0.78]
- x-range: [1e+10, 1e+18]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.740000
- tau: 14.134725142
- amplitude: 1.282187e-04
- tail ratio (sup/amp): 1.314336e+00
- tail decay exponent (sup envelope): 0.038635
- remainder majorant eta: 0.038635
- remainder majorant C_all: 5.089259e-03
- remainder majorant C_tail: 6.781864e-04
- remainder majorant tail window: [1.59132e+15, 1e+18]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.399330e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.038635` and `C_all=5.089259e-03` (finite range)
