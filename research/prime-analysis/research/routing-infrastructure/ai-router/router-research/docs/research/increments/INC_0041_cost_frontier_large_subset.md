# INC-0041: Cost Frontier Large-Subset Confirm

## Status
Completed.

## Trigger
`INC-0040` produced two new 4-seed strict-gate survivors:
- `HOPF_K25_BASE_IT60_P4`
- `HOPF_PHI2_BAND_IT60_P4`

Both are much faster than `R0` on the current proxy subset and both beat `R0` on MSE.

## Objective
Check whether the reduced-schedule cost frontier survives on a larger proxy subset without losing route health.

## Minimal Scope
1. Compare only:
   - `HOPF_K25_BASE_IT60_P4`
   - `HOPF_PHI2_BAND_IT60_P4`
   - `R0`
2. Increase proxy load relative to `INC-0040`:
   - larger `max_train`
   - larger `max_eval`
3. Keep the same strict seed-wise health gate.

## Decision Rule
- If only one reduced-schedule route survives, promote it as the sole operational lead.
- If both survive:
  - keep `HOPF_K25_BASE_IT60_P4` as the quality/runtime-balanced lead
  - keep `HOPF_PHI2_BAND_IT60_P4` as the widened efficient lead
- If neither survives, reopen cost diagnosis before reopening geometry.

## Result
- Config:
  - `configs/proxy_transfer_inc0041_cost_large_subset.json`
- Analysis:
  - `results/analysis/inc0041_cost_large_subset.json`
- Gate:
  - `docs/governance/gates/gate_20260306_093641.md`
- 4-seed means:
  - `HOPF_K25_BASE_IT60_P4`
    - `test_mse_after=0.003895705`
    - `total_sec=37.090`
    - `chart_opt=19.655`
    - `training_ema=16.140`
    - health fail on runtime
  - `HOPF_PHI2_BAND_IT60_P4`
    - `test_mse_after=0.003904061`
    - `total_sec=47.604`
    - `chart_opt=25.087`
    - `training_ema=21.057`
    - health fail on runtime
  - `R0`
    - `test_mse_after=0.003913707`
    - `total_sec=27.271`
    - health fail on shell collapse

## Decision
- The reduced-schedule cost rescue does not survive as an operational win on the larger subset.
- `HOPF_K25_BASE_IT60_P4` remains the best large-subset quality candidate.
- `HOPF_PHI2_BAND_IT60_P4` remains the widened large-subset quality candidate, but is clearly behind reduced-schedule pure Hopf under load.
- Next branch:
  - attack large-subset EMA pressure and chart cost directly before reopening geometry
