# #440 — test(router): #434 — band-matched controls: zeta claim re-attributed to storage banding (~0.66 MRR lever found)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T18:39:40Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-08-05T19:02:56Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/440
- **Merged:** 2026-08-05T19:02:56Z
- **Merge commit:** 4119b05b794252ff2c4e4bcbbd5b465de4bc47ca
- **Base / Head:** main / UOR-Foundation:i434-bandctl
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #434 #245. The pre-declared control run: **arm ZB (zeta band-matched) 0.2564 — fails the ≥0.5×Z bar (0.4466); arm CF (content vectors full-width) 0.8948 — ABOVE zeta's 0.8932.** The 0.89 headline was vector fullness, not zeta geometry; the state evolution contributes −0.0016. Verdict: RE-ATTRIBUTED TO BANDING, per the rule.

**The actionable finding: the #245 storage path keeps only one active_range band (1/16th) of each stored vector, costing ~0.66 MRR vs full-width on this corpus (0.2348 → 0.8948).** De-banding storage is an engineering change with a measured ~3.8× retrieval upside, cascading directly into router anchor accuracy. All parity checks (46,938) tie the reconstructed band to the stored vectors. Zeta-anchor task correctly skipped (would be full-width retrieval renamed).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

