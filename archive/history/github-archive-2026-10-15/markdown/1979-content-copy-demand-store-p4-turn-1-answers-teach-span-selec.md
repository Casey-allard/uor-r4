# #1979 — Content-copy demand store (P4): turn-1 answers teach span selection (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T00:15:51Z
- **Updated:** 2026-10-09T13:19:42Z
- **Closed:** 2026-10-09T00:17:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1979
- **Merged:** 2026-10-09T00:17:30Z
- **Merge commit:** 57d3bc2e79440651aefd34a5ab6dbb1657f190e3
- **Base / Head:** main / UOR-Foundation:deepseek/copy-store
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The data half of the fix the binding probe justified. Every earlier store answered turn 1 with a content-free acknowledgement - Noted., I see. - so the turn-1 response slot taught a register instead of span selection, and the probe caught the consequence: the model's own turn 1 was Noted. and its turn-2 reply was also Noted. This store forbids acknowledgements outright (12-entry forbidden set, 0 hits over 2,012 responses) and makes turn-1 answers the PREMISE'S OWN TOKEN SPAN, so the slot teaches copying. Counts: 1,006 documents, 10,757 response tokens under a 12,000 ceiling; turn-1 answers mean 4.83 BPE tokens, all 1,006 at or under 4 whitespace words. Two checks the earlier stores never made both return zero - turn 2's answer appears in NO user turn (0/1006) and is never token-identical to turn 1's answer (0/1006) - and applying the first of those eliminated three of the six families the brief asked for, because an attribute lookup, a quantity unit and a yes/no demand put the answer inside a user turn by construction. The store is verified with 0 offenders on all 16 mask clauses in two independent implementations, corruption rejected by the project's own EpisodeIndex, and 2,012 of 2,012 responses admitted with none excluded. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

