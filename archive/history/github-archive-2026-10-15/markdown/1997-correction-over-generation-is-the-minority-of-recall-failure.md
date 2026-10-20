# #1997 — Correction: over-generation is the minority of recall failures; a terminating decoder is worth +10 points (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T04:09:23Z
- **Updated:** 2026-10-09T13:22:52Z
- **Closed:** 2026-10-09T04:11:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1997
- **Merged:** 2026-10-09T04:11:03Z
- **Merge commit:** da1fa990c372f27258fb607c3a6b6995da623ecc
- **Base / Head:** main / UOR-Foundation:deepseek/truncation-test
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous rounds recorded the dominant residual failure as a correct value followed by over-generation and set out to test a copy-terminating decoder. THE TEST DOES NOT SUPPORT THAT AS STATED: of 49 misses at the full budget only 9 start with the value then continue, while 29 never contain the value at all. Exact-copy by budget on 84 cells: 3 -> 0.083, 5 -> 0.440, 8 -> 0.417, 40 -> 0.417; the no-pointer control is 0/84 at every budget. A post-hoc cut of frozen replies gives 4 -> 0.488 and an ORACLE cut at the value length gives 44/84 = 0.524 [0.418, 0.627], so a perfect copy-terminating decoder is worth +9 cells and about +10.7 points: real and modest. A MEASURED CONFOUND explains the first reading: the harness generates turn 1 premise reply with the same max_new_tokens, so in the natural condition the knob truncates the premise too; in the unconfounded pinned conditions truncation gains EXACTLY ZERO cells (Sure 10/28, Noted 13/28 at all budgets). THE SPLIT: exact at some budget 18/28 = 0.643; never exact at any budget 10/28 = 0.357; starts with the value at some budget 22/28; never even starts 6/28. So there are two failures, not one, and the larger is retrieval/selection, meaning a decoding fix addresses at most the continuation surplus on top of a 0.417 base. The next lever is the pointer matching step - a token-identity key instead of a hidden-state key. The recall result stands: 0.417 against a control of 0.000 at every budget. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

