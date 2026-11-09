# REPORT: one-model-end-to-end, claude-339

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [the language carry run](../../../../harness/prep/one-model-end-to-end-language-carry-2026-11-09.md). Base `fbf15e56f`. Training pin `721a4784f265efea6b490d5c3100779e4c4b1607`. Post-processing on `main` at `402b4091eb7f7e7ea174f96ed0bbd06cf58fa78b`.

**In one line:** Negative. Outcome row **4c**, for this configuration only: at step 16,384 no seed of L-aC puts the asked value on the first line more than 2 times in 256, its exact `−C` control scores the same on that count at both seeds, no cell has one complete reply, and stopping fell far below its floor (EOS 2 and 0 of 256, floor 64).

## What did not work, first

- **The registered read missed every bar (negative, row 4c).** All four cells were served and every registered instrument check passed.
  - **The panel's bar:** 0 complete rows and 0 good pairs in all four cells. The bar is 104 and 52 at both seeds.
  - **Recall did not move.** L-aC's first-line asked count is 0 and 2, against L-a's 2 and 0, claude-322's 14 and 2, and its own `−C`'s 0 and 2. The bar is at least 16 above all three, at both seeds.
  - **Stopping did not hold.** L-aC has 2 and 0 EOS stops, L-a 6 and 0, against the floor of 64 and claude-322's 109 and 86. Nearly every reply runs to the 64-token cap.
  - **Replies are narrower than claude-322's.** Distinct texts are 38 to 97 of 256, against its 94 and 125, and up to 172 of the 256 rows share one text (L-a seed 1).
  - **The carry changes replies, but not toward the value.** Against `−C`, the carried artifact's replies differ on 87 of 256 rows at seed 1 and 44 at seed 2, and the asked count does not rise on either.
- **Stopping is lost in both arms, including the no-carry guard.** So the loss happens without the carry. That does not clear the carry: at seed 1 L-aC has fewer EOS than its own `−C` at every point (18 against 39, 0 against 2, 2 against 4), and seed 2 reads 0 against 0 at 16,384. L-a differs from claude-322 in the ask heads, the pull and the trainer commit (`721a4784f` here, `ff5156bcc` there), and L-aC adds the carry. No no-ask control ran at the new commit, so this run cannot say which of them caused it.
- **The trend does not rise.** L-aC's asked count falls at seed 2 (5, 4, 2) and stays 0 at seed 1. EOS is lower at 16,384 than at 8,192 in every cell that had any (L-aC 18 to 2 and 14 to 0, L-a seed 1 14 to 6). Row 4u's rise needs at least 8 above L-a's; it is 0 and −2.
- **I-xg failed on the first pod (instrument, repaired).** L-aC seed 1's first pod snapshotted claude-322's seed-1 checkpoint and exported `9b01ed62…`, not claude-322's `86e51d5e…`. The job stopped before its full run as the sheet says, and sealed NOT_RUN at `runs/L-aC/s1` (EXIT 6).
  - **Cause:** `fbs-snapshot` builds the trunk tables on the GPU. Where two nearest atoms are almost tied, the GPU and the laptop choose differently: four quantized entries in `layers`, four trunk sections. The GPU snapshot itself is deterministic: two pods gave the same `6a424749…`.
  - **The repair:** every served export is the laptop snapshot (`cpu-fbs-export` on a base-inventory copy from the new `hamming-base-copy`), with the pod's ask entries spliced in and checked section by section against the pod's own export. I-xg's bar is unchanged. It passed with `86e51d5e…` for the registered `runs-3` round at 18:12:56Z.
