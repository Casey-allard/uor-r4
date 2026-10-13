# INC-0018: Phi Delta Retune

## Hypothesis
If the `phi_ratio` controller is a real improvement, then the old fixed-controller `delta_r=3.0` optimum should not be assumed to remain optimal. Retuning `delta_r` under the live `phi` controller should reveal whether `phi` can recover a routed hardware-efficiency edge rather than staying only a quality-first side branch.

## Configs
- Screen: `configs/proxy_transfer_inc0018_phi_delta_screen.json`
- Confirm: `configs/proxy_transfer_inc0018_phi_delta_confirm.json`

## Evidence
- Screen analysis: `results/analysis/inc0018_phi_delta_screen.json`
- Screen gate: `docs/governance/gates/gate_20260305_184239.md`
- Confirm analysis: `results/analysis/inc0018_phi_delta_confirm.json`
- Confirm gate: `docs/governance/gates/gate_20260305_185546.md`

## Screen Result
2-seed screen on the live `phi` controller (`train=3000`, `test=1500`):
- `R0`
  - `mse=0.00394501`
  - `total=26.446s`
  - `shells=1.0`
  - collapsed baseline
- `D30_FIXED_SG16`
  - `mse=0.00394314`
  - `total=26.225s`
  - `shells=3.5`
  - `shell_pmax=0.588`
  - health pass
- `PHI_D30_L120`
  - `mse=0.00394299`
  - `total=25.750s`
  - `shells=5.5`
  - `shell_pmax=0.544`
  - health pass
- `PHI_D32_L120`
  - `mse=0.00393778`
  - `total=28.177s`
  - `shells=3.5`
  - `shell_pmax=0.588`
  - health pass
- `PHI_D35_L120`
  - `mse=0.00393047`
  - `total=36.180s`
  - `shells=2.0`
  - `shell_pmax=0.787`
  - failed runtime gate

Screen reading:
- the `phi` controller clearly wants a different radial law than the fixed controller
- `delta_r=3.0` and `3.2` both survive the health gate
- `delta_r=3.5` improves MSE but pays too much runtime to remain a hardware-efficiency candidate

## Confirm Result
4-seed larger-subset confirm (`train=5000`, `test=2500`):
- `R0`
  - `mse=0.003907888`
  - `total=45.985s`
  - `shells=1.0`
  - `shell_pmax=1.000`
  - collapsed baseline
- `D30_FIXED_SG16`
  - `mse=0.003918454`
  - `total=47.476s`
  - `shells=3.0`
  - `shell_pmax=0.579`
  - `unseen=0.0005`
  - health pass
- `PHI_D30_L120`
  - `mse=0.003914423`
  - `total=50.308s`
  - `shells=3.25`
  - `shell_pmax=0.555`
  - `unseen=0.0002`
  - health pass
- `PHI_D32_L120`
  - `mse=0.003937115`
  - `total=40.801s`
  - `shells=6.0`
  - `shell_pmax=0.543`
  - `unseen=0.0020`
  - health pass

## Decision
- Promote `PHI_D32_L120` as the routed hardware-efficiency transfer lead.
- Retain `PHI_D30_L120` as the quality-first `phi` comparator.
- Demote `D30_FIXED_SG16` from transfer lead to fixed-controller comparator.
- Kill `PHI_D35_L120` as a hardware-efficiency replacement candidate on runtime.

## Interpretation
`INC-0018` resolves the main uncertainty left by `INC-0017`:
- `phi_ratio` is not just a quality-oriented controller
- once `delta_r` is retuned, `phi` can also win on the routed hardware-efficiency objective

The important trade now is cleaner:
- `PHI_D32_L120` is materially faster and more shell-open than both `R0` and `D30_FIXED_SG16`
- `PHI_D30_L120` remains the better routed-branch MSE point
- `R0` still holds the raw transfer-quality baseline

That means the transfer frontier is now genuinely split three ways:
- `R0` = raw quality baseline
- `PHI_D32_L120` = routed hardware-efficiency lead
- `PHI_D30_L120` = routed quality-first `phi` branch

Mechanistically, `delta_r` has moved under the unsaturated `phi` controller:
- fixed controller optimum: `delta_r=3.0`
- live `phi` hardware-efficiency optimum: `delta_r=3.2`

This supports the stronger reading that `delta_r` is part of the controller law, not just a neutral shell-index constant.

## Next Increment
`INC-0019`: hold the new `phi` lead steady while testing hybrid local zoom.

Recommended branch:
- keep `PHI_D32_L120` as the routed hardware-efficiency control
- test `phase4d_adaptive` as the coarse router with `complex2` as local zoom
- express the local complex branch explicitly as a discrete complex-multiplication / imaginary-field neighborhood hypothesis
- only activate a discrete `phi` step-ladder controller if the continuous `phi` lead later looks noisy or overly tuned
