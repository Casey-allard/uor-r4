# REPORT: one-model-end-to-end, claude-358

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3948.

The retrain the [sheet](../../../../harness/prep/one-model-end-to-end-copy-longest-2026-11-12.md) pre-registers: three seeds of the place-read model and one control with no place read, each trained 16,384 steps with `--hamming-copy-longest 714,28` on one pod, then exported and read on the laptop as integer artifacts.

**The sheet's row is 1, Better, replicated.** On the reading panel's 578 recall questions the retrained models state the remembered value on 314, 320 and 325, against 276, 296 and 265 for stage 3's models served with the same rule. The mean gain M is +40.7 against a bar of +29, and all three seeds gain. **The gain is on single recall questions on a panel. The nine-turn conversation read word for word is not better (below).** No milestone moves on this run.

| Gate | Reading | Status |
|---|---|---|
| G1 | the sealed `pre` stage passed every pod check; job 0, seal 0 | holds |
| G2 | five stages sealed and read back before the pod went down; the four checks passed (`watch-try-1.log`) | holds |
| G3 | I-f and I-u read on all four artifacts at 03:40Z on 13 Nov (`check-try-1.log`); the earliest file of any later unit that reads them dates from 05:51Z | holds |
| G4 | R, O, D, M, the row, replies word for word and the negatives are below, each from a named log | holds |
| G5 | one notebook row | holds |

Every log named here is in the kit, `local/claude/claude-358/`, which is not committed. The sealed stages are in the store under `results/claude/one-model-end-to-end-copy-longest-2026-11-12/run/<stage>/try-1`.

## What did not work, first

1. **The sheet's reading ran seven hours late, after later units had already used the models.** The pod went down at 03:11:48Z on 13 Nov and the instrument checks were read at 03:40Z. The session then went on to claude-359, claude-360 and claude-361, which read replies from the three retrained p artifacts as their baselines from about 06:00Z. This unit's own reading, the step that calls the row, ran at 10:11Z. The row came out Better, which is what licenses those artifacts as the line's models. Had it come out Level or Worse, three units' baselines would have rested on models the sheet did not license. The order the sheet asks for inside this unit (I-f and I-u before any reply) did hold.
2. **The conversation is not better.** Seed 2's retrained model against stage 3's, nine turns, served on the laptop (`extras-try-1b/chat-table.txt`; the stage 3 column is stage 3's model served with the same rule, and its reply ids are identical to claude-356's record, `extras-try-1b/compare-356.txt`):

   | Turn | User | Retrained | Stage 3 |
   |---|---|---|---|
   | 1 | Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18. | "Got it." | "Got it." |
   | 2 | What is mibtorn? | "Mibtorn?" | "Mibtorn is felsharp, quonzet is 18." |
   | 3 | What does a cow say? | "A cow says moo." | "A cow says moo." |
   | 4 | Say exactly this: trelvon dask pimmerholt zane | "trelvon dask pimmerholt zane\nAssistant." | "trelvon dask pimmerholt zane\nA" |
   | 5 | What is druvalk? | "That's 6673." | "That's 603." |
   | 6 | I'm tired today. | "Okay, I'll remember that." | "I'm a helpful assistant that runs on your computer." |
   | 7 | Tell me what quonzet was. | "It's 18." | "It's 18." |
   | 8 | What is two plus three? | "It's 18." | "I'm a helpful assistant that runs on your computer." |
   | 9 | What is my name? | "You're welcome!" | "You're welcome!" |

   The retrained model loses turn 2, which stage 3 answered. Turn 5 is wrong on both. Turn 8 repeats turn 7's answer. Both run on after the copied string in turn 4. Neither says it was not told a name in turn 9.
3. **The control gained too.** n-s2, which has no place read, went from 108 to 139 (D +31; 44 gained, 13 lost). That is one control seed from a lower start, so it sizes nothing, but with it this run does not show that the place read is what the rule helps. In the control the replies with no end token in 48 tokens rose from 102 to 131.
4. **Each retrained model also loses rows stage 3 had:** 51, 34 and 35 recall rows on the three seeds. Seed 2, word for word (`read-try-1/p-s2.*.own.jsonl`):

   | Row | Asked | Reference | Retrained | Stage 3 |
   |---|---|---|---|---|
   | 41 | "...kleerdran was?" | "Birkleerdran is roumklend." | "That's trerk." | "That's roumklend." |
   | 44 | "What was jortraid again?" | "It's 9029." | "Meet care care thatkaykaykay..." to the 48-token cap | "The day after Wednesday is 9029." |
   | 46 | "...remember what drundgil was?" | "It's chostdir." | "That's ch" | "That's chostdir." |

   And three it gained:

   | Row | Asked | Reference | Retrained | Stage 3 |
   |---|---|---|---|---|
   | 21 | "...was greerkgun again?" | "That's furtal." | "Greerkgun is furtal." | "Greerks for for for..." to the cap |
   | 32 | "What's my code word?" | "It is hudganku." | "The code word is hudganku." | "The code word is hu." |
   | 45 | "...what sterbras was?" | "Sterbras is 7290." | "That's 7290." | "That's 729." |

