# #2041 — Token-identity copy stop: new best 72/84 = 0.8571, F=1.0 recovered (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:57:12Z
- **Updated:** 2026-10-09T15:00:15Z
- **Closed:** 2026-10-09T14:59:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2041
- **Merged:** 2026-10-09T14:59:14Z
- **Merge commit:** 178d988f3131b8dfd88350a4b999f084a585f77b
- **Base / Head:** main / UOR-Foundation:deepseek/idstop-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The run-length stop fired on the prompt template (span inside the value's own span on only 20 of 79 cells at F=1.0) because values tokenize to ~5-7 ids. A stop keyed on TOKEN IDENTITY against the span the pointer itself selected cannot fire on the template. Result on 84 cells per arm: a NEW BEST of 72/84 = 0.8571 [0.7667, 0.9163] for arm-ptr at floor 0.5 with the identity stop, against 65/84 = 0.7738 previously. vs 65/84: +7 cells, 9 gained / 2 lost, McNemar p=0.0654, Newcombe CONTAINS ZERO - a better point estimate, not significant at 95% on 84 cells. vs the floor-0.5 plateau 59/84: +13, p=0.00235, significant. vs the same floor with no stop 49/84: +23, 23 gained / 0 lost, p=2.4e-07. F=1.0 is recovered, which the run-length rule could not do: arm-ptr 6 -> 71 (+25 over run-length, 25/0, p=2.98e-08), arm-a-w05 26 -> 70, arm-c-ctrl 7 -> 63. With the identity stop the floor becomes nearly irrelevant on arm-ptr (72/72/71 across F = 0.5/0.75/1.0). min_span=3 was run and is not a different instrument. Mechanism: channels unmoved; at arm-ptr F=0.5 the run-length rule fires on 26 cells with 1 exact while the identity rule fires on 23 with 20 exact and lands inside the value's span on 21. Honest limit: at F>=0.75 the fired span is fully inside the value's span only about a quarter of the time, so the mechanism is 'end the reply at the end of the first span the pointer sustains by token identity', not 'recognise the value'. Default proven bit-for-bit (288 + 288 records, 0 differing fields, 18/18 files); parser self-checked against known 35/84, 10/84, 24/84; four focused tests pass; a pre-registered offline prediction matched measurement. Also fixes a blocker this exposed: main could not compile its lib tests because #2009 landed only half the copy-stop change - fixed in #2040. Export still refuses an identity pointer, so this is an offline-capability result. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

