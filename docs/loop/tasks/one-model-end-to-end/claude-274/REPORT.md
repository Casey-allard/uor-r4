# REPORT: one-model-end-to-end, claude-274 (scoring and decoding at a set window)

Lab: claude. [ORDER](./ORDER.md). PR #3649, item #3407. Plan step: Casey's plan of 4 November, P2b, sheet unit 1 of 2. It gives P2b scoring and decoding at the window a model was trained at, and a first decode speed at long prompts.

**In one paragraph.** `train-in-space --score-rows --context N` now scores at an N-token window, at most the checkpoint's own.
- **What follows the window:**
  - every `--panel` split is cut into N-token windows; DEV and Fresh V2 keep 192;
  - `row_picks`' reply-table rows follow the forward's own window;
  - the serial decode, the batched decode and `reply_key_hits` slide over the model's own window (`config.context`).
- **What stays fixed:** scoring keeps its positions budget, 16 windows of 192 a forward, so a forward holds 16 windows at 192 and one at 2,048. This was amended in the ORDER before the GPU check, because a loaded checkpoint's forward keeps its intermediates.
- **What is refused:** off 192, the five flags whose TRAIN table or trace read 192-token windows are each refused by name.
- **New:** `--sample-opening last` decodes from each window's last reply opening, so a long window gives a long prompt. A scored file names its window only off 192.
- **The GPU check:** one RTX 5090 (`vast-59072880`, $0.975/h, 42 minutes up, about $0.69 in all) trained two 48-step checkpoints with claude-266's (b) recipe, at 192 and at 2,048. Then:
  - **G1:** the 192 checkpoint, scored by the base binary and by the head binary: all 10 files byte-identical, and the manifests equal apart from their seconds;
  - **G2:** the 2,048 checkpoint at `--context 2048 --sample-opening last`, batched and serial: all 10 files byte-identical, and the manifests equal;
  - **G3:** the same checkpoint at 192;
  - **memory:** every scoring pass's sampled peak was 5,324–5,356 MiB, the 2,048 ones included;
  - **the suite:** the crate's CUDA library suite passed on the same pod, 325 passed and 0 failed.

## Tokens per second first

Decode tok/s, per samples file of 16 greedy replies of up to 64 tokens, on one RTX 5090. Tokens decoded over the file's decode seconds; each pass ran once:

| checkpoint, window, opening | prompt tokens (min / mean / max) | greedy, memory on | greedy, memory off |
|---|---|---|---|
| c2048 at 2,048, `last` (G2, serial) | 1,770 / 1,963 / 2,034 | 11.7 | 12.6 |
| c2048 at 2,048, `last` (G2, batched) | the same | 10.5 | 11.3 |
| c2048 at 192, `last` (G3, batched) | 148 / 179 / 190 | 54.2 | 55.3 |
| c192 at 192, `first` (G1, head, batched) | 18 / 32 / 96 | 27.1 | 33.5 |
| c192 at 192, `first` (G1, base, batched) | the same | 38.1 | 42.9 |

- **On the same checkpoint**, a decode at prompts of about 1,960 tokens runs 10.5–12.6 tok/s. At prompts of about 180 tokens it runs 54–55 (greedy; the Flock arm 53.3). Paired by memory setting, that is 4.4–5.2× slower, for about 11× the prompt.
- **The decode recomputes its whole window each step.** At 1,960 tokens a token costs about 85 ms (87.5 s for 1,024 tokens). That is less than the positions alone would predict, so the ORDER's stop rule ("far longer than its positions predict") is not met.
- **DEV's short prompts (18–186 tokens) on the same checkpoint** decode at 32.2–43.1 tok/s with the 2,048 window (G2, serial; 23.9–30.8 batched) and 38.3–40.1 with the 192 window (G3, batched). No cost of the window setting shows beyond the pass-to-pass spread (What did not work, 5); one pass each settles nothing finer.
- These are the scorer's speeds on a 48-step model, not the served integer model's.

