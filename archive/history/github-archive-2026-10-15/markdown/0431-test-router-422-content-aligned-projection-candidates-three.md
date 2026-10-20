# #431 — test(router): #422 — content-aligned projection candidates: three NEGATIVES map a spread-vs-retrieval frontier

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T06:11:50Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-08-05T07:05:42Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/431
- **Merged:** 2026-08-05T07:05:42Z
- **Merge commit:** 0f1f7e2d15f79a9b10b45cdf3c8209a55fc8f730
- **Base / Head:** main / UOR-Foundation:i422-redesign
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #422 #306 #421. Three content-aligned candidates measured against the pre-declared rule (beat pre-#306 shim MRR 0.0743 by ≥0.020 AND beat shuffled), all behind R4_HOPF_REDESIGN=1 (base measurement reproduces unchanged):

| arm | filtered MRR | vs rule | occupancy |
|---|---|---|---|
| E data-PCA (per-block top PC, deterministic power iteration) | 0.0176 | −0.0568 | 448/512 |
| F magnitude+sign hybrid | 0.0308 | −0.0436 | 129/512 |
| G mean-direction | 0.0749 | +0.0005 (short of +0.020) | 53/512 |

**All NEGATIVE — but the pattern is the finding: a clean spread-vs-retrieval frontier.** Spread-maximizing directions (shipped Blake3, data-PCA) lose retrieval; retrieval-preserving directions collapse toward the magnitude/mean axis. G matches the pre-#306 MRR at 3.3× its occupancy — the closest point on the frontier — but no candidate gets both. Interpretation: at this sector binning, block content variance orthogonal to the mean carries little retrieval-aligned signal; a cheap projection swap will not fix #422. The redesign needs a deeper change (binning geometry, or retrieval-supervised directions) — scoped for the next phase (Ari).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

