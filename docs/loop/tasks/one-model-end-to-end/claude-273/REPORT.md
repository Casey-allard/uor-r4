# REPORT: one-model-end-to-end, claude-273 (training at a set window)

Lab: claude. [ORDER](./ORDER.md). PR #3646, item #3407. Plan step: Casey's plan of 4 November, P2 unit 3 ("the window as a setting"). It also gives P2b its first number: a training step at 2,048 against 192.

**In one paragraph.** `train-in-space --context N` now trains at an N-token window, from 2 to 4,096; 192 stays the default.
- **At 192:** at the head, a run without the flag and one with `--context 192` write the same run `config.json` apart from `batch`, because the field is left out at 192. That this matches the base rests on that serde skip and its negative gate: no training at 192 was compared with the base.
- **The training path uses the window's own length** everywhere:
  - TRAIN's cut, and the model's `config.context`, set through the new `StackModel::set_context`;
  - `assemble`, `trained_points`, `cut_census` and `gradient_norms`;
  - the turn memory's rows and its whole-window check, and the throughput line.
- **What stays at 192:** DEV's evaluation during training, `--score-rows` scoring and every decode. Every mode but training and `--grad-norms 1` now refuses another window by name.
- **A new guard:** `choose_sorted` refuses any Hamming-read tensor of 2³² elements or more, where candle 0.9.2's 32-bit launch size would wrap.
- **The GPU check:** one RTX 5090 (`vast-59067333`, $0.975/h, up from about 06:24Z, $0.18 spent by the job's end at 06:34Z, then saved and destroyed) ran 48 steps of claude-266's (b) recipe at each of 192 × 4, 2,048 × 4 and 192 × 16.
  - A step at 2,048 × 4 takes 0.603 s, against 0.104 s at 192 × 4: 5.8× the time for 10.7× the window positions.
  - Its sampled GPU memory peak is 20,012 MiB of the card's 32,607.
  - Every loss was finite. The crate's CUDA library suite passed on the same pod: 318 passed, 0 failed.

## Tokens per second first

No decode was run, so there is no decode tok/s. Training throughput, in window positions per second over steps 17–48 (batch × window × 32 steps over their summed step seconds):
- 7,373 at 192 × 4;
- 13,594 at 2,048 × 4;
- 22,767 at 192 × 16, claude-266's own batch.

A 2,048 window trains 1.84× as many positions a second as a 192 window at the same batch. These are window positions, not text tokens: on this TRAIN each 2,048 window is mostly padding (What did not work, 2).

## What did not work

1. **The memory figure is `nvidia-smi`'s, not the pool's own peak.** The ORDER asked for "the memory pool's peak", but `run_long` reads no pool attribute (claude-272 read `CU_MEMPOOL_ATTR_USED_MEM_HIGH` inside a test). The job sampled `memory.used` (140 ms apart on average, at most 271 ms) and split each run at its step-0 line.
   - **Before the step-0 line** the process has built the model and the turn-memory tables and run one DEV pass. That span tops out at 5,612–5,676 MiB in all three runs.
   - **After it**, training lifts the maximum to 8,076, 20,012 and 11,628 MiB. So training, not DEV, sets each peak.
   - **At each sample it is an upper bound on the pool's bytes in use:** it includes the CUDA context and any memory the pool keeps cached. Between samples it bounds nothing: a spike shorter than the 271 ms largest gap could be missed.
2. **TRAIN at 2,048 is mostly padding, so the loss here is not a learning result.** `mw-balp/train`'s documents run to about 258 tokens (claude-271's REPORT). `build_document_windows` puts one document in each window and pads it with BOS to the window. So each 2,048 window holds at most about 13% text.
   - The run's own epoch counts show it: 59,937 windows at 2,048 against 68,218 at 192, where documents past 192 tokens split into two windows.
   - **What the step time does measure:** a 2,048-position window's compute, because the read and the trunk run over every position.
   - **What it does not show:** that a model learns anything at 2,048. Between steps 16 and 48 no run's training loss moved by more than about 0.0003 (from 1.0044 to 1.00410–1.00435). That needs the long corpus, which is P2b's.
