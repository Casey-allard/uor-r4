# REPORT: one-model-end-to-end, claude-385 (V2(b) part 1: repair the vector reads V2(a) exposed)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3985. Base `cb91a1baa` (claude-381, V2(a), #3978).

The order came from Casey, 15 Nov 2026, relayed by the starting session: "can we please resolve the problems this is exposing? please figure out how to resole the issues". V2(b) is now two units. This one repairs every fault claude-381's REPORT named, each with a test that fails without its fix. The selection test, claude-386, comes next on the repaired reads, with its sheet's non-author FIT first. This unit trains nothing toward a bar. It is a float reference repair (D85) and an inventory-class unit under the native-space skill, so the notebook word is **inconclusive**.

**What is now true, at the head `44774d2a9eafc30514d1f26cddab28d414e98cf9` (wick try-4):**
- **Flags off is the same training.** Base `cb91a1baa` and the head train 20 steps of `p` with identical eval lines (3 of 3) and all 7 checkpoint files byte-identical. On the `--recall` B2 task with no read, 64 steps at `97b575bd9` (try-2), `recall.jsonl` 3 of 3 lines and both model files are identical to the base. Since then the head changed only when the Starling calibration fires, inside the read's own branch, and tests.
- **Every repair has a device test that fails with its own fix set aside.** 8 of 8 gates on wick, then 23 of 23 `vector_read` cuda tests green again. Two more host gates fail on the laptop.
- **The Starling reads now open.** On `p`, Lorentz reads 43% of rows at step 128 and 41% at 256, L2 41% and 40% (claude-381: none). On the B2 task the calibrated offset settles near 60 (Lorentz) and 64 (L2), against the fixed start 2.5.
- **The credit no longer reaches the trunk, and most of the early cost is gone.** Dev top-1 at step 256 of `p`, of 14,287: off 88; Lorentz 77, L2 79, Flock 64, prime-route 40. In claude-381 the same reads scored 4 to 10. One seed and 256 steps: a smoke.
- **Each read trains beside off at 1.05 to 1.10 times its step time,** the two started together on the same shared card.

## What did not work, first

1. **My first calibration fired on an eval forward.** The fresh-context review of the trainer slice found it: `run_long` armed the Starling calibration when the read was built, and the step-0 checkpoint's DEV panel ran first with the read active. So the trained offset was set from held-out rows, and the step-0 eval line already carried it. `run_recall` was right (armed after reading 0). Repaired at `a0a61e018` in two places:
   - The calibration now fires only inside `collecting`, a training forward. An eval forward leaves it pending: a device test checks that, and fails with the gate removed (n9).
   - `run_long` arms it after the step-0 checkpoint.
   The try-2 Starling `p` smokes ran with the fault (dev top-1 at 256: Lorentz 66, L2 56). Their logs are kept in the receipts; the table below is try-4's. At the head, both Starling arms' step-0 dev line is off's, 4 of 14,287 at dev loss 2.78726.
2. **The pick test was vacuous for Starling.** `the_recorded_picks_are_the_reference_choices` set the offset to 1.0, where the fixture reads no row, so device and reference agreed on all-zero picks. The guard I added on review (at least one row read) failed on wick at `a0a61e018`. The offset now sits midway between the two middle rows' nearest earlier distances, so some rows read and some do not (`44774d2a9`).
3. **Item 6 (prime-route memory) has no behavioural negative.** The 120 × 120 table gives the same energies as the 32-channel phase gap it replaces, so no choice test can tell them apart. Its test pins the table against the f64 reference and the cell-pair indexing (a shifted block pairing fails it, n6). The memory saving itself is not pinned by any test: the old `(batch, time, time, 32)` path could come back with every test green. Card peaks are whole-card on a shared card and do not isolate it either.
4. **The near-tie choices of prime-route can differ from claude-381.** The energy is now four f32 sums of table entries rounded from f64, where claude-381 summed 32 f32 gaps. It is closer to the f64 reference, and equal cell pairs still tie exactly, but a near-tie can resolve the other way.
5. **The step times are on a shared card.** claude-380's two `train-in-space` runs, 1.4–3.3 GB each, shared the 4090 throughout (`gpu-apps.txt` in every pair). Each pair started off and the read together, so the ratio is the measure: off itself ran at 0.46 to 0.47 s a step on `p` in try-2 and 0.74 to 0.75 s in try-4, against 0.174 s alone (claude-370).
6. **The B2 smokes say nothing about choice yet.** At 64 steps each read chose the asked fact on 0 to 6 of 512 held-out queries per far bucket. That is a plumbing check (the counts, the cut and the calibration run end to end), not a reading of the mechanism. claude-386 runs 4,096 steps at two seeds.
7. **Not done:** no read was trained past 256 steps, none was scored on replies, and none has an integer form (V2(c)).

## What changed

