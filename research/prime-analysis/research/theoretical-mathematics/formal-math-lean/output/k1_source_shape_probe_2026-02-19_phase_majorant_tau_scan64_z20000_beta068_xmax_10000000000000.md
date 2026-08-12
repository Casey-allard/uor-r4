# K1 Source Shape Probe

Generated: 2026-02-19T08:08:55.998721+00:00

- zeros used: 20000
- tau candidates: 64
- beta grid: [0.68]
- x-range: [5000, 1e+13]
- grid size: 1536
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.680000
- tau: 14.134725142
- amplitude: 7.827928e-03
- tail ratio (sup/amp): 1.337002e+00
- tail decay exponent (sup envelope): 0.034981
- remainder majorant eta: 0.034981
- remainder majorant C_all: 1.333311e-01
- remainder majorant C_tail: 2.480583e-02
- remainder majorant tail window: [5.57384e+09, 1e+13]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.344568e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.034981` and `C_all=1.333311e-01` (finite range)
