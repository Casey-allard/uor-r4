# K1 Source Shape Probe

Generated: 2026-02-19T08:18:41.823240+00:00

- zeros used: 50000
- tau candidates: 1
- beta grid: [0.76]
- x-range: [1e+07, 1e+20]
- grid size: 4096
- overall interpretation: near_strict_single_mode_tail_dominance_finite_range

## Overall Best Fit

- beta: 0.760000
- tau: 14.134725142
- amplitude: 2.835582e-04
- tail ratio (sup/amp): 9.995160e-01
- tail decay exponent (sup envelope): 0.143997
- remainder majorant eta: 0.143997
- remainder majorant C_all: 2.101201e-01
- remainder majorant C_tail: 2.101201e-01
- remainder majorant tail window: [2.25103e+19, 1e+20]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.801772e+00
- interpretation: near_strict_single_mode_tail_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.143997` and `C_all=2.101201e-01` (finite range)
