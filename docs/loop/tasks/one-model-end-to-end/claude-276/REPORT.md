# REPORT: one-model-end-to-end, claude-276 (P2b's run: the one model trained at the 2,048 window)

Lab: claude. [ORDER](./ORDER.md). PR #3657, item #3407. Plan step: Casey's plan of 4 November, P2b, the run unit (bound: 1 unit). It ran [P2b's sheet](../../../../harness/prep/one-model-end-to-end-long-context-2026-11-04.md) as pre-registered, at the pin `e1ad88afc5a7f43f5e7714d9ab70b8937aa6bdde`.

**In one paragraph.** This is the first model on this line trained on conversations of up to 2,048 tokens, and it does not state a held-out note's value back. Arm (L) trained at 2,048 × 4 for 8,192 steps on the long TRAIN, and arm (S) at 192 × 16 for 21,845 steps, at seeds 1 and 2, one RTX 5090 a seed. Scored on p-long's 609 recall queries, (L)'s replies state the answer in 0 (seed 1) and 1 (seed 2) of the 603 paired queries with the 2,048 window open, and in 0 with it shut at 192. **R1 fails at both seeds, so the sheet's Bar reads Negative.** That closes this configuration only: the (b) recipe, at 8,192 steps of 2,048 × 4, on this TRAIN. It does not close the window or the read. The sheet gives the classification, instrument or mechanism, to the next unit, and that unit is past P2b's bound (Next). The instrument checks I-0 to I-6 all hold, the run cost about $3.88, and every output is sealed in the store and was read back.

## Tokens per second first

Decode tok/s for each p-long queries file (609 queries, 32 tokens each, 19,488 tokens a file) on one RTX 5090. It is read from the `decode` pair `--score-rows` printed for the file, and each pass ran once:

| pass | memory on | memory off | seconds a file |
|---|---|---|---|
| (L) at 2,048, seed 1 | 15.2 | 15.2 | 1,284.5 / 1,282.3 |
| (L) at 2,048, seed 2 | 15.2 | 15.2 | 1,282.8 / 1,281.1 |
| (L) at 192, seed 1 | 466.2 | 468.9 | 41.8 / 41.6 |
| (L) at 192, seed 2 | 467.4 | 470.4 | 41.7 / 41.4 |
| (S) at 192, seed 1 | 512.7 | 515.8 | 38.0 / 37.8 |
| (S) at 192, seed 2 | 513.8 | 517.2 | 37.9 / 37.7 |

- **At 2,048 the same model decodes these queries about 31× slower than at 192** (15.2 against 466–470, paired by seed and memory setting). Two causes, not separated here:
  - each step re-runs its whole window, which is about 10× longer;
  - at 192 every query's prompt is cut to the same 192 tokens, so 16 decode in one forward. At 2,048 each prompt has its own length, and claude-270's batches take only equal lengths, so each decodes alone.
- **Training, from each arm's trainer clock:**
  - **(L) at 2,048:** 0.6449 and 0.6444 s a step, 12,703 and 12,712 positions a second;
  - **(S) at 192:** 0.1253 and 0.1255 s a step, 24,524 and 24,473 positions a second.

  A training position costs 1.93× as much at 2,048. Against claude-273's 0.6026 s and 0.1349 s, (L) is 7% slower and (S) 7% faster; that is one pass each, on other pods.
- **Time:** by the trainer clock, (L) trained for 5,283 s and 5,279 s, and (S) for 2,736 s and 2,742 s. At seed 1, job.log's step stamps span 5,405 s and 2,882 s, which includes loading. The score passes took 2,624 s and 2,622 s at 2,048, and 148 s and 138 s for each pass at 192.
- **Memory peaks:** training 20,048 and 20,112 MiB for (L), 11,664 and 11,600 MiB for (S). Every scoring pass peaked at 5,296 MiB.
- These are the float scorer's speeds, not the served integer model's.

## What did not work

1. **R1, the deciding reading, fails at both seeds: Negative.** (L) at 2,048 against (L) at 192, memory on, over the 603 paired queries (the census 6 left out):

   | seed | overall n | b | c | stated at 2,048 | stated at 192 | far n | far b | far c | under 192: 2,048 / 192 |
   |---|---|---|---|---|---|---|---|---|---|
   | 1 | 603 | 0 | 0 | 0 | 0 | 494 | 0 | 0 | 0 / 0 of 23 |
   | 2 | 603 | 1 | 0 | 1 | 0 | 494 | 0 | 0 | 1 / 0 of 23 |

   - b + c is under 10 at both seeds, so R1 cannot pass. The Bar pre-registers this as Negative.
   - **Unclassified.** The Bar says "The next unit classifies it first: instrument or mechanism". This REPORT names no cause.
