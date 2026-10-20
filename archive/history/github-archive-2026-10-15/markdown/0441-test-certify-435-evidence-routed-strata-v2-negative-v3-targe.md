# #441 — test(certify): #435 — evidence-routed strata (v2): NEGATIVE, v3 target identified

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T19:14:55Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-05T19:35:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/441
- **Merged:** 2026-08-05T19:35:53Z
- **Merge commit:** c38da595194d15f96c8bbccbb19a62aeafb7c4a5
- **Base / Head:** main / UOR-Foundation:i435-routing2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #435. Evidence-mass routing (per-position and per-story, K∈{8,32}, modulo controls): **all arms below flat** (best 34.0 vs 35.2; rule needed +2pp). Mechanism found: hard argmax-mass selection discards the counts other strata hold for the same key — the flat store aggregates them for free; content-vs-modulo routing differ by only ~1pp (mass votes track stratum size, not topic). The oracle edge persists (36.3 > 35.2) and its wins are DISTRIBUTION FLIPS on shared keys (oracle evidence-hit only 38%) — **v3 design target: soft mixture/interpolation of stratum distributions with per-key stratum-confidence weighting, not hard selection.** Prior rows reproduce exactly.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

