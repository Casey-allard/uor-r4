# K1 Source Shape Probe

Generated: 2026-02-19T08:39:58.569557+00:00

- zeros used: 100000
- tau candidates: 1
- beta grid: [0.74]
- x-range: [1e+07, 1e+20]
- grid size: 7168
- overall interpretation: borderline_single_mode_tail_dominance_finite_range

## Overall Best Fit

- beta: 0.740000
- tau: 14.134725142
- amplitude: 4.362697e-04
- tail ratio (sup/amp): 1.001150e+00
- tail decay exponent (sup envelope): 0.143662
- remainder majorant eta: 0.143662
- remainder majorant C_all: 3.178151e-01
- remainder majorant C_tail: 3.178151e-01
- remainder majorant tail window: [2.25138e+19, 1e+20]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.673784e+00
- interpretation: borderline_single_mode_tail_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.143662` and `C_all=3.178151e-01` (finite range)
