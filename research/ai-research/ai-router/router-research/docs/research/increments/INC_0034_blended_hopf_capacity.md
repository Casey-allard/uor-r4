# INC-0034: Blended Hopf-Capacity Law

## Status
Completed and killed as a promotion branch on 2026-03-06.

## Config
- Screen config:
  - `configs/proxy_transfer_inc0034_blended_hopf_screen.json`
- Analysis:
  - `results/analysis/inc0034_blended_hopf_screen.json`
- Gate note:
  - `docs/governance/gates/gate_20260306_024928.md`

## Implementation
- Added `sector_mode=phase4d_hopf_blend`.
- Added new router controls:
  - `--hopf_blend_lambda`
  - `--hopf_blend_chi_weight`
  - `--hopf_blend_shell_weight`
- Added blended Hopf diagnostics to:
  - `hyperbolic_router_so8.py`
  - `tasks/router_proxy_eval.py`
- Added test coverage for:
  - CLI exposure
  - valid sector routing
  - Hopf-blend regression cases

## Hypothesis
Pure Hopf quality is real, and `phi^2` widening is also real, but the current lattice branches over-concentrate `chi` and still cost too much.
The next branch should blend capped Hopf shell-capacity with a low-rank widening law rather than applying the lattice as a separate global overlay.

## Screen Result
2-seed means:
- `HOPF_K25_BASE`
  - `test_mse_after=0.003888756`
  - `total_sec=62.885`
  - `eval_sectors=4.0`
  - `adaptive_chi_bin_pmax=0.7834`
- `HOPF_PHI2_BAND`
  - `test_mse_after=0.003897103`
  - `total_sec=62.094`
  - `eval_sectors=10.5`
  - `adaptive_chi_bin_pmax=0.9418`
- `HOPF_BLEND_L110_C15_S05`
  - `test_mse_after=0.003911125`
  - `total_sec=59.899`
  - `eval_sectors=8.0`
  - `adaptive_blend_total_mean=13.188`
  - `adaptive_blend_score_mean=0.238`
  - `adaptive_chi_bin_pmax=0.7628`
- `HOPF_BLEND_L080_C10_S05`
  - `test_mse_after=0.003914592`
  - `total_sec=68.800`
  - `eval_sectors=9.0`
  - `adaptive_blend_total_mean=13.061`
  - `adaptive_blend_score_mean=0.320`
  - `adaptive_chi_bin_pmax=0.7720`
- `PHASE_K25_C035`
  - `test_mse_after=0.003909488`
  - `total_sec=58.010`
- `R0`
  - `test_mse_after=0.003911258`
  - `total_sec=45.506`
  - health-fails on shell collapse, but remains the operational runtime reference

## Reading
- The blended law did reopen capacity:
  - `adaptive_blend_total_mean` rose into the `12-14` range instead of staying at pure Hopf's `9`
  - effective sector count widened from `4` to `8-9`
- The cost/quality trade was not good enough:
  - neither blend beat pure Hopf on quality
  - neither blend beat the banded widened branch on runtime
  - neither blend came close to the runtime reference `R0`
- The `chi` concentration improved relative to `HOPF_PHI2_BAND`, but not enough to justify the added route law.

## Decision
- Keep `HOPF_K25_BASE` as the routed-quality lead.
- Keep `HOPF_PHI2_BAND` as the widened Hopf geometry candidate.
- Kill `phase4d_hopf_blend` as the active next branch.
- Do not spend a fairness-control batch on this family.

## Mechanistic Conclusion
The problem no longer looks like "find a better local widening correction."
The failure pattern across `INC-0031` through `INC-0034` now points to a deeper issue:
- pure Hopf quality survives
- local widening overlays repeatedly widen capacity
- but they do not improve the global quality/runtime frontier enough

That is stronger evidence that the route is still losing too much global geometric structure after the tangent/chart step.

## Process Note
The first attempt at this increment exposed a diagnostics-plumbing bug in `tasks/router_proxy_eval.py` for `phase4d_hopf_blend`.
The evaluator was fixed, tests were rerun, and the recorded result above comes from the clean rerun that produced `results/analysis/inc0034_blended_hopf_screen.json`.

## Next Branch
- `INC_0035_poincare_global_alignment.md`
  - treat Poincare-ball global alignment as the next primary geometry branch
  - only return to explicit cost decomposition if that deeper geometry branch still fails
