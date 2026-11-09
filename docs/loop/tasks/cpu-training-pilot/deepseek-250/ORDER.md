# ORDER: cpu-training-pilot, deepseek-250 (final ledger close)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `488f6b63f`. Parent ORDER: [deepseek-249](../deepseek-249/ORDER.md).

## Goal

Close the sole finding from the third fresh review:

1. Recompute `task_and_ledger_bytes` after all task and ledger files at this
   head exist.
2. Set `updated_utc` after that final measurement and refresh free bytes.
3. Record the final accounting in this order's REPORT.

No executable, manifest, evidence report, checkpoint result, Claude file,
serving path or GPU state changes.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-247/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-249/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-250/**`
- `docs/labs/cpu-training-pilot-real-ask-regression-2026-11-09/resource-ledger.json`

## Done when

1. The ledger's retained-byte total equals the actual task-plus-ledger tree.
2. Its timestamp follows the measurement.
3. Report policy, local policy, full gates and one final read-only review pass.
