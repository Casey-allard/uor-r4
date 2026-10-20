# #1946 — Add an optional turn-terminal weight to the dialogue objective (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:57:49Z
- **Updated:** 2026-10-09T13:22:46Z
- **Closed:** 2026-10-08T17:59:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1946
- **Merged:** 2026-10-08T17:59:36Z
- **Merge commit:** e0b714e992ee5cc39673bb1b264bede27cd9a5ec
- **Base / Head:** main / UOR-Foundation:deepseek/terminal-weight
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Add an optional turn-terminal weight to the dialogue objective

The session's elimination left token-level MLE length bias as the binding constraint on
instruction precision: after a complete answer the corpus's continuations carry roughly 250
times the token mass of its stops, so the per-token optimum is to keep writing. This is the
smallest intervention that attacks that ratio.

dialogue_episodes.rs weighs every response target 1.0 uniformly. The response's TERMINATING
target - the last real position, the <|eos|> that ends the reply - is now weighted by
UOR_TERMINAL_WEIGHT when that variable is set to a value at or above 1.0, and left at 1.0
otherwise, so existing runs are bit-for-bit unchanged unless they opt in. The supervised-target
statistic now tests weight > 0.0 rather than weight == 1.0 so it stays correct for weighted runs.

Checks: the ten dialogue_episodes tests pass. Three joint_campaign tests fail identically with
this change stashed, so they are pre-existing and unrelated.

The mask (verified byte-identical to the builder's hash) identifies the terminating position of
every response, so the weighting needs no new data and no new format.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

