# REPORT: float-training-module, deepseek-1 (FBS query cache)

Lab: deepseek. Item:
[float-training-module](../../../../roadmap/items/float-training-module.md) (#3801).
Order: [ORDER.md](ORDER.md). Base: `ff5156bcc`.

## What did not work, first

**Instrument ready for a sealed-checkpoint capture; no model result yet.**

`train-in-space --score-rows --query-cache-out DIR` now writes the active FBS
teacher-forced queries, targets, corpus positions, frozen nearest/writer
decisions, opening/copy flags, and the original shared vector table. The cache
is immutable and sealed. A separate CPU binary verifies it, fits centroid output
rows from TRAIN, builds an equal-observation-count shuffled-label control, and
evaluates all three tables on the recorded panels.

This is the active one-model-end-to-end track's centroid-refit step, not a
restoration of a retired mechanism. It does not change `run_long`,
`step_gradient`, the optimizer, micro-batching, serving defaults or Claude's
scorer calculations.

The 9950X evaluator parallelizes independent 64-query blocks across Rayon
workers and reuses each table row across the block. The cache's row order is
unchanged, and top-8 ties go to the lower token id.

## What changed

The float-training module gains an opt-in scorer cache, a sealed cache reader,
and a separate CPU centroid evaluator. `--score-rows` captures data from its
existing `row_picks` callback without changing the forward. The new binary
verifies the cache and evaluates original, centroid and matched-control tables.

## Why this is the next step

Claude-297 found that data moved the human-text trunk result while the tested
objective/step cells did not. The one track in `one-model-end-to-end` therefore
names centroid refit of output rows as the next readout experiment after step 1,
provided the trunk stays frozen.

The cache removes repeated GPU forwards from that experiment:

- capture the query trajectory once from the current checkpoint;
- fit and compare output rows on the workstation CPU;
- keep the frozen input table and decisions as controls;
- report the result against both original and matched-label shuffling.

The existing DeepSeek-237 public-artifact experiment already rejected a median
readout on two seeds and lost top-8 on both. That is a strong prior, not a
replacement for evaluating the sealed current checkpoint through the active
scorer. A future result must still pass its registered ranking and top-8 gates.

## LexLean-R4 decision

The full engineering read is [LEXLEAN-AUDIT.md](LEXLEAN-AUDIT.md).

LexLean is ahead as a browser product and exact-memory demonstration. It is not
ahead as evidence for generalized language capability. The one immediate code
transfer is its teacher-forced trajectory/readout separation. Its `/8` artifact
is a separate dialect, and its recurrence contains a sigmoid lookup that
UOR-R4's geometry and serving rules ban. Its engine's count-based “proofs”
compile authored prompt/reply pairs into memory; that is curriculum ingestion,
not learned language.

Canonical point/reply anchors, idempotent online admission and strict-loss
updates are worth studying inside M2/M3, after parity on their current UOR-R4
owners. Browser sharding remains Kimi's product lane.

## Cache identity

The immutable cache records:

- canonical checkpoint path and a sorted SHA-256 inventory of its files;
- the run `config.json` used to recover query space, and its SHA-256;
- pinned tokenizer, data root, query space, vocabulary and width;
- original table bytes and SHA-256;
- split, memory mode, context, row count, corpus-token hash and mask hash;
- query and decision-row file hashes;
- a seal over the complete manifest.

`QueryCache::open` refuses a bad seal, schema, tokenizer, shape, table, split
inventory, query/decision file hash, duplicate position, out-of-range decision
or non-finite table value. The evaluator additionally refuses an empty TRAIN or
panel, duplicate panel names, and using TRAIN as a panel. Its development
default is at least eight observations per fitted token.

Decision rows preserve target, corpus position, frozen nearest, frozen writer,
opening and copy flags. The cache does not store the native writer's kept-cell
IDs yet; that is required before rank-aware candidate evaluation and is not
claimed by this unit.

## CPU evaluation

`fbs-query-cache evaluate CACHE --train train --panel NAME [--memory 0|1]
[--minimum 8] [--out FILE]`:

1. opens and verifies the sealed cache;
2. loads TRAIN and each panel in the recorded memory mode;
3. evaluates the original table and checks every frozen nearest decision;
4. replaces eligible token rows by the TRAIN mean query for that token;
5. evaluates the centroid and a deterministic shuffle of centroid rows among
   tokens with the same observation count;
6. reports top-1, top-8, changed decisions, paired transitions, fit power and
   exact decision hashes.

If the original arm differs from even one frozen decision, status is
`CONTROL_FAULT`; candidate numbers are diagnostic only. This is deliberately
strict because GPU and CPU reduction order can differ.

The centroid table is an offline F32 development instrument. It is not an
export, does not alter input queries and is not a serving implementation.

## Measurements

No model result or timing claim is made. No checkpoint was available locally,
and `nvidia-smi` is absent, so this workstation cannot run the capture forward
or open a real panel. The source-level cache size scales as
`rows * width * 4 + rows * 18` bytes plus the table and manifest.

## Checks run at the head

Passed:

- `cargo test -p uor-r4-float-train --lib --no-fail-fast --locked --offline`:
  **292 passed, 0 failed**.
- `cargo check -p uor-r4-float-train --bin train-in-space --bin
  fbs-query-cache --locked --offline`: passed.
- Scoped `cargo clippy -p uor-r4-float-train --lib --bins --no-deps --locked
  --offline -- -D warnings`, with only the crate's pre-existing lint categories
  allowed: passed.
- Focused tests cover query/table/decision-byte mutation, sealed-manifest
  mutation, wrong memory mode, duplicate positions, immutable cache roots,
  blocked row order and lower-id top-8 ties.

Formatting:

- The new cache module, its binary, library export and small CLI edits were
  formatted.
- The large Claude-owned `in_space_long.rs` and `train-in-space.rs` were only
  patched at the cache seam and usage text; they were not globally reformatted.

Not run:

- No real checkpoint was available locally.
- `nvidia-smi` is absent; this workstation has no CUDA runtime for the capture
  forward.
- No cache was generated, no centroid panel was opened, and no capability or
  quality claim is made.

## Skills used

No system-provided Codex skill applied. Repository geometry, harness,
pre-registration and task-report rules were used directly.

## Still open

After this instrument lands, capture one FIT/GO-approved sealed current-FBS
checkpoint on the selected TRAIN and panels. Pre-register memory mode, context,
panel names and minimum observations, then run the CPU evaluator. Do not tune
the threshold after opening the panel.

Only after the cached original reproduces should the centroid result be treated
as evidence. Rank-aware or margin candidates come later and require capturing
each row's native kept-cell IDs into a schema revision.

## Plain summary

This change saves the model's internal query points during one ordinary scoring
run. A later CPU program can then test a different output table without running
the model again. The saved data is sealed and checked so experiments cannot
silently mix checkpoints or edit results. The capture and evaluation code are
built and tested, but no real model experiment has run yet.
