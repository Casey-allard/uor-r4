# ORDER: one-model-end-to-end, claude-296

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `5fc76143d`.

## Goal

Casey's one track (#2028 6675163438, 6 Nov 2026, "Yes, one track"): every claude unit goes to making the trunk learn human-written text. The bar is no-memory top-1 that beats its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band.

This unit classifies claude-295's flat loss on human text, instrument before mechanism (§4.2). It follows the prep sheet [`one-model-end-to-end-human-text-2026-11-06`](../../../../harness/prep/one-model-end-to-end-human-text-2026-11-06.md), FIT in round 2. The arms are at (256, D1), seeds 1 and 2, against claude-295's sealed control:
- (m1), fixed targets: `--train-targets mode:1`;
- (lr3): `--lr 0.003`;
- (near): `--objective nearest`.

It serves the item's done-when 3. The model to ship has to learn human text before its protocol can score it.

## Claim check

- **(a) The step.** This is step 2 of the one track's order: "finish claude-295 fast; the classification unit (claude-296, one-model-end-to-end); whatever it finds, until the bar is met". claude-295 merged as `9e2c0125bac53f92900cf62762dd3f4cd67ff5ae` (#3750).
- **(b) What it changes in the model.** It finds out whether the trunk learns human text when each context has a single target, and whether the learning rate or the objective moves the flat loss. The answer names the next change to the recipe of the model the project ships.

## Cancel probe (run first)

Run at 04:29Z on 6 Nov, before any code. Class: instrument.

1. **No hold.**
   - There is no `local/loop/STOP`, and #2028 has no `loop:pause` label (`loop next`: kill switch off).
   - #3407's last four comments are claude-281's two notes and DeepSeek's two copy-key notes, none of them a hold.
2. **The inputs are in the store.**
   - D1 is in `results/claude/scale-probe-2026-11-05/inputs/d1` (tokens 13.4 MB, mask 6.7 MB, manifest).
   - The control's sealed runs are in `runs/A` (`scale-probe-A.tar.gz`, `SHA256SUMS`).
   - The control's grades `grade/w256-d1-s{1,2}` and its no-memory rows are on the laptop, from claude-295.
3. **The caps:** `uor-pod status` shows 0 pods running, $0/h of $8/h, and 16 RTX 5090 offers from $0.563/h.
4. **The trainer is unchanged since claude-295's pin:** `crates/uor-r4-float-train/` has no diff from `e052abf41` at the base. So the control and the arms run the same code, apart from the new flag.

Result: GO.

## Done when

1. **The code.**
   - `--train-targets observed|mode:K` (K from 1 to 4), with its refusals, the I-m print and the sheet's unit tests.
   - Green on the laptop: the crate's tests and a stub-CUDA `cargo check`, with a negative gate for the `mode:1`-equals-brute-force-T2 test.
2. **The sheet** is at `docs/harness/prep/`, and its FIT is posted on #3407.
3. **The D67 pre-run review** of the job and watch scripts passes in a fresh context and is posted on #3407 before the pod goes up.
4. **The runs.** The six runs, or the stop rule's NOT_RUNs, are sealed under `results/claude/one-model-end-to-end-human-text-2026-11-06/` with a `SHA256SUMS` that was read back. The pod is down.
5. **The REPORT** gives:
   - training positions a second first;
   - Q1 and Q2 by the sheet's Bar;
   - I-0 to I-5 and I-m;
   - one row in `docs/research/notebook/float-training.md`.

   The item stays `ready`, and the REPORT says `References #3407`.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The sheet's stop rule:** I-0, I-3 or I-m failing stops everything. A failed arm is NOT_RUN at its failing step. Side by side out of memory goes back to one at a time.
- **The cap:** $3 and 4 hours. Arms still running at the cap are NOT_RUN.
- **A watcher's time or spend stop fires on a healthy run:** extend it and renew the lease; never cancel it.
- **Casey says "hold":** no further launches, and nothing is named or closed.
- **The laptop's free disk falls under 5 GiB:** downloads pause until my own store-sealed copies are cleared.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-training/examples/m-world.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-training/src/bin/m-world-recall-corpus.rs`
- `crates/uor-r4-training/src/bin/mix-chat-corpus.rs`
- `crates/uor-r4-training/Cargo.toml`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-training/src/milestone_world_v2.rs`