- **Three pod launches never reached SSH** (two for L-aC seed 1, one for L-a seed 2), each a different Vast offer on KR host 406325. `uor-pod` deleted each pod at 900 s, so about 45 pod-minutes were lost. The circuit breaker then refused a third launch, and `seed-339.sh` gained `UP_FORCE=1` for `--force-retry`.
- **CI went red twice on a test this PR does not touch:** `panel_m6_rustc_verifier::failing_assertion_is_reported`, "File exists (os error 17)". It's a capture-file name race in DeepSeek's verifier (#3821), sent to DeepSeek on #2028. Reruns pass.
- **The fresh review of the repair** found a refused artifact left under the served name, where `−C` would score it, and no check that the splice left the trunk alone. Both are fixed before any read, with tests and a negative gate (test-read 45/45).
- **I deleted two inputs of the read, then restored both before any read ran (L-003).** I cleared closed unit folders for disk. My reference check missed `read-339.sh`'s variable-built paths to the m2-fresh256 panel (claude-309) and claude-301's b64L grades.
  - **The panel:** refetched from the store into `local/claude/kept/panel`. Its four hashes equal those claude-322's sealed score recorded.
  - **The grades:** rebuilt from claude-301's runs-2 seals (`f7852f25…`, `1f24f03a…`) against D3 into `kept/claude-301-b64L`. Each `grade.json` is byte-equal to its pin (`e17a1354…`, `3170c779…`).
- **`serve` exited 0 with every cell NOT_RUN** (a dry run on the first L-aC seed 1 seal): a missing `config.json` skipped the cell without failing the read. Fixed before the real read: a NOT_RUN cell now sets the exit to 1. The dry run with the smoke's config then ran I-xa end to end on the repaired path (ok, carry 32, the trunk equal to `cpu-fbs-export`'s) and exited 1 on the missing full-run points.
- **All four first-round full runs stopped near step 14,700, so none reached `step-016384` (format range, row 0a, repaired).** Training as served rounds each table every step, and an ask table stores −8 to 32767/4096 (Q12 in `i16`). A few float weights drifted past the end and the rounding refused them: `d6.ask` at −8.0002 in three cells and `d2.ask` at 7.9999 in one. About 7,500 s of GPU per cell, $5.20 in all, gave only NOT_RUN seals (`runs/`, `runs-2/`, laptop).
  - It is not the carry: the no-carry arm stopped the same way, at the same step.
  - **The repair:** `hold_hamming_ask_in_exported_range` (commit `721a4784f`) holds each weight at the end of its range before rounding. A test pins it and fails on this same refusal with the hold set aside. All four reran from step 0 into `runs-3/`.
  - I had no range check on these tables before the full run; the smokes were too short to reach the end of the range.
- **The Hugging Face store filled (instrument, 17:13Z).** `uor-store put` refused with "Private storage limit reached", so L-a seed 2's seal was copied to the laptop and put in the store once it had room. Deleting files alone frees nothing. With Casey's approval, 784 old checkpoint files (15.4 GB, list in `local/claude/claude-339/store-clean/delete.txt`) were deleted and the history squashed (1,082 commits to 1). Writes worked again about 18:05Z.
- **The laptop's Hugging Face credential disappeared mid-unit (not by me).** Store access stopped until Casey logged in again; nothing else was tried.
- **Rerun launches lost:** one pod never answered SSH (`vast-59924022`, deleted at 900 s). L-a seed 2's first try ended `up exit=1` with its pod stopped (`vast-59928452`, deleted). Neither has a billing line I could find; the Cost section gives the estimate.
- **Trend points on the owner's early path are bad (9 Nov, 20:05Z).** L-aC seed 1 at 8,192 and 12,288 steps, carried and exact −C, scored 0 complete rows and 0 asked first lines on `m2-fresh256`. Carried EOS went 18 to 0 and distinct replies 137 to 44 between the two steps. They are not the registered read. Reports and hashes: [early/INDEX.md](early/INDEX.md).
- **Watchers changed mid-run on the owner's instruction (20:02Z).** "Do not kill a pod until the owner approves." The auto-down was removed: `watch-339-hold.sh` and `pull-seal-339.sh` hold each pod, leased, until an approval file exists. Finished pods bill while they wait.
- **Teardown rule, owner's words (9 Nov).** "Tear each pod down independently as soon as its own four checks pass": `job=0 seal=0`; all 41 required files; independent store read-back and `SHA256SUMS`; no GPU process. `--score-rows` is a required pod-side input. "Do not infer a final outcome from the 8,192/12,288 trend."
- **A launch failed I-xg on my launcher's own leftovers (instrument).** On `vast-59926331`, `read-339.sh` refused the first round's I-xg folder, compared nothing and reported FAIL. Each launch now names its own `IXG_OUT`. The job's NOT_RUN seal in `runs-3/L-aC/s1` moved byte-identical to `runs-3-void-ixg-collision/L-aC/s1`; it reads nothing about any arm.
- **`fetch-339.sh` with a relative path wrote into the main checkout,** because the script changes to the repo root. I moved the folder under `local/claude/claude-339/diag/` and added an absolute-path guard.
- **The fresh exact-head review found a defect in the ask-range hold (fixed).** `clamp` held an infinite weight at the end of the range, where the rounding used to refuse it by name. The hold now touches finite weights only, and a new test fails on `inf beside -9 refused` with the fix's non-finite arm undone. The merged trainer crate (`uor-r4-float-train`) differs from the training pin `721a4784f` on this non-finite path only. Finite trajectories do not change, so the sealed runs stand.
- **The fresh pre-run review of the rerun passed with minors,** all fixed or measured before launch: the seal pull now starts by itself, the margin's low side is measured (smallest 9.44 at 14,336), the launcher needs its seal root named, a failed copy is set aside. One nit is left: `job-339.sh`'s header comment, so its pinned hash doesn't move.

