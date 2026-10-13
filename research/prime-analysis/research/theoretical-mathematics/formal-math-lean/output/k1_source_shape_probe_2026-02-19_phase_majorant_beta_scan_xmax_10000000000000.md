# K1 Source Shape Probe

Generated: 2026-02-19T08:07:52.729846+00:00

- zeros used: 4096
- tau candidates: 1
- beta grid: [0.52, 0.54, 0.56, 0.58, 0.6, 0.62, 0.64, 0.66, 0.68, 0.7, 0.72]
- x-range: [5000, 1e+13]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.680000
- tau: 14.134725142
- amplitude: 7.877408e-03
- tail ratio (sup/amp): 1.476083e+00
- tail decay exponent (sup envelope): 0.037687
- remainder majorant eta: 0.037687
- remainder majorant C_all: 1.357407e-01
- remainder majorant C_tail: 2.875038e-02
- remainder majorant tail window: [5.58064e+09, 1e+13]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.464111e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.037687` and `C_all=1.357407e-01` (finite range)
