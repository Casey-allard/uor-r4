# #1930 — First response-region objective: the output form moves, and a frozen probe breaks (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T16:34:06Z
- **Updated:** 2026-10-09T13:21:54Z
- **Closed:** 2026-10-08T16:35:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1930
- **Merged:** 2026-10-08T16:35:47Z
- **Merge commit:** eb4a5e77133832badb7f04c6c9754d17b0fc1cc8
- **Base / Head:** main / UOR-Foundation:deepseek/response-region-first
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

dialogue-train with the extracted response mask under policy=role_only, 13,348 steps on the frozen 7.16M shape, final dev response nll 1.802750876406361. The output FORM moved for the first time this session - list-form, instruction-shaped replies with bullets and procedural openers where every previous arm produced prose describing the topic. It also cost three things: format collapse (every reply opens with a bullet, some pure bullet spam), placeholders leaking as literal text, and a broken instrument - the frozen first-token probe now returns a single space for all nine, because the model learned the corpus separator convention, so the probe reads the boundary rather than the answer. That is reported as an instrument problem rather than as 0 of 9. No improvement is claimed; the policy=full_prefix control with the same mask is running to separate the two effects, and the probe must skip the separator before judging. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