## What changed

- `crates/uor-r4-float-train/src/bin/hamming-base-copy.rs` (new), a laptop tool for this run's export. It is outside the item's `scope:` list, which `pr-check` reports as an advisory.
- `crates/uor-r4-float-train/src/fbs_export.rs` and `in_space_long.rs`: the ask-range hold and its two tests.
- The sheet, the ORDER, this REPORT, the notebook row, [early/](early/INDEX.md) and [read/](read/) (text reports and hashes).
- Laptop scripts in `local/claude/claude-339/` (not committed): `read-339.sh` (laptop snapshot), `seed-339.sh` (ROOT, UP_FORCE), `fetch-339.sh` (one assembled root), and the early and teardown scripts.

## mw-balp's licence and generator (claude-314's open item, answered)

TRAIN (d3mwcap6) holds mw-balp. claude-314 said its licence and generator are "recorded nowhere". They are recorded in three places:
- **Licence:** MIT, project-generated. Stated on the dataset card of [caseyallard/uor-r4-data](https://huggingface.co/datasets/caseyallard/uor-r4-data) (`memory-dialogue/mw-balp/`) and in the archived front README (`archive/front/README-2026-10-20.md`, its data table).
- **Generator:** `crates/uor-r4-training/examples/m-world.rs` (the card names `uor_r4_training::milestone_world_v2`), built at `dec3c096e4e3abe7ee382986a4ec1e83612aba12` (4 Oct 2026).
  - The command is in the set's own `attempt.json` (in the store's `data/ft-balp.tar` and on the card): `corpus world=v2 protocol=2 conversations=60000 seed=13 recall=off paraphrase_share=0.5`, with the 4,096-entry tokenizer.
  - It also reads 35 paraphrase files: `claude-instr-para__chat-q1..q4` and `claude-log-sieve__paraphrases-q3..q34`, written by project sessions.
  - Its train tokens have BLAKE3 `a95379b89f30f75b622ab907ba56e01eb9805860d51fc0cdeab323792f97a0d8`.
- **What is still missing:** the 35 paraphrase files are in neither the store's `data/` nor the dataset. So mw-balp can be used and checked from its published tokens, but not rebuilt from public pieces. Publishing them is a small follow-up; it changes no result.

## Measurements

**The panel at step-016384**, `m2-fresh256`, 256 rows and 128 pairs, `served-reply-panel --engine integer` at the pin. Median reply length is 64 and BOS and context stops are 0 in every row.

