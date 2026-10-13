# K1 Source Shape Probe

Generated: 2026-02-19T08:08:50.599490+00:00

- zeros used: 100000
- tau candidates: 1
- beta grid: [0.68]
- x-range: [5000, 1e+14]
- grid size: 1024
- overall interpretation: weak_single_mode_dominance_finite_range

## Overall Best Fit

- beta: 0.680000
- tau: 14.134725142
- amplitude: 6.986880e-03
- tail ratio (sup/amp): 1.342576e+00
- tail decay exponent (sup envelope): 0.053032
- remainder majorant eta: 0.053032
- remainder majorant C_all: 1.569932e-01
- remainder majorant C_tail: 4.075761e-02
- remainder majorant tail window: [2.54222e+10, 1e+14]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.552513e+00
- interpretation: weak_single_mode_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.053032` and `C_all=1.569932e-01` (finite range)