## What did not work

1. **The GPU job's first attempt was an instrument fault: EXIT 4 before any scoring.**
   - **What happened:** at `81139c20b`, both binaries were built into one `CARGO_TARGET_DIR`. Cargo hashes a path crate relative to its workspace root, so the base tree, fetched before the head build ended, counted as fresh. Its "build" took 0.14 s, and the base binary was the head's (both `6d1102d5…`).
   - **What caught it:** the job's own guard (`cmp` of the two binaries). The job's fresh pre-run review had passed this setup as safe.
   - **The rerun:** attempt 2, at `2206caecd`, builds each tree in its own fresh target folder. Its binaries differ: head `6891e913…8513`, base `2b893cb4…1f50`, the base built in 46 s.
   - **The record:** attempt 1's sealed folder (14 files) is stored beside attempt 2's and was not written into (L-002).
2. **`--sample-opening last` can cut a reference at the window's end** (the in-unit code review, medium; a gap in the ORDER, not a departure from it).
   - **Why:** a reference is the reply's scored targets left in the window. When the last opening sits near the window's end, the rest of the reply is in the next window.
   - **Measured on this panel:**
     - in G3 at 192, 7 of 16 long-panel references end without `<|eos|>` (for example a prompt of 190 tokens with a 3-token reference);
     - in G2 at 2,048, 0 of 16, where prompt plus reference reach at most 2,041;
     - every reference with `first` (G1) ends with `<|eos|>`.
   - **The fix belongs to claude-275,** whose per-query selector P2b scores recall with: it takes only openings whose reply ends inside the window. Here it is recorded in `last_openings`' doc and the bin's.
3. **`row_picks`' call site of the row stride is not pinned by a test** (the code review, low).
   - **What the test pins:** `the_reply_table_rows_follow_the_forward_s_window` tests `reply_table_rows` with an explicit window.
   - **Where the negative gate was applied:** inside that helper, not at the call that passes `window_time(ids.len(), windows.len())`.
   - **Why G2 cannot catch it:** at 2,048 a forward holds one window, so the stride never matters there. It matters only at windows of 193–1,536, where more than one window shares a forward.
   - **Carried to claude-275.**
4. **The batched decode was no faster at 2,048.**
   - **Why:** its budget, kept at 3,072 positions, fits one long prompt a forward. A prompt of 2,029 tokens reaches 2,048 with its reply.
   - **On the long panel:** the one equal-length pair was split into two single decodes.
   - **What was batched together:** only DEV's one equal-length pair (two 186-token prompts), in each of three files: greedy with the memory on and off, and the Flock rank rule. The Flock arms take the batched path too: `can_batch_decode` gates on the objective, scope, forced opening and served key, not on the rule.
   - **Next:** batching long prompts needs a larger positions budget, measured first (P3).
5. **Two passes of the same decode work differ by up to about 1.45× in decode time, so no speed difference between passes is established.**
   - **G1:** base and head decoded byte-identical files at 192, with every changed decode expression reducing to the old one there. Per file the head took −13% to +45% the base's time (decode 127.1 s against 162.3 s in all), with lower GPU use (23.2% mean against 29.0%).
   - **G2:** at 2,048 every long-panel group held one prompt, so both passes decoded the long arms one prompt a forward, the batched pass through its batched loop. The batched pass was slower on each long arm (10.5 against 11.7, 11.0 against 12.3, 11.3 against 12.6 tok/s); with one pass each, noise and the batched loop's own cost are not separated.
   - **What the run supports:** "no batched arm was faster", not a ratio. One file, DEV with the memory on, ran 23.9 batched against 43.1 serial, past the spread above; with one pass each, it is not established.
   - P3's timings need repeats.
