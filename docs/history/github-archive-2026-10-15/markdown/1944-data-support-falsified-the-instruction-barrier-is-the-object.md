# #1944 — Data support falsified: the instruction barrier is the objective (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:43:26Z
- **Updated:** 2026-10-09T13:19:50Z
- **Closed:** 2026-10-08T17:45:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1944
- **Merged:** 2026-10-08T17:45:06Z
- **Merge commit:** 212d98a9d1fd85f69e74c13c4f9814f566197e28
- **Base / Head:** main / UOR-Foundation:deepseek/data-falsified
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Data support is falsified: the barrier is the objective, not the corpus

The review of round 81 ranked data support first and token-level MLE length bias third, and
named the cheapest test: policy=truncated_prefix, one argument changed, same corpus, mask,
shape, objective and steps, which admits 2.14x the episodes and ALL 118 five-word-or-fewer
replies instead of 8, a 14.75x increase in short-form support. Its falsifier: if that arm still
shows no compliance and leaves the first-content-token distribution unmoved, data support is
falsified as the binding cause.

THE FALSIFIER FIRED. Read as the first two content tokens: one-word turns give "The color",
three-colours gives "Here's a", sky-blue yes/no gives "The sky", two plus two gives "The plus",
and the fish turn gives "Yes, fish" - the FIRST token right and the second wrong. Not one
instruction-form request is complied with at 14.75x the episode support, and "Yes, fish" is the
signature of continuation mass rather than of missing data.

Everything else is now eliminated by measurement rather than argument: capacity (2.8x params),
optimization (2x steps), decoding (sampling), the prompt protocol (a real defect, fixed), and
now data support (14.75x episodes). The model predicts the next token over a mixed-length reply
distribution whose continuations carry roughly 250 times the token mass of its stops, so the
per-token optimum after a complete answer is to keep writing.

This is the first time the binding constraint has been identified by elimination with a measured
arm per candidate. The fix is in the loss or the decoding - a turn-terminal weighting using the
verified mask's EOS positions, a completion-style estimand, or a terminating decoding rule - all
testable against the frozen instruments. None is claimed; the change has not been written.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

