# ORDER: cpu-training-pilot, deepseek-256 (manifest root receipt repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `3bc0928d1`. Parent ORDER:
[deepseek-255](../deepseek-255/ORDER.md).

## Goal

Repair the second fresh review of deepseek-255:

1. Make a basename-only manifest path such as `MANIFEST.json` resolve its
   containing directory as the current directory. Rust returns `Some("")` for
   that path's parent, and canonicalizing the empty path fails.
2. Add a regression that distinguishes a basename-only manifest path from a
   path with `./`.
3. Refresh the resource ledger's retained text bytes.
4. Correct the notebook row from five focused binary tests and stale timing to
   the final six-test measurement.

This remains a read-only checkpoint instrument. It does not change checkpoint
contents or make a B2, carry, capability, export or serving claim.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-254/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-255/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-256/**`
- `docs/labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. A basename-only manifest path uses the current directory and has a focused
   passing test.
2. The resource ledger and notebook record the final bytes, test count and
   timing.
3. Focused tests, full CPU library tests, all-target check, scoped Clippy,
   release build, report policy, local policy, all 11 gates and a third fresh
   exact-head review pass.

## Stop and ask if

Do not modify Claude-owned float-training files, its active run, serving,
export, GPU state or another lab's work.
