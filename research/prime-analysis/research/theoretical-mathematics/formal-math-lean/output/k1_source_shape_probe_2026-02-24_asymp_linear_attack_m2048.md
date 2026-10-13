# K1 Source Shape Probe

Generated: 2026-02-24T05:58:30.324437+00:00

- zeros used: 2048
- tau candidates: 128
- beta grid: [0.52, 0.55, 0.58, 0.6]
- x-range: [1e+07, 1e+20]
- grid size: 8192
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.600000
- tau: 14.134725142
- amplitude: 9.143844e-03
- tail ratio (sup/amp): 1.781047e+00
- tail decay exponent (sup envelope): 0.064514
- remainder majorant eta: 0.064514
- remainder majorant C_all: 3.256225e-01
- remainder majorant C_tail: 2.314432e-01
- remainder majorant tail window: [2.52293e+17, 1e+20]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.548276e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.064514` and `C_all=3.256225e-01` (finite range)
