# ORDER: one-model-end-to-end, claude-300

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `db843d578b33d8289e102cdd69c6d59d73d3c830`.

## Goal

Casey's one track (#2028 6675163438, "Yes, one track"): the trunk's no-memory top-1 must beat its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band, at every seed.

claude-298 (#3796) could not test batch 64. Every cell ran out of the RTX 5090's 32 GB, and the sheet's Stop rule names gradient accumulation as the next unit. This unit builds it:
- **A `--micro-batch m` flag** for `train-in-space`. Each step's `--batch` windows go forward and backward in forwards of `m` windows. Each micro-batch's loss is weighted by its share nᵢ/N of the step's scored rows, the gradients are summed, and one optimizer step follows.
- **The memory and reply rows of each micro-batch** are indexed within that micro-batch, as one forward needs them. Memory dropout keeps the window's place in the whole batch, so the draws do not change.
- **Off by default.** With no flag, or `m` equal to the batch, the step is the one it was, bit for bit.
- **A CUDA test** shows that 4 micro-batches of 2 give the same loss and gradients as one batch of 8, to float tolerance. A negative gate shows the test fails when the micro-batches are weighted by 1/k.

The batch-64 rerun is the next unit, on its own sheet with a memory gate.

## Claim check

- **(a) The step.** It is what claude-298's Stop rule names as next, on the "data moves it" branch of claude-297's sheet. claude-298 merged on 6 Nov.
- **(b) What it changes in the model.** Nothing yet: it lets the trunk train at batch 64 on one 32 GB card, so the next sheet can test whether a larger batch narrows the seed spread.

## Cancel probe (run first)

Run on 6 Nov, before any work. Class: instrument.
1. **No hold.** There is no `local/loop/STOP`, and #2028 has no `loop:pause`.
2. **`loop next`** offers claude no other GO unit on the one track. Its GO line is D84's CPU direction, which binds DeepSeek's CPU program, not claude.
3. **The trainer** has no accumulation flag on `main` (`grep -n micro in_space_long.rs` finds only "microseconds" in the profile clock).

Result: GO.

## Done when

1. `--micro-batch` is parsed, written to the run's config, and refused unless it divides `--batch`.
2. The CUDA equivalence test passes on a pod at the head, and its negative gate fails as expected. Both are recorded in the REPORT.
3. The crate's CPU tests, `cargo fmt --check` and `cargo clippy` pass at the head.
4. A fresh review of the diff is recorded at the exact head.
5. The REPORT gives the negatives first and ends `References #3407`.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The equivalence test fails beyond float tolerance:** the forward is not window-independent. Find which state couples windows before going on.
- **The pod cap:** one pod, $2.
- **Casey says "hold".**

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/**`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
