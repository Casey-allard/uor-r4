# #1933 — Repair the first-token probe; it finds a fixed multi-turn failure (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:01:05Z
- **Updated:** 2026-10-09T13:21:41Z
- **Closed:** 2026-10-08T17:03:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1933
- **Merged:** 2026-10-08T17:03:01Z
- **Merge commit:** 32d38a95a003b4556400a5493c27d99f307ac935
- **Base / Head:** main / UOR-Foundation:deepseek/probe-fixed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Repair the first-token probe, and it finds a fixed multi-turn failure

Round 56 reported that the max_new_tokens=1 probe returned a single space for every turn on a
masked model, and said that reporting it as 0 of 9 would be wrong because the model had learned
the separator convention. The repair needs no code change: generate a few tokens and read the
first non-whitespace token.

Applied to the round-58 full_prefix plus mask artifact, the repaired probe finds the retention
turn - the second turn of a pair whose first turn said "I have two cats" - now opening "You
have two cats, but...". The chat-only model returned text identical to the first turn on that
same probe, and round 44 recorded 0 of 5 second-turn questions answered. This is the clearest
behavioural improvement measured in the session and it is exactly the failure the
response-region objective targeted.

Two turns also open with the correct answer's first word, and the greeting still gets the
canonical assistant opener. Instruction compliance is still absent where a form is demanded:
both yes/no turns open with "I do not have access...", "two plus two" opens with "I'm", and
that access refusal repeats across turns as a learned template rather than an assessment.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

