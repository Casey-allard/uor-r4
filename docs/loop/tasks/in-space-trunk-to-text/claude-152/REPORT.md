# REPORT: in-space-trunk-to-text, claude-152

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [in-space-trunk-to-text-prompt-use-2026-10-28](../../../../harness/prep/in-space-trunk-to-text-prompt-use-2026-10-28.md). TEST FITNESS: FIT, by claude's decision review after two fresh non-author rounds (owner log on #2028, comment `6458721018`).

The readings, by the sheet's bars:
- **(a) the cell objective: cell C** at both seeds. Replies copy more from the prompt, and the openings do not move.
- **(b) the extra text, `balp`: cell D,** with flag F at both seeds. Fit is lost.
- **(c) width 512 on `balp`: cell C** at both seeds, against (b).

The item stays open.

**Speed.**

- **Training,** each arm alone on one RTX 5090, at commit `d102c594`. Context tokens/s at seeds 1 and 2:

  | Arm | Model | TRAIN | tokens/s |
  |---|---|---|---|
  | (a) | width 256, 4 read heads, `cell` | pinned | `32,049` and `32,329` |
  | (b) | the same | `balp` | `23,900` and `24,101` |
  | (b0) | width 256, no read | `balp` | `36,040` and `36,481` |
  | (c) | width 512, 8 read heads | `balp` | `12,512` and `12,523` |

  - **The cell objective costs nothing measurable.** claude-151's `nearest` twin trained alone at `32,013`.
  - **`balp` costs about 25%** at the same model. 62.5% of `balp`'s tokens are scored reply tokens, against 32% of the pinned TRAIN's. More scored rows per window is the candidate cause; it was not measured.
  - **Width 512 with 8 heads halves (b)'s rate.**
- **Decode,** in this runner: float, 64 tokens after 16, 64 and 192 context tokens. The runner re-runs the whole window for each token, so this times the runner, not a served decode.

  | Model | Seed 1 (16 / 64 / 192 context) | Seed 2 |
  |---|---|---|
  | width 256, 4 heads, arm (a) | `212 / 198 / 180` tokens/s | `211 / 198 / 180` |
  | width 256, 4 heads, arm (b) | `212 / 199 / 180` | `212 / 198 / 180` |
  | width 512, 8 heads, arm (c) | `152 / 142 / 122` | `152 / 141 / 121` |

  **Width 512 decodes 28–33% slower** than (b), seed for seed. There is no integer serving path for the read yet, so there is no integer decode figure.

## What did not work, first

- **No arm moves the openings.** At step 16,384, `O` (of 1,226) against its bar:

  | Arm | Seed 1 | Seed 2 | Bar |
  |---|---|---|---|
  | (a) | `306` | `245` | `344` |
  | (b) | `100` | `146` | `352` |
  | (c) | `151` | `165` | `197` |

  The probe already run sets the target: a plain lookup of the prompt's first 8 tokens names `607`. A reply's first word still does not follow what the user asked.
- **More text, as built, loses fit: (b) is cell D with flag F at both seeds.**
  - **Fit:** nearest top-1 `T` is `3,684` and `4,069`, against (a)'s `9,328` and `10,573` (`−5,644` and `−6,504`; the flag is `−500`).
  - **Copies:** `2,074` and `2,981` against a bar of `3,021`.
  - **Rest rows** fall to `1,563` and `1,009` of 9,094 (17% and 11%), from (a)'s 84% and 82%.
  - **Training loss stays at `1.000–1.003`** all the run; (a)'s falls to `0.60`.

  By the sheet, D keeps the pinned TRAIN. The replies show both sides: they repeat the user's numbers and lose the language. "Note these down: zundpim is 832, grarkfud is 4217." gets "Not that welcomeser: N what 832, I ref pro are off …, 832, 832, 832, 832," at seed 1.
- **Why (b) failed, as classified: configuration, not instrument.** The data reached the model as intended:
  - the loader takes only the sealed bytes (both sha256s pinned);
  - `balp`'s windows come from the same `build_document_windows` as the pinned TRAIN's;
  - in the first 2M tokens of `balp`, of the pinned TRAIN and of DEV, every reply starts right after the same token, id `28`;
  - (b)'s step 0 equals (a)'s exactly (I-2).

  Candidate causes, none tested here:
  - **Not the number of passes alone.** At step 2,048 both arms had seen the same number of windows, all for the first time. (a)'s training loss was already `0.979`, and `0.840` by step 4,096, still inside its first pass. (b)'s was `1.003` and `1.0006`.
  - **The mix:** about 85% of `balp` is general chat, 68% of whose tokens are scored replies. DEV is short dialogue, 33% reply.
  - **The trunk barely learns on `balp`:** the no-read control (b0) ends at `T = 1,004` and `1,046`. What (b) learns, it learns through the read.