| artifact | seed | asked first line | other first line | complete | good pairs | EOS | cap | collapse | distinct replies | most common | decode tok/s |
|---|---|---|---|---|---|---|---|---|---|---|---|
| L-aC carried | 1 | 0 | 3 | 0 | 0 | 2 | 254 | 1 | 60 | 77 | 682 |
| L-aC exact `−C` | 1 | 0 | 4 | 0 | 0 | 4 | 252 | 2 | 58 | 89 | 740 |
| L-aC carried | 2 | 2 | 3 | 0 | 0 | 0 | 256 | 0 | 46 | 143 | 505 |
| L-aC exact `−C` | 2 | 2 | 3 | 0 | 0 | 0 | 256 | 0 | 41 | 147 | 710 |
| L-a (no carry) | 1 | 2 | 7 | 0 | 0 | 6 | 250 | 4 | 38 | 172 | 723 |
| L-a (no carry) | 2 | 0 | 0 | 0 | 0 | 0 | 256 | 0 | 97 | 31 | 564 |

The decode rates are laptop rates from runs that were not timed under controlled load, so they are not a speed comparison. EOS per stratum: L-aC seed 1 reassert 2; its `−C` reassert 3, update 1; L-a seed 1 length2 4, reassert 2.

**The bars against the outcome map:**

| row | what it needs | reading |
|---|---|---|
| 0a, 0b | an arm NOT_RUN | no: all four cells `job=0 seal=0`, `EXIT 0` |
| 0c | I-xg, I-xa, I-cut or I-c fails | no: all passed (below) |
| 1 | at least 104 complete and 52 good pairs at both seeds | no: 0 and 0 |
| 2, 3 | recall moved | no: asked 0 and 2, needs at least 16 above L-a (2, 0), claude-322 (14, 2) and `−C` (0, 2), at both seeds |
| **4c** | recall not moved, `−C` within 4 of L-aC at both seeds | **yes:** the asked count differs by 0 at both seeds |
| 4u | a rise from 12,288 to 16,384 at least 8 above L-a's, at both seeds | not reached: L-aC's rise is 0 and −2, L-a's 0 and 0 |

**The trend**, asked and other first-line counts, EOS, and pull distance (complete rows are 0 at every point):

| cell | 8,192 | 12,288 | 16,384 |
|---|---|---|---|
| L-aC seed 1 | 0 / 0, EOS 18, 0.288 | 0 / 4, EOS 0, 0.294 | 0 / 3, EOS 2, 0.302 |
| L-aC seed 2 | 5 / 8, EOS 14, 0.351 | 4 / 4, EOS 1, 0.351 | 2 / 3, EOS 0, 0.350 |
| L-a seed 1 | 1 / 4, EOS 14, 0.287 | 2 / 4, EOS 5, 0.283 | 2 / 7, EOS 6, 0.285 |
| L-a seed 2 | 0 / 0, EOS 0, 0.272 | 0 / 0, EOS 0, 0.270 | 0 / 0, EOS 0, 0.269 |

**Held out**, `scale-probe grade` on all 1,460,697 rows from the pods' no-memory `--score-rows`, then `pair`:

| arm | against | verdict | d (seeds 1, 2) | mean d |
|---|---|---|---|---|
| L-aC | (b64L-D3) | behind | −34,428, −25,118 | −29,773 |
| L-a | (b64L-D3) | behind | −12,234, −17,832 | −15,033 |
| L-aC | claude-322 | inside | −18,972, +509 | |
| L-a | claude-322 | mixed | +3,222, +7,795 | |

Neither arm crosses the cost ceiling, a mean d below −58,212. The dev curve's hits at 16,384 are 6,266 and 6,797 for L-aC and 6,725 and 6,697 for L-a.

