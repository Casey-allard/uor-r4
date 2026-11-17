# ORDER: cpu-training-pilot, deepseek-255 (carry-hold containment repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `eeca55aa8`. Parent ORDER:
[deepseek-254](../deepseek-254/ORDER.md).

## Goal

Repair the fresh exact-head review of deepseek-254:

1. Make manifest containment hold from validation through the bytes parsed:
   - canonicalize the manifest root once;
   - retain and audit the canonical checkpoint directories that were checked;
   - reject duplicate canonical paths, not only duplicate path strings;
   - canonicalize and regular-file-check each checkpoint file;
   - on Linux, verify the opened file descriptor still resolves beneath the
     canonical root before reading it.
2. Add a test that a child-file symlink to a checkpoint outside the manifest
   root is refused.
3. Assert the rest-token gain mean in the grouped synthetic control.
4. Reject batch-only `--workers` in single-checkpoint mode.
5. Refresh the final `pr-check` size receipt.

This remains a read-only checkpoint instrument. It does not prove B2, carry,
capability, export or serving.

## Files in scope

- `crates/uor-r4-cpu-train/src/ask_offset_audit.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-254/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-255/**`
- `docs/labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json`

## Done when

1. The validated canonical root and checkpoint paths are the only paths the
   batch worker opens, and Linux checks the opened file descriptor before read.
2. Symlinked child files outside the root, canonical path aliases and mixed
   single/batch arguments are refused by tests.
3. The rest-token mean is pinned.
4. Focused tests, full CPU library tests, all-target check, scoped Clippy,
   release build, report policy, local policy, all 11 gates and a fresh
   exact-head review pass.

## Stop and ask if

Do not modify Claude-owned float-training files, its active run, serving,
export, GPU state or another lab's work. Do not claim path containment on a
platform where the pinned-descriptor check is unavailable for batch mode.
