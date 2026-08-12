# A3 Offdiag Lag Decomposition Probe

Generated: 2026-02-17T15:40:09.512246+00:00
Source A3 artifact: `research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json`

This is a sign-sensitive decomposition planning scaffold.
It prioritizes lag bands for the next analytic bound iteration.

## Current Anchor Constants

- `A_eta = 4.0`
- `C_eta = 1.9531254774350515`
- `A_H = 1.0999999999999999`
- `C_H = 4.526748350185781`

## Proposed Lag Bands (Priority Weights)

| Lag band edge | Priority weight |
|---:|---:|
| 1 | 0.444517 |
| 2 | 0.293763 |
| 4 | 0.163529 |
| 8 | 0.071419 |
| 16 | 0.022131 |
| 32 | 0.004221 |
| 64 | 0.000405 |
| 128 | 0.000015 |
| 256 | 0.000000 |
| 512 | 0.000000 |
| 1024 | 0.000000 |

## Next Analytic Targets

1. Bound positive offdiag in low-lag bands with explicit oscillatory cancellation constants.
2. Bound high-lag bands via coarse absolute control plus decay factor.
3. Recompose to a new symbolic `C_eta` candidate and compare to calibrated budget.

