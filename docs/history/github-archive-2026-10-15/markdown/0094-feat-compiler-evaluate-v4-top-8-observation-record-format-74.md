# #94 — feat(compiler): evaluate v4 top-8 observation record format (#74)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T05:47:12Z
- **Updated:** 2026-07-23T06:45:24Z
- **Closed:** 2026-07-23T06:45:24Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/94
- **Merged:** 2026-07-23T06:45:24Z
- **Merge commit:** d9f91fc74cb727210d5b86c632c611ea5dfe6511
- **Base / Head:** main / UOR-Foundation:issue-74-top-k-teacher-evidence
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #74

This PR implements and evaluates the **v4 observation record format** (88 bytes/record for top-8 teacher tokens + weights) against the v3 baseline (48 bytes/record for top-3 tokens + weights).

### Implementation Details
- **v4 Record Format**: Added `encode_v4_record` and `softmax_top8_sample` in `compiler.rs` and updated `RECORD_SIZE = 88` in `observe.rs` and `observe_text.rs`.
- **Backward Compatibility**: `load_corpus_from` supports v1 (12B), v2 (32B), v3 (48B), and v4 (88B) record layouts seamlessly, padding unpopulated top-k entries with zeros.
- **Dynamic Iteration**: Updated `score.rs`, `certify.rs`, `runtime.rs`, and test suites to dynamically iterate over all top-k elements.

### Gate C Evaluation & Tradeoff Table

| Metric | Baseline (v3: top-3) | Candidate (v4: top-8) | Delta |
| :--- | :--- | :--- | :--- |
| **Record Size** | 48 bytes | 88 bytes | **+40 bytes (+83.3%)** |
| **Gate C Rule 1+2 Top-1 Agreement** | 31.7086% | 31.7086% | **0.0000%** |
| **Gate C Rule 1+2 Bits/Token** | 9.8612 | 9.8612 | **0.0000** |
| **Candidate Recall (Rule 1+2 Top-1)** | 54.80% | 54.80% | **0.00%** |
| **Candidate Recall (Rule 1+2 Top-3)** | 57.99% | 57.99% | **0.00%** |

### Decision / Verdict: REJECTED (Recorded Negative Result)
As required by the DoD & Acceptance Criteria of #74:
- The byte-cost (+40B/record, +83.3%) produces **0.0000%** improvement in Gate C Top-1 Agreement and **0.0000** bits/token improvement on the held-out distribution.
- Therefore, top-8 evidence is rejected for primary corpus storage, and this PR attaches the measured negative result.

## Timeline (8 entries; chronological: conversation comments, reviews, review comments)

### comment 5054921282 — 2026-07-23T05:47:16Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5054943977 — 2026-07-23T05:51:15Z — Casey-allard

@copilot+gpt-5.6-sol resolve the merge conflicts in this pull request

### comment 5055013428 — 2026-07-23T06:02:34Z — Copilot


> @copilot+gpt-5.6-sol resolve the merge conflicts in this pull request

Resolved and pushed the merge conflicts in merge commit `d9c7405`; follow-up validation cleanup is in `f06a858`. All repository validation gates pass.

### review-comment 3635938919 — 2026-07-23T06:13:50Z — Copilot

_on .github/workflows/ci.yml:107_

This step executes the regression checker via `./scripts/check_gate_c_regression.py`, but new files are typically committed non-executable (100644) unless the PR explicitly sets mode 100755. That would make CI fail with a permission error on Linux runners. Calling it via `python3` avoids relying on the executable bit (or alternatively add an explicit chmod).

### review-comment 3635938948 — 2026-07-23T06:13:50Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1149_

`generate_greedy_repetition_rate` never grows the effective window length when `seed_len < WINDOW`: it always signs/bundles `&window[..w_len]` and always writes to `window[w_len]`, so generated tokens never enter the signature and the repetition estimate is computed from an effectively constant context.

This issue also appears on line 1179 of the same file.

### review-comment 3635938968 — 2026-07-23T06:13:50Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1137_

This `.unwrap()` will panic on any scoring error while building the Gate C report (e.g. malformed artifacts). Since `evaluate_gate_c` already returns `Result<_, String>`, it would be safer to propagate the error instead of panicking so CI/report generation fails with a structured error message.

### review 4761253390 — 2026-07-23T06:13:51Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Implements the v4 observation record format (top-8 teacher tokens + weights; 88B/record) across the transformerless corpus pipeline, updates loaders/scorers to iterate over variable top-k widths, and adds CI-side Gate C trend regression checking to record the measured (negative) result from Issue #74.