5. **The retrained models are worse than stage 3's when served with the old rule.** Under `--copy-rule latest-exact` they read 247, 254 and 257, below stage 3's 276, 296 and 265 under the new rule. The artifact carries its rule in the header and the reader serves it by default, so the product path is not affected, but a reader that overrides the rule reads 66 to 68 fewer.
6. **The repeat-a-string gain is one seed's.** Exact repeats of 104: seed 3 went from 1 to 36; seed 2 from 1 to 4; seed 1 from 3 to 6. That is +35 of seed 3's O gain of +38.
7. **Seed 3's row check is the lowest:** I-f 0.9860 against the 0.98 floor (572 of 40,719 scored rows differ from the float rows); the others are 0.9908 to 0.9917.
8. **The prediction missed low.** Written before the run: M between 0 and +30, Level the most likely row. M read +40.7, and seed 3's D of +60 is outside the predicted range for a job (−10 to +40). The size was guessed from the 26 to 33 replies the rule changed when only served; trained, it changed 503 to 544 of 979 replies per seed.
9. **The extras script's conversation step failed** (`extras-try-1.log`): `extras-358.sh` calls the chat reader on an artifact without `--tokenizer`. Its two "DIFFERENT reply ids" lines compare the failed step's empty output and mean nothing. The three conversations were rerun by hand with the flag into `extras-try-1b/`, and the comparison was rerun there. The script in the kit is not repaired.
10. **Which of the flag's two training changes gives the gain is not shown.** The flag changes the copy read's choice and the leave rule together; no run separates them.

## What changed

Nothing in the product source. This unit ran a pre-registered training and its reading.

- **Pod:** one RTX 5090 on Vast (Alberta), $1.042/h, `vast-60424033`, through `scripts/pod/uor-pod`. SSH was ready 34 s after creation. The chain ran `pre`, `p-s2`, `n-s2`, `p-s1`, `p-s3`; every stage ended job 0, seal 0. Training took 1,968 to 2,080 s per job. The pod went down at 03:11:48Z on 13 Nov after its four checks, at $3.07. The sheet expected $3.60 to $5.50.
- **Pin:** `f694d162d9b04d1226fd17c0a81825f00e9da007` (claude-357's merge commit).
- **Before launch:** a fresh reviewer who did not write `job-358.sh` read it against the sheet's pod checks (`prerun-review.md`): PASS with three findings, all fixed before the pod went up (the set-aside's scratch copy is now deleted on every exit; `fetch-358.sh` and `check-358.sh` now show a failed scoring pass; a stop message names both of its causes).
- **Exports** (SHA-256): p-s2 `f4667934627e99d709533f30505058fdf07912ddf3cfc9ade6ded5f415e8c680`, p-s1 `9a209883ff74cdeb6936fc11714d4dd4bb63947775609754c6a5f678e39df9bd`, p-s3 `9615060f4db98a4cc5b21220e454428b816cda1d47256cf85091963692e83d20`, n-s2 `056e8a5fa2d55d4027e658ab90c72f150a972282749339edcffe91b2bb227e3d`.

## Measurements

**The bar's reading** (`read-try-1.log`, `read-try-1/counts.txt`). The seed 16 reading panel, whole conversation as the prompt, 48 tokens at most. R is the recall questions (578) whose reply holds the told value as a whole word. O is the other 401 rows (297 other replies and 104 repeat-a-string rows) answered exactly. Stage 3's models are served with the same rule, so D is what the training adds beyond serving.

| Job | R retrained | R stage 3 | D | gained / lost | O retrained | O stage 3 | no end token in 48 |
|---|---|---|---|---|---|---|---|
| p-s2 | 314 | 276 | **+38** | 89 / 51 | 74 | 73 | 120 → 87 |
| p-s1 | 320 | 296 | **+24** | 58 / 34 | 76 | 74 | 127 → 109 |
| p-s3 | 325 | 265 | **+60** | 95 / 35 | 105 | 67 | 125 → 63 |
| n-s2 (control) | 139 | 108 | +31 | 44 / 13 | 76 | 72 | 102 → 131 |

Sum of D over the p jobs 122; **M = +40.7**. The sheet's bars: M at or above +29 (met), at least two D above 0 (three of three), no O down by more than 10 (none is down). **Row 1, Better; replicated, since all three D are above 0.**

**Reported readings, no bar.**

