# #1943 — Doubling training does not close the instruction-precision gap (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:37:41Z
- **Updated:** 2026-10-09T13:21:49Z
- **Closed:** 2026-10-08T17:39:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1943
- **Merged:** 2026-10-08T17:39:21Z
- **Merge commit:** a89dc95b814fb4a9e8684368ddcf3de9017ca105
- **Base / Head:** main / UOR-Foundation:deepseek/r7-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Doubling the training does not close the instruction-precision gap

The working configuration (19.9M, rrarrarr, read L2, full_prefix, the verified response mask)
trained for 24,414 steps - twice the arm it is compared against - finished with development
response nll 1.6801235877441423 and was exported and scored automatically by its watcher.

On the target it changes nothing. Read as the first two content tokens per reply, not one
instruction-form request is complied with: no bare Yes or No, no bare number, no one-word
answer. What is intact is everything the objective already bought - canonical greetings,
topical follow-ups, and BOTH retention turns now opening by answering the question asked
("My name", "Smallly, cats") where the chat-only model returned the previous turn's text
verbatim. The regression guard holds.

That rules out optimization as the cause and leaves the two explanations the round-81 review
ranked first and third: data support, which the queued policy=truncated_prefix arm tests with
one argument and 14.75x the short-form episodes; and the token-level MLE length bias, which
predicts that even with support the model continues after "Yes" because 31 continuations carry
about 250:1 of the token mass against 16 stops. If truncated_prefix fails as this arm did, the
objective becomes the leading explanation and the fix moves from data to the loss or decoding.

Also a third data point that development nll does not track this objective's quality: the
longer run's dev nll is worse (1.6801 against 1.4176) while its behaviour is equivalent.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

