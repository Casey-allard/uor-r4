# #415 — feat(certify): #399 B′ — self-anchor and confidence-gated arms: two-pass serving is VIABLE

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T20:49:43Z
- **Updated:** 2026-10-09T14:17:04Z
- **Closed:** 2026-08-04T21:09:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/415
- **Merged:** 2026-08-04T21:09:53Z
- **Merge commit:** e6b5e41811fc9e2649a22164c2ea6397c3b1aaf4
- **Base / Head:** main / UOR-Foundation:i399-bprime
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #399. B′ decision measurement (Casey approved the B′-then-A path): does the M2 forward-anchor lift survive when the engine supplies its OWN anchor instead of the true one? **Yes — fully.**

| scorer | top-1 | bits | live slice |
|---|---|---|---|
| 1+2 × fwd (true anchor) | 39.6% | 7.9222 | 40.9 vs 36.7 (73,806) |
| 1+2 × fwd SELF-anchor | 39.8% | 7.8944 | 41.2 vs 36.6 (74,679) |
| **1+2 × fwd GATED self** | **40.0%** | **7.8332** | **45.0 vs 39.4 (63,567)** |

Predicted-anchor accuracy is only 32.5%, yet self-anchoring slightly EXCEEDS the true-anchor arm — the fused row is keyed by the engine's own modal prediction and stays self-consistent with the scorer's evidence. The confidence gate (trust the anchor only where it resolved ExactContext) concentrates the effect: win/loss +6,800/−3,232 (2.10:1), bits 6.7244 vs 7.5453 on its slice. All pre-existing rows byte-identical; serving untouched (schema 19 instrumentation).

**Consequence: two-pass self-anchored serving (option B) is measurably viable with no external anchor source.** Next: FWDA artifact section + serving entry point.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

