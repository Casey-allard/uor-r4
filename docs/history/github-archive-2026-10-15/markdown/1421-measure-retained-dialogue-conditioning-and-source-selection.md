# #1421 — Measure retained dialogue conditioning and source selection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T06:41:55Z
- **Updated:** 2026-10-09T13:17:14Z
- **Closed:** 2026-09-27T06:43:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1421
- **Merged:** 2026-09-27T06:43:14Z
- **Merge commit:** bbce4cf62b3ad3d6965cb1f8702e3c557033ca0c
- **Base / Head:** main / UOR-Foundation:codex/dialogue-context-audit-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained R1d dialogue trainer starts recurrent windows at arbitrary token positions while supervising response tokens whose original prefixes may lie outside those windows. Add a Rust data-only audit that reconstructs the exact historical sampling stream, checks every supervision denominator, and measures complete-prefix eligibility by source dataset.

All2,237 curve rows match. Of6,280,627 response-target visits,74.6033% have all pre-response context outside the window. Only14,826/129,486 complete response prefixes fit within256 total IDs;89.4847% of those eligible response runs come from Everyday Conversations and Smol Constraints. The result makes population/weighting choices explicit before a future request-preserving sampling experiment. It does not establish the cause of weak generation or change any model, sampler, context limit or running study.

Validation: three focused optimized Rust interval/boundary checks pass; the actual data audit completed in3.210s with261,144,576B maximum child RSS and no resource stop. Source-bound build/check work89.734s, including both example tests and executable build. Native report seal, exact rowwise reconstruction, independent source/result review, touched rustfmt, diff whitespace and claim wording pass. The receipt binds source, executable, train tokens/mask, retained fit and all output hashes; historical mask binding remains retrospective. No model execution or new held-out scoring.

References #973. The canonical plan, current state and project map point to the result. Source changes are confined to one new training example; the separately owned dialogue trainer and frozen reader study are preserved.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

