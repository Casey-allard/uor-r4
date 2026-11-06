# ORDER: cpu-training-pilot, deepseek-201 (release full-sequence boundary claim)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `9f40967f1`.

## Goal

Record the protected merge and fresh-main verification of deepseek-200, then
remove its completed source and release claims. The experiment remains a
**CONTROL_FAULT** because full-history and reply-only stopping were
observationally identical; no termination or combined decode is promoted.

## Scope

- PR #3757 reviewed head `473361e70`;
- protected merge `9f40967f1`;
- fresh-main CPU and evidence checks;
- deepseek-200 delivery record;
- deepseek-201 release record and claim bookkeeping.

No model, serving, CUDA, float-training, exporter or Claude-owned source
changes.

## Done when

1. deepseek-200 records PR #3757, its reviewed head, merge and fresh-main
   verification.
2. No `deepseek/full-sequence-boundary` or
   `deepseek/full-sequence-boundary-release` claim remains.
3. Reviewed bookkeeping PR, protected merge and fresh-main verification finish.

## Stop rule

Stop on merge, source, evidence, result-state or claim drift.

Lab: deepseek
Order: docs/loop/tasks/cpu-training-pilot/deepseek-201/ORDER.md