**Recorded:**
- **Ask entries held at the ends of the range**, of 2,097,152 in each `step-016384`: at −32,768 and at 32,767, L-aC seed 1 17 and 18, seed 2 6 and 3; L-a seed 1 5 and 0, seed 2 4 and 5. By table (first asking table, then second): L-aC seed 1 1, 3 and 16, 15; seed 2 2, 2 and 4, 1; L-a seed 1 3, 0 and 2, 0; seed 2 2, 3 and 2, 2. Saturation is about 1 in 60,000 entries at most.
- **Carry gains** (4,096 at Q16 in `i32`, so nothing is held): seed 1 from −8.11 to 8.16, 337 negative; seed 2 from −8.10 to 8.14, 460 negative. 22 and 20 sit outside ±8.
- **Last logged loss:** 0.913, 0.878 (L-aC); 0.882, 0.858 (L-a). **Step time, I-t's 128-step rate:** 0.529 and 0.523 s (L-aC), 0.498 and 0.490 s (L-a). Full-run seconds per step, evaluation included: 0.538, 0.534 (L-aC), 0.507, 0.501 (L-a).

**Instrument checks, all passed:** I-xg on the rerun; I-xa at all four cells (the integer engine scores it, the default refuses by name); I-5 `same` on every split; I-c on both claude-322 artifacts; I-cut at both L-aC seeds (86 sections equal, 32 carry rows against none). Between the laptop's and the pod's snapshot at step-016384, 1, 1, 1 and 5 trunk sections differ (L-aC seeds 1 and 2, L-a seeds 1 and 2), from the near-tied quantization named under I-xg; the Hamming sections are equal. The registered `−C` is byte-identical to `fbs-toks --drop-hamming-carry` from `main` at both seeds (`6e85e4e2…`, `391392f5…`), so the two cut paths agree. The pinned scorer and `main`'s produce equal reply ids, stops and completeness on all four cells.

**Files.** Registered read in the store at `results/claude/one-model-end-to-end-language-carry-2026-11-09/read/` (artifacts, reports, grades, pairs; `SHA256SUMS.read` is `815552185963051205211802958c36c0da831e3651c4dd6c7145a17d13e9551a`). Seals at `.../runs-3/ARM/sS/`. Step-016384 artifacts:

| cell | SHA-256 |
|---|---|
| L-aC s1 carried | `ba221c27f846fc171856280135964e6a66a7a4ff5d7733b763570fa1d196b6e7` |
| L-aC s1 `−C` | `6e85e4e201e09771b56aa54d0ed680120ed00fbb082caaf7a8459681296da5e6` |
| L-aC s2 carried | `2741d9383a80c300b90ca2d5cbc1e06a9b664d913971f84ddbc2916a1625a430` |
| L-aC s2 `−C` | `391392f5c47af2cd6a1e091417dcc04c5340080474ebf6dad5d10ca2cfd277e5` |
| L-a s1 | `7b817369dbe0420a5dc409cdef84ce6598cafacae942b81931e2f7a88ebbed13` |
| L-a s2 | `df3c8f60999fe1161d02f7931d1c12b4a0a236ff32c4f8cc187506640277feb1` |

## What this does and does not show

**It shows,** for this configuration (b64L, d3mwcap6, 16,384 steps, two seeds), that the ask heads, the 32-row question carry and the found-copy pull do not put asked values into served replies: at most 2 of 256 first lines. The carry is present and changes up to 87 replies, and its removal moves the asked count by 0.

**It does not show that the carry cannot work.** A negative closes this configuration only. The asked count is at the floor, so "`−C` within 4" is close to the only reading available: two counts near 0 cannot separate. **TRAIN's distances are not counted** (the sheet names this for row 4c): the mix's `mw-balp` part comes from the panel's generator family, and D3's part is not counted, so whether 32 rows reach the question in training is unknown.
**It does not say why stopping went.** L-a loses it as well as L-aC, and this run has no arm that separates the ask heads, the pull and the trainer commit. L-aC adds the carry on top of those.

**It does not measure float-to-integer parity on these asking checkpoints.** "All passed" above means the registered checks; claude-336's parity was on a different model.

**The 8,192 and 12,288 points are the trend only.** The outcome comes from step-016384.

