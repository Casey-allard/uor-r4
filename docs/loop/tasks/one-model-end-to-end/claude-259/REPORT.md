# REPORT: one-model-end-to-end, claude-259 (the capacity sweep, trained as served)

Lab: claude. [ORDER](./ORDER.md). Sheet: [`one-model-end-to-end-capacity-2026-11-02.md`](../../../../harness/prep/one-model-end-to-end-capacity-2026-11-02.md). PR #3558, item #3407. Outputs: HF store `results/claude/one-model-end-to-end-capacity-2026-11-02/s1`, `s2` and `s3`, each a tar with `SHA256SUMS`, and `served/` (the six exports, with `SHA256SUMS`). Served evidence: [`docs/labs/one-model-end-to-end/capacity-2026-11-02/`](../../../../labs/one-model-end-to-end/capacity-2026-11-02/README.md).

**Reviews before the run.** The sheet's FIT (round 2, "TEST FITNESS: FIT") and its pre-run review (D67, PASS) are #3407 comment 6581697644, at sheet `47e1c5451` (its fixes at `f596a5e63`). The round-2 amendment (`4708039a0`, layout only, (W) dropped) came after that FIT. It was announced on #3407 (6582209024) and was not re-FIT.

**In one paragraph.** At this data and recipe, neither wider arm carried. No arm beat (b) on R1 at all three seeds (decision row 3), and (W), width 1,024, was not measured. (h) was worse than (b) at seeds 1 and 2. (w) has the most copies at every seed, but its R1 verdicts are better, worse and level, and it fails the sheet's R5 rule at seed 2. It stays the row-3 candidate, open until the width-512 export is repaired. No run was still rising: every rise from step 4,096 to 8,192 is below the bar of 143. Of DEV's strata, openings (601–605 of 1,226) and the rest (8,177–8,245 of 9,094) hold still in all nine runs; copies (2,002–3,140 of 4,130) carry the spread. In one served turn (b-s2, conv-a turn 3), a trace of the copy read shows each copy head keeping 16 earlier positions, led by the five that follow an earlier " is"; three of those five lead to a wrong value. The read keys an earlier position by the one token before it, and takes the mean of what it keeps (from the code). The working base for the next round stays (b), and that round changes the key.

## Tokens per second

- **Served decode on the M1** (`uor-chat --protocol 2`, a 177-token run minus a 1-token run, medians of 5): (b) 1,281, 1,381 and 1,385 tok/s at seeds 1–3; (h) 1,265 and 1,282 at seeds 2 and 3, and no usable reading at seed 1 (negative 13). (w) cannot be served (negative 9).
- **Training on a shared RTX 5090** (context tokens a second): (b) 10,730–12,618, (h) 8,947–10,168, (w) 7,414–9,592.

## What did not work, first

**The model's readings.**
- **A. No arm carried.** None was better than (b) on R1 at all three seeds (the table below).
- **B. (h), width 256 with 8 heads, was worse than (b)** on R1 at seeds 1 and 2 (−65 and −41, each past its floor) and level at seed 3. It also fails the sheet's R5 rule at seeds 1 and 2.
- **C. (w), width 512 with 8 heads, did not hold across seeds.** Its R1 verdicts are better, worse and level, and it fails the sheet's R5 rule (b − c above 0) at seed 2 (16 − 27 = −11).
- **D. The served replies are fluent but carry wrong values** at every export (the transcripts below).

**The run and its instruments,** each classed:

