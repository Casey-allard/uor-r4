# K1 Source Shape Probe

Generated: 2026-02-19T08:15:38.894846+00:00

- zeros used: 50000
- tau candidates: 1
- beta grid: [0.76, 0.78, 0.79, 0.8, 0.81, 0.82, 0.84]
- x-range: [1e+07, 1e+15]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.760000
- tau: 14.134725142
- amplitude: 4.703159e-04
- tail ratio (sup/amp): 1.257703e+00
- tail decay exponent (sup envelope): 0.031759
- remainder majorant eta: 0.031759
- remainder majorant C_all: 1.367987e-02
- remainder majorant C_tail: 1.569761e-03
- remainder majorant tail window: [1.59132e+12, 1e+15]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.403982e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.031759` and `C_all=1.367987e-02` (finite range)
