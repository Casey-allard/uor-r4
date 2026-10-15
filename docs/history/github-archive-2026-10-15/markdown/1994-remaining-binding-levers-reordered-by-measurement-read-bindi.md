# #1994 — Remaining binding levers reordered by measurement: read-binding supervision ranks last (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T03:37:53Z
- **Updated:** 2026-10-09T13:20:53Z
- **Closed:** 2026-10-09T03:39:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1994
- **Merged:** 2026-10-09T03:39:32Z
- **Merge commit:** 144b0bf97115bfcf41643fcfc51cb5a1e1baea1a
- **Base / Head:** main / UOR-Foundation:deepseek/levers-reordered
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Round 179 read the last untried supervision term interface. What it optimises is the point: it adds W * -log(m + 1e-6) where m is a read layer binding head attention mass on the EXPECTED VALUE HISTORY POSITIONS - it supervises WHERE THE READ LOOKS. The probe already measured that and it is already high: pointer mass 0.90-1.00 on the value span in 9 of 21 cells for the ungated pointer arm, read mass at or above TAU in 62 of 63 cells for the earlier store arms, control below TAU everywhere. The read lands; the failures are downstream - the continuation, the copy gate (unraisable without damage across four weights), and the identity mixture competition. So the term the review listed first among the untried ones optimises the least likely binding quantity. REORDERED: a token-identity pointer key instead of a hidden-state key; then a constrained or two-token decoder that draws the answer from the span and sidesteps the competition the gate sweep established, ranked second only because it is serving-side rather than learned; last, read-binding supervision. Falsifiable: if the first two fail while read-binding supervision succeeds, this ordering was wrong and will be corrected in public. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

