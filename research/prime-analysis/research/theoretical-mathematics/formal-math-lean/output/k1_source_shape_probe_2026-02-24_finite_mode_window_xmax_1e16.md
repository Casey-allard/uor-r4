# K1 Source Shape Probe

Generated: 2026-02-24T06:14:09.204525+00:00

- zeros used: 20000
- tau candidates: 64
- beta grid: [0.6]
- x-range: [1e+07, 1e+16]
- grid size: 4096
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.600000
- tau: 14.134725142
- amplitude: 1.217849e-02
- tail ratio (sup/amp): 1.892714e+00
- tail decay exponent (sup envelope): 0.100069
- remainder majorant eta: 0.100069
- remainder majorant C_all: 7.390031e-01
- remainder majorant C_tail: 7.200529e-01
- remainder majorant tail window: [1.59293e+14, 1e+16]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.465689e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.100069` and `C_all=7.390031e-01` (finite range)
