# K1 Source Shape Probe

Generated: 2026-02-24T06:14:11.298341+00:00

- zeros used: 20000
- tau candidates: 64
- beta grid: [0.6]
- x-range: [1e+07, 1e+18]
- grid size: 4096
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.600000
- tau: 14.134725142
- amplitude: 1.034123e-02
- tail ratio (sup/amp): 1.613798e+00
- tail decay exponent (sup envelope): 0.050297
- remainder majorant eta: 0.050297
- remainder majorant C_all: 2.249721e-01
- remainder majorant C_tail: 1.206895e-01
- remainder majorant tail window: [6.34872e+15, 1e+18]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.306105e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.050297` and `C_all=2.249721e-01` (finite range)