- **`crates/uor-r4-float-train/src/vector_reads.rs`:**
  - **Repair 1, the Starling offset from the address scale.** `calibrate_next_offset()` arms it. The next training forward sets the offset to the lower median of each row's nearest earlier distance (`median_nearest`, rows with no earlier position left out) before it scores, and `take_calibrated_offset()` reports it once.
  - **Repair 2, the stream stop.** By default the address reads `flat.detach()`, so the credit and the straight-through factor reach only the rotations and the offset. The payload still adds to the stream, and the main loss's gradient still runs through it. `VectorRead::with_stream_grad` and `VectorReadArm::stream_grad` turn it back on. The switch is saved with the read; a file without it loads as `true` (a claude-381 read).
  - **Repair 5, Lorentz precision.** `e = (|q - k|^2 - (l_q - l_k)^2) / 2` with `l_q - l_k = (|q|^2 - |k|^2) / (l_q + l_k)`. This is algebraically the old `l_q l_k - (|q|^2 + |k|^2 - |q - k|^2)/2 - 1`, but it cancels at the scale of `|q - k|^2`, not `|q| |k|`, and it removes the outer product of the lifts. The f64 reference uses the same form.
  - **Repair 6, the prime-route table.** A detached 120 × 120 table, `[a * 120 + b] = reference::cell_energy(a, b)`, indexed by the two cells of each block. The energy is `(batch, time, time)` per block, summed over 4 blocks.
  - **Repair 8, ties.** Every kind takes the latest earlier position on equal scores. Flock's K are `latest_best`: the latest extreme, removed from the mask, K times. A row with fewer earlier positions than K gets weight 0 at the extra ranks, and those picks point inside the window.
  - **Item 4's hooks:** `without_vector_read` (the read unset for a pass, restored on return and on error) and `recording_picks` (each row's choice, position + 1, 0 for none; Flock's first kept).
- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - **Repair 3, `--recall` takes a vector read.** The read is built with its own `FlockAdamW`, its credit is added inside the training forward only, its variables are saved with the model, and Starling is calibrated at the first training forward (the first micro-batch when `--micro-batch` splits a step).
  - **Repair 4, the cut reading.** Each recall reading adds `vector_read`: rows that read, chose the asked value and chose the other fact's value per bucket (`count_read_choices`, from `recording_picks`), and the held-out hits and two-class counts with the read cut (`cut_hits`, `cut_query`, a second pass under `without_vector_read`). It also records the weight's mean size, the offset, the calibrated offset and the training counts.
  - `run_long` calibrates after the step-0 checkpoint (item 1 above).
- **`bin/train-in-space.rs` (flags only):** `--vector-read-stream-grad 0|1` (0 by default), refused without a read. `--recall` no longer refuses a read.

**Size:** 1,202 lines added and 101 removed over 6 files, 802 of the changed lines in `vector_reads.rs` (about half of them tests). The ORDER gave no size estimate, so there is no ratio to report.

## Measurements

wick, RTX 4090 (`sm_89`), shared. Base `cb91a1baa`. The `p` recipe (claude-358), seed 2. B2 is the 8 Nov sheet's two-class recall with the 4-token header (`--recall 4 --recall-query 1 --recall-header 4`), 64 steps, eval every 32, seed 2. Read after layer 6 of 6, margin 1, credit 1, stream stop on.

**Flags off, `p`, 20 steps, eval every 10 (try-4, the head):** eval lines identical, 3 of 3; at step 20 `model.safetensors`, `hamming_read.safetensors`, `turn_memory.safetensors`, `turn_memory_reply.safetensors`, `hamming_place.json`, `config.json` and `.checkpoint-ready` identical, none on one side only (also at `97b575bd9`, try-2). Recall B2 with no read, 64 steps (try-2, `97b575bd9`): `recall.jsonl` 3 of 3 lines (timing cut) and `config.json`, `model.safetensors` identical; the base binary refuses `--recall` with a read.

**Each read beside off, started together, 256 steps of `p` then 64 steps of B2:**

| Arm | `p` s a step (off, read) | × off | B2 × off | dev top-1 (256) | dev loss (256) | `p` rows read (256) | chose a positive (256) |
|---|---|---|---|---|---|---|---|
| off | 0.46–0.75 | 1.00 | 1.00 | 88 | 2.77775 | – | – |
| Starling Lorentz (try-4) | 0.7510, 0.8011 | 1.07 | 1.08 | 77 | 2.78668 | 214,693 / 522,240 | 87,290 |
| Starling L2 (try-4) | 0.7429, 0.7899 | 1.06 | 1.08 | 79 | 2.79858 | 208,527 / 522,240 | 68,344 |
| prime-route (try-2) | 0.4698, 0.4985 | 1.06 | 1.05 | 40 | 2.79621 | 522,240 / 522,240 | 137,112 |
| Flock 4 (try-2) | 0.4711, 0.5148 | 1.09 | 1.10 | 64 | 2.78642 | 522,240 / 522,240 | 170,441 |

