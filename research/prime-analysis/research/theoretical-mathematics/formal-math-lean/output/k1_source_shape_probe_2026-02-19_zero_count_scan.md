# K1 Source Shape Probe: Zero-Count Scan

Generated: 2026-02-19T07:26:39.778566Z

- fixed beta: 0.60
- x-range: [1e4, 1e8], grid=10000
- tau candidates: first 64 zeta zeros

| zeros_used | best_tau | amplitude | tail_ratio_sup_to_amp | tail_decay_exponent | score |
|---:|---:|---:|---:|---:|---:|
| 16 | 14.134725142 | 3.680041e-02 | 2.159091 | 1.164987 | 3.242862 |
| 32 | 14.134725142 | 3.676209e-02 | 2.309558 | 0.705710 | 3.495116 |
| 64 | 14.134725142 | 3.666145e-02 | 2.256107 | 0.637566 | 3.516729 |
| 128 | 14.134725142 | 3.664144e-02 | 2.673857 | 0.635743 | 3.986426 |
| 256 | 14.134725142 | 3.663561e-02 | 2.928781 | 0.700047 | 4.277399 |
| 512 | 14.134725142 | 3.663507e-02 | 2.949572 | 0.594116 | 4.324110 |
| 1024 | 14.134725142 | 3.663439e-02 | 3.061240 | 0.586313 | 4.452186 |

## Reading

- best tau stays at the first zero (14.134725...), so the leading frequency remains stable.
- tail_ratio_sup_to_amp rises as more zeros are included (roughly 2.16 -> 3.06), indicating stronger multi-mode contamination of a single-mode fit.
- finite-range tail decay remains positive, but not enough to produce single-mode dominance under the current threshold.
- this supports using a theorem target that explicitly controls tail cancellation, not only leading-pair extraction.
