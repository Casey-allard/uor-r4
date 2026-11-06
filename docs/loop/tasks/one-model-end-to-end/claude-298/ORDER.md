# ORDER: one-model-end-to-end, claude-298

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `1f2bd76ce58503f25ef24e415dd193bf57086ddb`.

## Goal

Casey's one track (#2028 6675163438, 6 Nov 2026, "Yes, one track"): the trunk's no-memory top-1 must beat its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band, at every seed.

claude-297 (#3774) found two things. Data moves the trunk. And its best cell, (n3L), D3 at 32,768 steps and batch 16, sits `inside` D3's T2 because its two seeds disagree: 272,867 / 234,892 against 256,660. `pair`'s band includes the seed spread, so added seeds cannot turn it ahead.

This unit takes claude-297's data branch, "train longer at D3", with one change that also attacks the spread: batch 64 in place of 16, at seeds 1 and 2. It follows the sheet [`one-model-end-to-end-d3-batch-2026-11-06`](../../../../harness/prep/one-model-end-to-end-d3-batch-2026-11-06.md).

| arm | batch | steps | tokens against (n3L) |
|---|---|---|---|
| (b64S) | 64 | 8,192 | equal |
| (b64L) | 64 | 32,768 (16,384 if the throughput gate I-t says so) | 4× (2×) |

claude-297's sealed (n3L) is the batch-16 control. The trainer is unchanged between the two pins.

## Claim check

- **(a) The step.** This is step 1's follow-on in the roadmap item's one-track sequence: the "data moves it" branch of claude-297's sheet. claude-297 merged as `1f2bd76ce58503f25ef24e415dd193bf57086ddb`.
- **(b) What it changes in the model.** It tests whether the trunk's recipe meets the one-track bar when trained on more of D3 with a less noisy gradient. If it does not, it shows whether the seed spread is gradient noise.

## Cancel probe (run first)

Run at 14:42Z on 6 Nov, before any work. Class: instrument.
1. **No hold.**
   - There is no `local/loop/STOP`, and #2028's labels are `roadmap` and `roadmap:tracker`, with no `loop:pause`.
   - #3407's last comments are claude-296's merge note and claude-297's intent, FIT and D67 notes.
2. **The inputs are in the store:** D3 at `results/claude/scale-probe-2026-11-05/inputs/d3` (tokens 53.3 MB, mask 26.7 MB, manifest).
   - The panels and D3's seen set are where claude-297 read them: `local/claude/claude-295/seen/d3`, now in the main checkout's `local/claude`.
   - claude-297's (n3L) grades are at `local/claude/claude-297/grade/`.
3. **The caps:** `uor-pod status` shows 0 pods running and $0/h of $8/h.
   - The Vast adapter was rebuilt for this worktree at `local/claude/pp-target` and is used through `UOR_POD_VAST_BIN`.

Result: GO.

## Done when

1. **The sheet** is at `docs/harness/prep/`, and its FIT is posted on #3407.
2. **The D67 pre-run review** of the job and watch scripts passes in a fresh context. It is posted on #3407 before a pod goes up.
3. **The runs**, or the stop rule's NOT_RUNs, are sealed under `results/claude/one-model-end-to-end-d3-batch-2026-11-06/`, each pod's with a `SHA256SUMS` that was read back. The pods are down.
4. **The REPORT** gives, negatives first:
   - each arm against D3's T2 and unigram, by the sheet's Bar, on all rows and on unseen rows;
   - the levers, and each arm's seed spread beside (n3L)'s;
   - I-0 to I-5, I-t, I-s and I-r;
   - one row in `docs/research/notebook/float-training.md`.

   It ends `References #3407`.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The sheet's stop rule:** an I-0 or I-3 failure stops that pod. A failed arm is NOT_RUN at its failing step.
- **The cap:** $14, and 6 hours a one-card pod. When a watcher's stop fires on a healthy run, extend it and renew the lease; never cancel it.
- **Casey says "hold":** no further launches.
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

