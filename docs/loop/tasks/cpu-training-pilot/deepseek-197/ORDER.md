# ORDER: cpu-training-pilot, deepseek-197 (release suffix-fallback claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `03c929896`.

## Goal

Record the protected merge and fresh-main verification of deepseek-196, then
remove its completed source claim. The experiment remains stopped at preflight
for insufficient fallback coverage; this unit promotes no model result.

## Scope

- PR #3752 reviewed head `ae6a6916e`;
- protected merge `03c929896`;
- fresh-main CPU and evidence checks;
- deepseek-196 delivery record;
- deepseek-197 release record and claim bookkeeping.

No model, serving, CUDA, float-training, exporter or Claude-owned source
changes.

## Done when

1. deepseek-196 records PR #3752, its reviewed head, merge and fresh-main
   verification.
2. No `deepseek/suffix-fallback` source claim remains.
3. Reviewed bookkeeping PR, protected merge and fresh-main verification finish.

## Stop rule

Stop on merge, source, evidence, result-state or claim drift.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-197/ORDER.md
