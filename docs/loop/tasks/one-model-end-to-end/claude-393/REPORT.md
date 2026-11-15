# REPORT: one-model-end-to-end, claude-393 (a window-split gate credit, and the sheet for the first native read training run)

Lab: claude. Item: one-model-end-to-end (#3407). ORDER: [ORDER.md](ORDER.md). PR #4014.

Commits:
- `fa63a11c5`: the ORDER;
- `0b5e7861b`: the kernels, the wrapper and the timing test;
- `1286973474`: the sheet, with its FIT pending;
- `fc0651a8d`: the FIT's changes;
- the last commit: this REPORT and the notebook row.

## What did not work, first

1. **The split does not speed up the run's step.** The run's density is four target rows a window, 256 rows a batch. At width 256 and 64 × 192 rows, the step takes a median of 16.27 ms against main's 17.77 ms, 8% less. The serial loop skips rows without a rival at once, so at this density the credit kernel was never the cost.

   The step's time goes elsewhere: three `first_pick_read` passes, each O(time² · width) per window, and two `pick_loss` passes. Each call also copies the 25 MB stream to the device again. Both are named in Still open 1; neither was measured piece by piece.
2. **The first timing test had one density, and it showed no change** (17.44 ms against 17.38 ms). A second case was added, with a target on every row. It shows where the split works: 16.68 ms against 37.46 ms, 2.2 times faster.
3. **The FIT found the sheet's gain claim wrong.** The draft said claude-389's conversion "gives the weights at gain shift `g = 4`". claude-389 converts every state at `g = 0`. The sheet now has the trainer set `g = 4`.

   The FIT also found the draft's outcome map incomplete. A rise of 0.05 to 0.10 fit no outcome, and mixed readings fit none either. The map now has a fixed order, is judged per (state, bucket), and ends in a catch-all.
4. **The FIT did not read everything.** It stopped at its context budget before the device step code, `native_replay_pass` with `recall_payload_probe`, and the earlier sheet. The sheet's FIT line names these three for the run unit's pre-run review.

## What changed

- **`native_vector.cu`:** `native_q31_read_gate_credit` is replaced by two kernels.
  - `native_q31_read_gate_credit_partial`: one thread per (gate word, window). It sums that window's exact `__int128` conjugate products in row order and writes each sum as two 64-bit words, the low word and then `sum >> 64`.
  - `native_q31_read_gate_apply`: one thread per gate word. It rebuilds each partial as `high · 2^64 + low` (multiplying, because shifting a negative `__int128` left is undefined), adds them in window order, and writes `sat(gate − clamp(credit >> (step_shift + 1), ±2^32))` as before.
- **`build.rs`:** `NATIVE_VECTOR_ENTRIES` now lists seven entries. The PTX admission still rejects any float or matrix token.
- **`native_vector_device.rs`:** the step launches the two kernels. One new refusal is device-only: the step refuses when `2 · width · batch` does not fit `u32`, the grid's item count. The host rule has no such limit, and no test size reaches it.
- **`tests/native_vector_cuda.rs`:** `cuda_read_update_timing_at_a_run_size`, ignored by default. It runs at two target densities, checks device against host once, and prints the median step time over five steps. claude-392's three parity tests are unchanged.
- **The sheet** [`one-model-end-to-end-native-read-train-2026-11-15`](../../../../harness/prep/one-model-end-to-end-native-read-train-2026-11-15.md). It covers:
  - the arms: A-ext and A-4k from identity gates, with C0, C1, C2 (random targets), C2′ (other-fact targets), C3 (a warm start) and a replay;
  - the bar on held-out `chose_target` and on `R_ask` at ×16 from the native picks;
  - a ten-outcome map, its order, the base states, power and cost.

  Its line is `TEST FITNESS: FIT WITH CHANGES, applied`, and `prep-check` reads it as FIT.

- **`crates/uor-r4-integer/tests/sh_spin_probe.rs`**, one comment in Codex's file from #4013, reworded and its meaning kept. It now reads "no exponent anywhere".
  - The old comment named the banned token in order to say it is not used. The D23 interlock counted that, and four harness tests failed on every PR merged with main ("1 file above baseline, 0 -> 1").
  - Main's push CI runs only the docs jobs, so #4013 merged without this showing.
  - No code changed, and the baseline is not raised.

The host rule, the read, the session and the artifact are unchanged.

## Measurements

wick (RTX 4090, `sm_89`), `--release`. "Main's kernel" is the claude-392 copy on wick, `cmp`-equal to main's `native_vector.cu`, `native_vector_device.rs` and `build.rs`, run with this unit's test file. The figures are the second job's, at the committed test.

| Target rows (64 × 192, width 256) | Main's kernel, median of 5 | Window split, median of 5 |
|---|---|---|
| 256 (four a window: the run) | 17,772 µs | 16,272 µs |
| 12,224 (every row but the first) | 37,458 µs | 16,683 µs |

Device and host agreed at both densities, and every timed step equalled the checked one.

## Checks run at the head

- **Laptop:**
  - a stub-nvcc `cargo clippy -p uor-r4-discrete --features cuda --lib --test native_vector_cuda` shows nothing on the changed lines; the two existing warnings at test lines 297 and 298 are on main;
  - `native_vector_device.rs`, `build.rs` and the test file are rustfmt-clean;
  - `uor-harness prep-check` reads the sheet as FIT, complete.
- **wick:**
  - `cargo test --release --locked -p uor-r4-native-vector` passed;
  - `UOR_REQUIRE_CUDA=1 cargo test --release --locked -p uor-r4-discrete --features cuda --test native_vector_cuda -- --test-threads=1`: 10 passed, 1 ignored (the timing test);
  - the PTX holds both new entries.
- **Negative gates on wick.** Each sabotage was applied to the container's copy of `native_vector.cu` and confirmed with `diff`, and the sabotaged copy built. Each one fails both `cuda_read_update_matches_the_host_on_claude_391_fixtures` and `..._on_seeded_batches` ("losses or accept differ"):
  1. skip window 0's partial (the apply loop starts at 1);
  2. drop the high word (`credit += partial[0]` alone);
  3. add window 0's partial twice;
  4. drop each window's last row in the partial.

  After the restore, `cmp` matched the original and the full suite passed again (10 passed).
- **The sheet's FIT:** one fresh-context subagent at `1286973474`, given only the charter rule, the evidence files and the sheet. Its verdict was FIT WITH CHANGES. Required changes 1 to 5 and suggested changes 6 and 7 are applied at `fc0651a8d`.
- `uor-harness loop pr-check --local`: see the PR record.

## Skills used

`native-space-cycle`: the ORDER's five items. One operator definition, the host `train_pick_step`; its device twin is restructured, not redefined. claude-392's oracle (every field of `Q31ReadTrainStep`) is unchanged and green. The step is integer only.

## Still open

1. **The step's real cost.** Three `first_pick_read` passes, the before-pass not sharing its addresses with the read (claude-391 Still open 6), and a fresh stream copy on each call. These dominate at the run's density. A device-resident stream per batch would remove the copies. Not needed for the sheet's run, whose cost is under an hour.
2. **The run itself.** The next unit builds the sheet's three tools:
   - TRAIN dumps with targets files;
   - the native trainer example;
   - the probe's `--native-picks`.

   Its pre-run review covers what the FIT left unread. Then it runs the arms on wick.
3. **From claude-391:** Still open 3 (the weights' update), 4 (a step-size schedule) and 5 (one credit shift per batch) stand unchanged.

## Plain summary

The GPU step that teaches the read's gates now adds its credit window by window instead of one row at a time. It gives the same integers as before, and breaking the split in any of four ways makes a test fail. When every row carries a target, the step is twice as fast. At the next run's own density, four targets a window, it barely changes: that step's time goes to its three read passes.

The unit also wrote the plan for that run: train the read's gates from scratch in integers, on the states of a trunk whose float read already finds the asked fact, then check held-out picks and answers. An outside reviewer passed the plan after seven fixes.
