# INC-0015: Ridge Discrimination Under Controller Saturation

## Hypothesis
The surviving `D30` vs `SG18` ridge may not actually be two live axes.
If the shell controller is saturating, then once `delta_r` is fixed:
- changing `adaptive_shell_growth` should no longer change route health
- the dominant remaining geometry variable should be `delta_r`

## Motivation
`INC-0014` left two nearby healthy branches:
- `R5A_SG16_C10_D30`
- `R5A_SG18_C10_D35`

The next step was to stop treating them like unrelated flag wins and test whether they were really different mechanisms.

## Config
- `configs/proxy_transfer_inc0015_ridge_discrimination.json`

Matrix:
- `delta_r in {2.8, 3.0, 3.2, 3.5}`
- `adaptive_shell_growth in {1.6, 1.8}`
- `seeds=0,1`
- larger-subset proxy (`train=5000`, `test=2500`)

## Evidence
- Analysis: `results/analysis/inc0015_ridge_discrimination.json`
- Gate note: `docs/governance/gates/gate_20260305_174043.md`

## Key Result
For every tested `delta_r`, the paired `SG16` and `SG18` routes were effectively identical on the metrics that matter:
- `test_mse_after`
- `eval_shells`
- `shell_pmax`
- `test_unseen_rate`
- `adaptive_shell_mult_mean`

Observed shell multiplier:
- `adaptive_shell_mult_mean ≈ 2.4596` across the healthy ridge
- this is `exp(c_target + c_hyst) = exp(0.85 + 0.05) = exp(0.9)`

That means the current controller is living in a saturated cap regime.

## Paired Results
- `D28`
  - `RIDGE_D28_SG16`
    - `mse=0.003920306`
    - `total=49.982s`
    - `shells=2.0`
    - `shell_pmax=0.572`
    - `unseen=0.0`
    - `mult=2.459603`
  - `RIDGE_D28_SG18`
    - `mse=0.003920339`
    - `total=47.649s`
    - `shells=2.0`
    - `shell_pmax=0.572`
    - `unseen=0.0`
    - `mult=2.459603`
- `D30`
  - `RIDGE_D30_SG16`
    - `mse=0.003919331`
    - `total=60.198s`
    - `shells=4.0`
    - `shell_pmax=0.523`
    - `unseen=0.0008`
    - `mult=2.459602`
  - `RIDGE_D30_SG18`
    - `mse=0.003919378`
    - `total=54.007s`
    - `shells=4.0`
    - `shell_pmax=0.523`
    - `unseen=0.0008`
    - `mult=2.459603`
- `D32`
  - `RIDGE_D32_SG16`
    - `mse=0.003914771`
    - `total=51.573s`
    - `shells=2.0`
    - `shell_pmax=0.809`
    - seed-wise health fail
    - `mult=2.459603`
  - `RIDGE_D32_SG18`
    - `mse=0.003914724`
    - `total=50.102s`
    - `shells=2.0`
    - `shell_pmax=0.809`
    - seed-wise health fail
    - `mult=2.459603`
- `D35`
  - `RIDGE_D35_SG16`
    - `mse=0.003921623`
    - `total=54.643s`
    - `shells=4.0`
    - `shell_pmax=0.599`
    - `unseen=0.0004`
    - `mult=2.459580`
  - `RIDGE_D35_SG18`
    - `mse=0.003921682`
    - `total=49.856s`
    - `shells=4.0`
    - `shell_pmax=0.599`
    - `unseen=0.0004`
    - `mult=2.459603`

## Decision
- Treat `adaptive_shell_growth` as non-discriminative inside the current capped regime.
- Collapse the live search space from two variables to one:
  - keep searching `delta_r`
  - stop spending research time on `SG16` vs `SG18` under the current controller law
- Queue a 4-seed delta-only confirm before changing the lead.

## Interpretation
This increment is the strongest mechanism result in the repo so far:
- the controller cap is dominating the shell branch
- the current shell-growth knob is mostly cosmetic once the cap is active
- `delta_r` is not just "another hyperparameter"; it is the active radial law in the current regime

This also creates a clear next theoretical step:
- the right next branch is not more shell-growth search
- it is a new cap/merge law that avoids hard saturation

That is the point where a `phi`-derived ratio law becomes plausible:
- keep `pi` for angle/time normalization
- replace the fixed cap/hysteresis band with `phi`-structured split/merge ratios

## Next Increment
`INC-0016`: 4-seed delta-only confirm around `D28`, `D30`, and `D35` with `SG` fixed.
