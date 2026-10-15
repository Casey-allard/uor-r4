# #1078 — Learn contextual role reading over frozen compound binding

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-03T01:49:35Z
- **Updated:** 2026-10-09T14:16:52Z
- **Closed:** 2026-09-03T01:51:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1078
- **Merged:** 2026-09-03T01:51:39Z
- **Merge commit:** 6ebcf4bf48ed82addd0a38c7fe368b989815f772
- **Base / Head:** main / UOR-Foundation:issue-1077-language-interface
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Replace fixed role-position lookup with a learned contextual role reader while retaining the qualified compound-binding model. The new reader uses every valid clause token, including a competing owner name, and passes full soft role mixtures into the frozen four-fact-plus-null Q/K/V core and 4,096-token decoder.

One fixed 512-update reader fit qualifies all 25,600 primary answers and 358,400 supervised role decisions at 100%, including two withheld clause/owner-phrase combinations. Every identical-bag owner contrast receives both correct answers. The construction value-cycle control gives 0/8,192 original supported answers and 8,192/8,192 reassigned answers per view, with all UNKNOWN answers correct and both role and binding attention unchanged.

The result is `LANGUAGE_INTERFACE_HELDOUT_PASSED`. A fresh process reproduces the complete evidence exactly. Fit/evaluation/replay took 49.9201 seconds, peak 1.457169 GiB; synthetic timing selected four CPU threads over eight. Sixteen focused checks, Ruff, claim wording and independent source/preparation/evidence reviews provide scoped verification. Broad QA remains dormant; protected-queue compatibility statuses acknowledge transport only.

Includes frozen preparation/fit/result/replay envelopes, the appended measurement record and six current project mirrors. This is supervised local owner disambiguation with supplied clause boundaries, known vocabulary, a seen question form and already observed semantic worlds. It does not establish general English, generation or geometry advantage. The ordinary learned interface differs numerically from the canonical hard-field oracle (maximum logit difference 0.0623424); the next separate R4 qualification must use the learned ordinary path as its reference. Both reader and core stay fixed, with no R4 run or geometry expansion in this issue.

References #1077. #973 remains open and #954 remains blocked. Final native tracker/milestone reconciliation and issue completion follow verification of the protected merge.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

