# K1 Source Shape Probe

Generated: 2026-02-19T08:07:43.168792+00:00

- zeros used: 4096
- tau candidates: 1
- beta grid: [0.6]
- x-range: [5000, 1e+14]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.600000
- tau: 14.134725142
- amplitude: 2.374283e-02
- tail ratio (sup/amp): 2.166656e+00
- tail decay exponent (sup envelope): 0.077869
- remainder majorant eta: 0.077869
- remainder majorant C_all: 4.464585e-01
- remainder majorant C_tail: 4.324323e-01
- remainder majorant tail window: [2.49404e+10, 1e+14]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.802003e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.077869` and `C_all=4.464585e-01` (finite range)
