# #1975 — P1 probe: readout failure downstream of retrieval, plus an EOS method correction (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T23:38:16Z
- **Updated:** 2026-10-09T13:19:08Z
- **Closed:** 2026-10-08T23:39:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1975
- **Merged:** 2026-10-08T23:39:49Z
- **Merge commit:** 292b231d3803511a6b5c5d3cba270ae399800e96
- **Base / Head:** main / UOR-Foundation:deepseek/binding-p1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The expert review P1 over four rows differing only in the value inside turn 1, inference only on four checkpoints. Method correction: a raw-text single-turn emulation of a pinned turn 1 omits the EOS closing turn 1, worth up to 1.33 nats of DELTA, so naive pinned runs are invalid; the pinned implementation was validated bit-exactly against the official binary. Result: the pre-registered branch 4 for all three store arms - read mass above TAU on the expected span while the answer still prefers a prior. arm-as has read mass above TAU on 4/4 rows in every condition with every row c_readout and DELTA pinned Sure. of +1.91/-0.11/+2.60/+1.67 yet 0/4 correct argmax and 0/4 exact; the no-store control has read mass below TAU 4/4 in every condition and yet produces the value in prose, which the frozen two-token rule scores a miss. Branch 1 and branch 2 are excluded; branch 3 holds only for the control. DELTA itself tracks token prior, so it was not treated as evidence. The failure is readout, not retrieval, and the next changes are the review P3 and P4. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

