# #1010 — feat(#973): compile and qualify source-free R4 softmax trace student

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T11:21:21Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-08-30T11:22:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1010
- **Merged:** 2026-08-30T11:22:24Z
- **Merge commit:** 02b6a08a3dfd8c485d5d22353e0f37ad0cfd9926
- **Base / Head:** main / UOR-Foundation:issue-973-r4-softmax-trace-compiler
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and qualifies the first bounded source-free compilation rung after the accepted R4/Spin causal-softmax reference.

- Adds `R4SoftmaxTeacherTraceV1` at the exact transported attention seam.
- Adds canonical `R4SoftmaxTraceStudentV1` bytes with teacher-distilled Q16, observed-count, and document-permuted arms.
- Runtime prediction uses only artifact bytes plus token history: no source-model calls, softmax, floating point, or sampling.
- Preserves the construction-before-reveal boundary, exact causal/R4 work audits, and byte/CID replay.
- Reconciles the living roadmap, research, lifecycle, programme, glossary, and public claim boundary.

## Frozen result

Implementation revision: `3cbeab64ba40f4445f878741b87ec6a487fcee00`

On the nine non-BOS held-out context-bearing positions:

| Arm | covered teacher CE | teacher top-1 | actual-next top-1 |
|---|---:|---:|---:|
| distilled | **2.660721** | **3/9** | **2/9** |
| count | 9.678894 | 2/9 | **2/9** |
| permuted | 4.342019 | 1/9 | 1/9 |

Artifact CID: `blake3:e3b48b8bd113bf71be2fe9ecb64257b4eb1516303966d9d6c2c5cbe9d46adfac`

Freeze CID: `blake3:bb19fc6f6976aca6dfd8c67c470fd1fb70a1e1e74763800fdcb635f135325df7`

Result CID: `blake3:e48b4172e02fc84eef9e00024ac6602b790d8230026a979d2f71c552ddca0cd4`

Terminal: `PASS_SOURCE_FREE_TRACE_STUDENT_ADVANCE_GEOMETRIC_STATE_COMPILER`

The source-free continuation is still negative product evidence: `He was born` enters a period-2 `, Scotland` loop. This establishes bounded trace distillation, not geometric advantage, coherent generation, general inference, reasoning, WASM readiness, or release readiness.

## Verification

- focused core student tests: 5 passed
- focused trace-experiment tests: 3 passed
- CLI parser test: passed
- strict modified-crate clippy, format, claim wording, no_std, and wasm checks: passed
- full workspace sweep exposed one exact-executor source-list omission; repaired and the exact failing test reran green
- independent replay recomputed every metric, CID, predicate, and continuation token with no P0/P1 finding
- tracked base64 artifact decodes byte-identically; tracked freeze matches the local seal

## Next gate

`R4SoftmaxTraceStateStudentV1`: first add fail-closed trace/bundle reload and tamper coverage, then compile construction traces into causal recurrent R4/Spin state and compare against the frozen suffix, an equal-budget plain recurrent cell, and transport/state-permuted controls.

References #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