- **(c)'s gain is on a TRAIN that (b) rejected.** Against (b), (c) passes copies and has no flag F. But its fit, `T = 5,434` and `4,957`, is below (a)'s and below the M1 count table's `7,002`. By the sheet, C makes width 512 the width for later runs, and sends the next unit to a larger width or more steps. Its stated limits:
  - (c) changes the width and the head count together;
  - width 512 has no no-read control of its own;
  - its false-pass rate is about 6%;
  - **width 512 on the pinned TRAIN, with the cell objective, was not run.** That is the next run.
- **No checkpoint meets the item's done-when 2 with done-when 1.**
  - (a) meets done-when 1 at both seeds (`O = 306` and `245 ≥ 197`; `T = 9,328` and `10,573 ≥ 7,002`), but its `R16` is `0.0198` and `0.0098`, against `0.05`.
  - (b)'s and (c)'s `R16` read `0.32` to `0.75`, but neither meets done-when 1.
- **(a)'s replies are fluent templates, still without the user's words.** At seed 1:
  - "Remember, the code word is kandgind." gets "Hi is I'll remember?";
  - "My lucky number is 5246." gets "Okay, I'll remember that.".

  At seed 2, one reply brings the number back: "My lucky number is 5246." gets "Hi is 5246.". The code word gets "Remember,,.".

  (c) copies the words and loses the language: at seed 1 the same two prompts get "Got itionsab an say'll rememberwachgind." and "words 5246.".
- **Done-when 3 has a gap.** The run's config records `--train-split`'s directory and both sha256s, but not its token count. The loader checks the count against the split's own `UORT` header, and the sha256 fixes the bytes, so the count is determined: `99,322,259`. Writing it into the config is left for the next unit that touches the loader.

## What worked

- **The cell objective lets replies copy from the prompt: (a) is cell C at both seeds, with no flag F.**
  - **Copies** (of 4,130 copied rows): `1,517` (36.7%) and `2,933` (71.0%). The bar against the `nearest` twins is `1,381`, and against the no-read floor `1,173`.
  - **Rest rows:** they gain less (`+3.3` and `+2.1` points, against `+5.5` and `+39.8`), so the copy gain is not general fit.
  - **Fit rose:** `T = 9,328` and `10,573`, `+641` and `+1,864` over the twins. Every step-16,384 checkpoint in claude-146's and claude-151's runs read `9,233` or less.
  - **Openings spread out.** (a) picks `73` and `80` distinct opening tokens over the 1,226 openings, where the re-scored `nearest` twins pick 4 and 3. They spread without becoming more right.

  By the sheet, `cell` is the objective for later runs.
- **The new test separates copying from general fit.** Re-scored at step 16,384 (seeds 1 / 2), copied-row hits are:
  - claude-145's no-read runs: `985 / 1,088` (24% / 26%);
  - claude-146's Hamming runs: `1,119 / 1,291` (27% / 31%);
  - claude-151's spin runs: `1,133 / 1,521` (27% / 37%).

  The read copied a little before this unit. With the cell objective it copies 37% and 71%, and with width 512 on `balp`, 81%.

## What changed

- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - **`PointObjective::Cell`:** the hinge on the single nearest wrong token, `relu(1 + max_r s(r) − s(target))`. `nearest` averages it over the 8 nearest. Both decode as the whole table's nearest token (`decodes_nearest`).
  - **The copied-row test in `NearestReading`:** `copied_rows` and `copied_hits` count reply rows whose previous token and target stand side by side in the window's prompt; `rest_rows` and `rest_hits` count reply rows that neither open nor copy.
  - **`TrainSplit` and `load_named_split`:** TRAIN from another `UORT` split, refused unless its tokens and mask hash to the sha256s given. It also checks the header's count, the opening BOS and every id below the vocabulary.
  - **`LongConfig.width`.**
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:**
  - `--objective cell`;
  - `--train-split DIR --train-sha256 HEX --train-mask-sha256 HEX`, required together;
  - `--width N`, default 256.
