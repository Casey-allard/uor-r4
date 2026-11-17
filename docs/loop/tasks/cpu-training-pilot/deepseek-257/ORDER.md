# ORDER: cpu-training-pilot, deepseek-257 (non-regular-file refusal)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `89b688a9c`. Parent ORDER:
[deepseek-256](../deepseek-256/ORDER.md).

## Goal

Repair the third fresh review of deepseek-256:

1. Open Linux batch checkpoint files nonblocking and with no-follow semantics,
   then validate the opened descriptor as a regular file before reading.
   A FIFO under the manifest root must be refused, not block a worker.
2. Add a focused test that a checkpoint child path which is a directory is
   refused as non-regular.
3. Escape both `|w_y|` occurrences in the notebook table so the repository's
   row parser sees six cells rather than ten.
4. Correct the synthetic checkpoint's ask-head count from two to one ask head
   at each of two depths.
5. Record that the third review requested changes and this unit repairs them.

This remains a read-only checkpoint instrument. It does not change checkpoint
contents or make a B2, carry, capability, export or serving claim.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-254/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-255/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-256/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-257/**`
- `docs/labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. FIFO and other non-regular child paths fail closed without a blocking open.
2. The notebook row parses as exactly six cells with the expected fields.
3. Focused tests, full CPU library tests, all-target check, scoped Clippy,
   release build, report policy, local policy, all 11 gates and a fourth fresh
   exact-head review pass.

## Stop and ask if

Do not modify Claude-owned float-training files, its active run, serving,
export, GPU state or another lab's work.
