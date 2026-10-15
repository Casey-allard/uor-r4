# #1054 — Research: exact-byte Zoology transfer and optimizer-clock gap (#1053)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T14:18:09Z
- **Updated:** 2026-10-09T14:16:50Z
- **Closed:** 2026-09-02T14:19:29Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1054
- **Merged:** 2026-09-02T14:19:29Z
- **Merge commit:** 1cc1ce115a84d5d10b13d0c97ec0232425b70e0b
- **Base / Head:** main / UOR-Foundation:issue-1053-zoology-exact-transfer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #1053. References #973.

## Delivered
- Added the isolated `ZoologyExact1045TransferV1` lifecycle over the exact open #1045 data.
- Bound the full local implementation closure plus `pyproject.toml` and `uv.lock`.
- Proved that a saved passing checkpoint finalizes without another optimizer step.
- Preserved historical #1050 implementations and evidence.

## Frozen result
`STOCK_CELL_TRANSFER_MISS`: final development 984/8,192 (12.01171875%), NLL 6.79664158821106, after 64 epochs / 1,024 optimizer updates / 4,194,304 training query presentations. Primary wall 324.061369 seconds. CPU preflight selected four Apple Accelerate threads over one and eight. Binding control: `NOT_RUN_PRIMARY_MISS`; no extra fit or geometry change.

Result CID: `blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1`.

This is a bounded population/training-recipe miss, not a serialization diagnosis or attention falsifier. The nominal 64-epoch schedule allowed only 1,024 updates and annealed to zero, while the source-positive run first showed strong binding around 2,940 updates and passed at 3,920. #1050's positive remains intact. A separately scoped optimizer-clock correction is recommended, not executed.

## Evidence
- 31 focused loader/model/lifecycle tests and C0 passed.
- Fresh-process artifact inference reproduced exact final logits/NLL/top-1.
- Independent read-only audit matched all 20 exported tensors to the checkpoint and verified every parameter's 1,024 optimizer steps.
- Python format/baseline Ruff, Rust format, claim wording, and diff integrity passed.
- Broad QA, generation, reasoning, R4 comparison, and lowering were not run.
- Protected queue acknowledgements are transport only, not verification evidence.

Current summaries and the append-only research record are reconciled with the measured outcome and its limits.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