- **`hamilton_sheet.rs`** gains `build_hamilton_model_at`, and **`spiralcore_fbs_sheet.rs`** gains `build_fbs_model_at`. They build at a given width; the old builders call them at `SHEET_WIDTH`.
- **Tests:**
  - **host:** `a_copied_row_repeats_a_pair_the_prompt_holds_and_not_one_the_reply_made`, `a_named_split_loads_only_under_the_sha256s_it_names`, and two width tests;
  - **CUDA:** `the_cell_objective_pushes_only_the_nearest_rival`.
- **The item's scope** is widened to those two sheet files and the writer, on Casey's direction (in the ORDER).
- **The prep sheet**, and this REPORT and its notebook row.

## Measurements

### The runs

`train-in-space --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S`, with each arm's flags from the sheet, one arm at a time per RTX 5090. Every pod built and tested its own binary from `d102c594`. All four builds have sha256 `5e2f5d01cc3b897f11fb06f238033bcdb869d3ca99eb3dcba3157732cb30202b`.

At step 16,384, seed 1 / seed 2:

| Arm | `T` (of 14,287) | `O` (of 1,226) | Copies (of 4,130) | Rest (of 9,094) | `R16` | Read weight | Train loss | Cell |
|---|---|---|---|---|---|---|---|---|
| (a) | `9,328 / 10,573` | `306 / 245` | `1,517 / 2,933` | `7,600 / 7,490` | `0.0198 / 0.0098` | `0.508 / 0.559` | `0.604 / 0.618` | C |
| (b) | `3,684 / 4,069` | `100 / 146` | `2,074 / 2,981` | `1,563 / 1,009` | `0.320 / 0.467` | `1.016 / 1.122` | `1.0004 / 1.0006` | D, flag F |
| (b0) | `1,004 / 1,046` | `68 / 69` | `76 / 47` | `864 / 936` | `0.633 / 0.205` | none | `1.0004 / 1.0005` | floor |
| (c) | `5,434 / 4,957` | `151 / 165` | `3,342 / 3,330` | `2,002 / 1,531` | `0.611 / 0.752` | `0.321 / 0.283` | `1.0005 / 1.0003` | C |

The references, re-scored here at step 16,384 (seed 1 / seed 2):

| Run | `T` | `O` | Copies | Rest |
|---|---|---|---|---|
| claude-146's Hamming twins (`nearest`), (a)'s reference | `8,687 / 8,709` | `298 / 200` | `1,119 / 1,291` | `7,299 / 7,239` |
| claude-145's no-read runs, (a)'s copy floor | `8,607 / 8,706` | `247 / 229` | `985 / 1,088` | `7,391 / 7,427` |
| claude-151's spin runs, reported | `8,487 / 9,063` | `208 / 291` | `1,133 / 1,521` | `7,158 / 7,296` |

**The bars,** computed from those references as the sheet states (`m = 3·√(N·p(1−p))` at the better reference seed):

| Arm | Openings bar | Copies bar, against the reference | Copies bar, against the floor | Copy gain over rest gain (seed 1 / 2) |
|---|---|---|---|---|
| (a) | `344` (`298 + 45.1`; the sheet writes `343`) | `1,381` (`1,291 + 89.4`) | `1,173` (`1,088 + 84.9`) | `+5.5 > +3.3` / `+39.8 > +2.1` points |
| (b) | `352` (`306 + 45.5`) | `3,021` (`2,933 + 87.5`) | `102` (`76 + 25.9`) | not reached: fails the reference bar |
| (c) | `197` (`146 + 34.0 = 180`, raised to `197`) | `3,068` (`2,981 + 86.4`) | `102` (`76 + 25.9`) | `+8.7 > +4.8` / `+8.5 > −0.4` points |

- **The groups overlap at openings.** A reply's first row is a copied row when its pair already stands in the prompt; 163 of the 1,226 openings are. So copies, rest and openings add up to more than 14,287. Rest rows exclude both.
- `R16 = shift(16) / shift(0)` from `--reach` on the step-16,384 checkpoint. At step 0 the untrained trunk reads `0.66–0.73` (width 512) and `0.73–0.81` (width 256, claude-151).
- (a)'s `T` passed both twins' step-16,384 values by step 6,144 at seed 1 (`9,005`) and by step 2,048 at seed 2 (`9,027`). (b)'s peaked early, at `5,878` (step 4,096, seed 1) and `4,698` (step 2,048, seed 2), and then fell.

**Instrument checks,** at every seed:

