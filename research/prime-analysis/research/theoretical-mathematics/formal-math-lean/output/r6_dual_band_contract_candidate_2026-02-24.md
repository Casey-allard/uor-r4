# R6 Dual-Band Contract Candidate

Generated: 2026-02-25T03:55:04.790469+00:00

## Candidate pack
- beta: 0.6
- tau_main: 14.134725142
- a_main: -0.000275542978508
- b_main: -0.00903849347864
- amplitude_main: 0.00904269253577
- amplitude_total: 0.0121341248226
- dominant_index: 0
- mode_count: 6
- power majorant: C_tail=0.156028348352, eta_tail=0.0477702265358, x0_tail=2.53031507648e+17
- tail_ratio_sup_to_amp_total: 1.45103003169

## Checks
- beta_gt_half: True
- tau_pos: True
- main_nonzero: True
- c_nonneg: True
- eta_pos: True
- mode_count_ge_6: True
- dominant_index_valid: True
- power_majorant_contract_pass: True
- near_strict_ratio_check: False
- near_strict_ratio_margin: 0.451030031688

## Interpretation
- power_majorant_status: candidate-compatible-finite-range
- near_strict_status: not-yet-satisfied-finite-range
- note: This is a finite-range theorem-candidate extraction from probe outputs. It is not an unconditional proof term.
