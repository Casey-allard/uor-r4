# #1973 — Expert review: cross-turn binding diagnosis and ranked proposals (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T23:11:56Z
- **Updated:** 2026-10-09T13:19:12Z
- **Closed:** 2026-10-08T23:13:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1973
- **Merged:** 2026-10-08T23:13:35Z
- **Merge commit:** 9129ed45de5c1140a29a65de4a4b23015f3d68f2
- **Base / Head:** main / UOR-Foundation:deepseek/binding-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

An expert review of the one problem the night isolated. It reframes it substantially. First, the same architecture ALREADY bound: with no demand store at 7.16M and terminal weight 60 the retention name turn answered My name on both turns, and the cats turn was recorded as You have two cats - the identical pair that now fails. So binding is not absent from the model; it is destroyed by the demand store presence. Second, the failure strings are the store own register: Noted. and I see. are hard-coded as every family turn-1 answer in the generator, and 3. is the count family answer. Third, the store validation tested the wrong thing: the transformation fraction checks the answer against turn 1 ANSWER but never against turn 1 PREMISE, where the value usually sits, so the store never forced ignoring the premise. It diagnoses conditionally-wired binding that is outcompeted, identifies the missing mechanism as an address from the question to the value span, and ranks ten proposals. The cheapest and most decisive is the pre-registered binding probe on existing checkpoints with turn 1 pinned and values swapped, measuring the log-probability margin; if the margin is large with a correct argmax when turn 1 is pinned to a canonical reply while form compliance holds, the mixture explanation is ruled out and the fix is data plus commitment weighting rather than a mechanism change. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