1. **Round 1's smoke failed on GPU memory.** This was an instrument fault in my layout, not a reading of any arm.
   - (W), width 1,024 with 16 heads, ran out of memory at 49 s alone on a 32 GB RTX 5090.
   - (b) and (w) ran out together while sharing GPU 0 three ways with (h).
   - Round 2 changed only the layout (sheet amendment `4708039a0`; #3407 comment 6582209024).
2. **(W) was dropped** under the ORDER's stop rule. Class: instrument (it does not fit a 32 GB card at batch 16), not a reading of width 1,024. It is reported as absent, not level, so decision row 2 cannot be reached. The width-1,024 question stays open for a larger card or a recipe change.
3. **Seed 1's first pod never opened SSH.** It was deleted after 900 s, about $0.45.
4. **Pod prices rose between the review and the launch:** $6.28/h for the three pods, against the sheet's $4.20/h. That is still inside the $8/h cap.
5. **I never projected the scoring time.** The round-2 review flagged it, and I answered that the 90 minutes bound only training. Scoring is CPU-bound: about 0.77 s per greedy sample on one core, with the GPU at about 22%. One arm's score plus snapshot took 3,439–10,037 s; seed 1's pod ran 5.99 h.
   - The watcher's $24 budget and 4 h limit would have stopped a healthy run. I raised them to $45 and 8 h and noted it on #3407 (6583833708).
   - This is the second cost overrun, after round 1's.
6. **(w)'s training on seed 1 took 3,677 s,** just past the ORDER's "about an hour per run" line. It was not cut, because steps had to stay equal across arms (FIT round 1, blocker 3).
7. **Step 1's chat negatives were an instrument fault.** uor-chat's default `--protocol 1` mis-tokenizes every turn: a lone space token after the colon, which the corpus places almost only before digits.
   - Step 1's "the model does not reply" was withdrawn on #3407 (6584644252).
   - This unit serves with `--protocol 2`.
   - Lesson L-007 again.
8. **Round 1's stop record** (`round1-stopped.txt`, seed 3), verbatim: "every arm ran out of GPU memory in the smoke at 16:19:56Z; the job then hung in a bare wait (no arm left to score, so it waited on its nvidia-smi logger); claude stopped pids 910 and 912 at 16:28:57Z".
9. **(w) cannot be exported, so its R6 is an instrument fault, not a reading** (the sheet's I-3 rule). `fbs-export` refuses (w) at all three seeds with "the head width exceeds 256".
   - The FBS export writes its trunk's shape with `heads: 1` (`crates/uor-r4-integer/src/stack/format.rs:1575`). An all-recurrence trunk has no attention heads; the Hamming read's heads are its own section.
   - The shape check `width / heads > 256` (`format.rs:508`) then refuses every FBS trunk wider than 256.
   - The ORDER asked for a smoke that "runs `run_long` and exports". The sheet's I-1 listed the snapshot but not the export, and I exported only (b)'s smoke on the laptop. A smoke export of (w) would have found this before the run. R1 to R5 stand, because they are read from the GPU's files.
   - The repair is code outside this unit's scope: its own unit.
10. **The watcher's spend line was wrong after the first pod went down.** I wrote the down markers as clock times (`19:39Z`), not epochs. The watcher's subtraction failed (`watch.sh` line 23, "syntax error in expression"), and it printed $7.57 to $7.84 where the true spend was about $22. The markers were rewritten as epochs at about 19:50Z. No stop depended on it: the budget was $45.
11. **The watcher's stall stop fired on a healthy pod** at 21:43:40Z on seed 1. Arm (h) was still scoring: one process at 100% CPU, the GPU at 13%. The greedy pass without memory writes no file until it ends, and on (b) that pass took 28 minutes. The 30-minute window did not fit the scorer's silent passes. The watcher only exits, so the pod was untouched. I re-armed it with a 90-minute window and renewed the lease (#3407, 6589239237).
12. **`grade.sh` read the dropped arm (W) as (w).** macOS's disk ignores case, so `out/W-scored` opens `out/w-scored`. The grader printed W rows equal to w's instead of "absent". Those rows are discarded here; (W) is absent. No reading of (b), (h) or (w) changes.
13. **h-s1's decode tok/s is not a usable reading.** Class: instrument (the laptop's load), not a reading of (h). Two runs of the protocol gave 610 and 1,659 tok/s at a load average of 5.7. Its 177-token runs ranged from 0.44 to 1.17 s. b-s1's runs were stable (0.397–0.417 s), and h-s2 and h-s3 read 1,265 and 1,282.
14. **The decision table's row-3 unlock cannot run yet.** Its candidate is (w), which cannot be exported (negative 9), so its three more seeds could not be read on served files. Class: instrument. The decision review below orders the unlock after the export repair.
15. **b-s2's parity summary line was overwritten.** A second `parity.sh` run refused to overwrite `b-s2-toks.json` (os error 17) and wrote that refusal over `b-s2-toks.out`. I re-ran `fbs-toks` on b-s2 into a new file: its output is byte-identical to the first run's, and it reports 26 capped of 310. The committed `parity/b-s2-toks.out` is that re-run's.

## What worked

- Every arm that fit in memory trained, scored and snapshotted at every seed (all exit codes 0), and no seed was cut.
- (b) and (h) exported at every seed and pass I-3: 0–1 first tokens of 310 differ from the float file.
- The sealed outputs are in the HF store, each tar's sha256 matched between the pod and the laptop.

## What changed

Documents only; no code changed in this unit.
- The sheet, `docs/harness/prep/one-model-end-to-end-capacity-2026-11-02.md`, with its round-2 amendment.
- This REPORT and the ORDER, in `docs/loop/tasks/one-model-end-to-end/claude-259/`.
- One row in `docs/research/notebook/float-training.md`.
- The served evidence, in `docs/labs/one-model-end-to-end/capacity-2026-11-02/`: the three conversations, the 18 transcripts, the decode timings, the parity summaries and the trace.

## Measurements

No seed was cut (`steps.txt` holds no `CUT`); every arm ran 8,192 steps at every seed.

### Training and DEV (R4), all nine runs

DEV scores 14,287 nearest-readout rows (`dev_scored_rows`). Of them, 1,226 open a reply, 4,130 copy the prompt, and 9,094 do neither (the rest). The first two overlap: 163 rows both open a reply and copy, so the three counts sum to 14,450 (`fold_picks`, `crates/uor-r4-float-train/src/in_space_long.rs:1098`). "Rise" is the gain from step 4,096 to 8,192; R4's bar for "still rising" is more than 143.

| Arm | Seed | Train wall (s) | Context tok/s (shared GPU) | DEV nearest | Rise | Openings | Copies | Rest | Writer hits |
|---|---|---|---|---|---|---|---|---|---|
| b 256/4 | 1 | 2,567 | 10,730 | 11,627 | +116 | 603 | 2,856 | 8,245 | 482 |
| h 256/8 | 1 | 3,101 | 8,947 | 10,726 | +98 | 601 | 2,002 | 8,197 | 436 |
| w 512/8 | 1 | 3,677 | 7,414 | 11,851 | +37 | 603 | 3,097 | 8,228 | 434 |
| b | 2 | 1,994 | 12,618 | 11,835 | +34 | 603 | 3,119 | 8,190 | 596 |
| h | 2 | 2,475 | 10,168 | 11,849 | +42 | 604 | 3,104 | 8,218 | 357 |
| w | 2 | 2,623 | 9,592 | 11,866 | +58 | 602 | 3,140 | 8,201 | 334 |
| b | 3 | 1,998 | 12,591 | 11,788 | +64 | 605 | 3,073 | 8,188 | 191 |
| h | 3 | 2,487 | 10,116 | 11,771 | +12 | 602 | 3,069 | 8,177 | 228 |
| w | 3 | 2,629 | 9,571 | 11,884 | +79 | 605 | 3,130 | 8,227 | 245 |

- **R4: no arm is still rising** at any seed. The largest rise is +116, against the bar of 143.
- **Most of the gain comes within one pass over TRAIN.** At step 2,048, every run's DEV nearest is under 2.4% below its final value. b-s1 gains the most (11,357 to 11,627, 2.3%); five of the nine gain under 1%. A step reads 16 × 192 = 3,072 tokens, so step 2,048 is about 0.84 of TRAIN's 7,512,829 ids (arithmetic).
- **Openings and the rest hold still; copies carry the spread.** Openings sit at 601–605 in all nine runs and the rest at 8,177–8,245. Copies range from 2,002 to 3,140. (w) has the most copies at every seed (3,097, 3,140 and 3,130, against (b)'s 2,856, 3,119 and 3,073). Before the turn memory, claude-153 saw width 512's copies clear its bars at one of two seeds ([REPORT](../../in-space-trunk-to-text/claude-153/REPORT.md)). h-s1's low total is its copies.
- **The writer fails at every arm** (191 to 596 of 14,287). That is writer-admission's (#3394) problem, unchanged by size.

### R1, R2 and R3 on p-w (1,202 prompts), paired against (b) at each seed

Each cell is b − c, then the verdict (floor 2·√(b + c), with b + c ≥ 10).

| Arm | Seed | R1 right prefix | R2 exact | R3 opening |
|---|---|---|---|---|
| h | 1 | 22 − 87 = −65, worse | 0 − 4, level | 18 − 79 = −61, worse |
| h | 2 | 36 − 77 = −41, worse | 3 − 10, level | 23 − 47 = −24, worse |
| h | 3 | 41 − 41 = 0, level | 5 − 1, level | 30 − 30, level |
| w | 1 | 89 − 51 = +38, better | 10 − 3, level | 71 − 49 = +22, better |
| w | 2 | 41 − 75 = −34, worse | 3 − 8, level | 24 − 57 = −33, worse |
| w | 3 | 53 − 43 = +10, level | 4 − 2, level | 41 − 22 = +19, better |

- **Summed R1:** (h) −106, (w) +14.
- **The seed matters more than the arm.** (b)'s own right-prefix sum on p-w is 537, 989 and 643 at seeds 1, 2 and 3. Exact replies stay at 4–15 of 1,202 in every run.

### R5 on p-v (393 prompts), paired against (b) at each seed

The sheet's R5 rule is b − c above 0 at every seed (row 1). The verdict column uses R1's floor, 2·√(b + c), for comparison.

| Arm | Seed | b | c | b − c | Floor | Verdict | R5 rule (b − c above 0) |
|---|---|---|---|---|---|---|---|
| h | 1 | 7 | 26 | −19 | 11.49 | worse | fails |
| h | 2 | 9 | 31 | −22 | 12.65 | worse | fails |
| h | 3 | 18 | 9 | +9 | 10.39 | level | passes |
| w | 1 | 28 | 15 | +13 | 13.11 | level | passes |
| w | 2 | 16 | 27 | −11 | 13.11 | level | fails |
| w | 3 | 21 | 8 | +13 | 10.77 | better | passes |

Neither arm passes R5 at every seed.

### The decision table's reading

- **Row 1 is not reached:** no arm is better at all three seeds.
- **Row 2 is not reached:** (W) is absent, and (w) is better at seed 1.
- **Row 3, keep and iterate.** The rule's candidate is (w), the largest summed R1 above 0 (+14 from +38, −34 and +10).

**My decision review (reversible by Casey).** The row's unlock is "the next unit runs it to its plateau at 3 more seeds before anything ships". It cannot run yet, because (w) cannot be exported (negative 9), so its served half (R6, I-3) could not be read. That is an instrument fault, so it neither closes (w) nor counts against it.
- **(w) stays the row-3 candidate, open.** Its readings so far: R1 +38 better, −34 worse and +10 level (sum +14); R5 fails at seed 2; the most copies at every seed; rises of +37, +58 and +79, each below the bar of 143.
- **The width-512 export repair comes first, as its own unit** (`format.rs:508`, `:1575`). (w)'s three more seeds follow it, so they are read on served files as well as on the GPU's.
- **The working base for the next round is (b), width 256 with 4 heads,** for cost and served-ability. It trains fastest (10,730–12,618 context tok/s, against (w)'s 7,414–9,592), serves at 1,281–1,385 tok/s, and exports today. That is a choice of where to build next, not a verdict on (w).
- **The next lever is the copy read's key,** from the trace below. Copies are the stratum that moves, and the served replies fail there.

This closes (h) only, at this data and recipe (CHARTER §4.1): it was worse than (b) on R1 at seeds 1 and 2 and level at seed 3, and fails R5 at seeds 1 and 2. (w) is inconclusive. (W) was never measured and stays open. It closes no mechanism.

### Served (R6), `uor-chat --protocol 2 --max-tokens 64`

| Export | Decode tok/s | Load average |
|---|---|---|
| b-s1 | 1,281 | 5.7 |
| h-s1 | not read (610 and 1,659; negative 13) | 5.7–5.8 |
| b-s2 | 1,381 | 11.5 |
| b-s3 | 1,385 | 10.9 |
| h-s2 | 1,265 | 10.3 |
| h-s3 | 1,282 | 9.7 |

(w) refused to export at every seed (negative 9). Decode uses step 1's method: a 177-token run minus a 1-token run, each the median of 5 runs, timed with zsh's `EPOCHREALTIME`. The conversations, transcripts and timings are in [`docs/labs/one-model-end-to-end/capacity-2026-11-02/`](../../../../labs/one-model-end-to-end/capacity-2026-11-02/README.md).

**The transcripts read the same at every arm and seed.** The replies are fluent and in the right form, but the values are wrong:
- conv-a: "Got it, your lucky number is 47." / "Your lucky number is 47." / then "Got it, your friend is 47." for Maya / "Your friend is 47." / a context reset / "You grew up in Lismouth." (h-s1: "Got it, your friend is.").
- conv-c: "Hey! Nice to hear from you." is right. "The opposite of hard is soft." (b-s1, b-s3) is right; the others say "happy", "blue" or "dirty". "Repeat after me: red and green." gets "The cat is happy." "I have a dog called Cheedse." gets "Got it, your dog is" and then a wrong or broken value.
- conv-b: "hello", "I'm doing well, thank you!" or "Pretty good, thank you!", then loops. b-s1 opens with a broken line, and h-s1 repeats one broken line three times.
- **The 192-token context resets the conversation at turn 5** of conv-a and conv-c.

### I-3 parity

First tokens that differ from the float file, out of 310 DEV prompts. The sheet's bar is at most 9 (`parity.sh` printed 8, the same rule rounded down; no export came near either).

| Export | First tokens that differ | Whole replies that differ |
|---|---|---|
| b-s1 | 0 | 97 |
| h-s1 | 0 | 30 |
| b-s2 | 0 | 26 |
| b-s3 | 1 | 29 |
| h-s2 | 0 | 32 |
| h-s3 | 0 | 46 |

`fbs-toks` reports 26 replies capped at every export. The order matched on every prompt.

### Cost

Pods: $28.00 (seed 1: 5.99 h at $2.33/h; seed 2: 3.49 h at $1.775/h; seed 3: 3.61 h at $2.171/h), plus $0.45 for seed 1's first pod. About $28.45 in all, against the $45 budget. All three pods are deleted.

**Scoring was the run's critical path** (negative 5; Codex's FYI, relayed by Casey on 2 Nov). `job.sh` scores (b) and then (h) one after the other on GPU 0, each CPU-bound at about 22% GPU. The next sheet budgets from a timed scoring smoke.

## Also found during this unit (reported, not acted on in scope)

- **Why the value was wrong in one served turn: the copy read kept the right position and could not prefer it.** The trace was run on the laptop with Casey's go ("um yes I think so"), using an existing instrument. It covers one export and one turn; the other exports and turns were not traced.
  - `fbs-toks served/b-s2.uorlut --hamming-trace` ran over the 86 ids that uor-chat served in conv-a turn 3 (the evidence folder's `trace/`; trace sha `e7a8e73b6c6e4ce4587038006496386021977b581f345fd8b5c9a81a88fe0a23`).
  - At the reply's " is" (position 80), each copy head at both depths keeps 16 earlier positions, and the five that follow an earlier " is" lead its list. Three of those lead to 47 (positions 8, 26 and 51), one to " my" (37) and one to Maya (64). The served next id was the space before 47.
  - The other 11 kept positions' distances, and so their weights, were not read here: the trace holds each head's 64-bit codes, not its distances or margin.
  - From the code: the key is the one token before a position (`crates/uor-r4-float-train/src/flock_read_hamming.rs:14-19`), and a head weights each kept position `max(0, m − d) / m` and reports their mean (`:33-36`). A two-token key (" friend", " is") matches only position 64.
  - The same frame-not-slot failure was recorded on the 29M pointer in October (`docs/mechanisms/pointer-key.md`), where a key naming the slot was never built.
- **The training corpus holds no memory lines.** That corrects step 1's REPORT.
  - In the pinned TRAIN (`mw-balp/train/tokens.u16`, 7,512,829 ids), the " Memory:" sequence (ids 379 970 2852 28) occurs 0 times. DEV (`ft/dev`, 42,888 ids) holds 164.
  - Step 1's REPORT (m4-export-uorlut01-cli/claude-259, #3559) says "In the training world, every recall question carries a system turn". That holds for M-world v2's evaluate world and for DEV, not for the corpus this model trained on.
  - `m-world corpus world=v2` already takes `recall=oracle` (`crates/uor-r4-training/examples/m-world.rs:13-17`), so a TRAIN with memory lines needs no new code.
- **The data cannot teach a long context.** TRAIN conversations have a median of 117 tokens and a maximum of 248; DEV's maximum is 258. The generator caps episodes at 256 (`milestone_world_v2.rs:96`). Casey made a realistic context a path step (#3471).
- **The speed sweep** (the lab's `next-round/speed-targets.md`):
  - no dense matrix product in the served FBS step;
  - the whole-table L1 nearest is 64% of a served token;
  - in training, the launch-bound backward is 62% of a step, and the table search writes 256 MiB intermediates.
- **Main's `--train-as-served 1` grew while these runs ran.**
  - These runs trained under claude-258's form: the export's own rounding of the trunk tables, the Hamming reads and the turn-memory weights before each step, with float master values kept for the optimizers.
  - deepseek-3 (#3563, merged 18:23Z on 2 Nov) and deepseek-4 (#3565, merged 18:45Z on 2 Nov) added served surrogates in the forward pass: D11 snaps, Q16 residuals, dynamic i16 activations and the Q12 scored query ([deepseek-4 REPORT](../deepseek-4/REPORT.md)). Neither is in this unit's history.
  - So "trained as served" in this REPORT means claude-258's form, and the I-3 parity above is that form's. The next round trains with main's.

## Checks run at the head

- `grade.sh` on p-w and p-v over all three seeds (`(W)` rows discarded, negative 12); R4 read with `jq` from each run's `progress.jsonl`. At this head, `bash grade.sh … p-v` was re-run: its six (h)/(w)-against-(b) prefix rows match the R5 table cell for cell.
- `fbs-export` for every arm at seeds 1–3: (b) and (h) exported, (w) refused at each seed. `parity.sh` on the six exports: the table above. b-s2's `fbs-toks` re-run: byte-identical to the first run (`cmp`), 26 capped of 310 (negative 15).
- The six exports' sha256s, in `SHA256SUMS` beside them in the HF store's `served/`, and in the evidence README.
- Each seed's tar: sha256 on the pod equal to the laptop's (seed 1 compared by script), then stored with `SHA256SUMS`.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and the fresh reviews: recorded on the PR at the head.

## Skills used

None.

## Still open

One unit each, in this order (my decision, logged for Casey):
1. **The copy read's key, trained as served on (b):** an ordered two-token key and a slot key against the current read, with memory lines in TRAIN (`recall=oracle`), at 3 seeds, timed from a scoring smoke. The item's scope already holds `flock_read_hamming.rs` and `m-world.rs`; the integer read (`stack/hamming.rs`) must be added.
2. **The width-512 export** (`format.rs:508`), then (w)'s three more seeds, the row-3 unlock.
3. **A realistic context** past 192 tokens (#3471), then the served memory line.
4. **The writer's admission** (#3394) and the batched scorer.

The item stays open: its own bars (end to end on unseen prompts, with tokens per second) are not met. References #3407.

## Plain summary

We trained the model at three sizes, three times each, exactly as it is served, and talked to the results on the laptop. The largest size did not fit the graphics card, so it was not measured. Neither of the other two beat the smallest at every seed on this training data, and none was still improving at the end. The middle size copies the most values from earlier in the chat, but it cannot be exported to the laptop yet, so it gets three more runs once that is fixed. Meanwhile we keep building on the smallest model, which is the cheapest to train and serve. In one served chat we looked inside to see why the model said "your friend is 47". When it looked back, the first places it kept were the five that came after an earlier "is": three of them were followed by 47 and one by Maya, and it blends what it keeps. It can only look things up by one word. The next round teaches it to look things up by the words that name the fact.
