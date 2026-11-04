# ORDER: one-model-end-to-end, claude-274 (scoring and decoding at a set window)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `daa0748dd`. Draft PR #3649.

## Goal

claude-273 made the training window a setting: `train-in-space --context N` trains at N tokens. Scoring still cannot follow it.
- `--score-rows` cuts every split into 192-token windows (`windows_of`, `in_space_long.rs:6517`).
- Every decode slides over the last 192 tokens.
- `--context` is refused with `--score-rows` (`context_applies`, `train-in-space.rs:1247`).

So a model trained at 2,048 can be trained but not scored or decoded at 2,048. P2b's run cannot be judged until it can.

This unit makes `--score-rows` score at `--context N`:
- every `--panel` split is cut into N-token windows; DEV and Fresh V2 keep 192;
- every row and decode site reads its own window's length;
- decodes slide over the model's own window;
- a `--sample-opening last` selector decodes from each window's last reply opening, so a long window gives a long prompt.

At 192 nothing changes, and a GPU check compares base and head byte for byte, which claude-273 could not.

- **Plan step:** Casey's 4 November plan, P2b, sheet unit 1 of 2. It provides scoring and decoding at a window, the first of the three things P2b's sheet needs.
- **What it changes in the model:** the window the model is scored and decoded at. The float decode then reads as far back as the model was trained to.
- **No prereg** (§4.5): the GPU part is a finite instrument check, byte equality and a parity, with no model result claimed.

**Moved to sheet unit 2 (claude-275):**
- the long held-out panel with a query sidecar, and a selector at each query's opening;
- the long TRAIN corpus with `exclude=`, and the census over a named TRAIN;
- then the sheet, its FIT and its pre-run review.

## Cancel probe (run first)

1. Does any flag already set a scoring window?
2. Which scoring and decode sites read 192?
3. Would decoding at the model's own window change any checkpoint trained at 192, or any test?
4. Which opening does `sample_windows` decode from in a chained window?
5. Does any open order hold these files? The claim's overlap check passed.

**Result, at `daa0748dd`: not cancelled.** The finding is about the instrument, not the mechanism: scoring's window is a constant.
1. No. `context_applies` refuses `--context` outside training and `--grad-norms 1`, and `score_rows` takes no window.
2. At `in_space_long.rs`:
   - the split windows: `windows_of` (:6518) and `load_dev` (:5865);
   - `row_picks`' reply-table rows, `row.window * SHEET_CONTEXT + row.offset` (:1371). This is a defect at another window: since claude-273, `assemble` lays rows out at `b · window + offset`, so at 2,048 the reply table would move the wrong rows without a word;
   - the serial decode's slide (:5375, :5386);
   - the batched decode's budget and clamp (:5505, :5516) and its slide (:5639, :5640);
   - `reply_key_hits` (:5800);
   - the opening keys: `keyed_openings` (:2643), with TRAIN's opening points cut at 192 (:6631);
   - the decision trace and force: `batch_force` (:1587), `record` (:1672), `traced_rows` (:1813), `check_forced` (:1828, :1846).
3. No. Every checkpoint before claude-273 has `config.context = 192` (`hamilton_sheet.rs:2205`). Every decode test builds its model with `build_fbs_model`, at 192.
4. The first. `sample_windows` (:4933) takes windows whose first scored target opens a reply. In a chained conversation that is the first reply, after a short first turn, so a decode at a long window never gets a long prompt.
5. The claim passed with no overlap.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-274/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **Scoring at a window.**
   - `--context N` is accepted with `--score-rows`, and still refused in every other mode but training and `--grad-norms 1`.
   - Every `--panel` split is cut into N-token windows. DEV and Fresh V2 keep 192.
   - An N above the checkpoint's own window is refused by name.
2. **Each site reads its own window.**
   - `row_picks`' reply-table rows take the forward's own window length.
   - Decodes slide over the model's own window (`config.context`): the serial decode, the batched decode, and `reply_key_hits`.
   - The batched decode's budget is `SHEET_EVAL_BATCH ×` that window.
   - **Amended before the GPU check:** the batched decode's budget, and the row forward's, stay at `SHEET_EVAL_BATCH × 192` positions; only a group's reach is capped at the model's window. A loaded checkpoint's weights are variables (`StackModel::load`, `Var::from_tensor`), so every forward keeps its intermediates and its memory grows with its positions. Scoring was measured only at 16 × 192 = 3,072 positions a forward, about 5.2 GiB (claude-270). 16 × 2,048 = 32,768 is 10.7 times that. So the row forward takes 16 windows at 192, as before, and one at 2,048.
3. **Refused at N other than 192, by name:**
   - `--decision-trace` and `--decision-force` (Q5 is parked);
   - `--opening-keys`, `--served-key` and `--keyed-decode`, whose TRAIN table is cut at 192.
4. **`--sample-opening first|last`, default `first`.** `last` decodes from each eligible window's last reply opening that has at least 16 prompt tokens.
5. **Scored files carry their window.** Each `ScoredFile` records `context`, left out at 192, so a 192 manifest is as before.
6. **Host tests on the laptop.** No tensor maths, per D24/D29.
   - The reply-table rows at a 2,048 window are `window · 2,048 + offset`.
   - `exact_length_batches` at 192 is as before, and at 2,048 its budget is 16 × 2,048 (amended: its budget stays 16 × 192 positions, and a group's reach is capped at 2,048; a row forward holds 16 windows at 192 and one at 2,048).
   - `reply_key_hits` reads the window the decoder sees.
   - The last-opening selector's windows start at the last opening, and their prompt, reference and position follow it.
   - `context_applies` accepts scoring.
   - A `ScoredFile` at 192 writes no `context`.
   - **Negative gates:** set aside the row stride, and its test fails; set aside `reply_key_hits`' window, and its test fails.
7. **One GPU check.** The smallest RTX 5090, one lease, under an hour. It trains two 48-step checkpoints with claude-266's (b) recipe, one at 192 and one at 2,048. A small chained panel from claude-271's generator serves only as an instrument; it is not held out. Then:
   - **G1:** the 192 checkpoint is scored with DEV, the panel and `--samples 16` by the base binary and by the head binary. Every output file is byte-identical, and the manifest lines are equal apart from their seconds.
   - **G2:** the 2,048 checkpoint is scored at `--context 2048 --sample-opening last`, batched and serial. Its samples and rows files are byte-identical between the two.
   - **G3:** the same checkpoint at `--context 192`, for rows and decode tok/s against prompt length beside G2.
   - It records each pass's sampled GPU memory peak. On the same pod, the crate's CUDA library suite passes.
8. **A REPORT, negatives first,** and one notebook row.

## Stop and decide

- **G1 differs:** something at 192 changed, which is a bug. Fix it; never loosen a check.
- **G2's parity fails:** classify it. A fault in the batched decode at long lengths is fixed here if it is this unit's code. Otherwise it is recorded as P2b's first blocker, and P2b scores serially.
- **A decode at 2,048 does not fit,** or takes far longer than its positions predict: record it. A smaller batch at long windows is then P2b's decision.
