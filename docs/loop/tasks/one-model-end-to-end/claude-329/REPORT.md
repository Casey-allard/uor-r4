# REPORT: one-model-end-to-end, claude-329

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-question-keyed-read-2026-11-08.md](../../../../harness/prep/one-model-end-to-end-question-keyed-read-2026-11-08.md).

## What did not work, first

- **The sheet is NOT FIT after two rounds, so this unit stops at the sheet.** That is the ORDER's stop rule (step 2). No code was written and no GPU was used.
- **Round 1** (fresh non-author reviewer, at `3c7a10f9a`):
  - **Blocker:** the reproduction check I-R asked for byte-identical `recall.jsonl`. That can never pass, because every line records `train_seconds` and `train_context_tokens_per_second` (`in_space_long.rs`, `RecallReading`).
  - **Major:** in B2 the question row `Q(K_j)` is not scored. So the found-copy pull, defined on rows that carry a training target, never reaches `a[Q]`. A B2 failure therefore could not be blamed on the carry.
  - **Four minors:**
    - outcome 4's distance was measured from step 512, not step 0;
    - the sheet did not say that the task loss shares `key_rotation` with the copy heads;
    - the attribution used a copy-head cut;
    - the sheet did not say that B0-a cannot test the ask path.
- **Round 2** (a different fresh reviewer, at `59b69fad4`):
  - All six round 1 findings are confirmed fixed.
  - **One blocker remains:** I-R gated `hits` against `rows` at every checkpoint, and the step-0 line fails that. Read as an absolute bar at step 4,096, it is also failed by run-to-run noise: claude-328's seed 2 far bucket is 0.980.
  - **Two minors:**
    - outcome 3's second branch cannot separate the ask heads' stream write from the trunk's carry;
    - the sheet did not say how B2-aq finds its rows.
  - The reviewer's verdict: NOT FIT "on the I-R wording alone", and fit after that fix.
- **The round 2 fixes are applied but not reviewed.** Under the stop rule, they are not accepted on the author's word. A fresh FIT of the fixed text is owed.

## What changed

- **The prep sheet** `docs/harness/prep/one-model-end-to-end-question-keyed-read-2026-11-08.md`. It covers:
  - the mechanism: ask heads, which start as copy heads at offset 0 and learn a per-token ask offset `a`;
  - the training signal: the found-copy pull, a Hamming distance toward the detached key of the earlier position holding the target;
  - the arms B0-a, B1-a, B1-a0, B2-a, B2-aq and B0-h, with the no-read twins;
  - the bar, the outcome map, the instrument checks I-1, I-A1 to I-A5, I-4, I-6, I-7 and I-R, and the cost (about $1 for two seeds).
- The sheet's FIT line records both rounds.
- This REPORT.
- No code, bar, status, scope or cap changed. No notebook row is written, because there is no result.

## Measurements

None. This unit ran no training or evaluation. The facts the sheet rests on were read from the code and the store:
- **The type heads' detached choice and their single query rotation:** `flock_read_hamming.rs:1155–1185` and `:1716–1724`. Both reviewers confirmed this reading.
- **claude-328's sealed result:** the tarball `results/claude/one-model-end-to-end-query-recall-2026-11-08/runs/s1/query-recall-s1.tar.gz` lists `oh/run/B0-h-s1/recall.jsonl`. I checked its `.list` on 8 Nov.
- **The arithmetic behind the chance rate:** 16 kept positions out of about 184–190 is 0.084–0.087, matching claude-328's measured kept rate for the type heads.

## Checks run at the head

- The banned-token scan of all three files (the sheet, the ORDER and this REPORT; the §3.2 and §3.3 names): 0 hits, after the ORDER's one hit was reworded at the review's request.
- `uor-harness loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-329/ORDER.md`: PASS.
- No cargo check was needed: the diff is docs only.

## Skills used

None beyond the loop's own steps. The two FIT reviews ran as fresh read-only reviewer subagents, under suborders `local/claude/claude-329/orders/1.md` and `2.md`.

## Still open

- **claude-330:** a fresh non-author FIT of the sheet as fixed after round 2. If it is FIT, the code follows behind `--hamming-ask`, `--hamming-attract` and `--hamming-attract-at`, default off, with I-1 and I-A1 to I-A5 as unit tests with mutations. Then the D67 pre-run review and the two-seed run.
- **The build map is already read.** It is carried into claude-330's ORDER, which is a local draft until that unit commits it:
  - The ask query depends only on the token point and `a`, so the pull can be computed from `ids` and the table outside the forward pass.
  - The recall training loop calls `batch_loss` at `in_space_long.rs:12741`, where the pull term is added.
  - `RecallQuery` carries `query_pos` and `ask_value` for B2-aq.

## Plain summary

The read failed on questions whose word differs from the key, because it never learns where to look. This unit wrote the test of a fix: each question word gets a learned offset that the read's own distance pulls toward the right key. Two independent reviews found real flaws in the plan. The first round's are fixed. The second round left one wording fault, now corrected, but the rules say a third look is owed. So the next unit reviews the fixed plan, then builds and runs it.
