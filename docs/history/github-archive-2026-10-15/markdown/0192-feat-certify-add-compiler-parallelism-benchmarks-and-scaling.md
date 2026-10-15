# #192 — feat(certify): add compiler parallelism benchmarks and scaling certificates (#175)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-25T04:54:22Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-07-25T05:13:45Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/192
- **Merged:** 2026-07-25T05:13:45Z
- **Merge commit:** 946fb25e03c79e5d1f16a536ccb807ac9aa9b0a1
- **Base / Head:** main / UOR-Foundation:issue-175-compiler-scaling-benchmarks
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Implements Issue #175: Compiler parallelism benchmarks and scaling certificates.

- **Normative Specification ([](file:///Users/casey.allard/uor-r4/docs/compiler_scaling_certificate.md))**: Defines empirical metrics (wall-clock time, CPU utilization, peak RSS, throughput), thread sweep evaluation matrix (=1, 2, 4, N$), 5-way bottleneck classification taxonomy (`Scaling`, `BandwidthLimited`, `TeacherLimited`, `SequentialFinalization`, `OversubscriptionPenalty`), and CPU-only environment rules.
- **Scaling Certificate Engine ([)**: Implements `CompilerStageMetrics`, `StageScalingClassification`, `HardwareMetadata`, `CompilerScalingReport`, and `CompilerScalingEngine`.
- **Proof Model Verifier & Vocabulary**: Added `verify_compiler_scaling_certificate_compliance` in `crates/uor-r4-proof-model/src/structural_guarantees.rs`; updated `docs/formal_vocabulary.md` to v0.1.13.

Closes #175.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5077015008 — 2026-07-25T04:54:35Z — Casey-allard

@copilot please review this PR

### comment 5077021611 — 2026-07-25T04:56:19Z — Copilot


> @copilot please review this PR

Reviewed. I validated the changes in this PR by running focused tests on the touched crates (`uor-r4-graph-certify` and `uor-r4-proof-model`), including the new scaling-certificate coverage, and everything passed. I did not find issues requiring code changes.


---
_Comment/review ids in this file: 5077015008, 5077021611_