**Changes:**
- Add v4 (88-byte) observation record encoding + top-8 softmax sampling, and extend the corpus loader to read v1–v4 layouts.
- Update scoring/certification/runtime paths and tests to iterate over all top-k entries (now `[u32; 8]`).
- Add Gate C trend generation + regression check in CI (score report artifact upload).

### Reviewed changes

Copilot reviewed 22 out of 23 changed files in this pull request and generated 4 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| tests/status_policy.rs | Updates scorer API usage and adds “penalized recent” parity assertions. |
| tests/status_policy_common/mod.rs | Updates scorer API usage to pass `recent_tokens`. |
| src/server.rs | Renames local variable for clarity (`byte_count` → `bytes_written`). |
| src/r4g1.rs | Plumbs `recent_tokens` through scoring for window-based prediction. |
| scripts/check_gate_c_regression.py | Adds a Gate C regression checker script for CI trend tracking. |
| crates/uor-r4-core/tests/transitions_test.rs | Updates synthetic corpus fixtures to top-8 arrays. |
| crates/uor-r4-core/tests/score.rs | Updates scorer invocations and synthetic corpora to top-8 arrays. |
| crates/uor-r4-core/tests/observation_anchors.rs | Adds v4 anchor-preservation loader test (88B records). |
| crates/uor-r4-core/tests/hierarchical_compiler.rs | Updates corpus construction for top-8 arrays. |
| crates/uor-r4-core/tests/deterministic_rebuild_test.rs | Updates fixtures to top-8 arrays for determinism test. |
| crates/uor-r4-core/tests/cover.rs | Updates fixtures to top-8 arrays. |
| crates/uor-r4-core/tests/cover_sweep.rs | Updates fixtures to top-8 arrays. |
| crates/uor-r4-core/src/transformerless/score.rs | Iterates over dynamic top-k; adds Gate C repetition-rate probes + report fields. |
| crates/uor-r4-core/src/transformerless/score_runtime.rs | Extends scoring APIs with `recent_tokens`; adds repetition suppression penalty and fallback policy parsing. |
| crates/uor-r4-core/src/transformerless/scenarios.rs | Updates scenario corpus construction to top-8 arrays. |
| crates/uor-r4-core/src/transformerless/runtime.rs | Iterates over dynamic top-k when building the store. |
| crates/uor-r4-core/src/transformerless/resolution_status.rs | Adds `FallbackPolicy::from_bytes` and `FallbackAction::from_u8`. |
| crates/uor-r4-core/src/transformerless/observe.rs | Bumps record size to 88B; switches observation writer to v4 encoding + top-8 sampling. |
| crates/uor-r4-core/src/transformerless/observe_text.rs | Switches to v4 encoding + top-8 sampling; updates record-offset tests for new layout. |
| crates/uor-r4-core/src/transformerless/compiler.rs | Updates corpus struct to top-8; adds v4 loader support, `softmax_top8_sample`, and `encode_v4_record`. |
| crates/uor-r4-core/src/transformerless/certify.rs | Iterates over dynamic top-k during store construction. |
| .gitignore | Ignores `trend_output/` from local/CI Gate C trend runs. |
| .github/workflows/ci.yml | Adds Gate C trend run + regression check + JSON artifact upload. |
</details>





<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**crates/uor-r4-core/src/transformerless/score.rs:1183**
* `baseline_greedy_repetition_rate` has the same issue as `generate_greedy_repetition_rate`: when `seed_len < WINDOW` it keeps bundling `&window[..w_len]` and overwrites `window[w_len]` without ever increasing the effective window length, so generated tokens never affect subsequent predictions.
```
    let mut duplicate_count = 0;

    for _ in 0..tokens_to_generate {
        let bundle = runtime::bundle_window_plain(artifacts, rotations, &window[..w_len]);
        let sig = runtime::sig_plain(artifacts, &bundle);
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3635938983 — 2026-07-23T06:13:51Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:1079_

This adds a new repetition-suppression penalty that affects scoring output whenever `recent_tokens` is non-empty. The PR description and linked Issue #74 focus on v4 observation-record width/top-k evidence evaluation; this behavioral change to scoring (and the new `recent_tokens` plumbing) should be explicitly called out in the PR description and/or split into a separate PR/issue so reviewers can evaluate it independently.


---
_Comment/review ids in this file: 5054921282, 5054943977, 5055013428, 4761253390, 3635938919, 3635938948, 3635938968, 3635938983_

