# #1945 — Run card: the objective change the elimination left (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:47:18Z
- **Updated:** 2026-10-09T13:21:21Z
- **Closed:** 2026-10-08T17:48:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1945
- **Merged:** 2026-10-08T17:48:57Z
- **Merge commit:** 016789dfd25986717e8b5a03795de7b61612e1f1
- **Base / Head:** main / UOR-Foundation:deepseek/objective-change-runcard
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Run card: the objective change the elimination left

Round 93 closed the elimination - capacity, optimization, decoding, the prompt protocol and
data support have each been ruled out by a measured arm - leaving token-level MLE length bias,
which alone predicts the observed failure signature "Yes, fish": first token right, second
wrong. The fix must make a completed answer worth something to the loss.

This card specifies where. Per the round-81 review, dialogue_episodes.rs:572-582 weights every
response token equally and everything before it zero, and the verified mask identifies each
response's terminating EOS as the last position of a masked run. So the terminal token is already
addressable, and the cheapest change is turn-terminal weighting - multiply that position's weight
by a factor and change nothing else - with a length-normalised completion term as the second
experiment and a terminating decoding rule as a near-free diagnostic that isolates the diagnosis
from the fix.

Judging uses panel v2 read as the first two content tokens with the echo item removed, fluency
and the retention pairs as guards, and explicitly NOT development nll, which four arms have now
shown does not track this objective's behaviour.

The falsifier is recorded: if turn-terminal weighting at every factor tested still produces no
compliance, the continuation-mass explanation is wrong too and the remaining candidate is that
the requested form is not represented in the prefix at all - a representation question, and the
first result of the night pointing back at the architecture.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

