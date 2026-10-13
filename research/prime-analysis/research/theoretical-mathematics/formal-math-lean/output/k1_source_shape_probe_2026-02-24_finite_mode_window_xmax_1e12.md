# K1 Source Shape Probe

Generated: 2026-02-24T06:14:05.004393+00:00

- zeros used: 20000
- tau candidates: 64
- beta grid: [0.6]
- x-range: [1e+07, 1e+12]
- grid size: 4096
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.600000
- tau: 14.134725142
- amplitude: 1.736940e-02
- tail ratio (sup/amp): 2.874560e+00
- tail decay exponent (sup envelope): 0.342687
- remainder majorant eta: 0.342687
- remainder majorant C_all: 3.796864e+02
- remainder majorant C_tail: 3.796864e+02
- remainder majorant tail window: [1.00282e+11, 1e+12]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 4.247350e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.342687` and `C_all=3.796864e+02` (finite range)