- **I-1:** `123 passed; 0 failed; 5 ignored` on each of the four pods' builds. The negative gate: with the cell hinge's max over rivals replaced by their mean (`nearest`), `the_cell_objective_pushes_only_the_nearest_rival` failed with exit 101 and its own `FAILED` line, on every pod. The copied-row test's host gate, with its prompt-only check widened to the whole window, failed on the laptop before the runs.
- **I-2:** (a)'s, (b)'s and (b0)'s step-0 `dev_nearest` equal claude-146's twins exactly: hits, openings and distinct opening picks are `2/0/730` at seed 1 and `5/3/681` at seed 2.
- **I-3:** every arm with a read ends with its weight above 0 (`0.28–1.12`, in the table).
- **I-4:** every loss is finite.
- **I-5:** the re-scored claude-146 checkpoints reproduce their stored step-16,384 hits and openings exactly: `8,687/298` and `8,709/200`.

### Evidence

- **Location:** `caseyallard/uor-r4-store:results/claude/prompt-use-2026-10-28/`. It holds one tar per pod: `pod1-s1.tar`, `pod4-s2.tar`, `pod3-c-s1.tar` and `pod5-c-s2.tar`. Each holds the pod's whole job folder:
  - the job log and every step's out, err and exit;
  - each arm's config, progress, samples and checkpoints;
  - reach, inspect and decode;
  - the gate's patch and its output;
  - the re-scored references.
- **Seals:** each tar has its own `.SHA256SUMS`, written on the pod. `SHA256SUMS-claude-152` lists all four, sha256 `0d4f4943e04699b13c94714fcbadace6723f46b23ef56958ff0fac9c88f20611`:
  - `pod1-s1.tar`: `d901d4dcc1d21ab34c3adcdf268fcef8a09b3545fc4d0910c53fb15d352e28ee`;
  - `pod4-s2.tar`: `643f2cbc8cb0b46a5ab8c0f5371a886ca04ddb700833cc067f124731cfcaaf63`;
  - `pod3-c-s1.tar`: `f34e46f6cbde75bcda73210ebe00f2157934afc2b20c5eaf3990c723017d74f5`;
  - `pod5-c-s2.tar`: `fd993330282fab6869dfbe57669163a875bca0d2f39d1a16bd35f3c8c787284b`.
- **Checks:** every tar was read back from the store to the laptop and checked against the hash the pod wrote, before that pod came down.
- **Cost:** four Vast RTX 5090 pods at `$0.563/h`, 6.2 pod-hours in all: about `$3.50`. All four are down. A fifth create returned no contract and left an exited instance, `58038282`. It was destroyed and checked gone within minutes.

## Checks run at the head

The code has not changed since the runs' commit, `d102c594d126725c233fe7ebc5cc4e61f583d731`. `git diff d102c594 -- crates/ Cargo.lock` is empty. Later commits touch only the sheet, this REPORT and the notebook.

