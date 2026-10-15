# #1937 — The corpus has no short-form replies; instruction precision is a data gap (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:20:31Z
- **Updated:** 2026-10-09T13:20:50Z
- **Closed:** 2026-10-08T17:22:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1937
- **Merged:** 2026-10-08T17:22:09Z
- **Merge commit:** a27e9b602f811cbc81e58892fe69fd6bdff068d1
- **Base / Head:** main / UOR-Foundation:deepseek/length-distribution
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The corpus has no short-form replies, so instruction precision is not learnable from it

Measured the response-length distribution from the verified response mask over 129,465
assistant replies in chat-v0-p2/train (82,248,461 tokens, 71.1 percent response region):

  median 350 tokens, mean 438, p90 919, max 3,925
  <= 1 token   0.0 percent
  <= 3 tokens  0.0 percent
  <= 10 tokens 0.1 percent
  <= 50 tokens 7.8 percent
  > 200 tokens 67.8 percent

There is not one one-to-three-token reply in the corpus. That answers the instruction-precision
question four arms failed to move: when a turn asks for exactly one word, a yes/no, or a
number, the model has no such reply in its training target distribution to imitate, so it
writes prose about the request - which is precisely what the panels show.

It also explains why each candidate cause was refuted in turn. Capacity helped what capacity
can help (the refusal default, a factual answer, format awareness) but cannot invent a reply
form absent from the data. The objective fixed register and multi-turn conditioning because
those ARE in the data. Sampling changes style, not the support of the distribution.

This corrects rounds 75-76: scaling capacity is NOT the answer for this target. More tokens of
the same corpus remain refuted (round 48's pre-registered failure, and the 214M comparison at
1.0966 against 0.9607), but a corpus whose response-length distribution includes short forms is
the binding constraint - a much smaller and more specific ask than 1.39B more tokens.

Next, before any training run aimed at instruction precision: estimate the form distribution of
the task-formatted sources already used (summarize, rewrite, constraints) with the same mask
method, since that determines whether the short-form data can come from existing licence-clean
material or needs an owner decision.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