| Reading | p-s2 | p-s1 | p-s3 | n-s2 |
|---|---|---|---|---|
| R under `--copy-rule latest-exact` at serving | 247 | 254 | 257 | 82 |
| Replies changed by that override, of 979 | 225 | 227 | 228 | 208 |
| Exact repeats of 104 | 1 → 4 | 3 → 6 | 1 → 36 | 5 → 7 |
| Float pass, replies equal to the reference of 1,108 on the scoring panel | 187 | 187 | 224 | 157 |

The float pass's count (`float-exact-358.txt`) is a laptop count from each job's `samples-p-greedy-memory.json`, reply against reference up to the end mark. The claude-361 sheet cites 188 for p-s2; the one-row difference is not explained here.

**Speed** (`extras-try-1/toks-*.out`; `fbs-toks --read`, Apple M1, two rounds each). Tokens per second.

| Artifact | Decode, read-prefill | Decode, plain | Context prefill, read | Context prefill, plain |
|---|---|---|---|---|
| p-s2 retrained | 1,567 / 1,636 | 1,564 / 1,622 | 4,309 / 4,499 | 1,225 / 1,292 |
| stage 3's p-s2 with the rule's field | 1,587 / 1,624 | 1,577 / 1,641 | 4,365 / 4,469 | 1,242 / 1,286 |

On this one pair of artifacts, two rounds, one laptop, serving speed is the same.

## Checks run at the head

| Check | Where | Result |
|---|---|---|
| Step 1: I-o on all four comparison artifacts, the patch at the sheet's hash, I-w 578 of 578 | laptop, `step1-358.log` | hold |
| Step 2: I-p, I-g with its set-aside, I-3, three 64-step smokes with I-m, I-s, I-r, I-k, I-n, I-b | pod, the sealed `pre` stage | job 0, seal 0 |
| I-t: 16,384 steps, the last checkpoint, a snapshot and a config | `check-try-1.log` | all four |
| I-m: the trainer's and the scoring pass's marker lines; no not-run or scoring-lost record | `check-try-1.log` | all four |
| I-c: the config equals stage 3's except the flag | `check-try-1.log` | 29 fields equal on the p jobs, 27 on n-s2 |
| I-h: the header field is in the export with the run's config and absent without it; the reply half on the panel | `check-try-1.log`, `read-try-1.log` | all four; 0 rows differ |
| I-f: share of 40,719 scored rows where the artifact picks the float model's token, floor 0.98 | `check-try-1.log` | 0.9911, 0.9917 (n-s2), 0.9908, 0.9860 |
| I-u: the export with the field differs from the float rows on fewer rows than the one without | `check-try-1.log` | 364 vs 1,274; 340 vs 1,427; 375 vs 1,264; 572 vs 1,490 |
| The pod's four checks before down | `watch-try-1.log` | pass |

No source changed, so no build or test suite ran on this branch beyond the docs gates.

## Skills used

None beyond the project loop.

**Reviews.** Each by a fresh `delegate:reviewer` subagent of this Claude Code desktop session.

| Review | Of | Verdict | What was taken |
|---|---|---|---|
| Pre-run | `job-358.sh` against the sheet, before launch | PASS, three findings | all three, before the pod went up |
| Report | the REPORT and the notebook row at `fbd3f70b3`, every figure against its log and the row against the sheet | APPROVE WITH CHANGES: three medium, four low; the row call sound | all seven: two unlogged figures now have kit logs (`extras-try-1b/compare-356.txt`, `float-exact-358.txt`), "67 or 68" corrected to 66 to 68, and four wordings narrowed. Not checked by the reviewer: the six word-for-word rows in item 4 |

## Still open

- **The row's unlock:** the flag joins this line's recipe and the retrained artifacts replace stage 3's as the line's models. claude-359 to claude-361 already build on them.
- **The wrong first token of a value,** which the rule does not touch (the sheet's next step under row 1), and the cut values in rows such as 46.
- **The control's gain** (item 3): how much of M belongs to the place read is not separated.
- **Which training change gives the gain** (item 10).
- **Conversation quality:** turns 2, 5, 8 and 9 above. Questions about things never told and stopping after a copied string are not addressed by this unit.
- **`extras-358.sh`** needs `--tokenizer` on its chat step before it is reused.

## Plain summary

Training with the new copy rule made the model's reply hold the told value on more single "what was X?" questions on the reading panel: about 41 more of 578 on average, on all three seeds, which clears the bar set before the run. A control model with no place read gained about as much, so the run does not show why it helps. In a nine-turn conversation the retrained model is not better: it loses one answer the old model gave and repeats an earlier answer to an unrelated question. The reading that calls the result ran seven hours after the pod finished, after later work had already started using the new models.
