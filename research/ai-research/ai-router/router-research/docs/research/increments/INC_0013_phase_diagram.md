# INC-0013: Shell-Control Phase Diagram

## Hypothesis
The adaptive shell-control surface is not a smooth tuning problem. It should contain distinct regimes:
- collapsed radial routing
- a healthy shell-active band
- an over-dispersed band

If that is true, the next lead should come from mapping the boundary, not from locally nudging the previous winner.

## Motivation
`INC-0012` promoted `R5A_SG16_C10_D35`, but the lead still sat close to the shell concentration cutoff. The next task was to determine whether that route lived inside a real healthy operating band or on a narrow ridge.

## Alias Map
- `PD_CENTER` = `R5A_SG16_C10_D35` (the `INC-0012` transfer lead entering this increment)
- `PD_SG18` = `R5A_SG18_C10_D35`
- `PD_T080` = `R5A_SG16_C10_T080_D35`
- `PD_D30` = `R5A_SG16_C10_D30`
- `PD_D40` = `R5A_SG16_C10_D40`
- `PD_C06` = `R5A_SG16_C06_D35`
- `PD_C12` = `R5A_SG16_C12_D35`

## Configs
- Screen: `configs/proxy_transfer_inc0013_phase_diagram_screen.json`
- Confirm: `configs/proxy_transfer_inc0013_phase_diagram_confirm.json`

## Evidence
- Screen analysis: `results/analysis/inc0013_phase_diagram_screen.json`
- Confirm analysis: `results/analysis/inc0013_phase_diagram_confirm.json`
- Strict confirm review: `results/analysis/inc0013_phase_diagram_confirm_strict.json`
- Screen gate: `docs/governance/gates/gate_20260305_163649.md`
- Confirm gate: `docs/governance/gates/gate_20260305_164628.md`
- Strict confirm gate: `docs/governance/gates/gate_20260305_164628_strict.md`

## Screen Result
Single-seed screen on PTB proxy (`train=3000`, `test=1500`, seed `0`) showed a real phase structure rather than a single optimum:

- `PD_D40`
  - `test_mse_after=0.0039184`
  - `total_sec=27.074`
  - `eval_shells=1`
  - `shell_pmax=1.000`
  - stable collapse candidate
- `PD_CENTER`
  - `test_mse_after=0.0039264`
  - `total_sec=26.602`
  - `eval_shells=2`
  - `shell_pmax=0.676`
  - healthy interior ridge candidate
- `PD_SG18`
  - `test_mse_after=0.0039265`
  - `total_sec=24.875`
  - `eval_shells=2`
  - `shell_pmax=0.676`
  - faster nearby challenger to `PD_CENTER`
- `PD_T080`
  - `test_mse_after=0.0039345`
  - `total_sec=23.820`
  - `eval_shells=2`
  - `shell_pmax=0.723`
  - conservative healthy point
- `PD_D30`
  - `test_mse_after=0.0039460`
  - `total_sec=38.104`
  - `eval_shells=5`
  - `shell_pmax=0.508`
  - lower-`delta_r` boundary / possible over-dispersion
- `PD_C06`
  - `test_mse_after=0.0039695`
  - `total_sec=27.521`
  - `eval_shells=12`
  - `shell_pmax=0.335`
  - high-dispersion healthy point

Screen reading:
- `delta_r` defines a real regime change, not a bookkeeping constant
- `D35` looked like the best quality/runtime ridge
- `D40` already looked like true shell collapse
- lower convergence and lower `delta_r` both opened shells more strongly, but with signs of dispersion pressure

## Confirm Result
Two-seed confirm on PTB proxy (`train=3000`, `test=1500`, seeds `0,1`) under the original mean-based gate:

- `PD_CENTER`
  - `test_mse_after=0.0039278`
  - `total_sec=37.159`
  - `eval_shells=2.0`
  - `shell_pmax=0.792`
  - failed mean gate due runtime
- `PD_SG18`
  - `test_mse_after=0.0039279`
  - `total_sec=31.994`
  - `eval_shells=2.0`
  - `shell_pmax=0.792`
  - passed mean gate
