# CTRL-0003: Hopf Frontier Confirm

## Status
Completed.

## Objective
Audit whether the current live Hopf frontier survives a stricter, fairer comparison against `R0`.

## Routes
- `R0`
- `HOPF_K25_BASE`
- `HOPF_PHI2_BAND`

## Why This Control Exists
`INC-0039` failed as a new branch but unexpectedly produced a stronger frontier:
- `HOPF_K25_BASE` beat `R0` on both quality and runtime while passing the route-health gate
- `HOPF_PHI2_BAND` beat `R0` on quality and matched runtime while passing the route-health gate

That is important enough that the next move should be confirmation, not another geometry branch.

## Minimal Scope
1. Re-run the three-route frontier with `run_order=seed_major`.
2. Use `seeds=[0,1,2,3]`.
3. Keep the same proxy subset unless the runtime is clearly cheap enough to increase it.
4. Apply the same strict seed-wise health gate.

## Result
Analysis:
- `results/analysis/ctrl0003_hopf_frontier_confirm.json`
Gate:
- `docs/governance/gates/gate_20260306_085323.md`

4-seed means:
- `HOPF_K25_BASE`
  - `test_mse_after=0.003937984`
  - `total_sec=44.838`
  - `eval_shells=2.25`
  - `eval_sectors=4.0`
  - health pass
- `HOPF_PHI2_BAND`
  - `test_mse_after=0.003921230`
  - `total_sec=51.541`
  - `eval_shells=2.25`
  - `eval_sectors=9.25`
  - health fail on `runtime_ratio_vs_r0>1.150`
- `R0`
  - `test_mse_after=0.003946853`
  - `total_sec=42.409`
  - `eval_shells=1.0`
  - health fail on shell collapse

Timing read:
- `HOPF_K25_BASE`: `chart_opt=39.959s`, `training_ema=4.143s`
- `HOPF_PHI2_BAND`: `chart_opt=40.658s`, `training_ema=10.133s`
- `R0`: `chart_opt=40.634s`, `training_ema=0.856s`

## Decision
- promote `HOPF_K25_BASE` as the current healthiest routed branch
- keep `HOPF_PHI2_BAND` as the widened-quality candidate only
- keep `R0` as the transfer control baseline, but explicitly note that it fails the route-health standard
- move next to `INC-0040` cost decomposition instead of another geometry branch
