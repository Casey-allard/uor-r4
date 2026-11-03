# ORDER: repair the served-rounding item registration, deepseek-79

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance: repair a new item). Base: `a9d0461e2`.

## Goal

Repair the registration review's scope and acceptance findings without
registering a model result or changing serving code.

## Scope

- `docs/roadmap/items/served-rounding-cost.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-78/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-79/**`

## Work completed

- deepseek-78 registered the item and task records at `3b6987c79` and
  `6e3ce8da7`.
- The exact-head review requested changes because the item could select an
  activation or table-copy repair after profiling but did not scope the files
  that own those mechanisms.
- The same review found that "rerun the comparison with the doors" did not
  require the repaired configuration to pass them.

## Done when

1. The item scope names the float-training activation and export-rounding files
   that its profiling can select, as well as the integer snapshot quantizer.
2. The operational done condition requires both seeds at or below 2x plain
   training and the registered quality-smoke rule not to find served materially
   below plain.
3. The REPORT's changed-file and line accounting matches the exact diff.
4. All checks and an exact-head review pass at the repaired head.

## Stop if

The repair marks the item done, claims a speed result, changes a model, or edits
Claude's active copy-key branch.
