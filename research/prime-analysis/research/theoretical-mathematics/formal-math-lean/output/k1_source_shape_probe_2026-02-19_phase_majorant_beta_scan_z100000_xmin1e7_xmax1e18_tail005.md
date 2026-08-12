# K1 Source Shape Probe

Generated: 2026-02-19T08:17:07.210253+00:00

- zeros used: 100000
- tau candidates: 1
- beta grid: [0.74, 0.75, 0.76, 0.77]
- x-range: [1e+07, 1e+18]
- grid size: 1536
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.740000
- tau: 14.134725142
- amplitude: 5.051580e-04
- tail ratio (sup/amp): 1.011913e+00
- tail decay exponent (sup envelope): -0.000000
- remainder majorant eta: 0.000000
- remainder majorant C_all: 9.567702e-03
- remainder majorant C_tail: 5.111760e-04
- remainder majorant tail window: [2.85348e+17, 1e+18]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.554990e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.000000` and `C_all=9.567702e-03` (finite range)
