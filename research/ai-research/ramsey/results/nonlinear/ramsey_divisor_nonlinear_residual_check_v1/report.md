# Ramsey Divisor Nonlinear Residual Check v1

## Setup
- Reused existing feature table; no new simulations.
- Divisor candidates: cross_block_contrast, composite_frac, d_std.
- Nonlinear model: pairwise interactions + optional n interactions only.

## Residual Targets: Linear vs Nonlinear
- delay: linear_resid_r2=-0.0016, nonlinear_resid_r2=0.0524, delta=0.0539
- force: linear_resid_r2=0.0183, nonlinear_resid_r2=0.2600, delta=0.2417
- delta_delay_vs_random: linear_resid_r2=-0.0082, nonlinear_resid_r2=0.0089, delta=0.0171
- delta_force_vs_random: linear_resid_r2=-0.0105, nonlinear_resid_r2=0.0555, delta=0.0660

## Mirrored-vs-Base Residual Gap: Linear vs Nonlinear
- delay: linear_gap_r2=-0.0314, nonlinear_gap_r2=-0.0174, delta=0.0139
- force: linear_gap_r2=-0.0196, nonlinear_gap_r2=0.0296, delta=0.0493

## Verdict
- Decision: **KEEP**
- Rationale: Limited nonlinear interactions reveal meaningful new residual and mirrored-gap signal.

## Limits
- Minimal nonlinear check only; still finite and correlational.
- No claim that divisor counts are the mechanism.
