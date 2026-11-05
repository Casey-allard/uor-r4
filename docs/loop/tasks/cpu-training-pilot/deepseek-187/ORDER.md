# ORDER: cpu-training-pilot, deepseek-187 (release table-anchor claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `63ce79e85`.

## Goal

Record the protected merge and final fresh-main verification of deepseek-186,
then remove the completed table-anchor release claim.

## Scope

- PR #3740 reviewed head `4ee7e1231`;
- protected merge `63ce79e85`;
- final fresh-main CPU and evidence checks;
- deepseek-186 delivery record and the stale claim.

No model, serving, CUDA, float-training, exporter or Claude-owned source
changes.

## Done when

1. deepseek-186 records PR #3740, reviewed head, merge and fresh-main checks.
2. No table-anchor claim remains.
3. Reviewed bookkeeping PR, protected merge and fresh-main verification finish.

## Stop rule

Stop on merge, source, evidence or claim drift.