- `PD_C12`
  - `test_mse_after=0.0039301`
  - `total_sec=27.853`
  - `eval_shells=3.0`
  - `shell_pmax=0.840`
  - passed mean gate
- `PD_D30`
  - `test_mse_after=0.0039431`
  - `total_sec=29.851`
  - `eval_shells=3.5`
  - `shell_pmax=0.639`
  - passed mean gate
- `PD_T080`
  - `test_mse_after=0.0039496`
  - `total_sec=31.186`
  - `eval_shells=3.0`
  - `shell_pmax=0.733`
  - passed mean gate
- `PD_C06`
  - `test_mse_after=0.0039538`
  - `total_sec=27.952`
  - `eval_shells=9.0`
  - `shell_pmax=0.400`
  - passed mean gate

## Governance Correction
This increment exposed a flaw in the previous transfer-health rule:
- the gate only checked mean route-health
- that allowed a route to pass even if one seed fully collapsed or crossed the shell concentration cutoff

The repo was updated during this increment:
- `tools/proxy_sweep.py` now supports `enforce_seed_health`
- promotion review now requires seed-wise health in multi-seed transfer increments

## Strict Seed Review
Re-scoring the completed confirm batch under seed-wise health changed the branch ranking:

- `PD_CENTER`
  - failed strict review because `seed1_shell_pmax=0.908`
- `PD_SG18`
  - failed strict review because `seed1_shell_pmax=0.908`
- `PD_C12`
  - failed strict review because `seed0_eval_shells=1` and `seed0_shell_pmax=1.000`
- `PD_D40`
  - failed strict review because both seeds collapsed to one shell
- `PD_D30`
  - passed strict review
  - `test_mse_after=0.0039431`
  - `total_sec=29.851`
  - `eval_shells=3.5`
  - `shell_pmax=0.639`
  - `unseen_rate=0.0020`
- `PD_T080`
  - passed strict review
  - `test_mse_after=0.0039496`
  - `total_sec=31.186`
  - `eval_shells=3.0`
  - `shell_pmax=0.733`
  - `unseen_rate=0.0000`
- `PD_C06`
  - passed strict review
  - `test_mse_after=0.0039538`
  - `total_sec=27.952`
  - `eval_shells=9.0`
  - `shell_pmax=0.400`
  - `unseen_rate=0.0010`

## Decision
- Demote the `D35` ridge (`PD_CENTER` / `PD_SG18`) from promoted-lead status under strict seed review.
- Promote `PD_D30` as the current provisional strict-health proxy-transfer lead.
- Keep `PD_T080` as the conservative interior healthy comparator.
- Keep `PD_C06` as the high-dispersion comparison branch.
- Treat `PD_C12` and `PD_D40` as collapse-side evidence, not active lead candidates.

## Interpretation
The important result is not that `SG18` won. It did not survive stricter review.

What the phase diagram actually says:
- `delta_r=4.0` is on the collapse side
- `delta_r=3.5` carries a narrow high-quality ridge, but one seed already crosses the shell concentration wall
- `delta_r=3.0` opens a healthier radial band and remains competitive on quality/runtime
- lower convergence (`C06`) produces a stable high-dispersion regime, which is useful as a comparison branch even though it is not the quality leader
- the healthy operating band exists, but it is narrower and more structured than the mean-only gate implied

## What Changed In The Project Direction
Before this increment:
- transfer lead = `R5A_SG16_C10_D35`
- open problem = map the shell-control phase boundary

After this increment:
- provisional strict-health transfer lead = `R5A_SG16_C10_D30`
- open problem = verify whether the low-`delta_r` branch is truly robust or just a different boundary effect
- governance change = multi-seed transfer promotions must use seed-wise health, not mean-only health

## Next Increment
`INC-0014`: larger-subset and 4-seed strict-health robustness test around:
- `R0`
- `R5A_SG16_C10_D30`
- `R5A_SG16_C10_T080_D35`
- `R5A_SG16_C06_D35`
- `R5A_SG18_C10_D35` as a narrow-margin challenger
