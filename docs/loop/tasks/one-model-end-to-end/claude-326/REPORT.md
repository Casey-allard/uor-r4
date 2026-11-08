# REPORT: one-model-end-to-end, claude-326

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Owner decision: "Yes, pivot assessment" (#2028, 6726003019). Deliverable: [ASSESSMENT.md](ASSESSMENT.md).

## What did not work, first

- **The pivot came late.** D21 asks for a pivot card after three merged PRs on one line that leave the headline where it was. This line had more than ten from claude-309 on, all with 0 complete replies. It took Casey's observation, "the last four days worth of testing runs have basically turned out all of them completely negative", to start this assessment.
- **The cause has not been tested.** The assessment names the most likely blocker from the record and proposes the test (claude-327); it runs nothing. Under §4.1 no mechanism is closed here.
- **The first-token measurement has never been run on a claude checkpoint.** Every first-target number in the assessment is from DeepSeek's SFT artifacts (notebook rows 17, 18).
- **claude-325's reading is not in it.** Its seeds were still training at review time; its outcome row decides how claude-327 orders its two parts (ASSESSMENT, Q4).

## What changed

- `ASSESSMENT.md` (new), which answers the ORDER's four questions:
  1. the blocker;
  2. what is and is not established about choosing the queried value;
  3. the levers;
  4. the smallest test.

  Each number is cited to a notebook row on `main` at `fa5004861`, a REPORT or a source line.
- No code, data, bar, status, scope or cap changed.

## Measurements

None were taken; this unit ran no compute. The numbers the assessment rests on are all from the record:

| What | Number | Source |
|---|---|---|
| Complete replies on `m2-fresh256`, every served configuration 4–8 Nov | 0 of 256; pairs 0 of 128 | rows 13–15, 17, 21–23, 25, 26, 28, 30, 34 |
| First target, teacher-forced (DeepSeek's SFT artifact) | 0/256 top-1, top-8 and top-32; median rank 956 / 1,189 of 4,096 | row 18 |
| Later targets after a correct first token | 1,614 / 2,576 top-1; 2,574 / 2,576 top-8 | row 18 |
| Opening query spread | 0.26249, against 2.06252 over all rows | row 18 |
| Exact-token recall, 27 Oct probe (query = key) | 0.984–0.994 far buckets; no read 0.000–0.002 | row 109 |
| First line holds the asked value / the other value (claude's served replies) | 25 / 21 and 76 / 36; 14 / 11 and 2 / 6; 0 / 0 and 32 / 30 | rows 15, 14, 13 |

## Checks run at the head

- `uor-harness loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-326/ORDER.md`: PASS at the head.
- **Fresh-context decision review, round 1** (delegate:reviewer, given only the ORDER and the ASSESSMENT): PASS WITH CHANGES, 4 major and 7 minor, all applied at `8084db326`:
  - **M1:** the headline overclaimed mechanism from DeepSeek's checkpoints.
  - **M2:** claude-321's seed 2, at 76 / 36, contradicted "copied, not selected".
  - **M3:** Part B's query was the current token, unlike the panel; a two-class arm and a header-gap arm were added.
  - **M4:** the fail now reads "mechanism-side for the configuration", per §4.1.
- **Round 2:** PASS WITH CHANGES, all eleven fixed, 4 minor, applied in the final commit:
  - the training mix cited to rows 21 and 32, with the format/choice confound it raises;
  - row 81's (w) arm;
  - the one-user later-target top-8 (2,574);
  - the closed-configuration wording of row 13.

  Its design note, to run B0 at row 109's own settings, is in Part B.

## Outcome

**The replies fail at the reply's first token,** where the model has to choose which of the prompt's two values the question asks for. Data, scale and the objective each moved something measurable, and none reliably moved this choice.

**The leading hypothesis is the mechanism:**
- the copy heads cannot make the choice by construction;
- no learned-query read on record has learned where to look.

**The record cannot yet tell data from mechanism:**
- the first-token ranks are from DeepSeek's OASST2-heavy SFT artifacts only;
- claude-321's seed 2 chose the asked value 76 to 36.

Three readings decide it:
1. claude-325's run;
2. Part A on claude's seals;
3. Part B, the probe whose query is not the key, with its two-class and header-gap arms.

**The decision (lab, §5.1):** no new one-knob language run on this line starts until claude-327's Part A is read and its Part B has run. The decision changes no bar, status, scope or cap.

## Skills used

- None beyond the loop itself (ORDER, REPORT, the fresh review).

## Still open

- **claude-327:**
  - Part A, on the laptop: the first-target rank and a Hamming trace at the first reply position, on claude-322's, claude-323's and claude-325's seals.
  - Part B, a finite GPU probe: the 27 Oct recall probe with a two-class query that is not the key, then a header gap, against its own induction control and a no-read arm.
  - Part B needs its prep sheet, a non-author FIT and a pre-run review (§4.5).
- **claude-325's outcome row**, folded in when it lands. Row (b) puts the bare-answer recipe, scaled, ahead of Part B; rows (c) and (e) put Part B first.

## Plain summary

The last four days' runs all scored zero complete replies. They each changed the data, the size or the training signal, and they did fix stopping, but they never touched the step where the replies go wrong.
- **Where it goes wrong:** the first word of the answer, where the model has to pick which of two facts the question asked about. Once it has the right first word, it copies the rest almost every time.
- **Why it most likely can't:** the part of the model that copies from the prompt looks for the current word, not for the question, and the part that could follow the question has never learned to.

The next unit tests that directly, with a small synthetic recall task where the question word differs from the stored word, before any more language runs.
