# Ramsey Divisor Residual Test v1

## Setup
- Baseline model fit first on structural predictors only.
- Divisor observables tested on baseline residuals (individually and grouped).
- Reused existing feature table; no new simulation campaign.

## Residual Increment Results
- delay: baseline_cv_r2=0.6999, combined_cv_r2=0.7214, lift=0.0215, residual_explained_r2=0.0715
- force: baseline_cv_r2=0.6652, combined_cv_r2=0.7433, lift=0.0781, residual_explained_r2=0.2332
- delta_delay_vs_random: baseline_cv_r2=0.2256, combined_cv_r2=0.2324, lift=0.0067, residual_explained_r2=0.0087
- delta_force_vs_random: baseline_cv_r2=0.2540, combined_cv_r2=0.2493, lift=-0.0046, residual_explained_r2=-0.0062

## Strongest Individual Residual-Side Divisor Features
- delay:
  - composite_frac: lift=0.0010, residual_explained_r2=0.0032
  - d_std: lift=0.0007, residual_explained_r2=0.0024
  - prime_frac: lift=0.0006, residual_explained_r2=0.0019
- force:
  - cross_block_contrast: lift=0.0060, residual_explained_r2=0.0178
  - composite_frac: lift=0.0045, residual_explained_r2=0.0133
  - d_std: lift=0.0029, residual_explained_r2=0.0085
- delta_delay_vs_random:
  - prime_frac: lift=0.0009, residual_explained_r2=0.0012
  - composite_frac: lift=0.0009, residual_explained_r2=0.0012
  - cross_block_contrast: lift=0.0003, residual_explained_r2=0.0004
- delta_force_vs_random:
  - cross_block_contrast: lift=0.0023, residual_explained_r2=0.0031
  - block_var_mean: lift=0.0002, residual_explained_r2=0.0002
  - level_shadow_sum: lift=0.0000, residual_explained_r2=0.0000

## Mirrored-vs-Base Residual Gap
- delay: group_gap_cv_r2=-0.0356, best_single=cross_block_contrast (-0.0314)
- force: group_gap_cv_r2=0.0048, best_single=level_shadow_sum (-0.0166)

## Verdict
- Decision: **REFINE**
- Rationale: Divisor features explain partial residual structure, but not consistently strong across all targets.

## Limits
- Residual analysis is still correlational and finite small-n.
- Divisor features may remain partially entangled with unmodeled interactions.
