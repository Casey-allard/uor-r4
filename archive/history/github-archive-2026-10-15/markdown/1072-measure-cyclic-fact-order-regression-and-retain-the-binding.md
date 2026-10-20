# #1072 — Measure cyclic fact-order regression and retain the binding reference (#1071)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T00:02:02Z
- **Updated:** 2026-10-09T14:16:51Z
- **Closed:** 2026-09-03T00:03:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1072
- **Merged:** 2026-09-03T00:03:15Z
- **Merge commit:** 88b084450067aa770099072f85348b00adbc1e6b
- **Base / Head:** main / UOR-Foundation:issue-1071-cyclic-facts
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The cyclic fact-order fit regresses against retained #1067 in every tested order. Canonical accuracy falls from 45.5933% to 20.7764%; the candidate scores 20.7764–21.0083% across four rotations. Owner/object paired correctness and all slot criteria miss. Keep #1067 as reference and preserve this negative result.

The change adds the frozen rotation trainer, three-history development exclusions, matched four-order evaluation, conditional fresh transfer and exact replay. Source and preparation were published before the sole 3,920-update fit. The reference's complete canonical score and diagnostic reproduce exactly. All 15 focused checks pass, alongside independent source, preparation, evidence and record reviews. Fresh-process replay matches the complete evidence. Fit, comparison and replay total 315.589 seconds, peaking at 0.78595 GiB.

Fresh development remains unscored. No R4, geometry change or generation occurred. Broad QA remains dormant; required queue statuses acknowledge transport only.

The record and six current mirrors recommend a separate learned compound-key/value binding prototype with explicit fixed-grammar roles, ordinary softmax, a full 4,096-token head and counterfactual controls. No successor prototype ran here.

Closes #1071. References #973 and #820; #954 remains blocked.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

