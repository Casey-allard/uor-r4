# #1278 — Learn unknown-neighbor query roles with retained-output preservation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T18:06:54Z
- **Updated:** 2026-10-09T13:17:58Z
- **Closed:** 2026-09-14T18:07:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1278
- **Merged:** 2026-09-14T18:07:56Z
- **Merge commit:** cfe89267605ade42be41db776afaaf7c8dcc08db
- **Base / Head:** main / UOR-Foundation:codex/unknown-neighbor-learning
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Unseen names and trained question words could collapse to the same query-role observation, rejecting otherwise valid or conflicting continuations. Retain the enclosing artifact's trained query vocabulary, remap canonical neighbor identities exactly, and learn masked-name observations from frozen output-compatible credit. Constrain UNKNOWN/wildcard induction while retaining specific alternatives. Serving observation, routing and generation algorithms remain unchanged.

The final candidate passes 492/492 open-development paths (parent 468/492), all 2,976 training answers, and 300 + 200 + 6,688 retained comparisons. Anchors-only and no-projection controls each remain 468/492. Two rejected candidates and their 320/80 historical regressions are preserved; the final candidate restores all historical outputs and preserves both negatives' successful transfer outputs. Normal model remains unpromoted; independent held-out transfer is the next step, not a completed claim.

Validation: final optimized build passes 51 focused tests; actual report gate PASS_LEARNED_UNKNOWN_NEIGHBORS; cargo fmt --check; claim-wording check; git diff --check. Sealed reports and frozen source/binary/artifact hashes are verified. Charged model/build/evaluation work is 959381 ms, within the recorded 1200000 ms local step allowance; no external spending or cleanup. Historical queue statuses are compatibility acknowledgements, not tests.

Result and limits: docs/native_geometric_unknown_neighbor_973.md and docs/evidence/native_geometric_unknown_neighbor_973.json. Current state records the independent no-fit transfer panel next.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