3. **At the GPU pin, `--context` was ignored by `--score-rows` and every other non-training mode** (found by the scout for the next unit). A model scored with `--context 2048` would have been scored on 192-token windows without a word.
   - **The fix:** at `aa46d1273` the bin refuses the flag there by name (`context_applies`), as `--train-as-served` and `--served-forward` already are. The new test `a_window_other_than_192_is_a_training_setting` FAILS when the refusal is set aside.
   - **The pod ran at `7bc88fc31`.** The head differs from it only in that check in `train-in-space.rs`, a check that training mode does not trigger; the pod's binary predates it.
4. **`set_context`'s shape check cannot fire on today's trunk.** `validate_native_config` admits only `r` layers, so no native model has a weight sized by its window (the ORDER's cancel probe, 3). The check guards a future layer kind. No test trips it, and only the accept path ran (on the GPU, at 2,048).
5. **The launch guard did not fire on the GPU.** At 2,048 × 4 the largest checked tensors are each difference chunk, at 4 × 128 × 2,048 × 64 = 2²⁶ elements under claude-272's 256 MiB budget, and the `(batch · time, time)` tensors, at 2²⁴. The refusal at 2³² rests on the host test of its arithmetic.
6. **The job script's first draft had three faults,** found by its fresh pre-run review (PASS) and fixed before it shipped:
   - its memory peak could have been DEV's, fixed by timestamping each log line and splitting at step 0;
   - the mean step time has to be differenced, `(t₄₈ − t₁₆) / 32`, to leave out step 0's diagnostics and the first launches' PTX compile;
   - each data fetch overwrote the last one's log.
7. **The P2 bound (2–3 units) is passed.** This unit covers training at a window only. Scoring and decoding at a window, and the long TRAIN corpus and panel, move to P2b's sheet unit. Logged for Casey on #2028 (comment 6624171117).

## What changed

- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:**
  - the `--context N` flag, checked by `in_space_long::training_context` (2 to 4,096, otherwise refused with the value named);
  - `context_applies`, which refuses a window other than 192 outside training and `--grad-norms 1`;
  - the usage line and its doc paragraph;
  - one test.
- **`crates/uor-r4-float-train/src/geometric_stack_native.rs`:** `StackModel::set_context` sets `config.context` after `validate_native_config`. It refuses the change if any weight's shape would differ (`config.shapes()`). The weights and the init RNG do not depend on the window, so a model built at 192 and set to 2,048 holds the same weights.
- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - `LongConfig.context`, skipped in `config.json` at `SHEET_CONTEXT` (`is_sheet_context`);
  - `training_context`, and `build_long_model`: the FBS model at the run's seed and width, set to the run's window;
  - `load_train` and `start_memory` cut TRAIN at the window, and `load_train` checks each window's length;
  - `assemble` records each window's start in the flat ids, so a scored row is `start + offset` (before, `b · 192 + offset`);
  - `trained_points` and `gradient_norms` take the forward's window from `window_time(ids.len(), batch)`, which refuses a batch of 0, no ids, or ids that do not divide into the batch;
  - `cut_census` and `window_input_mask` take the window's length;
  - `run_long` builds its model at the window and uses it for the turn memory's rows (two sites), its whole-window check and its throughput line.
- **`crates/uor-r4-float-train/src/flock_read_hamming.rs`:** `launch_fits`, a checked `u64` product under 2³². `choose_sorted_in` applies it to the `(batch · time, time)` distance and to each `(batch, chunk, time, width)` difference, and returns an error before any launch.
- **Six host tests** (no tensor maths, per D24/D29):
  - `a_config_writes_its_window_only_off_the_sheets`: no `"context"` at 192, and `"context": 2048` at 2,048;
  - `a_training_window_is_two_to_four_thousand_ninety_six_tokens`: 2, 192, 2,048 and 4,096 accepted; 0, 1 and 4,097 refused;
  - `a_forward_window_is_its_ids_over_its_batch`;
  - `assembled_rows_follow_the_window`: documents of 300, 40 and 2,100 tokens give 14 windows at 192 and 4 at 2,048. Every row is `b · context + offset`, with input `tokens[p − 1]` and target `tokens[p]`;
  - `a_launch_fits_below_two_to_the_thirty_two_elements`: 65,535 × 65,537 fits and 65,536 × 65,536 does not; 16 × 2,048 × 2,048 × 64 does not;
  - `a_window_other_than_192_is_a_training_setting` (the bin).

## Measurements

On the RTX 5090 (32,607 MiB, driver 595.71.05), release build at `7bc88fc31`, bin sha256 `0496a848…af44`. Pinned TRAIN `mw-balp/train` (`3dee858f…`, mask `b5e7352d…`) and DEV `ft/dev` (`0919e90f…`), checked against claude-266's sums. Each run is 48 steps with DEV every 16, seed 1, and claude-266's (b) recipe:
- `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0`;
- `--turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500`;
- `--train-as-served 1 --width 256 --hamming-read 4`.

| run | window × batch | mean step, steps 17–48 | positions / s | GPU memory peak, training span | build + DEV span | training loss at 16 / 32 / 48 |
|---|---|---|---|---|---|---|
| c192-b4 (`--context 192`) | 192 × 4 | 0.1042 s | 7,373 | 8,076 MiB | 5,644 MiB | 1.00445 / 1.00435 / 1.00432 |
| c2048-b4 (`--context 2048`) | 2,048 × 4 | 0.6026 s | 13,594 | 20,012 MiB | 5,676 MiB | 1.00444 / 1.00441 / 1.00435 |
| c192-b16 (no flag) | 192 × 16 | 0.1349 s | 22,767 | 11,628 MiB | 5,612 MiB | 1.00440 / 1.00418 / 1.00410 |

- **The configs.**
  - c192-b4 and c192-b16 write no `context` in their run `config.json`. The two files are equal apart from `batch`, so the flag at 192 and no flag write the same file.
  - c2048-b4's run config has `"context": 2048`.
  - Each saved model's own `config.json` (`models/step-000048`) carries its window: 192, 2,048 and 192.
- **DEV**, at 192 in every run: `dev_loss` 2.794–2.820, top-1 at most 10 of 14,287 rows. DEV's top-1 moved by no more than 9 rows; no noise band was measured, and no learning is claimed.
- **The ratio the ORDER's stop rule reads:** 0.6026 / 0.1042 = 5.78×, under its ~20×. No step at 2,048 failed.

## What this shows, and what it does not

- **It shows:**
  - a model trains at a 2,048 window on one 5090 at batch 4: 0.60 s a step, with a sampled peak of 20,012 MiB (19.5 GiB) of the card's 31.8 GiB;
  - the read and the trunk at 2,048 cost 5.8× a 192 step, not the 10.7× of the positions, which suggests the GPU is under-used at 192 × 4;
  - at 192, the flag and no flag write the same run `config.json` at the head, apart from `batch`. That this matches the base rests on the serde skip, the row-layout test and their negative gates; no training at 192 was compared with the base.
- **It does not show:**
  - that a model learns, or recalls across turns, at 2,048 (What did not work, 2);
  - batch 8 or more at 2,048, which is untested;
  - the pool's own peak (What did not work, 1);
  - any scoring or decode at a window.
- **A projection for P2b, not a measurement:** P2b's 8,192 steps at 2,048 × 4 would take about 8,192 × 0.6026 ≈ 4,940 s (82 min) of training steps, plus its DEV passes.

## Next

1. **P2b's sheet unit, now 2 units:**
   - scoring and decoding at a set window;
   - an opening selector at a chained conversation's end queries. Today `sample_windows` takes each window's first opening, which in a 2,048-token conversation is never a query;
   - the long TRAIN corpus and a long held-out panel from claude-271's generator, named by sha256, with the exclusion drop rate checked;
   - then the sheet, its FIT and its pre-run review.

   P2b's run trains at 2,048 × 4, on this unit's numbers.
2. **P4 (#3471):** reconcile the exported window. `fbs_export.rs:293` exports `config.context`, while the integer session's point key is pinned at 192.

## Checks run at the head

- **On the laptop**, at `aa46d1273`: `cargo test -p uor-r4-float-train --no-fail-fast` gives 0 failed:
  - the library 230 passed (225 at the base, plus five new tests);
  - `fbs-snapshot` 7, `spiralcore-fbs-sheet` 4, `train-in-space` 6 (5 plus the new one);
  - the dependency check 2 and the kernel controls 12.
- **Negative gates**, each fix set aside and then restored:
  - `assemble`'s window start back to `b · SHEET_CONTEXT`: `assembled_rows_follow_the_window` FAILS at `in_space_long.rs:11502`;
  - the serde skip removed: `a_config_writes_its_window_only_off_the_sheets` FAILS;
  - `context_applies`'s refusal removed: `a_window_other_than_192_is_a_training_setting` FAILS at `train-in-space.rs:1313`, and the restored run passes all six.
- **The stub-`nvcc` type-check**, at the head: `cargo check --offline -p uor-r4-float-train --features cuda --all-targets` passes. `cargo clippy` with the same flags shows no warning on a changed line. Its two `too_many_arguments` warnings (`decode_speed`, `inspect_run`) predate this unit.
- **On the pod**, at `7bc88fc31`:
  - the crate's CUDA library suite: 318 passed, 0 failed, 8 ignored, with the five new library tests each named `ok`;
  - the three runs exit 0, and the six config checks pass.
- **The seal:** 67 files (`SHA256SUMS.files`, sha256 `b824bac7…4ff9`), all OK on the laptop.
- **The store:** `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-training-window-2026-11-04` (71 files: the 67 sealed ones plus the seal, its error file, `DONE` and `EXIT`). `SHA256SUMS.files` and `c2048-b4/progress.jsonl` read back byte-identical, and the laptop lists the folder.
- **`rustfmt`:** this crate is not rustfmt-clean at its base. The new lines follow each file's own style.
- **`loop gates --full`**, `loop report --check` and `loop pr-check --local`: run at the head before review.
- **Review:** a fresh-context review of the diff and this REPORT, recorded on PR #3646 by `loop review`. The GPU job had its own fresh pre-run review: PASS, with three fixes applied before it shipped (What did not work, 6).

## Skills used

None applied.

## Still open

1. **Scoring and decoding at a set window,** and the long TRAIN corpus and held-out panel: P2b's sheet unit (What did not work, 7).
2. **The exported window** (P4): `fbs_export.rs:293` exports `config.context`, so a model trained at 2,048 exports a 2,048 window while the integer point key is pinned at 192.
3. **The pool's own peak** is not recorded by `run_long`. If P2b's batch choice needs it, the job or `run_long` should read `CU_MEMPOOL_ATTR_USED_MEM_HIGH`.
4. **`set_context`'s refusal is unreachable on today's trunk** (What did not work, 4).
5. **The ignored timing test `the_read_at_long_windows`** now meets the new guard at 2,048 × 16 unchunked, where candle used to fail with `CUDA_ERROR_INVALID_VALUE`. It still prints "does not fit". It was not run in this unit.
6. **claude-270's served-mode parity check** is still owed, by the next GPU unit that scores a served checkpoint.

## Plain summary

The model used to train on pieces of conversation 192 word pieces long, because that length was fixed in the code. Now it can be set when training starts, anywhere from 2 to 4,096, and leaving it at 192 writes the same settings as before.

On one gaming GPU, a training step on 2,048 word pieces took 0.6 seconds and ran in about 20 GB of the card's 32 GB, four pieces of conversation at a time. That is about six times as long as a step on 192 for nearly eleven times as many positions, which suggests the card is under-used at the short setting.

This does not show that the model learns to remember across a long conversation. The test data is short, so the long pieces were mostly empty filler. The next units build long practice conversations and a way to score at the long length, and then train on them.

References #3407.
