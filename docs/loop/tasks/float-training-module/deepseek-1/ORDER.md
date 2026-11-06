# ORDER: float-training-module, deepseek-1 (FBS query cache)

Lab: deepseek. Item: [float-training-module](../../../../roadmap/items/float-training-module.md)
(#3801 records the query-cache work).
Base: `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`.

## Goal

Port the useful LexLean trajectory separation as a checked current-architecture
instrument: emit teacher-forced FBS queries and targets once, then evaluate
output-table candidates on this workstation CPU without rerunning the trunk.

The first unit establishes the cache and CPU evaluator. It does not train,
change optimizer or micro-batch behavior, or claim model capability.

## Cancel probe

Class: instrument. The current `row_picks` already gives `score_rows` each
forward's selected query tensor and returns the matching targets, positions,
nearest picks and writer picks. `--score-rows` owns the full split and memory
setup. This is a bounded optional output path, so the answer is **GO**.
LexLean's latest `/8` runtime is a non-canonical dialect with a banned sigmoid
lookup. Only its teacher-forced trajectory separation is in this unit; see
[LEXLEAN-AUDIT.md](LEXLEAN-AUDIT.md).

## Done when

1. `--score-rows --query-cache-out DIR` writes an immutable cache containing the
   unchanged original table, memory and no-memory query rows, targets,
   positions, nearest/writer decisions and opening/copy flags.
2. The cache manifest binds checkpoint files, tokenizer, corpus tokens/mask,
   context, table shape/hash and every cache file hash. Existing, reordered,
   mixed, corrupt or changed-input caches fail closed.
3. `fbs-query-cache evaluate` runs without CUDA, verifies the cache and applies
   centroid and deterministic matched-label output rows to the cached panel
   queries without changing the input table.
4. The original arm reproduces every frozen nearest decision exactly. The report
   gives top-1, top-8, changed decisions, power and paired centroid/original and
   centroid/control comparisons.
5. Focused tests include one bit, one byte, one row, one split, one table and
   one checkpoint identity mutation, all red.
6. `cargo test -p uor-r4-float-train --lib`, formatting, scoped Clippy and the
   D23/D25/D32 ratchets pass.

## Stop and ask if

Ask means decide, record here and in REPORT, and continue.

- The cache cannot stream without holding all query rows in memory.
- The current scorer's query order cannot be bound to returned row picks.
- A code path would need to change `run_long`, `step_gradient`, the optimizer or
  serving defaults.
- A real checkpoint/cache run would require a GPU lease; this build unit stops
  before that run.

## Files in scope

- `crates/uor-r4-float-train/**`
- `docs/loop/tasks/float-training-module/deepseek-1/**`
- `docs/research/notebook/float-training.md`