2. **The model fails even where both windows see the note.** In the under-192 bin, 23 queries whose note is fewer than 192 tokens back, (L) states 0 of 23 at either window at seed 1, and 1 at 2,048 against 0 at 192 at seed 2. So distance is probably not the whole story; that is a lead for the classification, not a finding. At counts this small, the bin cannot do its job on the sheet, which is to tell a scorer broken at 192 from a failing model. I-4 is what checks the grader: all 609 references state their answer. This agrees with claude-205: "Neither trunk puts a rare value from the prompt into the reply."
3. **No other reading separates anything.**
   - **R2 (memory on against out, at 2,048):** seed 1 states 0 and 0; seed 2 states 1 and 1, the same query (b 0, c 0).
   - **R2b (the same at 192):** 0 and 0 at both seeds.
   - **R3 ((S) against (L) at 192):** 0 and 0 at both seeds.
   - **By kind:** the one stated answer is a number. Over the 284 number and 319 word answers, nothing else is stated.
   - The turn memory, built from TRAIN, holds no held-out value, so R2 near 0 was expected. This run cannot show what it adds to recall.
4. **Most replies do not answer the query. These are counts I made after the outputs existed: descriptive, not readings, and no bar rests on them.**
   - **What (L) says at 2,048, memory on, seed 1:**
     - 162 of 609 replies contain "remember": the commonest, " Okay, I'll remember all of that." (82), is TRAIN's acknowledgement of a note turn;
     - next come greetings and mood replies, " I'm good, thanks for asking." (19) and " I'm sorry you feel nervous…" (18 and 11 in its two commonest variants; 44 replies mention "nervous");
     - then " It's green." (16) and " I'm not sure, you didn't tell me." (9).
   - **Replies whose first token is the reference's,** at (L) 2,048, (L) 192 and (S) 192: 43, 24 and 14 at seed 1; 61, 43 and 22 at seed 2.
   - These suggest the trunk often answers a different kind of turn than the one asked. That is a lead for the classification, not a finding.
5. **The long window did not lower the training loss.** At equal positions, (L)'s final loss is 0.639 and 0.644, and (S)'s 0.642 and 0.546. This is reported only: the sheet reads no bar from training loss.
6. **One departure from the job plan, none from the sheet.** The seed folders went to the store from each pod directly, not through a laptop tar, to keep the laptop's disk (21 GiB free) to one copy. Each seal was checked on its pod before the upload (`sha256sum -c`) and again on the laptop after a full read-back (Checks).

## What changed

No code. This unit adds:
- this REPORT and the ORDER (with its pre-run review);
- the sheet's PRE-RUN line, and now its outcome line;
- one row in the [float-training notebook](../../../../research/notebook/float-training.md);
- a paragraph in the [item](../../../../roadmap/items/one-model-end-to-end.md).

## Measurements

- **The pods:** seed 1 on `vast-59085368` and seed 2 on `vast-59086035`, each 1 × RTX 5090 at $0.563/h on Vast (KR). They were up 09:39Z and 09:45Z, and deleted at 13:09Z: about 3.5 h and 3.4 h, about $3.88 in all, against the sheet's $9–12.
- **The job:** `local/claude/p2/job-p2b.sh`, SHA-256 `da1938f7e685a2c88a8ace48f6237115be2f0e2ef85ec296405c5e5089b8997e`, checked on each pod before launch. It started at 09:51:44Z and 09:51:47Z, and both ended `EXIT 0 all steps passed` at 13:03:39Z and 13:03:32Z.
- **I-0:** the CUDA library suite at the pin passed on each pod, with every named test `ok`.
- **I-1:** both smokes exited 0, with finite loss.
- **I-2:** FULL. (L)'s smoke step of 0.6406 and 0.6401 s projects 5,247 and 5,243 s, under 3 hours, so no arm was cut.
- **I-3:** every input folder's `SHA256SUMS` passed, and `check-queries` printed `checked 609 queries, 0 failed`.
- **I-4 to I-6,** from the grade at both seeds:
  - every reference states its answer (609 of 609);
  - at 2,048 every prompt holds its whole conversation (609);
  - all six recall files pair query by query, 609 long, each at its own window.