6. **At the first GPU pin, three of Done-when 3's five flags were refused off 192 only indirectly.**
   - **Which:** `--served-key` and `--keyed-decode` fail on "needs `--opening-keys 1`". `--decision-force` fails only after the checkpoint loads.
   - **The fix, at `2206caecd`:** `window_bound_flags` names each one asked for, before the checkpoint loads. Its test `each_192_bound_flag_is_refused_by_name_off_192` FAILS with the earlier condition put back, and the file was restored to the same sha256 (`d7cee0bd…`).
7. **The memory figures are `nvidia-smi`'s,** sampled about every 140 ms over each pass, not the pool's own peak. Each is the highest sample, so a lower bound on the true peak; nothing between samples is seen.
8. **claude-270's served-mode parity check is covered only thinly.**
   - **What ran:** both checkpoints were trained with `--train-as-served 1`, and the scorer read that from the run config, so every pass scored in served mode. G2 is a batched-against-serial parity at current code in served mode.
   - **Why it is thin:** only three groups were decoded together (DEV's pair, in three files), and all three matched byte for byte.

## What changed

- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - `score_rows` takes `context` and `last_opening`:
    - `training_context` checks the window;
    - `window_bound_flags` refuses off 192, by name, `--opening-keys`, `--served-key`, `--keyed-decode`, `--decision-trace` and `--decision-force`, before the checkpoint loads;
    - a window past the checkpoint's own is refused by name, after load;
    - every `--panel` split is cut at `context`; DEV, Fresh V2 and TRAIN's opening points stay at 192;
  - `reply_table_rows` takes the forward's own window; `row_picks` passes `window_time(ids.len(), windows.len())`;
  - `windows_per_forward`: `SHEET_EVAL_BATCH × SHEET_CONTEXT` positions a forward, clamped to 1–16 windows. `row_picks` chunks by it;
  - `exact_length_batches(prompts, reply_tokens, context)`: the budget stays 16 × 192 positions; a group's reach is capped at `context`;
  - the serial decode, the batched decode and `reply_key_hits` slide over `model.config.context`;
  - `last_openings`: each window with a reply opening at least 16 tokens in is cut to a clone whose scored lists start at its last such opening. Then `sample_windows`' stride takes them;
  - `ScoredFile.context`, skipped at 192 (`is_sheet_context`), set at every construction site.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:**
  - `--sample-opening first|last`, refused outside `--score-rows`;
  - `context_applies` accepts scoring as well as training and `--grad-norms 1`;
  - the module doc;
  - the test `a_window_other_than_192_trains_or_scores`.
- **Seven new library host tests** (no tensor maths, per D24/D29):
  - `the_hit_rate_reads_the_window_the_decoder_sees`: 299 keys at 2,048 against 191 at 192; 7 at 8;
  - `the_reply_table_rows_follow_the_forward_s_window`: rows 4, 5, 2,052 and 2,053 for two windows at 2,048;
  - `a_scoring_forward_holds_sixteen_windows_of_192_positions`: 16 at 192 and at 96, 3 at 1,024, 1 at 2,048 and 4,096;
  - `exact_length_batches_cap_the_reach_at_the_model_s_window`: forty 1,000-token prompts go 16 / 16 / 8 at 192 and twenty groups of 2 at 2,048;
  - `the_last_opening_starts_each_window_at_its_latest_reply`: scored offsets 28–30, prompt `tokens[..29]`, reference 400–402;
  - `a_scored_file_names_its_window_only_off_192`;
  - `each_192_bound_flag_is_refused_by_name_off_192`.

## Measurements

On the RTX 5090 (32,607 MiB, driver 595.71.05), release builds at `2206caecd` (head, `6891e913…8513`) and `daa0748dd` (base, `2b893cb4…1f50`).
- **Data:** pinned TRAIN `mw-balp/train` (`3dee858f…`, mask `b5e7352d…`) and DEV `ft/dev`, checked against claude-266's sums.
- **The panel:** 64 chained conversations from claude-271's generator (`m-world-recall-corpus conversations=64 chain=2048 context=2048 protocol=2 seed=274`): 125,651 tokens, the longest 2,048, tokens `6af58bb2…`, mask `6969c6b1…`. An instrument only, NOT held out.
- **Scoring:** every pass ran `--samples 16 --reply-tokens 64 --retain-rates 1 --panel long=… --seed 1`.

| pass | binary, checkpoint | flags | exit, seconds | GPU memory peak | files against its pair |
|---|---|---|---|---|---|
| train c192 | head | `--context 192`, 48 steps | 0 | 8,044 MiB | |
| train c2048 | head | `--context 2048`, 48 steps | 0 | 19,916 MiB | |
| g1-base | base, c192 | | 0, 137 s | 5,324 MiB | G1: 10 SAME, 0 not; manifests SAME |
| g1-head | head, c192 | | 0, 172 s | 5,356 MiB | |
| g2-batched | head, c2048 | `--context 2048 --sample-opening last` | 0, 409 s | 5,356 MiB | G2: 10 SAME, 0 not; manifests SAME |
| g2-serial | head, c2048 | the same, `--batched-decode 0` | 0, 344 s | 5,356 MiB | |
| g3-c192 | head, c2048 | `--context 192 --sample-opening last` | 0, 143 s | 5,356 MiB | (no pair) |

- **Rows:** DEV has 14,287 rows in every pass; the panel has 39,992 at both 2,048 and 192. The rows phase and load take about 17–18 s at 2,048 against about 10 s at 192. The hit counts are a 48-step model's (13–24 nearest hits) and claim nothing.
- **The window check:** in G2, the panel's files carry `"context": 2048` and DEV's carry none. In G1 and G3 no file carries one. The saved models' own configs read 192 and 2,048.
- **Prompts:** with `first` (G1), the panel's prompts at 192 are 18–96 tokens (mean 32). With `last`, they are 148–190 at 192 and 1,770–2,034 at 2,048 (mean 1,963).
- **The suite:** the crate's CUDA library suite at `2206caecd`: 325 passed, 0 failed, 8 ignored. All eight of this unit's named tests are `ok`, the CUDA-only `exact_length_batches_match_one_at_a_time_decodes` among them.

## What this shows, and what it does not

- **It shows:**
  - at 192, the head scores and decodes exactly as the base did, in served mode, on every file and manifest line;
  - a model trained at 2,048 can be scored and decoded at 2,048, with its batched and serial decodes byte-identical;
  - on one 5090, scoring at 2,048 had the same sampled GPU memory peak as at 192 (5,356 MiB, a lower bound; What did not work, 7), as the positions budget would predict;
  - a decode at prompts of about 1,960 tokens runs at 10.5–12.6 tok/s, against 54–55 (greedy) at about 180 on the same checkpoint;
  - `last` gives long prompts at a long window.
- **It does not show:**
  - any learning or recall across turns: the checkpoints are 48 steps on short TRAIN, and the panel is an instrument, not held out;
  - a speed difference between base and head, or between batched and serial (What did not work, 5);
  - parity at windows of 193–1,536, where several windows share a forward (What did not work, 3);
  - whole references with `last` at short windows (What did not work, 2);
  - the integer model's speed at long windows.

## Next

1. **claude-275, P2b's sheet unit 2:**
   - the long held-out panel with a query sidecar, and a per-query selector that takes only openings whose reply ends inside the window (What did not work, 2);
   - the long TRAIN corpus via `exclude=`, and the census over a named TRAIN;
   - a test of `row_picks`' row stride at a window of 193–1,536, or the risk kept on record (What did not work, 3);
   - then the sheet, its FIT and its pre-run review.
2. **The P2b run:**
   - train at 2,048 × 4 on the long corpus (claude-273: about 0.60 s a step) and score recall across turns against a no-memory control (L-010);
   - at 11–13 tok/s, 16 replies of 64 tokens take about 80–90 s a file at 2,048.
3. **P3:**
   - batch long prompts: a larger positions budget, measured first (What did not work, 4);
   - repeat every timing (What did not work, 5).

## Checks run at the head

- **On the laptop, at the head:** `cargo test -p uor-r4-float-train --no-fail-fast` gives 0 failed:
  - the library 237 passed (230 at the base, plus seven new tests);
  - `fbs-snapshot` 7, `spiralcore-fbs-sheet` 4, `train-in-space` 6, the dependency check 2 and the kernel controls 12.
- **The stub-`nvcc` type-check** (`cargo check --features cuda --all-targets`) passes at the head. `cargo clippy` with the same flags shows no warning on a changed line. Its two `too_many_arguments` warnings (`decode_speed`, `inspect_run`) predate this unit.
- **Negative gates**, each fix set aside and then restored:
  - the row stride inside `reply_table_rows` back to 192: `the_reply_table_rows_follow_the_forward_s_window` FAILS (196 and 197 for 2,052 and 2,053);
  - `reply_key_hits`' window back to 192: `the_hit_rate_reads_the_window_the_decoder_sees` FAILS (191 for 299);
  - `window_bound_flags` back to the earlier condition: `each_192_bound_flag_is_refused_by_name_off_192` FAILS.
- **On the pod, at `2206caecd`:** the CUDA library suite, the two runs, G1, G2, G3, the window check and the config check, all exit 0 (job `EXIT 0`, 07:33:38Z–08:00:45Z).
- **The head and the GPU pin:** the head differs from `2206caecd` only in doc comments (`last_openings`, the bin's `--sample-opening`, two test doc comments) and this unit's docs.
- **The seal:** attempt 2's 176 files (`SHA256SUMS.files`, sha256 `a6c8712b…bfc0`) and attempt 1's 14 are all OK on the laptop. The tar's sha256 on the laptop matches the pod's.
- **The store:**
  - `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-score-window-2026-11-04/runs` holds 198 files: both attempts' folders without their fetched inputs, and the step-48 checkpoints' sha256 list. The checkpoints themselves (205 MB each) are not stored.
  - `g2.txt` and `SHA256SUMS.files` read back byte-identical, and the laptop lists all 198. The panel is at `…/panel-274` (7 files).
- **The pod:** `vast-59072880` was released and deleted at 08:03:07Z, after the read-back.
- **`rustfmt`:** this crate is not rustfmt-clean at its base. The new lines follow each file's own style.
- **`loop gates --full`**, `loop report --check` and `loop pr-check --local`: run at the head before review.
- **Reviews:**
  - **the GPU job's own fresh pre-run review:** PASS, with `STALL=40` applied. It missed the shared target folder (What did not work, 1);
  - **a fresh-context code review** of `daa0748dd..2206caecd`: APPROVE with low items, and no path where 192 can drift. Its items are What did not work 2 and 3, a check that `opening_points` keys every later opening (it does), and a merged test doc comment (fixed);
  - **the claims review** of this REPORT is recorded on PR #3649 by `loop review`.

## Skills used

None applied.

## Still open

1. **Whole references for the last or query opening:** claude-275 (What did not work, 2).
2. **`row_picks`' row stride at windows of 193–1,536:** claude-275 (What did not work, 3).
3. **Long-prompt batching, and repeated timings:** P3 (What did not work, 4 and 5).
4. **Served-mode parity with larger batches** (What did not work, 8).
5. **The exported window** (P4, #3471): `fbs_export.rs:293` exports `config.context`, while the integer point key is pinned at 192.

## Plain summary

A model can now be tested at the same length of conversation it was trained on, instead of always in pieces of 192 word pieces. At the old length, the new code gives exactly the same answers as the old code, file for file.

On one gaming GPU, a model trained on 2,048 word pieces was tested at 2,048. Its fast and slow answer-writing paths gave identical answers, and its measured memory peak was the same as at 192. With about 2,000 word pieces of conversation before each answer, it wrote about 12 word pieces a second, against about 54 with 180 before it.

The model here is a tiny 48-step test, so this says nothing yet about whether it remembers across a long conversation. The next unit builds the long test conversations and the long training data for that run.

References #3407.
