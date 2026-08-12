# K1 Source Shape Probe

Generated: 2026-02-19T08:25:01.550341+00:00

- zeros used: 100000
- tau candidates: 1
- beta grid: [0.76]
- x-range: [1e+07, 1e+22]
- grid size: 1536
- overall interpretation: borderline_single_mode_tail_dominance_finite_range

## Overall Best Fit

- beta: 0.760000
- tau: 14.134725142
- amplitude: 2.815246e-04
- tail ratio (sup/amp): 1.005032e+00
- tail decay exponent (sup envelope): 0.008350
- remainder majorant eta: 0.008350
- remainder majorant C_all: 8.187666e-03
- remainder majorant C_tail: 4.282654e-04
- remainder majorant tail window: [1.02276e+19, 1e+22]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.781544e+00
- interpretation: borderline_single_mode_tail_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.008350` and `C_all=8.187666e-03` (finite range)