- **R5 (short context, reported only).** Top-1 nearest rows at 192, memory on, then off:

  | | DEV (14,287 rows) | p-v (15,961 rows) |
  |---|---|---|
  | (L), seed 1 | 9,229 / 8,168 | 9,056 / 9,062 |
  | (L), seed 2 | 9,220 / 8,066 | 8,989 / 8,998 |
  | (S), seed 1 | 9,515 / 7,470 | 8,425 / 8,432 |
  | (S), seed 2 | 9,969 / 8,852 | 9,745 / 9,753 |

- **The grade:** `local/claude/p2/grade-p2b.sh` at `e40789617a59da6f3706c1d380f86813dda1c622d35c276c44aaeb7520fe76f4`. It was run once on files copied from the pods, and once on the store read-back. The two outputs are identical apart from the folder names. Its output (`grade-276.json`) has SHA-256 `216f02b209f60d2d29fb54f9a5097625c87a3beb852af8cdca56166e4d64fb9c`.

## What this shows, and what it does not

- **It shows:** this configuration, the (b) recipe at 8,192 steps of 2,048 × 4 for about two epochs of the long TRAIN, does not state held-out note values. That holds with the window open or shut, near or far, and with the memory on or out, at two seeds. It also shows what the window costs: 1.93× a training position, and about 31× a decode token on these query files.
- **It does not show:**
  - that a 2,048 window cannot carry recall (§4.2);
  - which part fails: the trunk's routing to the asked turn, the read's copy of a held-out multi-token value, or the training budget;
  - anything about the served integer model.

  No reading here is a served result, a step-4 claim or a milestone claim.

## Next

1. **The classification, owed by the sheet's Bar, is past P2b's bound.** Casey's plan gave P2b one sheet unit and one run unit, and both are used. As the plan says, this is recorded in PLACE.md with one note to Casey on #2028, and P3 (faster scoring) goes next. The classification comes before P4 (#3471), whose base was to be (L)'s recipe on a Positive or Partial result.
2. **Laptop leads for that unit,** from these sealed files with no new GPU work:
   - whether the trunk answers the asked turn's type at all;
   - whether the model states note values in TRAIN-like conversations, in distribution;
   - how the first-token matches in What did not work 4 split by distance.
3. **P3 now has its numbers.** The score pass at 2,048 took 2,622–2,624 s of a seed's 11,505–11,515 s (09:51:4xZ to 13:03:3xZ), 2,563–2,567 s of it the two query files, with every 2,048 prompt decoded alone. The P3 scout notes are in `local/claude/p3/`.

## Checks run at the head

- **No code changed** since the pin: only this unit's docs, the sheet's PRE-RUN and outcome lines, the notebook row and the item paragraph.
- **The seals,** each checked twice, on the pod before upload and on the laptop after a full read-back from the store:
  - **seed 1:** 279 of 279 OK; `SHA256SUMS.files` has SHA-256 `ca0b8aeda4e23a6f793df8b503534a48ff090724632dc384c66a0b4b27424b23`;
  - **seed 2:** 279 of 279 OK; `3c26a25280457e5b58c44e1bb7fc82dcd6a51b176fa86f49f6c5161a376598f3`.
- **The store:** `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-long-context-2026-11-04/s1` and `s2` each hold 283 files: the 279 sealed files, their `SHA256SUMS.files`, and the run's `DONE`, `EXIT` and `job.log`. The inputs are left out; they are under `…/inputs/`.
- **The pods:** each was released and deleted after its read-back; `uor-pod status` shows 0 running.
- **`loop report --check` and `loop pr-check --local`** were run at the head before review.
- **Reviews:**
  - the pre-run review: GO WITH FIXES, made before launch (ORDER);
  - the claims review of this REPORT is recorded on PR #3657 by `loop review`.

## Skills used

None applied.

## Still open

1. The classification of this Negative, instrument or mechanism (Next 1). It gates P4.
2. P3, faster scoring: the 2,048 query pass decodes one prompt a forward.
3. Whether a long window carries recall under any recipe. This run closes one configuration, not the window.

## Plain summary

For the first time, the model was trained on long conversations of up to about 2,000 word pieces, in which a fact is told early and asked about much later. Then it was tested on 160 conversations it had never seen, with 609 such questions.

It almost never answered with the fact: once in 609 at one seed and never at the other, whether it could see the whole conversation or only the last 192 word pieces. Often it replied as if to a different kind of message: "Okay, I'll remember all of that." It missed even when the fact was only a few lines back.

This says that this way of training, at this size, doesn't teach the model to repeat a new fact back. It doesn't yet say why. Finding the cause is the next step on this line; first comes making the tests faster.

Training at the long length costs about twice as much per word piece. Writing an answer with the long conversation in view ran about 15 word pieces a second, against about 470 with the short view.

References #3407.
