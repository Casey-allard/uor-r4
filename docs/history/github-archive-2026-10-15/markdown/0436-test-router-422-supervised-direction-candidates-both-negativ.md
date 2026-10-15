# #436 — test(router): #422 — supervised direction candidates: both NEGATIVE; linear projection space exhausted

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T16:37:56Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-08-05T17:02:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/436
- **Merged:** 2026-08-05T17:02:54Z
- **Merge commit:** a622df5913fb55169622dc7f681f7c36e80e1ea1
- **Base / Head:** main / UOR-Foundation:i422-supervised
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #422. Fisher retrieval-supervised directions (within = shared-continuation pairs) and supervised-sign magnitude, measured on the original 500k corpus for frontier comparability: **arm H fisher-dir 0.0195 MRR / 430 sectors; arm H2 mag+fisher 0.0428 / 121 — both NEGATIVE** vs the ≥0.0763 rule (both beat shuffled). With phases 1-3 (shipped Blake3 0.0045/456, pre-#306 0.0743/16, PCA 0.0176/448, mag+sign 0.0308/129, mean-dir 0.0749/53), seven projections including two supervised all fall on one spread-vs-retrieval frontier. **Conclusion: retrieval-aligned signal is not linearly recoverable from Hopf block inputs at this sector binning — the linear per-block projection design space is exhausted.** Honest next scope: binning geometry (lambda/chi lattice) or non-block representations. All prior rows byte-identical.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