## Checks run at the head

At the head: the host tests at the end of this list, and CI on the PR. The script gates are earlier, as dated.

- **On the scripts the registered `runs-3` pods ran, on the laptop at 17:22Z to 17:28Z, after their last edit at 17:19Z and not at this PR's head** (job `4e1272b8…`, chain `5eb46601…`, seal `e1ca7e1b…`, the pinned hashes checked on every pod):
  - job mutations 37 of 37 caught;
  - test-gates 81/81, test-seal 18/18;
  - test-read 45/45 with the trunk check's negative gate.
- **test-watch ran earlier, at about 15:12Z, and is timing-sensitive:** one run caught 9 of 10. The miss was the mutant that drops `end()`'s own `trap '' TERM INT HUP`, under a laptop load average of about 7. Two reruns caught 10 of 10. The test sends its signals 0.4 s and 0.7 s into a 2 s hold, so under load the first can land before the window. The job is unchanged, and this is a test that can pass by luck, named here.
- **Host test of `hamming-base-copy`:** `cargo test -p uor-r4-float-train --bin hamming-base-copy`, 1 passed. The other code in this PR is the ask-range hold in `fbs_export.rs` and `in_space_long.rs`, with its two tests, `an_ask_weight_past_the_served_range_is_held_at_its_end` and `every_ask_is_held_and_a_weight_not_finite_is_still_refused`, 2 passed on the host.

## Cost

First round: four 5090 pods at $0.563/h, about $5.20, plus the first L-aC seed 1 pod and three SSH-failed pods. Rerun, from the pods' final lines: $1.50, $1.52, $1.44 and $1.40, $5.86 in all. The two lost launches and the I-xg collision pod have no billing line I could find; at $0.563/h I estimate under $0.50 for the three. The unit's pods total at least about $11. Each pod went down 75 to 136 s after its chain ended, under the owner's teardown rule. The laptop did the exports, the panels and the grades.

## Next

**Row 4c's next step is a decision review, between a question-row placement on chat windows (the `both` placement that passed at both seeds on the recall task) and TRAIN's distance count.** It belongs to the next unit, read against the whole project. My view going in: count TRAIN's distances first, since it is laptop work and says whether 32 rows reach the question at all. The review should weigh the lost stopping, which the outcome map does not cover. It is the larger gap for a usable reply, it appears without the carry, and no arm here separates the ask heads, the pull and the trainer commit (L-aC adds the carry).

**Small follow-ups:** publish mw-balp's 35 paraphrase files; make `test-watch` marker-driven.

## Skills used

- None beyond the loop itself: the sheet with its FIT and pre-run review, the owner's per-pod teardown checks, the registered read and the fresh exact-head review.

## Still open

- **Row 4c's decision review:** a question-row placement on chat windows, or TRAIN's distance count first. The next unit.
- **Stopping:** both arms lost it against claude-322. No arm here separates the ask heads, the pull and the trainer commit, and L-aC adds the carry, so the cause is open. A no-ask control at the new commit would settle the trainer commit's part.
- **TRAIN's distances are not counted.** Whether 32 rows reach the question in training is unknown.
- **mw-balp's 35 paraphrase files** are not published, and **`test-watch`** still depends on signal timing.

Closes nothing: #3407 stays open.

## Plain summary

We taught the model two new things in one run: a part that looks back for the answer to a question, and a short-term memory (the carry) that holds the question while the model reads. Four copies trained on rented GPUs for about $6 in the final round (about $11 with the first round, which stopped early), and each machine was shut down as soon as its results were saved and checked.

It did not work at this setting. Out of 256 test conversations, the model named the right value on the first line at most twice, and it never wrote a complete correct reply. Removing the memory changed some replies but not that count. The model also mostly stopped ending its replies, with or without the memory, so that happens without it. We can't yet say what caused it.

This rules out this one setting, not the idea. The next step is to decide between placing the memory differently and first counting how far back questions sit in the training data.
