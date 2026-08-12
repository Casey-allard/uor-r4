# INC-0017: Phi-Ratio Controller

## Hypothesis
If the saturated fixed cap is the reason `adaptive_shell_growth` went dead, then replacing it with a `phi`-ratio convergence law should:
- reopen the shell controller as a live axis
- preserve healthy shell activation under strict seed-wise review
- potentially improve transfer quality without collapsing back into the fixed-cap ridge

## Controller Change
The adaptive shell controller now supports:
- `adaptive_converge_mode=fixed`
- `adaptive_converge_mode=phi_ratio`

The new branch keeps `pi` in the time-expanded divergence field and uses `phi` in the shell convergence law:
- the divergence field is unchanged
- the threshold band remains `c_target + c_hyst`
- convergence pressure is computed from shell expansion ratio rather than a hard linear overflow cap

This turns the fixed exact cap into a softer `phi`-scaled pressure law.

## Configs
- Screen: `configs/proxy_transfer_inc0017_phi_ratio_screen.json`
- Confirm: `configs/proxy_transfer_inc0017_phi_ratio_confirm.json`

## Evidence
- Screen analysis: `results/analysis/inc0017_phi_ratio_screen.json`
- Screen gate: `docs/governance/gates/gate_20260305_181815.md`
- Confirm analysis: `results/analysis/inc0017_phi_ratio_confirm.json`
- Confirm gate: `docs/governance/gates/gate_20260305_183037.md`

## Screen Result
2-seed screen on the retained `D30` / `SG16` branch (`train=3000`, `test=1500`):
- `D30_FIXED_SG16`
  - `mse=0.00394314`
  - `total=30.429s`
  - `shells=3.5`
  - `shell_pmax=0.639`
  - health pass
- `D30_PHI_L100`
  - `mse=0.00392903`
  - `total=27.523s`
  - `shells=3.0`
  - `shell_pmax=0.666`
  - health pass
- `D30_PHI_L120`
  - `mse=0.00394299`
  - `total=27.217s`
  - `shells=5.5`
  - `shell_pmax=0.544`
  - health pass
- `D30_PHI_L140`
  - `mse=0.00393956`
  - `total=25.701s`
  - fails seed-wise shell concentration (`seed0_shell_pmax>0.850`)

## Confirm Result
4-seed larger-subset confirm (`train=5000`, `test=2500`):
- `R0`
  - `mse=0.003907888`
  - `total=41.442s`
  - `shells=1.0`
  - `shell_pmax=1.000`
  - collapsed baseline
- `D30_FIXED_SG16`
  - `mse=0.003918454`
  - `total=42.805s`
  - `shells=3.0`
  - `shell_pmax=0.579`
  - `unseen=0.0005`
  - health pass
- `D30_PHI_L120`
  - `mse=0.003914423`
  - `total=44.637s`
  - `shells=3.25`
  - `shell_pmax=0.555`
  - `unseen=0.0002`
  - health pass
- `D30_PHI_L100`
  - `mse=0.003932333`
  - `total=45.349s`
  - `shells=7.75`
  - `shell_pmax=0.486`
  - `unseen=0.0036`
  - health pass

## Decision
- Keep `D30_FIXED_SG16` as the transfer hardware-efficiency route lead.
- Track `D30_PHI_L120` as the best healthy `phi`-controller quality branch.
- Kill `D30_PHI_L140` as an over-concentrated controller setting.
- Demote `D30_PHI_L100` to a slower, over-dispersed comparison branch.
- Do not promote `phi_ratio` over the fixed controller yet.

## Interpretation
`INC-0017` proves something important even though it does not replace the lead:
- the `phi` controller is real
- it reopens the shell controller as a live mechanism branch
- the fixed-cap saturation was not just a logging artifact or a narrow search illusion

But the first healthy `phi` branch is quality-first, not hardware-first:
- `D30_PHI_L120` improves routed-branch MSE vs `D30_FIXED_SG16`
- `D30_FIXED_SG16` remains faster
- both routed branches are slower than `R0` in this confirm batch

That means the repo should now treat the transfer branch as a three-way split:
- `R0` = raw proxy quality baseline
- `D30_FIXED_SG16` = routed hardware-efficiency lead
- `D30_PHI_L120` = routed quality-first controller candidate

Mechanistically, this also changes the next search space:
- under the fixed law, `delta_r` was the only live radial control axis
- under `phi_ratio`, the controller is live again
- so the old fixed-regime `delta_r=3.0` optimum should no longer be assumed optimal for `phi`

## Next Increment
`INC-0018`: retune the radial law under the live `phi` controller.

Recommended branch:
- hold `D30_PHI_L120` as the base controller
- scan `delta_r` around the old fixed optimum (`3.0`, `3.2`, `3.5`)
- keep strict seed-wise route-health gates
- test whether `phi` can recover a hardware-efficiency edge once `delta_r` is retuned for the unsaturated regime
