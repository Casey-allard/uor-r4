# #418 — feat(certify): #399 — strict per-step gate on the draft-anchor arm: second falsification, B-mode closed

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T00:07:34Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-08-05T00:28:59Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/418
- **Merged:** 2026-08-05T00:28:59Z
- **Merge commit:** 91f24e694f1a6e2a32e5006aa6c0bdd84acb5a33
- **Base / Head:** main / UOR-Foundation:i399-strictgate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #399. Rescue variant for the falsifier-1 negative (#417): the draft anchor is trusted only when EVERY intermediate greedy step resolved ExactContext, not just the final one.

**Result (schema 21, natural wiki-obs; prior rows byte-identical): STILL NEGATIVE.** Strict-gated 35.8% / 9.3593 all; live slice 42.0% vs Rule 1+2 43.0% (n=59,653); win/loss +827/−1,449. The strict gate selects even easier positions (base 43.0% there) and the channel still loses — drift is not concentrated in low-confidence steps.

**With #417's record this is the second falsification: standalone two-pass B-mode is CLOSED.** A-mode (anchors as inputs, +4.2pp live, drift-immune) is the adoption path; its serving surface follows in the next PR.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

