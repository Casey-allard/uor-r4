# K1 Phase-Majorant Xmax Trend by Tail Window (2026-02-19)

Generated: 2026-02-19T08:26:51.433116+00:00

Fixed config:
- beta: 0.74
- zeros_used: 100000
- x_min: 10000000.0
- tau: first_zeta_zero_locked

## tail_frac = 0.1
- ratio range: [0.996728, 1.043819]
- eta range: [0.002421, 0.039516]
- fit intercept (x_max -> inf proxy): 0.878157
| x_max | ratio | eta | interpretation |
|---:|---:|---:|---|
| 1000000000000000000 | 1.043819 | 0.004692 | borderline_single_mode_tail_dominance_finite_range |
| 100000000000000000000 | 0.996728 | 0.039516 | near_strict_single_mode_tail_dominance_finite_range |
| 10000000000000000000000 | 1.006703 | 0.031055 | borderline_single_mode_tail_dominance_finite_range |
| 999999999999999983222784 | 1.000145 | 0.002421 | borderline_single_mode_tail_dominance_finite_range |

## tail_frac = 0.2
- ratio range: [1.002720, 1.052821]
- eta range: [0.001016, 0.016115]
- fit intercept (x_max -> inf proxy): 0.851471
| x_max | ratio | eta | interpretation |
|---:|---:|---:|---|
| 1000000000000000000 | 1.052821 | 0.009866 | borderline_single_mode_tail_dominance_finite_range |
| 100000000000000000000 | 1.021280 | 0.016115 | borderline_single_mode_tail_dominance_finite_range |
| 10000000000000000000000 | 1.011266 | 0.009774 | borderline_single_mode_tail_dominance_finite_range |
| 999999999999999983222784 | 1.002720 | 0.001016 | borderline_single_mode_tail_dominance_finite_range |

Finite-range evidence only; not theorem-level closure.
