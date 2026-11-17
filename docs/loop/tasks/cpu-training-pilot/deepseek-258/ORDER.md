# ORDER: cpu-training-pilot, deepseek-258 (exact resource receipt repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `3a700910a`. Parent ORDER:
[deepseek-257](../deepseek-257/ORDER.md).

## Goal

Repair the fourth fresh exact-head review of deepseek-257:

1. Correct the resource ledger's `storage.task_and_ledger_bytes` after all
   deepseek-258 text is final. At `3a700910a` the ledger declares `82,835`
   bytes, but the four task folders plus the ledger total `82,836`; the final
   deepseek-257 size edit added one REPORT byte after the ledger measurement.
2. Record that the fourth review requested changes for this exact receipt and
   that this order repairs it.

No Rust source, execution, artifact, notebook row or scientific claim changes.
The repaired receipt must equal the final tree's task-folder plus resource-ledger
byte total. Recompute it after writing the ORDER and REPORT; do not assume the
legacy value or a predicted new value.

This remains a read-only checkpoint instrument. It makes no B2, carry,
capability, export or serving claim.

## Files in scope

- `docs/loop/tasks/cpu-training-pilot/deepseek-257/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-258/**`
- `docs/labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json`

## Done when

1. The final `task_and_ledger_bytes` equals the exact sum of all files under
   deepseek-254 through deepseek-258 plus this resource ledger.
2. The deepseek-257 report records the fourth review's requested change and
   deepseek-258 as its repair.
3. Report policy, local policy, the applicable checks and all 11 gates pass,
   and a fifth fresh exact-head review approves the final head.

## Stop and ask if

Do not modify Claude-owned float-training files, its active run, serving,
export, GPU state or another lab's work.
