# ORDER: cpu-training-pilot, deepseek-260 (parity and scope repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `8751530f1`. Parent ORDER:
[deepseek-259](../deepseek-259/ORDER.md).

## Goal

Repair the seventh fresh exact-head review of deepseek-259:

1. Re-scope the already-committed updates to deepseek-255 and deepseek-256,
   which were not listed in deepseek-259's files-in-scope block. Record the
   seventh review and this repair in all three reports.
2. Commit a machine-readable expected-versus-observed parity receipt for the
   real eight-checkpoint audit. It must identify each `recall.jsonl` source,
   bind its SHA-256, retain the expected and observed question/header/value/
   rest gains and both per-depth type-weight means, and record exact equality
   for every arm.
3. Clarify the time-bounded 06:07Z store observation and identify only the
   synthetic retained reports as test-harness output; the real report is
   release-CLI output.
4. Recompute `storage.task_and_ledger_bytes` after every text edit, including
   the new order and parity receipt.

This remains a read-only checkpoint instrument. It makes no B2, carry,
capability, export or serving claim.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-254/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-255/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-256/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-259/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-260/**`
- `docs/labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. The 255/256 report changes are explicitly re-scoped and attributed to this
   repair order.
2. The parity receipt records eight exact expected/observed matches, including
   all per-depth type-weight means, and its hash is recorded.
3. The previous 404 is unambiguously time-bounded and report provenance is
   correct.
4. The final task-plus-ledger byte receipt is exact.
5. Report policy, local policy, all 11 gates and an eighth fresh exact-head
   review pass.

## Stop and ask if

Do not modify Claude-owned float-training files, its merged run, serving,
export, GPU state or another lab's work.
