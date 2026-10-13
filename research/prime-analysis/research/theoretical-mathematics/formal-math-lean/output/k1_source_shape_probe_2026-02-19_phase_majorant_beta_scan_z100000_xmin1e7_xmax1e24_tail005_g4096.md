# K1 Source Shape Probe

Generated: 2026-02-19T08:36:53.568699+00:00

- zeros used: 100000
- tau candidates: 16
- beta grid: [0.7, 0.72, 0.74, 0.76]
- x-range: [1e+07, 1e+24]
- grid size: 4096
- overall interpretation: borderline_single_mode_tail_dominance_finite_range

## Overall Best Fit

- beta: 0.700000
- tau: 14.134725142
- amplitude: 7.677604e-04
- tail ratio (sup/amp): 1.006718e+00
- tail decay exponent (sup envelope): 0.005341
- remainder majorant eta: 0.005341
- remainder majorant C_all: 2.168743e-02
- remainder majorant C_tail: 1.030900e-03
- remainder majorant tail window: [1.4227e+23, 1e+24]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.702834e+00
- interpretation: borderline_single_mode_tail_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.005341` and `C_all=2.168743e-02` (finite range)
