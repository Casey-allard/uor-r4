# #437 — test(certify): #433 — evidence-sparsity audit: ceiling re-attributed to the teacher signal

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T16:55:55Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-08-05T17:21:00Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/437
- **Merged:** 2026-08-05T17:21:00Z
- **Merge commit:** 4feae8446228b91d9c49ca9165acdd712e726eb6
- **Base / Head:** main / UOR-Foundation:i433-sparsity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #433 #320. Oracle evidence ladder (k=1..8 exact-context, both targets). **wiki10k, gate-C target: target-stream unigram floor 26.82%, any-k oracle 27.51% — the measured 26% sits AT the evidence ceiling, and the ceiling is the TEACHER: P(next==t_argmax)=6.4% on wiki vs 70.2% home (stories15M off-distribution, argmax near-degenerate). Original corpus: oracle 52.7% vs measured 36.5% — a real ~16pp representation gap in the home regime.** Consequences on the issues: #320 is the critical path for broad-corpus work; representation work (#422/#434/#435) has a quantified 16pp home-regime target.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

