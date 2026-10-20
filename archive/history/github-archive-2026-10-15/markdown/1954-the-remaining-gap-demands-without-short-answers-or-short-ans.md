# #1954 — The remaining gap: demands without short answers, or short answers without demands (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T19:17:40Z
- **Updated:** 2026-10-09T13:21:52Z
- **Closed:** 2026-10-08T19:19:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1954
- **Merged:** 2026-10-08T19:19:19Z
- **Merge commit:** a41b16517a05050cce89733f98d4ea74cc34c524
- **Base / Head:** main / UOR-Foundation:deepseek/remaining-gap
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The remaining gap, stated precisely: demands without short answers, or short answers without demands

The synthetic generator's own documentation says its replies "always keep the single canonical
answer form, so an arm that varies forms varies how a fact is stated and asked, NOT HOW IT IS
ANSWERED", and forms is clamped to 1..=8. So it cannot supply a demand-bearing turn at any
setting: it produces short answers reliably (20,000 documents at a 14.9 percent response
fraction against the chat corpus's 68.9 percent) but its user turns are relation phrasings,
never an instruction on the answer's form.

The chat corpus is the mirror image. It contains 17,205 turns whose user text says the answer
must or should do something, and 142 in the whole contract family - so it has demands, whose
replies are prose at a median of 78 words - but only 26 one-word replies in 129,465 responses,
of which 4 survive the context filter.

So the project possesses each half separately and neither together: demands without short
answers in the corpus, short answers without demands in the synthetic store, and the pair
nowhere. That is the precise reason the combination arm reproduced the stopping signal's
behaviour and moved nothing on the one-word and numeric classes - nothing the model has trained
on ever shows a demand and its matching short answer in the same turn.

The artefact to build is therefore named: a demand-bearing short-form store with instruction
turns paired with replies that are exactly that form, in the literal-role protocol with the mask
convention the trainer requires. If that still leaves those classes non-compliant, the per-token
objective is the last explanation standing and the fix moves to the loss or the decoding.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

