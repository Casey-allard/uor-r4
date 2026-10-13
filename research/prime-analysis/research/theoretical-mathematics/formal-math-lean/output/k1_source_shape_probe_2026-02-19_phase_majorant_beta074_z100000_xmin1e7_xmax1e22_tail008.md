# K1 Source Shape Probe

Generated: 2026-02-19T08:25:12.488790+00:00

- zeros used: 100000
- tau candidates: 1
- beta grid: [0.74]
- x-range: [1e+07, 1e+22]
- grid size: 1536
- overall interpretation: borderline_single_mode_tail_dominance_finite_range

## Overall Best Fit

- beta: 0.740000
- tau: 14.134725142
- amplitude: 4.178722e-04
- tail ratio (sup/amp): 1.006703e+00
- tail decay exponent (sup envelope): 0.045593
- remainder majorant eta: 0.045593
- remainder majorant C_all: 2.083093e-02
- remainder majorant C_tail: 4.178625e-03
- remainder majorant tail window: [6.42418e+20, 1e+22]
- remainder majorant max ratio (all grid): 1.000000
- remainder majorant max ratio (tail): 1.000000
- score: 3.678957e+00
- interpretation: borderline_single_mode_tail_dominance_finite_range

## Candidate Finite-Range Witness

- Normalized model: `E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`
- Candidate remainder majorant: `|remainder(x)| <= C_all * x^{-eta}` with `eta=0.045593` and `C_all=2.083093e-02` (finite range)