- **CUDA library tests on an RTX 5090,** on each of the four pods' builds: `123 passed; 0 failed; 5 ignored`, and the negative gate bit on each.
- **Host:** `cargo test --locked -p uor-r4-float-train --no-fail-fast` at the head: 75 library tests and every other target pass, 93 in all, 0 failed.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **In-unit reviews,** each by a fresh reviewer that never saw the plan:
  - two FIT rounds, both NOT FIT, each fixed, and claude's decision review at the two-round limit (listed in the sheet);
  - **the exact-head code review: APPROVE.** It confirmed:
    - the cell hinge's axis and shapes, and that the rival set excludes the target;
    - that `nearest` is unchanged;
    - `copies_prompt`'s index arithmetic: the pair always sits strictly before the row, inside the window, at prompt positions;
    - both sha256s checked with no fallback to the pinned TRAIN;
    - no width-256 assumption left, with reload using the recorded width.

    Its four low-severity notes are in [Still open](#still-open). One changed this REPORT: the width-256 host test compares two entry points that now share one path, so it cannot catch a change from main.
  - **the claims check of this REPORT** confirmed about 130 claims and found six slips, all fixed:
    - decode and training rates cut off instead of rounded;
    - three bar breakdowns whose parts did not add up;
    - "28–32%" for what reaches 33%;
    - a plain-summary line stronger than the re-scores;
    - (b0)'s missing `R16`;
    - the twins' pick count.

## Skills used

- **holding-the-goal.** Casey asked for three things: a probe of what matters, if quick; more training text and a wider trunk; and a retrain with the Hamming read under a geometric margin objective, "so that we can have a better test for if the response is taking the user's input into consideration". The probe already existed and was reused. The test was built first (copied rows, against a no-read floor), and every retrain was read by it.
- **root-cause-debugging.** (b)'s loss stall was classified before it was read as a result. The data's bytes, window builder, reply alignment and step 0 were each checked, and all were sound. So it is a configuration result, and the unit says so (CHARTER §4.1, lesson L-007).
- **verify-before-done.** Every bar was recomputed from the re-scored references, and every tar was read back against its pod's hash before the pod came down.

## Still open

- **The next run, by the three cells together:** the cell objective, the pinned TRAIN and width 512 with 8 heads, at seeds 1 and 2. It is the one combination this unit's decisions point to, and it was not run. A width-512 no-read control on the pinned TRAIN goes with it.
- **More text, differently:** `balp` held to dialogue like DEV's (the `mw-balp` part, or chat filtered to short replies), or more steps. (b)'s stall says the mix matters, not that text does not.
- **Openings:** no arm moves them. The probe's `607` comes from the prompt's first 8 tokens, beyond the trunk's reach of about 6. The copy heads now carry the user's words, but nothing carries the prompt's first words to the reply's first token.
- **Done-when 3's gap:** write `--train-split`'s token count into the run's config.
- **The code review's four low-severity notes,** for the next unit that touches these files; no run is affected:
  1. The copied, opening and rest tally inside `nearest_reading` has no test of its own. The host test pins `copies_prompt`, the predicate. A host-side tally function with counts asserted on the hand-built window would pin the split itself.
  2. Copied rows can be opening rows (163 of 1,226 here). Make the classes exclusive, or keep reporting the overlap.
  3. The width-256 host test compares two entry points that share one path. A sha256 of main's seed-1 initial tensors would let it fail.
  4. `--width` accepts any number; "a multiple of 8" is documented but checked only deep in the model build. Refuse others in the CLI.
- **The item stays open:** no checkpoint has `R16 ≥ 0.05` together with done-when 1.

## Plain summary

Casey asked for a better test of whether a reply uses the user's input, and for three retrains of the trunk with the Hamming read. The test counts reply tokens that the user's message already holds as a pair, such as a code word or a number. Before this unit the read named 27–37% of them, a few points above the trunk with no read at all (24–26%).

- **The geometric margin objective works:** it scores the target against the single nearest wrong token. The read now names 37% and 71% of those tokens at the two seeds, and overall accuracy rose above every earlier final checkpoint (`9,328` and `10,573` of 14,287). Replies stay fluent but mostly templated. At seed 2 one brings back the user's number ("Hi is 5246."), and the code word does not come back.
- **13 times more training text, as mixed, hurt.** The model learned to repeat the user's numbers and lost the language. The data was checked and loads correctly; the mix is mostly long general chat, unlike the test's dialogues.
- **The wider trunk on that text** copied the most (81%), but stayed far below the smaller model's overall accuracy.
- **Openings:** a reply's first word still does not depend on the question.

The next run is the margin objective with the wider trunk on the original text. The runs cost about `$3.50`, and all four pods are down.

## Done when

1. **Met.** `--objective cell` is built. `the_cell_objective_pushes_only_the_nearest_rival` (CUDA) checks that the loss is positive inside the margin and zero beyond it, and that it counts only the nearest rival, where `nearest` averages three. Its negative gate, max replaced by mean, bit on every pod.
2. **Met.** The copied-row test is in `NearestReading`. A host test pins the split on a hand-built window, and its negative gate, the prompt check widened to the whole window, failed.
3. **Met, with one gap.** `--train-split` reads `balp`, refused unless both sha256s match. The run's config records the directory and both sha256s, not the token count (in What did not work).
4. **Met.** `--width N` builds the trunk, the table and every read at `N`. Checkpoints record `width` (`512` in (c)'s, `256` in the others). The width-256 build is what main builds by construction: the old builders now call the new ones at `SHEET_WIDTH`, and the only other change sets `config.width` to that same value. I-2's exact step-0 match with claude-146's twins, built from main, agrees. `the_width_builder_at_256_is_the_sheet_builder` (host) compares the two entry points tensor by tensor. Since they now share one path, it cannot catch a change from main (the code review's note, in Still open). The suite is green, with the CUDA tests on each pod.
5. **Met.** The pre-registered runs ran under one sheet, with FIT and a pre-run review: (a), (b) and (c) at two seeds each, plus the (b0) control. The sheet's bars decide them.
6. **Met on merge.** This REPORT leads with tokens/s, and the result has one notebook row ([float-training](../../../../research/notebook/float-training.md)).

The item's own done-when is not met, so it stays open.
