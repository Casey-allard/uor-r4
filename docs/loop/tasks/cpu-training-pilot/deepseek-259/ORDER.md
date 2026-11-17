# ORDER: cpu-training-pilot, deepseek-259 (real-seal audit repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `81451387a`. Parent ORDER:
[deepseek-258](../deepseek-258/ORDER.md).

## Goal

Repair the sixth fresh exact-head review after Claude-334's run merged:

1. Audit the real sealed `B2-aqH` and `B2-aH` checkpoints at 4,096 and
   16,384 steps on both seeds with the manifest runner and eight workers.
   Verify the downloaded seed tarballs against Claude-334's recorded hashes,
   retain the manifest and complete report as text evidence, and compare every
   grouped gain and per-depth type-weight mean with Claude's `recall.jsonl`.
2. Replace the stale `NOT_RUN` statements in deepseek-254, deepseek-257,
   deepseek-258 and the notebook with the real audit result. Preserve the
   original 06:07Z HTTP 404 as a time-bounded observation, not a current fact.
3. Correct the D84 projection. Its five components currently total 3,603.25
   seconds against a 3,600-second wall limit. Make the components sum to the
   admitted 3,600-second increment and record the real audit's measured wall,
   CPU and peak-RSS readings.
4. Add a Linux FIFO regression with a bounded wait. A regression to a blocking
   open must fail the test without hanging it.
5. Recompute `storage.task_and_ledger_bytes` after all evidence and records are
   final.

This remains a read-only checkpoint instrument. It makes no B2, carry,
capability, export or serving claim.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-carry-hold-audit.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-254/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-257/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-258/REPORT.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-259/**`
- `docs/labs/cpu-training-pilot-carry-hold-grouped-2026-11-09/resource-ledger.json`
- `docs/research/notebook/cpu-training.md`

## Done when

1. Eight real held checkpoints pass the hash-bound CPU audit, and all eight
   grouped gain/type-weight readings match Claude-334 exactly.
2. Every current `NOT_RUN` statement is replaced by the real result while the
   earlier 404 remains explicitly time-bounded.
3. The D84 projection components sum to 3,600 seconds.
4. The FIFO regression passes and cannot hang indefinitely on failure.
5. Focused tests, full CPU library tests, all-target check, scoped Clippy,
   release build, report policy, local policy, all 11 gates and a seventh fresh
   exact-head review pass.

## Stop and ask if

Do not modify Claude-owned float-training files, its merged run, serving,
export, GPU state or another lab's work.
