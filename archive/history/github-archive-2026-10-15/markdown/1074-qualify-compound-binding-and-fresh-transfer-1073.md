# #1074 — Qualify compound binding and fresh transfer (#1073)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T00:35:59Z
- **Updated:** 2026-10-09T14:16:52Z
- **Closed:** 2026-09-03T00:38:25Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1074
- **Merged:** 2026-09-03T00:38:24Z
- **Merge commit:** d7a369cb53a0311ff35111bb20a99c1aaba7dfde
- **Base / Head:** main / UOR-Foundation:issue-1073-compound-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous token-level binding candidates did not qualify on owner/object counterfactuals. This adds one frozen, 286,976-parameter model with independent learned compound Q/K, shared fact-location V, learned null, ordinary softmax and a full vocabulary head. Fixed grammar supplies causal roles; there is no target-derived routing or answer filter.

At the unchanged 3,920-update dose, the final candidate passes every declared gate: all four construction orders are 100% on 8,192 supported and 2,048 absent queries; all fresh orders are 100% on 1,024 supported and 256 absent queries. Full construction predictions are order-identical (maximum logit difference 5.7220459e-6). Cycling only the four fact values preserves attention exactly, changes all supported predictions to their reassigned locations and preserves all absent answers.

The frozen preparation, actual fit ledger, result, exact fresh-process replay and interpretation are published in docs/r4_zoology_compound_binding_1073.md and its four JSON envelopes. The six current mirrors identify unchanged-R4 preservation of the complete four-fact-plus-null weighted mixture as the next experiment. This qualifies structured binding in familiar grammar; learned English parsing, general language, free generation and R4 preservation for this artifact remain unestablished.

Validation: 18 focused tests, source/preparation/evidence reviews, Ruff, claim wording and exact full-evidence replay. One fit; 2,007,040 presentations; 63.7994 seconds cumulative; peak 1.323975 GiB. No broad workspace/release QA; required queue statuses acknowledge transport only.

References #1073. Parent #973 remains open; #954 remains blocked.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

