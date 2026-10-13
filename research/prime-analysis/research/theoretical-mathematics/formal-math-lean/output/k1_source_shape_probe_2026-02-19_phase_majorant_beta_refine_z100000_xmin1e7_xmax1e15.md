# K1 Source Shape Probe

Generated: 2026-02-19T08:15:58.104018+00:00

- zeros used: 100000
- tau candidates: 1
- beta grid: [0.74, 0.76, 0.78]
- x-range: [1e+07, 1e+15]
- grid size: 1536
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.740000
- tau: 14.134725142
- amplitude: 7.148911e-04
- tail ratio (sup/amp): 1.259484e+00
- tail decay exponent (sup envelope): 0.035053
- remainder majorant eta: 0.035053
- remainder majorant C_all: 2.006840e-02
- remainder majorant C_tail: 2.517879e-03
- remainder majorant tail window: [1.58966e+12, 1e+15]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.307289e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.035053` and `C_all=2.006840e-02` (finite range)
