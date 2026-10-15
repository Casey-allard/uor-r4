# #92 — feat(r4g1): implement bounded integer-only repetition control (#79)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T05:34:38Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-07-23T07:19:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/92
- **Merged:** 2026-07-23T07:19:33Z
- **Merge commit:** d74f6cf69923ac654fa7ee00dc62b48207e77ffa
- **Base / Head:** main / UOR-Foundation:fix-issue-79-repetition-control
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #79

This PR introduces bounded, allocation-free repetition control to `GraphScorer::score_candidates`.

### Implementation Details:
1. **Recency Window**: `score_candidates` and `score_candidates_legacy` now accept a bounded `recent_tokens` slice (max 32).
2. **Penalty Application**: Saturated `-2,000,000` logic is dynamically applied via `ScoreQ::from_raw` during cloud prediction for any token appearing in the recent context.
3. **Telemetry & Gate C**: 
   - A deterministic greedy simulation on the first 5 viable test seeds is performed inside `evaluate_gate_c`.
   - `repetition_rate_rule12` and `repetition_rate_baseline` are now properly serialized inside the `score_report.json` for downstream monitoring.

### Validation
Passes all unit and integration tests (zero regressions on Gate C accuracy). Verified that the runtime penalty applies seamlessly without triggering memory allocations.

## Timeline (10 entries; chronological: conversation comments, reviews, review comments)

### comment 5054849230 — 2026-07-23T05:34:42Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5055051106 — 2026-07-23T06:08:19Z — Casey-allard

@copilot+claude-opus-4.8 resolve the merge conflicts in this pull request

### comment 5055164583 — 2026-07-23T06:25:01Z — Copilot


> @copilot+claude-opus-4.8 resolve the merge conflicts in this pull request

Resolved the merge conflicts in `77389f8` (true two-parent merge of `origin/main`).

Conflicts were in `src/r4g1.rs` and `src/server.rs`, caused by main's PR #93 landing a canonical ResolutionStatus/D4 policy architecture (issue #78) that superseded this branch's parallel implementation. I resolved them in favor of main's architecture (`predict_window_status` / `generate_into_status` / `PredictDecision` / `PolicyStatus` / `R4g1Text`) while preserving this PR's unique #79 contribution — the bounded integer-only repetition control in `GraphScorer::score_candidates`. The new `score_candidates(sig, recent_tokens)` signature was threaded through main's callers and its new `status_policy*` test fixtures.

All gates pass on the merge preview: build, `clippy -D warnings`, `fmt --check`, full workspace tests (incl. `status_policy` and `score`), the no_std ladder, and κ reproduction.

### review-comment 3636101862 — 2026-07-23T06:49:10Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1138_

`generate_greedy_repetition_rate` uses `.unwrap()` on `score_candidates`, which will panic the Gate C evaluation/CI trend job on any artifact/scoring error and lose the actionable error context. Prefer returning a `Result<f64, String>` from this helper and propagating it with `?` from `evaluate_gate_c`.

### review 4761466802 — 2026-07-23T06:49:11Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR extends the transformerless graph scoring path to support bounded repetition suppression via a recency window, and adds Gate C telemetry/CI tracking to monitor regression and repetition-rate trends over time.

**Changes:**
- Add a bounded `recent_tokens` parameter to `GraphScorer::score_candidates` and apply a saturated suppression penalty to recently seen tokens.
- Add deterministic greedy repetition-rate measurement to Gate C evaluation and serialize repetition telemetry into `score_report.json`.
- Wire a Gate C trend job into CI (producing/uploading `trend_output/score_report.json`) and update tests/callers for the new `score_candidates` signature.

### Reviewed changes

Copilot reviewed 9 out of 10 changed files in this pull request and generated 4 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| tests/status_policy.rs | Update scorer calls to pass an empty recency window. |
| tests/status_policy_common/mod.rs | Update helper scoring calls to pass an empty recency window. |
| src/r4g1.rs | Keep status-policy scoring deterministic by passing empty `recent_tokens`. |
| scripts/check_gate_c_regression.py | Add a CI-friendly regression check for Gate C trend output. |
| crates/uor-r4-core/tests/score.rs | Update tests for new `score_candidates(sig, recent_tokens)` signature. |
| crates/uor-r4-core/src/transformerless/score.rs | Add repetition-rate simulation + serialize repetition telemetry into score report. |
| crates/uor-r4-core/src/transformerless/score_runtime.rs | Implement recency-window penalty in `GraphScorer::score_candidates` and parse fallback policy. |
| crates/uor-r4-core/src/transformerless/resolution_status.rs | Add artifact-bytes decoding helpers for fallback policy/actions. |
| .gitignore | Ignore `trend_output/` artifacts generated by CI trend tracking. |
| .github/workflows/ci.yml | Add Gate C trend generation + regression checking + artifact upload. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3636101880 — 2026-07-23T06:49:11Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:848_

`recent_tokens` is documented/used as a bounded recency window (max 32), but `score_candidates` accepts any slice length and then does `contains()` checks per candidate. Clamping (or rejecting) longer inputs here makes the API safer and keeps the cost deterministically bounded.

### review-comment 3636101891 — 2026-07-23T06:49:11Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:636_

`GraphScorer::fallback_policy` is introduced as a `pub` field but does not appear to be used outside `score_runtime.rs`. Exposing it publicly unnecessarily expands the API surface and locks in representation details; prefer `pub(crate)` (or a getter) until an external consumer is demonstrated.

### review-comment 3636101914 — 2026-07-23T06:49:11Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1549_

`build_score_report` now serializes repetition-rate telemetry into `ScoreReportGraph` (`graph_repetition_rate`/`baseline_repetition_rate`), which changes the JSON shape. The report `schema` value is still 3, so downstream consumers that key off `schema` may misinterpret the payload; this should bump the schema version (and ideally update the schema-history comment).

### comment 5055433614 — 2026-07-23T07:00:33Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5055491221 — 2026-07-23T07:07:49Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved. I merged `origin/main`, fixed all conflicts in this PR branch, and pushed follow-up fixes in `8bc7ebe` and `16c25fa`.


---
_Comment/review ids in this file: 5054849230, 5055051106, 5055164583, 5055433614, 5055491221, 4761466802, 3636101862, 3636101880, 3636101891, 3636101914_

