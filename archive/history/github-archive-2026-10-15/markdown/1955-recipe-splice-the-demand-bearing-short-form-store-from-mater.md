# #1955 — Recipe: splice the demand-bearing short-form store from material already held (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T19:21:30Z
- **Updated:** 2026-10-09T13:19:54Z
- **Closed:** 2026-10-08T19:23:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1955
- **Merged:** 2026-10-08T19:23:10Z
- **Merge commit:** 17264073176bfdbea3bcd582b4ebd08be54f7232
- **Base / Head:** main / UOR-Foundation:deepseek/demand-recipe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Recipe: build the demand-bearing short-form store by splicing the two halves the project has

The project holds demands (the corpus's contract turns) and short answers (the synthetic
store's canonical replies) as TOKENISED MATERIAL, so the store the last two rounds identified as
missing can be built by splicing rather than by authoring text, a new licence or a new
generator.

Select the explicit demand turns from the corpus; take the canonical answer runs from the
synthetic store; splice per document as BOS, user turn, the five Assistant marker ids, the short
answer, the genuine EOS (measured as 1 from the chat store's 129,465 terminations); pair
demands with answers of the MATCHING form, because the point is that the pair appears together;
mask 0 on BOS, the user turn and the markers and 1 on the answer and its EOS, which are the three
clauses dialogue_episodes.rs:385-405 enforces and the checker used on the synthetic store
validates; write the 64-byte header with its token count at offset 8, the field that silently
broke the last rebuild, plus the manifest the trainer requires.

Then mix at a small weight, train with policy=full_prefix and UOR_TERMINAL_WEIGHT=60, export and
read panel v2.

Every other candidate has been eliminated by a measured arm - capacity, optimization, decoding,
the prompt protocol, raw data support, data volume in the wrong shape, and the stop signal alone.
What has never existed is a demand and its matching short answer in the same turn. If the classes
still do not comply after this, the per-token objective is the last explanation standing and the
work moves to the loss or the decoding.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

