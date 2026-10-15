# #410 — refactor(graph-runtime): #400 — remove syntactic_morphism_score (measured-dead, verified identical)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T16:11:51Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-04T16:29:26Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/410
- **Merged:** 2026-08-04T16:29:26Z
- **Merge commit:** 7d83825d33ca9c7e8d44b5aab5515e50da482975
- **Base / Head:** main / UOR-Foundation:i400-cd-removal
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #400 with the full three-part record: (1) scope — the certify C-row surface never calls the term; (2) root cause — context_backoff short-circuits the graph path before node candidates; (3) measurement — on a rows-on production-shaped bundle the node-candidate path ran 0/1,998 sampled positions; the term executed never. Removal verified behavior-identical on the serving surface (36.49%/40.68% before and after). Hard-coded token-id gating in the same dead path removed with it; pub cayley_dickson module retained (no engine references remain). Part of #393 §7.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

