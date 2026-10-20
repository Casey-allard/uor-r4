# #1597 — Accept dialogue protocol 2 in grounded session checkpoints

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T03:24:45Z
- **Updated:** 2026-10-09T13:17:34Z
- **Closed:** 2026-10-02T03:56:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1597
- **Merged:** 2026-10-02T03:56:37Z
- **Merge commit:** 1941ed0689cc57ddd48247f1de7ee7ea19002c6c
- **Base / Head:** main / UOR-Foundation:codex/grounded-protocol2-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

GroundedSession could not consume protocol-2 emitters because checkpoint identity validation admitted only protocol1. Reply display and snapshot validation also retained the protocol2 content-leading space.

Add `CheckpointIdentity::from_tokenizer_version(..., version, ...)` for explicit1/2 selection, with the existing constructor preserving version1 bytes. Checkpoint reload admits only those supported schemas and retains protocol-digest binding. Grounded sessions now use the bound protocol's reply-text rule while preserving raw generated IDs, recall-line segment boundaries and exact history.

Lab: Codex. Coordination and file ownership agreed on #1552 before editing. No Claude corpus/training/evaluator file changes. References #1552 and #962; this interface change does not qualify copying or language capability.

Validation at70b5d867: source review, rustfmt and diff check passed. Executed `cargo test --offline -p uor-r4-training --lib protocol_ -- --nocapture`:4passed,0failed (3newchecks plus retained protocol panel-count test). Tests cover explicitversion/unknownschema/resealedmismatch, protocol2actual-loaded-emitterrecall+snapshot continuation and protocol1preservation. Cold compilation10m28s; executedtests0.09s. No fitting/heldout evaluation or copy-quality claim. Five protected CI status names are compatibility acknowledgements, not those tests.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

