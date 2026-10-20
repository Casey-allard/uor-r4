# #1958 — Breakthrough: demands paired with matching short answers produce form compliance (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T19:50:16Z
- **Updated:** 2026-10-09T13:19:53Z
- **Closed:** 2026-10-08T19:52:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1958
- **Merged:** 2026-10-08T19:52:05Z
- **Merge commit:** 2bcd56b89108809b4b111136beeeb828e54581c1
- **Base / Head:** main / UOR-Foundation:deepseek/demand-breakthrough
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Breakthrough: demands paired with matching short answers produce answer-form compliance

Built a demand-bearing short-form store: 2,860 documents whose user turn DEMANDS a response form
and whose assistant turn is exactly that form (670 one-word, 580 yes/no, 550 numeric, 520
short-list, 540 short-factual), 16,410 response tokens, validated against 16 clauses with a tool
that refuses to write on any offender - proven by fault injection - and admitted in full by the
real training loader, against the chat corpus's FOUR eligible one-word episodes.

Mixed with the hash-proven chat corpus and trained with policy=full_prefix and
UOR_TERMINAL_WEIGHT=60, it complies across every form class on panel v2: "Green." and "Honey."
for one-word demands, "Yes." and "No." for yes/no, "7." and "4." for numeric, "Red, blue," for a
list, and "Paris." for a short factual - with the correct answers.

Every previous arm on the same panel produced prose for all six classes: the chat-only arm, the
role_only arm, full_prefix at 1x and 2x and 2.8x, truncated_prefix, and the synthetic-content
arm. The predicted cause is confirmed: the project held demands without short answers and short
answers without demands, and never the pair.

The deepest doubt of the night is also answered empirically - a per-token objective over a
mixed-length distribution can commit to a single token once the demonstration exists. And the
stop signal compounds rather than substitutes: it is present here, and it alone did not move
these classes.

What remains: retention turns no longer answer ("I do"), so multi-turn conditioning is a separate
behaviour and the mixture weight may need adjusting to preserve it; and the mixture weight, the
retention pair and a longer run are the natural next sweep with the instruments already frozen.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

