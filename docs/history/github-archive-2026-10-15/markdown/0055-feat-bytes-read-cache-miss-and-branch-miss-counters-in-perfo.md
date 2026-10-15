# #55 — feat: bytes-read, cache-miss, and branch-miss counters in performance certificates (closes #27)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T03:21:39Z
- **Updated:** 2026-07-22T03:29:08Z
- **Closed:** 2026-07-22T03:29:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/55
- **Merged:** 2026-07-22T03:29:04Z
- **Merge commit:** e307007cd64cae0e209654900ffcc269a4c916cb
- **Base / Head:** main / UOR-Foundation:feature/issue-27-performance-certificate-counters
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #27: Performance Certificate data structures with hardware operational metrics (bytes_read, cache_miss_estimate, branch_miss_estimate, ops_breakdown), hardware metadata, baseline threshold checking, and CBOR serialization.

## Timeline (6 entries; chronological: conversation comments, reviews, review comments)

### comment 5041451781 — 2026-07-22T03:21:42Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3627223292 — 2026-07-22T03:25:12Z — Copilot

_on crates/uor-r4-core/tests/performance_certificate_test.rs:4_

`HardwareMetadata` is imported but never used in this test file, which will trigger an `unused_imports` warning (and typically fails CI when running clippy with `-D warnings`).

### review 4750728336 — 2026-07-22T03:25:13Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds new transformerless evaluation and certification components to support Issue #27 by introducing performance certificate structures/serialization and a shortlist recall evaluator with gating logic.

**Changes:**
- Introduces `PerformanceCertificate` / `PerformanceProfiler` with hardware metadata, operational metrics, CID verification, and CBOR (de)serialization.
- Adds `ShortlistEvaluator` producing recall metrics and Gate H / Decision D5-related outputs plus CBOR roundtrip support.
- Adds new unit tests covering metrics, gating behavior, and CBOR roundtrips for both features.

### Reviewed changes

Copilot reviewed 5 out of 5 changed files in this pull request and generated 4 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/shortlist_evaluator_test.rs | New tests validating recall metrics, Gate H/D5 behavior, and CBOR roundtrip for shortlist reports. |
| crates/uor-r4-core/tests/performance_certificate_test.rs | New tests validating profiler metrics, threshold behavior, CID verification, and CBOR roundtrip for performance certificates. |
| crates/uor-r4-core/src/transformerless/shortlist_evaluator.rs | New shortlist recall evaluator/report types with gating logic and CBOR serialization helpers. |
| crates/uor-r4-core/src/transformerless/performance_certificate.rs | New performance certificate types, CID computation/verification, and CBOR serialization, plus a profiling helper. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new transformerless modules behind `cfg(not(target_arch = "wasm32"))`. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3627223351 — 2026-07-22T03:25:13Z — Copilot

_on crates/uor-r4-core/src/transformerless/performance_certificate.rs:72_

`compute_cid` hashes `serde_json` output and uses `unwrap_or_default()`, which silently turns serialization failures into hashing an empty payload. Elsewhere in transformerless certificates, the CID is computed over CBOR bytes (via `ciborium::into_writer`), which is also what `to_cbor_bytes()` emits. Using JSON here risks non-canonical / inconsistent CIDs and undermines determinism expectations.

### review-comment 3627223383 — 2026-07-22T03:25:13Z — Copilot

_on crates/uor-r4-core/src/transformerless/performance_certificate.rs:110_

`PerformanceProfiler::profile` hard-codes a 64-byte cache line size while also emitting `HardwareMetadata { cache_line_size_bytes }`. If the metadata default (or future inputs) diverge from 64, the certificate will contain self-inconsistent metadata vs. metric calculations.

### review-comment 3627223414 — 2026-07-22T03:25:13Z — Copilot

_on crates/uor-r4-core/src/transformerless/shortlist_evaluator.rs:115_

`worst_routing_error` is computed using `min(scores_graph.len(), scores_teacher.len())`, so if the vectors differ in length the tail is silently ignored and the reported worst error can be too small (which can incorrectly pass Gate H). It’s safer to treat a length mismatch as a hard failure for this metric (or at least force the gate to fail).


---
_Comment/review ids in this file: 5041451781, 4750728336, 3627223292, 3627223351, 3627223383, 3627223414_

