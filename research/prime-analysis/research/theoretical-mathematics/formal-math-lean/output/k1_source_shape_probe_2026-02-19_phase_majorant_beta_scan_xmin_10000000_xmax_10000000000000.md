# K1 Source Shape Probe

Generated: 2026-02-19T08:08:06.161129+00:00

- zeros used: 4096
- tau candidates: 1
- beta grid: [0.6, 0.64, 0.68, 0.72, 0.76]
- x-range: [1e+07, 1e+13]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.760000
- tau: 14.134725142
- amplitude: 6.078908e-04
- tail ratio (sup/amp): 1.540923e+00
- tail decay exponent (sup envelope): 0.057125
- remainder majorant eta: 0.057125
- remainder majorant C_all: 1.818427e-02
- remainder majorant C_tail: 3.987535e-03
- remainder majorant tail window: [7.96744e+10, 1e+13]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.410945e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.057125` and `C_all=1.818427e-02` (finite range)
