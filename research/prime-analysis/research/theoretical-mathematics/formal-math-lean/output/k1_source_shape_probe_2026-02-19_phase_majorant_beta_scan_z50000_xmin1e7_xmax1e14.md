# K1 Source Shape Probe

Generated: 2026-02-19T08:14:40.832552+00:00

- zeros used: 50000
- tau candidates: 1
- beta grid: [0.6, 0.64, 0.68, 0.72, 0.76, 0.8, 0.84, 0.88]
- x-range: [1e+07, 1e+14]
- grid size: 2048
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.800000
- tau: 14.134725142
- amplitude: 2.469479e-04
- tail ratio (sup/amp): 1.339932e+00
- tail decay exponent (sup envelope): 0.064374
- remainder majorant eta: 0.064374
- remainder majorant C_all: 1.204166e-02
- remainder majorant C_tail: 1.997641e-03
- remainder majorant tail window: [3.56073e+11, 1e+14]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.440341e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.064374` and `C_all=1.204166e-02` (finite range)
