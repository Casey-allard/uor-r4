# #1935 — Capacity removes the refusal template and fixes a factual answer (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:11:47Z
- **Updated:** 2026-10-09T13:19:17Z
- **Closed:** 2026-10-08T17:13:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1935
- **Merged:** 2026-10-08T17:13:40Z
- **Merge commit:** 6c8f3f2e777a0dd72ca4a5380c05e3a2be6adb1a
- **Base / Head:** main / UOR-Foundation:deepseek/capacity-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Capacity removes the refusal template and fixes a factual answer

The 19.9M response-region arm (rrarrarr, width 512, heads 8, read L2, full_prefix plus the
extracted mask, 12,207 steps) finished with development response nll 1.4175610855135288 and
was exported and scored automatically by the watcher installed in round 67.

On the two frozen instruments it improves on the 7.16M arm of the same recipe in three ways.
The refusal template is gone under GREEDY decoding - 0 of 22 - where the smaller arm produced
it repeatedly and round 73 could only remove it by sampling; so the template was a small-model
default rather than only a decoding artefact. The largest-planet question is now answered
correctly ("Jupiter") where the smaller arm said "the absence of the planet". And the model
now reasons about the requested form: "Imagine a response with exactly three bullet points
using markdown. Here is the first sentence:" - it has absorbed what the request is, even
though it does not yet emit a clean list.

What is still wrong: one-word and exact-count requests are not obeyed precisely, "how many
days in a week" gives the right shape with the wrong value, and repetition persists.

Conclusion: capacity is the lever for reliability and the response-region objective is the
configuration to scale. The remaining gap is precision of instruction following, now a single
named target. Worth noting for future metric choices: the larger arm's development nll is
slightly WORSE (1.4176 against 1.3806) while its behaviour is better on every qualitative
measure.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

