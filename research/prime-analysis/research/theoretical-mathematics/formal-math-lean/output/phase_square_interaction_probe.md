# Phase-Square Interaction Probe

Generated: 2026-02-16T23:05:42.854706+00:00

- n_max=300000, split=150000
- prime-square boundaries used: 101

## base
- test_R2: 0.122848
- test_accuracy: 0.919017
- feature_count: 7

## with_high_r_terms
- test_R2: 0.129933
- test_accuracy: 0.919011
- feature_count: 10

## with_phase_terms
- test_R2: 0.122846
- test_accuracy: 0.919011
- feature_count: 11

## with_high_r_and_phase
- test_R2: 0.129930
- test_accuracy: 0.919011
- feature_count: 14

## with_piecewise_square_phase
- test_R2: 0.129920
- test_accuracy: 0.919011
- feature_count: 19

## with_piecewise_and_phase
- test_R2: 0.129892
- test_accuracy: 0.919011
- feature_count: 23

## with_local_square_coords
- test_R2: 0.131093
- test_accuracy: 0.919004
- feature_count: 15

## boundary_band_ensemble
- test_R2: 0.128167
- test_accuracy: 0.919004
- feature_count: 90

## Gains vs base
- test_r2_gain_high_r: +0.007085
- test_r2_gain_phase: -0.000002
- test_r2_gain_both: +0.007082
- test_r2_gain_piecewise: +0.007072
- test_r2_gain_piecewise_full: +0.007044
- test_r2_gain_local_square: +0.008245
- test_r2_gain_band_ensemble: +0.005319
- test_acc_gain_both: -0.000007
- test_acc_gain_piecewise: -0.000007
- test_acc_gain_piecewise_full: -0.000007
- test_acc_gain_local_square: -0.000013
- test_acc_gain_band_ensemble: -0.000013
