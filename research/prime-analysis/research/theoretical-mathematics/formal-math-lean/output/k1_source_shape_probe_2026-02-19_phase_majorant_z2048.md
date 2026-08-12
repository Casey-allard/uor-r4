# K1 Source Shape Probe

Generated: 2026-02-19T07:38:12.389782+00:00

- zeros used: 2048
- tau candidates: 48
- beta grid: [0.5, 0.52, 0.55, 0.58, 0.6]
- x-range: [10000, 1e+09]
- grid size: 8000
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.600000
- tau: 14.134725142
- amplitude: 3.311246e-02
- tail ratio (sup/amp): 2.766424e+00
- tail decay exponent (sup envelope): 0.132310
- remainder majorant eta: 0.132310
- remainder majorant C_all: 1.179248e+00
- remainder majorant C_tail: 1.179248e+00
- remainder majorant tail window: [1.00115e+08, 1e+09]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 4.209719e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.132310` and `C_all=1.179248e+00` (finite range)
