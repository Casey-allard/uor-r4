# Ramsey Nonlocal Shadow Test v1

## Setup
- Reused existing ablation instance table (no new simulation campaign).
- Families: base recursive, mirrored recursive, random baseline, balanced binary control.
- Goal: test divisor-count observables as candidate arithmetic shadows, not as full mechanism.

## Model comparison (CV R^2)
- Target `delay`:
  - baseline_plus_divisor: cv_r2=0.7283, cv_rmse=0.3646
  - baseline_struct_only: cv_r2=0.6999, cv_rmse=0.3831
  - divisor_observables_only: cv_r2=0.7057, cv_rmse=0.3794
- Target `delta_delay_vs_random`:
  - baseline_plus_divisor: cv_r2=0.2478, cv_rmse=0.4289
  - baseline_struct_only: cv_r2=0.2256, cv_rmse=0.4352
  - divisor_observables_only: cv_r2=0.1938, cv_rmse=0.4441
- Target `delta_force_vs_random`:
  - baseline_plus_divisor: cv_r2=0.2665, cv_rmse=0.0444
  - baseline_struct_only: cv_r2=0.2540, cv_rmse=0.0448
  - divisor_observables_only: cv_r2=0.2229, cv_rmse=0.0457
- Target `force`:
  - baseline_plus_divisor: cv_r2=0.7492, cv_rmse=0.0359
  - baseline_struct_only: cv_r2=0.6652, cv_rmse=0.0415
  - divisor_observables_only: cv_r2=0.7302, cv_rmse=0.0373

## Top divisor-feature correlations (absolute)
- level_shadow_sum vs delay: r=0.8075
- d_mean vs force: r=-0.8067
- d_mean vs delay: r=0.7972
- level_shadow_sum vs force: r=-0.7830
- d_std vs force: r=-0.7822
- composite_frac vs force: r=-0.7793
- d_std vs delay: r=0.7704
- composite_frac vs delay: r=0.7539

## Mirrored > base behavior by n
- n=8: mean(mirrored-base delay)=0.0000
- n=10: mean(mirrored-base delay)=0.2639
- n=12: mean(mirrored-base delay)=0.0250
- n=14: mean(mirrored-base delay)=0.3208
- n=16: mean(mirrored-base delay)=0.1667
- n=18: mean(mirrored-base delay)=0.1250
- n=20: mean(mirrored-base delay)=0.0000
- n=22: mean(mirrored-base delay)=-0.0611
- n=24: mean(mirrored-base delay)=0.0571
- n=26: mean(mirrored-base delay)=0.0258
- n=28: mean(mirrored-base delay)=0.2431

## Interpretation
- Divisor-count observables are treated as candidate shadows only.
- Added value is judged by incremental CV performance over size/order/depth/asymmetry baselines.

## Verdict
- Decision: **REFINE**
- Rationale: Divisor observables show partial/weak incremental value, but evidence is not strong or uniform.

## Limits
- This is finite small-n exploratory modeling, not causal proof.
- Features are hand-crafted proxies and may miss deeper coupling structure.
