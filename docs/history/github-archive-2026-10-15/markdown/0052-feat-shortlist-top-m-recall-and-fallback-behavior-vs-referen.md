# #52 — feat: shortlist top-M recall and fallback behavior vs reference classifier (closes #26)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T03:11:28Z
- **Updated:** 2026-07-22T04:01:02Z
- **Closed:** 2026-07-22T04:00:59Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/52
- **Merged:** 2026-07-22T04:00:59Z
- **Merge commit:** c3d50a01ed94de343d760cb1d8fb3d03db45fdf5
- **Base / Head:** main / UOR-Foundation:feature/issue-26-shortlist-topm-recall-evaluator
- **Reviews / inline review comments:** 1 / 5
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #26: Top-1/top-M candidate shortlist recall evaluation (M=1,3,5,10,20), false-negative rate, conditional direct fidelity, worst-case routing error computation (Gate H), Decision D5 trigger-gated fallback rules, and CBOR ShortlistRecallReport.

## Timeline (11 entries; chronological: conversation comments, reviews, review comments)

### comment 5041392002 — 2026-07-22T03:11:34Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3627219803 — 2026-07-22T03:24:37Z — Copilot

_on crates/uor-r4-core/src/transformerless/shortlist_evaluator.rs:98_

The top-K recall loop scans each shortlist up to 6 times (`contains`, `first`, and multiple `take(k)` passes). This adds unnecessary work on large evaluation sets and risks subtle inconsistency if the list contains duplicates. You can compute the first match position once and derive all top-K counters from it.

### review-comment 3627219818 — 2026-07-22T03:24:37Z — Copilot

_on crates/uor-r4-core/src/transformerless/shortlist_evaluator.rs:112_

`worst_routing_error` is computed over `min(scores_graph.len(), scores_teacher.len())`, which can silently under-estimate the max error when the vectors differ in length. Also, `(a - b).abs()` can overflow on extreme values. Consider treating length mismatch as an evaluation failure and computing diffs in a wider type.

### review 4750724343 — 2026-07-22T03:24:38Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds Phase 5/6 “transformerless” evaluation primitives for (a) explicit resolution status + fallback policy (Decision D4 / Theorem 12) and (b) shortlist top‑M recall reporting with Gate H / Decision D5 trigger-gated fallback signaling, including CBOR report roundtrips to support certification artifacts for Issue #26.

**Changes:**
- Introduces `ResolutionStatus`, `CalibratedFeatures::classify()`, and `FallbackPolicy`/`FallbackAction` for D4 fallback semantics.
- Implements `ShortlistEvaluator` + `ShortlistRecallReport`/`ShortlistMetrics` with CBOR serialization for Gate H-style shortlist recall reporting.
- Adds integration tests covering classification/policy behavior and shortlist report metrics + CBOR roundtrip.

### Reviewed changes

Copilot reviewed 5 out of 5 changed files in this pull request and generated 5 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/shortlist_evaluator.rs | New evaluator/report types for shortlist recall + Gate H/D5 signals, with CBOR encode/decode. |
| crates/uor-r4-core/src/transformerless/resolution_status.rs | New resolution-status classification + fallback policy types for Decision D4. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new transformerless modules. |
| crates/uor-r4-core/tests/shortlist_evaluator_test.rs | Integration tests for shortlist metrics, D5 trigger behavior, and CBOR roundtrip. |
| crates/uor-r4-core/tests/resolution_status_test.rs | Integration tests for Theorem 12-style classification and D4 fallback policy mapping. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3627219828 — 2026-07-22T03:24:38Z — Copilot

_on crates/uor-r4-core/src/transformerless/shortlist_evaluator.rs:131_

`fallback_rate` and `conditional_direct_fidelity` are currently hard-coded (`0.25/0.05`) and set to `top1_recall`, which doesn’t reflect the actual observed evaluation outcomes described for Gate H certification (top-M recall, fallback rate, conditional fidelity without fallback). Also `worst_err <= 1000` is a magic threshold; using a named constant and deriving the other metrics from counts will make the report meaningful and auditable.

### review-comment 3627219846 — 2026-07-22T03:24:38Z — Copilot

_on crates/uor-r4-core/src/transformerless/resolution_status.rs:29_

`self.calibrated_radius * 2` can overflow `u32` and wrap, which would misclassify very large radii. Using `saturating_mul(2)` avoids incorrect wraparound in boundary cases.

### review-comment 3627219862 — 2026-07-22T03:24:38Z — Copilot

_on crates/uor-r4-core/src/transformerless/resolution_status.rs:35_

`self.score_margin.abs()` can overflow on `i32::MIN` (panic in debug / wrap in release depending on build), which would make `classify()` non-deterministic across build profiles. `unsigned_abs()` avoids this overflow while preserving the intended “magnitude < 10” check.

### comment 5041498737 — 2026-07-22T03:26:57Z — Casey-allard

@copilot+claude-fable-5 resolve the merge conflicts in this pull request

### comment 5041533959 — 2026-07-22T03:32:35Z — Copilot


> @copilot+claude-fable-5 resolve the merge conflicts in this pull request

Merge conflicts resolved in f2cd415. The conflict was in `resolution_status.rs::classify()` — kept main's check ordering (BackedOff before Novel) with this branch's overflow-safe `unsigned_abs()` margin check. All workspace tests pass.

### comment 5041556766 — 2026-07-22T03:36:39Z — Casey-allard

@copilot+gpt-5.6-sol resolve the merge conflicts in this pull request

### comment 5041597361 — 2026-07-22T03:43:18Z — Copilot


> @copilot+gpt-5.6-sol resolve the merge conflicts in this pull request

Resolved in d43bf6a by merging latest `main` and preserving the reviewed overflow-safe shortlist routing-error calculation. The PR now reports `mergeable_state: clean`; targeted and full workspace tests pass, and CodeQL found 0 alerts.


---
_Comment/review ids in this file: 5041392002, 5041498737, 5041533959, 5041556766, 5041597361, 4750724343, 3627219803, 3627219818, 3627219828, 3627219846, 3627219862_

