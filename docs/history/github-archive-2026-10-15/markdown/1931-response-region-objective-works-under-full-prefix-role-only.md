# #1931 — Response-region objective works under full_prefix; role_only caused the list collapse (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T16:55:17Z
- **Updated:** 2026-10-09T13:21:56Z
- **Closed:** 2026-10-08T16:57:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1931
- **Merged:** 2026-10-08T16:57:02Z
- **Merge commit:** 06bdf0b8bfdf1953121bb73e35de17165de47236
- **Base / Head:** main / UOR-Foundation:deepseek/response-region-verdict
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The control declared in round 56 - same mask, same recipe, policy=full_prefix - is measured: final dev response nll 1.3805535499925063 against role_only's 1.802750876406361.

Its replies read like an assistant: a greeting returns a canonical help opener, the France question returns the correct one-line answer, and several turns produce the shape of an assistant explaining what it can and cannot do and offering what it can instead. The bullet spam is gone entirely.

That answers round 56's open question - the collapse was role_only's, not the mask's. Training only on response tokens over-weights the corpus's bullet-heavy assistant style; training with the mask as a region signal does not.

Not yet a good assistant: Paris is right, the largest planet is not, two plus two degenerates, one opener repeats, and the first-token probe remains confounded for masked models. The register arrived before the reliability.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

