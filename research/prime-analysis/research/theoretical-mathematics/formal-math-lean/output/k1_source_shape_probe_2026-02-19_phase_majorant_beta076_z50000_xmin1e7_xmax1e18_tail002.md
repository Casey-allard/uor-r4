# K1 Source Shape Probe

Generated: 2026-02-19T08:17:02.126006+00:00

- zeros used: 50000
- tau candidates: 1
- beta grid: [0.76]
- x-range: [1e+07, 1e+18]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.760000
- tau: 14.134725142
- amplitude: 3.500676e-04
- tail ratio (sup/amp): 1.006702e+00
- tail decay exponent (sup envelope): -0.000000
- remainder majorant eta: 0.000000
- remainder majorant C_all: 7.218436e-03
- remainder majorant C_tail: 3.524139e-04
- remainder majorant tail window: [6.09609e+17, 1e+18]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.502418e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.000000` and `C_all=7.218436e-03` (finite range)
