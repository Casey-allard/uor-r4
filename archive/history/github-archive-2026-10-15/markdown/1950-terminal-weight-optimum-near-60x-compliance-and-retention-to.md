# #1950 — Terminal-weight optimum near 60x: compliance and retention together (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T18:42:15Z
- **Updated:** 2026-10-09T13:22:44Z
- **Closed:** 2026-10-08T18:43:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1950
- **Merged:** 2026-10-08T18:43:53Z
- **Merge commit:** b15094d49e6887e68c54c5416ddeeb9d43dfdbee
- **Base / Head:** main / UOR-Foundation:deepseek/terminal-optimum
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The terminal-weight optimum sits near 60x: compliance and retention together

Three doses, same shape, corpus, mask, policy, steps and seed; only UOR_TERMINAL_WEIGHT differs.
At 20x yes/no compiled 2 of 3 and one retention turn answered; at 200x compliance reverted,
retention broke and list artifacts returned; at 60x yes/no compiles ("Yes, the", "Yes, that's"),
BOTH retention turns answer "My name", the follow-up responds ("You can"), and the France and
largest-planet openers are intact.

So the ordering on the behaviours that matter is 60 > 20 > 200, and 60x is the first setting at
which compliance and the existing behaviours hold TOGETHER rather than trading against each
other. That is the most useful result of the night on the chat path.

It also confirms the causal story the elimination left: token-level MLE length bias, where after
a complete answer the corpus's continuations carry about 250:1 the token mass of its stops. A
single scalar that changes only what a completed answer is worth to the loss moves compliance
up, back down, and to a better point in between - which no data, scale or decoding change
managed tonight.

One-word and numeric turns still do not comply at any dose, producing procedures and phrases
rather than the requested token. Either the weight that reaches them lies outside [20, 200],
which the 200x collapse makes unlikely, or those classes need something beyond terminal
weighting. That is the next, now narrow, question.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

