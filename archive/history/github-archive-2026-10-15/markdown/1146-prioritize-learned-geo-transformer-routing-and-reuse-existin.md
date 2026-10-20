# #1146 — Prioritize learned geo-transformer routing and reuse existing foundations

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T02:25:09Z
- **Updated:** 2026-10-09T13:17:16Z
- **Closed:** 2026-09-06T02:49:20Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1146
- **Merged:** 2026-09-06T02:49:20Z
- **Merge commit:** 04bf2f9c6d6549cba9541b9f7acebe272283d14f
- **Base / Head:** main / UOR-Foundation:codex/geo-transformer-direction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The immediate plan still prioritized residual NoWrite scoring while postponing learned nonlinear transformations. Following the owner's clarification, #1139 now builds a jointly learned geo-transformer block: semantic placement, bounded context admission and selected integer/table computation connected to language prediction and short composition. #1140 remains the subsequent multi-operation qualification; composition supplies a learning signal during #1139.

The canonical plan names reuse boundaries for UOR/addr, NAF/GNAF, the graph compiler/R4G1, XOR/popcount selection and the currently pinned uor-matmul Atlas lookup/add implementation. Existing binding/memory results and the #1145 sparse execution improvement remain retained. Prior NoWrite score-bound work is secondary and unimplemented. The new block is explicitly unimplemented and unqualified; offline Rust matrix operations remain permitted and dense transformer serving remains excluded.

Validation: claim-wording check, existing architecture-policy executable, local file-link resolution and git diff --check pass. Reviewed the relevant original routing, NAF, graph-runtime and pinned uor-matmul source. No Rust source or dependency change; no new compilation, training, model evaluation or performance measurement was run. Cumulative model time remains 1,363.884/1,800 seconds. No storage-cap change, material deletion or paid compute.

Updates #973, #1139 and #1140; none is closed by this plan clarification.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

