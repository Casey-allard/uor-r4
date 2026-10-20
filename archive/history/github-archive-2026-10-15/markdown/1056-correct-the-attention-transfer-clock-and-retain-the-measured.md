# #1056 — Correct the attention transfer clock and retain the measured gain (#1055)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T16:33:27Z
- **Updated:** 2026-10-09T14:16:50Z
- **Closed:** 2026-09-02T16:34:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1056
- **Merged:** 2026-09-02T16:34:51Z
- **Merge commit:** f2f2396bedeafaa8377eb67232f02e82ca8b45d9
- **Base / Head:** main / UOR-Foundation:issue-1055-optimizer-clock
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- Correct the exact-data transfer optimizer clock: 196 updates per source-clock block, resumable complete shuffle traversal, strict early stop, durable checkpoints, and measured CPU selection.
- Retain the development gain: 12.01171875% → 68.2861328125%; NLL 6.7966416 → 2.6761019. The frozen >99% target remains missed (`CLOCK_MATCHED_TRANSFER_MISS`); the conditional binding control did not run.
- Preserve the refused resource preflight and user-authorized timing retry. The single primary fit took 1,078.926785 seconds on eight Apple Accelerate CPU threads: 3,920 updates and 16,056,320 training queries. No CUDA/MPS.
- Update current summaries and append-only records. The user approved a separate bounded continuation of the saved checkpoint; none ran within #1055.

## Focused evidence

- The 15 named clock/contract/sampler/recovery tests passed in 0.982 seconds before freezing implementation `47ebdabc`; Python formatting and scoped Ruff baseline passed. Code did not change after preparation.
- Fresh-process inference reproduced exact final logits and result. Independent read-only checkpoint audit matched all 20 exported tensors, 3,920 AdamW steps, scheduler block 20, 245 complete sampler cycles, and RNG. Verification added 8,192 inference decisions and no fitting.
- Claim wording and diff integrity pass. Broad workspace/BDD/WASM/release QA: `NOT_RUN`. Queue acknowledgements are transport only, not test evidence.
- Result CID: `blake3:3cb810f09a118cfb70752643f5d9e60d0e42780dc6e47dc4f99224cbd69af0ee`.

Closes #1055.
References #973.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

