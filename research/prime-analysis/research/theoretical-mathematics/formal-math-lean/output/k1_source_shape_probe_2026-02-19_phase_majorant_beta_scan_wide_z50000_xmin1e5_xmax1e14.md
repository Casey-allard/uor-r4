# K1 Source Shape Probe

Generated: 2026-02-19T08:15:14.258814+00:00

- zeros used: 50000
- tau candidates: 1
- beta grid: [0.55, 0.58, 0.6, 0.62, 0.64, 0.66, 0.68, 0.7, 0.72, 0.74, 0.76, 0.78, 0.8, 0.82, 0.84, 0.86, 0.88, 0.9, 0.92, 0.94]
- x-range: [100000, 1e+14]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.700000
- tau: 14.134725142
- amplitude: 3.329312e-03
- tail ratio (sup/amp): 1.448818e+00
- tail decay exponent (sup envelope): 0.067108
- remainder majorant eta: 0.067108
- remainder majorant C_all: 9.822449e-02
- remainder majorant C_tail: 3.052466e-02
- remainder majorant tail window: [7.11178e+10, 1e+14]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.540247e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.067108` and `C_all=9.822449e-02` (finite range)
