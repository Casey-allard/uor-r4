# #442 — test(certify): #434 — ARM BF full-width anchor supply: de-banding rule PASSES (+2.1pp anchor accuracy)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T20:15:19Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-08-05T20:43:23Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/442
- **Merged:** 2026-08-05T20:43:23Z
- **Merge commit:** 0550d5db9be1087c29bd39d503bf8fb151332171
- **Base / Head:** main / UOR-Foundation:i434-bf
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #434 #245 #421. The de-banding payoff test: **ARM BF (full-width retrieval anchor supply) free top-1 8.17% vs banded ARM B 7.64% (+0.54pp), vs nulls 2.14/3.09; anchor accuracy 11.4% vs 9.3% (+2.1pp). All three pre-declared conditions PASS — de-banding pays.** Honest read: the 3.8× retrieval-MRR gain converts to a modest anchor gain — retrieval quality is no longer the binding constraint on anchor supply; the continuation side-table is. Decision (Casey, C→A): **de-banding ADOPTED** as an implementation item — consumer audit of band-sparsity assumptions first, then full-width storage in the #245 path; priority moderate (the case = 3.8× raw retrieval + measured anchor gain).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

