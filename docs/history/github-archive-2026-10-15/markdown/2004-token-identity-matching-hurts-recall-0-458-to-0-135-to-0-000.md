# #2004 — Token-identity matching HURTS recall: 0.458 to 0.135 to 0.000 (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T10:32:51Z
- **Updated:** 2026-10-09T13:20:38Z
- **Closed:** 2026-10-09T10:34:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2004
- **Merged:** 2026-10-09T10:34:30Z
- **Merge commit:** 4593071feca4abf4e0a0396789d200e276b3c6fc
- **Base / Head:** main / UOR-Foundation:deepseek/identity-negative
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The token-identity pointer score tested against the learned hidden-state match on the same 28 invented values, three conditions, 96 cells per arm, scored in one session with the probe that reads the model directly. baseline arm-ptr 44/96 = 0.458 [0.362,0.558]; identity 0.5 gives 13/96 = 0.135 [0.081,0.218]; identity 1 gives 0/96 = 0.000 [0.000,0.038]. The term hurts monotonically in its weight: half weight costs about 32 points with non-overlapping intervals and full weight destroys recall, reproducing the no-pointer control despite the model having a pointer; the baseline reproduces its earlier measurement, which makes the matched comparison credible. WHY: reachability is identical across all three arms at 0.4729, so the span is addressable in every configuration and what differs is the score ability to discriminate; and the term is anti-correlated with the correct source here because the value tokens appear in the premise, so every candidate near the premise shares them and a local-window Jaccard cannot distinguish the answer location from its neighbours. A first extraction printed 0/96 for all three arms including the baseline; that was caught by comparing against the known 0.417 and was a parser error reading the id n_vantel as the value. WHAT THIS SETTLES: both ways of injecting token identity into the pointer are now rejected - token-id keys via a route and token overlap as a score term - so the remaining lever is to sharpen the learned match rather than replace it. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

