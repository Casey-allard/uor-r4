# R6 Dual-Band Contract Candidate

Generated: 2026-02-24T14:03:13.241056+00:00

## Candidate pack
- beta: 0.55
- tau_main: 14.134725142
- a_main: -0.00173217875415
- b_main: -0.0356813333869
- amplitude_main: 0.0357233536431
- amplitude_total: 0.0533641405061
- dominant_index: 0
- mode_count: 12
- power majorant: C_tail=0.274332646513, eta_tail=0.0343010349523, x0_tail=6.3723302336e+15
- tail_ratio_sup_to_amp_total: 1.25751520604

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
- near_strict_ratio_margin: 0.257515206044

## Interpretation
- power_majorant_status: candidate-compatible-finite-range
- near_strict_status: not-yet-satisfied-finite-range
- note: This is a finite-range theorem-candidate extraction from probe outputs. It is not an unconditional proof term.
