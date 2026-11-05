# REPORT: integer-long-context, claude-287 (slide, blocks and carry past the trained window, measured and decided)

Lab: claude. [ORDER](./ORDER.md). PR #3696, item #3471 (References #3471; done-whens 3 to 5 remain, for claude-288). Plan step: P4 in Casey's plan of 4 November, step 5 of THE ORDER (#2028 6634218223). Base `d6c2c3ecd`.

**In one paragraph.**
- **The decision** (done-when 1, the decision review on #3471): **carry, provisional.**
  - a7r, the served model, reads row 5 of the sheet's table, because its two seeds disagree on carry.
  - P2b's (L), trained at 2,048, reads row 4: carry holds at both seeds. As the sheet says, that row is what the next long checkpoint re-reads.
- **What it changes in the model that serves:** the next unit, claude-288, serves carry in the integer session. That keeps the trunk's state and the read's keys across the whole conversation instead of stopping at 192.
- **What it changes in the model that trains:** nothing in this unit.
- **What the numbers do not say:** why carry holds on (L) and not at a7r's seed 1. The two checkpoints differ in their trained window, and also in their run and step, their panel, their carry context and how far carry runs past the window (about 10.7 times against 2). On (L), carry only ties slide, and no rule recalls a far note.

## Tokens per second first

Float batched greedy decode of each panel's queries on the RTX 5090. This is not the integer session's speed, which claude-288 measures.

| rule | a7r (W 192, p-long) | (L) (W 2,048, p-long4k) |
|---|---|---|
| slide | 957 tok/s | 18.5 tok/s |
| blocks | 626 tok/s | 67 tok/s |
| carry | 145 tok/s, run at 2,048 | 42 tok/s, run at 4,096 |

- a7r's carry rate matches claude-281's 137–148 tok/s at 2,048.
- **Carry is the slowest float decode on a7r,** and slower than blocks on (L). So the table's "carry, on serving cost" does not rest on these rates. It rests on the integer session's carry, which claude-242 served past 192 at 4,647 tok/s at 192 and 1,851 at 2,112. claude-288 measures it on the served model.
- **Slide is the costly rule at a long window.** (L)'s slide passes took 2,660 s each, against 1,320 s estimated. 2,206 s of that is the two queries decodes at 18.5 tok/s. The stride-128 scoring, where each window reads 2,048 tokens to score its last 128, is the rest.

## What did not work

- **Carry on a checkpoint trained at 192 is worse at one seed.**
  - At a7r seed 1, carry's reply agreement past W is 0.0138, against slide's 0.0339 and blocks' 0.0267. Both gaps are past the 0.0100 band. That is 89 agreeing tokens under carry against 173 under blocks, of 6,469.
  - Carry's loss past W is the highest of the three rules at both a7r seeds: +0.0194 and +0.0143 over slide. Both are inside the band (0.0197), and only because a7r's two seeds differ by 0.033. At seed 1 the margin is 0.0003.
  - **The class: mechanism, in one configuration** (carry on a7r, a 192-trained checkpoint, at 2,048). Every instrument check passed.
- **(L)'s one "carry beats blocks" rests on one token:** 173 against 130 agreeing tokens of 4,262, where the band is 42.6. Were it a tie, (L)'s row would still be 4.
- **A third call sits at the band's edge, and could move a row.** At (L) seed 2, carry's loss past W is 0.00634 below blocks', against a band of 0.00688. Had it crossed, carry would beat blocks at both seeds, and (L) would read row 3 (carry, not provisional).
- **No rule is shown to use earlier context.**
  - Agreement past W is 1–5% under every rule.
  - Recall (R5) is 0: no census-clean far word is stated under any rule, of 306 on p-long and 189 on p-long4k. The one far note stated (a7r seed 2, carry, 1,849 tokens back) has a numeric answer, which the census-clean filter excludes.
  - So the panels do not yet test what earlier context is for. That is why row 4 is provisional, as the sheet expected. Row 5 is provisional because a7r's seeds disagree; re-reading it on a checkpoint that recalls is the decision review's extension.
- **The sheet's slide query prompt was misworded.** It says slide's prompt is "the W positions before the reply". The code ends the W-token window at the reply's last input, as the base scorer's `query_prompts` does.
  - So slide's prompt holds W + 1 − n tokens before a reply of n tokens: 172 to 187 at a7r, and 2,028 to 2,043 at (L), for replies of 6 to 21 tokens.
  - Under blocks, a reply's first token reads anywhere from 1 to W tokens, so neither rule's prompt is always the longer.
  - Every slide query in the run used this. The sheet now records it under "Changes after the run".
- **The memory adds nothing past the window (L-010).**
  - On a7r, memory on is worse past W than memory off, under every rule: R1 is 0.0024 to 0.0035 higher, and R2 0.003 to 0.013 lower.
  - On (L), it changes R1 by under 0.0001 and leaves R2 the same.
  - With the memory out, both tables read the same rows.
- **The pre-run review found an instrument defect, fixed before any GPU spend.** The ORDER's blocks ("`build_document_windows` at W, as today") read a conversation's last, shorter chunk from the W tokens before its last target, back into the previous block. That made blocks no forgetting control at conversation tails, and for (L) at every target past W.
  - `build_document_blocks` now reads each block from its own start.
  - Two tests pin it: with the dispatch line set aside, 14 tests passed and 2 failed.
  - This departs from the ORDER's wording, and is recorded in the sheet's "Changes after the pre-run review".
- **The first pod never answered SSH** in 900 s, and was deleted. A second machine ran the job. `uor-pod up` retries offers on a create failure only, not on an SSH failure.

## What changed

- **`hamilton_sheet.rs`:**
  - `build_document_windows_sliding`, which scores each target once, each seeing at least W − s + 1 tokens;
  - `build_document_blocks`, which reads each block from its own start;
  - their tests.
- **`in_space_long.rs`:** `--score-rows --past-window slide|blocks|carry` and `--past-window-stride`. The changes:
  - the per-rule cut and query prompts;
  - `set_context(C)` for carry;
  - the per-band rows, hits and `query_loss` under the checkpoint's own objective (`cell`);
  - the per-band query agreement and recall;
  - the new file `past-window-{split}-{memory|no-memory}.json`;
  - the refusals (Fresh V2, no panel, `--context` other than W, the arms this measurement does not read).
  - Without the option, every output is as at the base. That rests on reading the code (two exact-head reviewers); no CPU test pins it, since `score_rows` runs on the GPU. One refusal order changed: a panel that fails both the windowing and the query checks now reports the windowing error first (#3714).
- **`bin/train-in-space.rs`:** the flags and the header line.
- **The prep sheet,** `docs/harness/prep/integer-long-context-2026-11-05.md`, with its FIT round 1 and the pre-run review's changes.
- **The p-long4k panel** (`chain=4096`, seed 2771) and its census, stored under `results/claude/integer-long-context-2026-11-05/inputs/`.
- **One row** in `docs/research/notebook/m4-serving.md`.
- **The item file:** `stack/session.rs` and `stack/mod.rs` return to the scope for claude-288, and "Where it stands" is updated.

## Measurements

**The run.**
- **Pod:** one RTX 5090 on Vast ($0.563/h), at pin `be2defc00004daa8c020350c6a6f4baeb8aa2fda`.
- **Binary:** `train-in-space-be2defc0`, sha256 `b38eec462ce751e69d4699d24ccdcc8fee8eb5676af22d9b3e39c170a7940c1a`.
- **The job:** all 12 passes, exit 0, $1.70 spent against the $3 bound, with (L)'s slide at stride 128.
- **Sealed** in the store under `results/claude/integer-long-context-2026-11-05/c287/`. Its `SHA256SUMS.files` holds 378 files, sha256 `bd18638e1e78fc925a4ed1aea0c764236c792fcabdb7ef472ac451d34b2f68e4`. Read back 378 of 378 before `down`.

**Instrument checks:**
- **I-0:** 366 passed, 0 failed, 12 ignored, including all 16 named past-window tests.
- **I-1:** passed.
  - DEV's rows files are byte-identical across the three rules.
  - DEV's sample files are byte-identical between slide and blocks. Under carry, one sample of 16 in one of 12 files differs (a7r seed 1, DEV Flock sample 11), from position 194, past 192, as the sheet expected; it is reported only.
- **I-2:** passed. a7r's band 0 `loss_sum` differs from blocks' in the 9th significant digit, for slide and carry alike, inside carry's tolerances. Slide's mismatch is recorded as batch composition, as the sheet decided in advance, and carry's is inside its own tolerances. The pattern is not explained: slide and carry agree bit for bit, and blocks is the one that differs.
- **I-3 to I-8:** passed. I-5 and I-6 are also read on the laptop: every file's objective is `cell`, and carry ran at 2,048 and 4,096, in 160 windows.

**The readings, memory on.**
- **R1** is the mean `cell` loss over targets at offset W or more.
- **R2** is the agreement over queries whose note is at least W back.

| | R1 blocks | R1 carry | R1 slide | R2 blocks | R2 carry | R2 slide | row |
|---|---|---|---|---|---|---|---|
| a7r s1 | 0.64982 | 0.65907 | 0.63971 | 0.0267 | 0.0138 | 0.0339 | 5 |
| a7r s2 | 0.68516 | 0.68708 | 0.67276 | 0.0255 | 0.0255 | 0.0244 | |
| (L) s1 | 0.69067 | 0.68556 | 0.68529 | 0.0305 | 0.0406 | 0.0307 | 4 |
| (L) s2 | 0.69695 | 0.69061 | 0.69049 | 0.0375 | 0.0408 | 0.0359 | |

**The bands:**
- **a7r:** b_loss 0.0197 (the ceiling) and b_agree 0.0100.
- **(L):** b_loss 0.0069 and b_agree 0.0100.

**By band (R3):**
- **On a7r,** carry sits about 0.01 to 0.03 above slide from 384 to 1,920, and below slide and blocks in the last band, 1,920 to 2,048.
- **On (L),** carry and slide agree within 0.0003, and carry's agreement is the highest at both seeds.

**Reported, never deciding:**
- **R4, nearest hit rate past W** (blocks / carry / slide): a7r 0.678 / 0.667 / 0.705 and 0.671 / 0.659 / 0.697; (L) 0.5765 / 0.5788 / 0.5773 and 0.5760 / 0.5769 / 0.5763. Writer hits are 1.3–2.9% under every rule.
- **R5:** 0 census-clean far words stated, under every rule and seed.
- **`blocks_crossing`:** 32 of a7r's blocks replies past W cross their block's end, and none of (L)'s.

**Laptop readout:** `local/claude/claude-287/score.sh`, a bash and jq script, gitignored. It runs the instrument checks first, then the sheet's fixed jq, then the table. The fresh decision reviewer re-derived its bands, comparisons and rows. Its I-1 sample, I-5, I-6 and R5 checks were added at that review's request.

## Checks run at the head

- **The laptop checks:**
  - the float crate's library suite;
  - the stub-nvcc check;
  - the 16 past-window tests, with the negative gate for the blocks fix (14 passed and 2 failed with it set aside);
  - `uor-harness loop gates --full`;
  - `loop report --check`.
- **On the pod at the pin:** the CUDA library suite (I-0).

## Skills used

None beyond the loop's own steps.

## Still open

- **claude-288, done-whens 3 to 5:**
  - the integer session serves carry, with a test against the float forward's carry past 192 and a negative gate;
  - tok/s and resident memory against context out to 8,192;
  - the pipeline map's gap 6.
- **The row is provisional.** The sheet re-reads row 4 on the first checkpoint that recalls far notes inside its window; the decision review applies the same re-read to a7r's row 5.
- **The served long-context model.** a7r is read only out to 2,048, so serving it to 4,096 rests on (L)'s row. Which checkpoint serves long conversations is P5's to schedule. A reading that would separate the trained window from the other differences is a7r under carry at twice its window, the stretch (L) was read at.
- **`uor-pod up`** should try the next offer when SSH never answers. Noted, not fixed here.
- **#3714:** refusals the scorer should make (a missing run config, a short losses vector, carry over panels of different C, blocks at W = 0), and the default path's missing test. None touched this run; the next unit that runs the scorer lands them.

## Plain summary

The served chatbot stops after 192 tokens. Three ways to go further were measured on the GPU, on two models and two seeds each:
- **slide:** re-read the last 192 tokens for every new one;
- **blocks:** start fresh every 192;
- **carry:** let the model's running state carry through the whole conversation.

On the model trained on 2,048-token windows, carry is as good as sliding and at least as good as starting fresh, and it carries its state through the whole conversation. On the model trained on 192-token windows, carry is worse at one of its two seeds, with less than half of slide's reply agreement. So the chatbot will carry its state, provisionally. The next unit makes the served chatbot do that and times it out to 8,192 tokens.

None of the models yet use what was said long ago in a conversation, so the test cannot yet show that any rule remembers better. The choice is read again on the first checkpoint that does.