- **Prime-route and Flock ran at `97b575bd9`.** The head changed only when the Starling calibration fires and tests, which these two kinds never reach. Off is the same run in every pair: the training is deterministic, so off's losses and dev line are identical in all six pairs.
- **Last train loss at 256:** off 1.00320; Lorentz 1.00331, L2 1.00326, prime-route 1.00332, Flock 1.00324 (claude-381: 1.00455 to 1.00484).
- **Card peak MiB per pair (off, read), whole card with the neighbours:** Lorentz 16,269 and 19,757; L2 16,493 and 19,341; prime-route 13,530 and 16,186; Flock 13,434 and 17,882.
- **Positive rows:** 402,974 of the 522,240 rows at step 256 have an earlier position followed by their own next token. Descriptive: no control was run.
- **B2 at step 64, per far bucket of 512 held-out queries:** prime-route read 512 and 512, chose the asked value 2 and 6, the other 2 and 3; Flock read 512 and 512, asked 2 and 2, other 5 and 2; Lorentz read 321 and 311, asked 1 and 0, other 0 and 3, offset 59.50; L2 read 355 and 349, asked 2 and 5, other 3 and 1, offset 64.33. The cut reading equals the uncut one at step 0 in every pair (`step0-cut.txt`), as it must with the weight at 0.

## Checks run at the head

- **Laptop:** `cargo test -p uor-r4-float-train --lib vector_read` 9 passed; `--lib in_space_long` 200 passed; `--bin train-in-space` 16 passed. Host negative gates: the cut not unsetting the read fails `the_cut_and_the_pick_recording_restore_what_they_found`; an upper median fails `the_offset_start_is_the_lower_median_of_the_rows_with_an_earlier_position`.
- **wick, `build-u` at the head (try-4):** clean checkout, `cargo +1.97.1 build --release --locked --features cuda`, binary sha256 `509f378e553b8dd270dc7e37b7f0f3bfb9c007585e2330000b2f430c574dbf95`; `--lib vector_read` 23 passed; `--bin train-in-space` 16 passed.
- **wick, `neg` at the head (try-4):** each patch applied alone in a clean checkout, its named test run and required to fail (ran 1, failed 1), the file restored, then the `vector_read` tests again, 23 passed:
  - n1, calibration off: `a_known_positive_opens_the_calibrated_read_and_is_chosen`
  - n2, stream detach removed: `the_stream_stop_leaves_the_trunk_no_credit_gradient`
  - n5, the old Lorentz form: `the_lorentz_chord_keeps_the_hyperbolic_order_at_the_state_scale`
  - n6, block pairing shifted: `the_cell_pair_table_is_the_reference_energy`
  - n7, factor sign flipped: `the_payload_gradient_is_minus_the_chosen_distance_gradient`
  - n8, Flock back to the earliest-tie argmin: `equal_scores_take_the_latest_position_for_every_kind`
  - n9, calibration at an eval forward: `the_calibrated_offset_is_the_median_nearest_distance`
  - n10, a chosen rank not removed: `every_kept_rank_takes_the_latest_left_on_a_tie`
- **Review (fresh context, three slices, at `97b575bd9`):** the read's code APPROVE, its tests APPROVE, the trainer plumbing CHANGES on one finding (item 1 above), repaired at `a0a61e018`. The tests reviewer's minor findings repaired in the same commit: credit cleared after the calibration test, an at-least-one-read guard (which then caught item 2), and a direct test that every rank Flock keeps is the latest left on a tie (n10). The rest are under Still open.
- **Banned ops:** `grep -nEi "softmax|sigmoid|tanh|gelu|softplus|\.exp\(|matmul|einsum|log_sum_exp"` on `vector_reads.rs`: no line. The device path has no `log`; host-side `ln` builds the phase table and the f64 reference only.
- **Receipts:** every log of tries 2–4 (no checkpoints), the job kit and the ten mutation patches are in `local/claude/claude-385/`, 692 files, `SHA256SUMS` sha256 `0da349245b47f1022caa2ab6fdb3907081a3521525deb5ccba1fbacb4b52ca5f`. try-3 is the failed `build-u` of item 2 above.

## Skills used

`native-space-cycle` (read before the work: five items, none met, so an inventory-class unit, recorded in the ORDER); the loop's ORDER/REPORT form; wick kit `local/claude/wick/wick.sh` with a completion watcher on every chain.

## Still open

- **claude-386, the selection test:** the sheet's FIT, then 20 arms of 4,096 steps on the repaired reads. The sheet predates the stream stop and the calibration and must name both.
- **Item 6's memory** wants a measured peak at a large context on a quiet card, or a test on the tensor shapes held.
- **From the review (minor, not repaired):**
  - `count_read_choices` compares the chosen position's next token with the asked value, not the asked value's position. With values unique inside a window (the sheet's bijection), the two agree.
  - `recording_picks` keeps only Flock's first kept pick, so the cut reading cannot show the other K − 1.
  - Unsetting the read clears a calibrated offset not yet taken; every caller takes it first.
- **V2(c):** the winner's integer form, native state schema, shared operator and bit-identical replay oracle (D85).

## Plain summary

The four look-back reads that claude-381 added now work as intended. The two Starling reads actually read (about half the rows), the learning signal that taught them no longer pulls on the rest of the model, and the scores are more precise and use less memory. The recall test can now run with a read and with that read switched off for comparison. With the reads off, training is byte-for-byte the same as before. Each fix has a GPU test that fails when the fix is taken out. Nothing here says yet which read is best: that is the next unit's test.
