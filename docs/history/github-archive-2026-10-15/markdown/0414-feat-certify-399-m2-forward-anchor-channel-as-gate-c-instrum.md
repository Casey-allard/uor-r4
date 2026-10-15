# #414 — feat(certify): #399 M2 — forward-anchor channel as Gate C instrumentation (first in-pipeline positive)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T19:10:08Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-04T19:40:18Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/414
- **Merged:** 2026-08-04T19:40:18Z
- **Merge commit:** 5de012f7faa243d74bbe98c61afd11d02d27bde6
- **Base / Head:** main / UOR-Foundation:i399-m2-fwd-channel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #399 (M2 of the substrate track; not a close — the adoption decision follows).

**What:** the forward-anchor channel of the anchor-infill program (#394/#399), added to the score pipeline's own Gate C eval as instrumentation. Certifier-side only — the serving scorer is untouched (emitted `score.r4g1` κ unchanged: `blake3:6fccc422…`). Table: (lookahead distance, next-anchor token) → predecessor counts from the construction split, infill stride 4. Fusion: the measured product law in the scorer's own units (ScoreQ = ln×2⁻¹⁶ → `score_q/65536 + smoothed-ln(fwd)`), floors on one scale per the #387 accounting rule, canonical tie-break. Report schema 18: `rule12_fwd_fused`, live-slice pair, `fwd_vs_rule12(_live)` win/loss.

**Measured (natural wiki-obs, 100,306 held-out positions; all pre-existing rows byte-identical to the #393 substrate run):**

| scorer | top-1 | bits/token |
|---|---|---|
| Rule 1+2 | 36.5% | 8.3534 |
| TLA3 store baseline | 39.1% | 8.4461 |
| **1+2 × fwd-anchor (M2)** | **39.6%** | **7.9222** |

Live slice (73,806 positions, 73.6%): fused **40.9%** vs Rule 1+2 36.7% (+4.2pp), bits 7.7437 vs 8.3299. Win/loss on live: +6,649 / −3,550 (1.87:1). **First configuration to beat the store baseline in-pipeline, and the first in-pipeline confirmation of the harness composition law.**

Adoption into serving is a separate decision under the engine quality-gate governance (Alex/Ari); this row is the data for it.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

