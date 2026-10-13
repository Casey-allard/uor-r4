# INC-0020: Hybrid Local-Convergence Rescue

## Hypothesis
`phase4d_complex_local` can be rescued if local complex zoom is opened with explicit convergence control instead of unconditional local capacity.

## Mechanism Change
- Added hybrid-local controller parameters:
  - `hybrid_local_min_k`
  - `hybrid_local_target`
  - `hybrid_local_hysteresis`
  - `hybrid_local_converge_lambda`
- Fixed the local controller law:
  - old local activation was normalized against absolute shell-scale headroom and stayed effectively dead
  - new local activation is ratio-based: `local_ratio_pressure / phi`, then reduced by local convergence pressure
- Added hybrid diagnostics:
  - local drive
  - local ratio / ratio pressure
  - local convergence
  - local activation
  - effective local bin count

## Evidence
- Screen config:
  - `configs/proxy_transfer_inc0020_hybrid_rescue_screen.json`
- Screen analysis:
  - `results/analysis/inc0020_hybrid_rescue_screen.json`
- Screen gate:
  - `docs/governance/gates/gate_20260305_195327.md`
- Confirm config:
  - `configs/proxy_transfer_inc0020_hybrid_rescue_confirm.json`
- Confirm analysis:
  - `results/analysis/inc0020_hybrid_rescue_confirm.json`
- Confirm gate:
  - `docs/governance/gates/gate_20260305_200621.md`

## Screen Result
- `HYB4_M2_T010_C005`
  - `mse=0.003936500`
  - `total=29.562s`
  - `shells=2.0`
  - `unseen=0.000667`
  - healthy
- `PHI_D32_L120`
  - `mse=0.003937783`
  - `total=29.962s`
  - healthy

Decision from screen:
- promote `HYB4_M2_T010_C005`
- keep `HYB4_M2_T005_C005` as the slightly more open comparator

## Confirm Result
4-seed larger-subset means:
- `R0`
  - `mse=0.003907888`
  - `total=43.329s`
  - collapsed, not health-pass
- `HYB4_M2_T010_C005`
  - `mse=0.003920267`
  - `total=46.116s`
  - `shells=2.25`
  - `unseen=0.0012`
  - health-pass
- `HYB4_M2_T005_C005`
  - `mse=0.003923083`
  - `total=46.062s`
  - `shells=2.25`
  - `unseen=0.0020`
  - health-pass
- `PHI_D32_L120`
  - `mse=0.003937115`
  - `total=45.038s`
  - `shells=6.0`
  - `unseen=0.0020`
  - health-pass

## Reading
- The hybrid branch is no longer blocked.
- The rescue mechanism works:
  - `local_min_k=2` creates a stable two-way local split
  - the ratio-based local controller keeps the local field from exploding
- `HYB4_M2_T010_C005` is now the healthiest routed-quality branch on this proxy.
- But the hybrid branch is not a hardware-efficiency lead:
  - it improves healthy routed quality versus `PHI_D32_L120`
  - it does not beat `R0` on runtime
  - it is slightly slower than `PHI_D32_L120`

## Decision
- Promote `HYB4_M2_T010_C005` to routed-quality branch status.
- Retain `PHI_D32_L120` as the fastest healthy routed branch.
- Do not claim a routed hardware-efficiency win from `INC-0020`.

## Next Increment
- `INC-0021`: discrete `phi` step-ladder controller to recover runtime / stability in the `phi` family under the larger-subset regime.
- `INC-0022`: hybrid runtime rescue only if the quality gain remains worth carrying.
