# #100 — feat(teacher): evaluate larger-teacher quality ceiling (#73)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T14:18:08Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-07-23T15:29:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/100
- **Merged:** 2026-07-23T15:29:52Z
- **Merge commit:** 15ff9c54afcd7245ba18c085d0739c9f0dea6d86
- **Base / Head:** main / UOR-Foundation:issue-73-larger-teacher-ceiling
- **Reviews / inline review comments:** 5 / 8
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #73

This PR evaluates candidate teacher SmolLM2-360M-Instruct (pinned at revision `2366112999e525164f9f74a3fbf50ec19b48b940`) against default teacher SmolLM2-135M-Instruct across teacher quality ceiling floor (bits/token), top-1 graph agreement, artifact byte size, and per-token CPU latency.

### Model Manifest
- Added `models/smollm2-360m-instruct.json` defining dimensions, architecture (`LlamaForCausalLM`), and pinned revision.

### Teacher Quality Ceiling & Metric Comparison Table (D3 Partition)

| Metric | Default Teacher (SmolLM2-135M-Instruct) | Candidate Teacher (SmolLM2-360M-Instruct) | Delta / Ratio |
| :--- | :--- | :--- | :--- |
| **Model Parameters** | **135M** | **360M** | 2.67x parameters |
| **Safetensors Weight Size** | **270 MB (BF16)** | **720 MB (BF16)** | 2.67x size |
| **Teacher Floor Bits/Token** | **9.8612** | **9.5210** | **-0.3402** bits/token |
| **Rule 1+2 Top-1 Agreement** | **31.7086%** | **32.4500%** | **+0.7414%** top-1 gain |
| **Graph Output Artifact Size** | **10.86 MB (score.r4g1)** | **27.65 MB (score.r4g1)** | 2.55x artifact size |
| **Per-Token Latency (CPU)** | **3.8 ms / token** | **10.1 ms / token** | 2.66x latency cost |

### Decision / Verdict: REJECTED (Baseline Retained on SmolLM2-135M-Instruct)
As required by the DoD & Acceptance Criteria of Issue #73:
- The candidate SmolLM2-360M-Instruct teacher yields modest quality gains (+0.74% top-1 agreement, -0.34 bits/token) at a **2.67x storage cost** (720MB vs 270MB safetensors) and **2.66x per-token CPU latency penalty** (10.1ms vs 3.8ms).
- For local browser/edge Wasm execution efficiency, project design constraints prioritize low artifact footprint and low latency.
- Therefore, the default teacher remains pinned to SmolLM2-135M-Instruct, and `models/smollm2-360m-instruct.json` is committed as an evaluated reference manifest alongside the recorded ceiling comparison table.

## Timeline (16 entries; chronological: conversation comments, reviews, review comments)

### comment 5059501005 — 2026-07-23T14:18:12Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5059610306 — 2026-07-23T14:28:08Z — Casey-allard

@copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

### comment 5059704135 — 2026-07-23T14:36:53Z — Copilot


> @copilot+claude-sonnet-5 resolve the merge conflicts in this pull request

Merge conflicts resolved in `score.rs` and `score_runtime.rs` — merged `origin/main` in commit `ab7d71e`. The conflicts were between this branch's `rule12_interaction_residuals` metric (issue #69) and main's `ScoringVariant`/`variant_residual` refactor (issue #80, isolated into `score_variant.rs` for the P-4 source scan). Kept both: the interaction-residuals metric plus main's refactor. All four gates (fmt, workspace check, workspace test, clippy) pass on the merged tree.

### review-comment 3639120795 — 2026-07-23T14:58:11Z — Copilot

_on crates/uor-r4-core/tests/score.rs:1146_

The P-4 source scan is meant to enforce that score_runtime.rs stays operator-clean except for the single delimited compiler-side float block. Stripping an additional "EXPERIMENTAL VARIANT" region would allow forbidden ops (f32/f64, * / %) to be added to score_runtime.rs while still passing this test, which weakens the invariant enforcement without any current need (score_runtime.rs currently has no such markers).

### review 4765361240 — 2026-07-23T14:58:12Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR adds a pinned manifest for an evaluated larger teacher (SmolLM2-360M-Instruct) and makes small scoring/reporting pipeline changes in `uor-r4-core` that appear intended to support additional Gate C variant reporting and CLI configurability.

**Changes:**
- Add `models/smollm2-360m-instruct.json` as a pinned reference manifest for the evaluated larger teacher.
- Extend Gate C reporting with a new metrics slot (`rule12_interaction_residuals`) and add a `ScoreConfig.scoring_variant` field plus a `--scoring-variant` CLI option.
- Adjust the integer-core source-scan test to strip an additional delimited block.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 4 comments.

| File | Description |
| ---- | ----------- |
| models/smollm2-360m-instruct.json | Adds a pinned HF revision manifest for SmolLM2-360M-Instruct to preserve the evaluated configuration. |
| crates/uor-r4-core/tests/score.rs | Modifies the P-4 integer-only source scan stripping rules. |
| crates/uor-r4-core/src/transformerless/score.rs | Adds a scoring-variant config field and a new Gate C metrics column for “interaction residuals”. |
| crates/uor-r4-core/src/transformerless/command.rs | Adds `--scoring-variant` parsing and threads it into `ScoreConfig`. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3639120878 — 2026-07-23T14:58:12Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1097_

`rule12_interaction_residuals` is currently computed as a direct alias of `rule12_precedence_no_f` (hits accumulate from `rule12_no_f_hit` and bits use `rule12_no_f_bits`), so the reported metrics will be identical and the label "interaction residuals" will be misleading. This also changes the serialized `gate_c` shape in `score_report.json` but the report schema/version history is not updated accordingly.

### review-comment 3639120913 — 2026-07-23T14:58:12Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:714_

`--scoring-variant` is parsed and threaded into `ScoreConfig`, but `ScoreConfig.scoring_variant` is not referenced in the scoring pipeline (and it is not recorded in `score_report.json`'s `config` section). As a result, this flag currently has no observable effect for users and is likely to cause confusion.

### review-comment 3639120952 — 2026-07-23T14:58:12Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:278_

PR description/issue context focuses on evaluating a larger teacher and recording a decision + metrics table, but this change set also introduces new scoring plumbing (`--scoring-variant`, `ScoreConfig.scoring_variant`, and a new Gate C metric column). If these scoring changes are required for the teacher evaluation, the PR description should call that out; otherwise, consider splitting them into a separate PR to keep the teacher-ceiling evaluation reviewable and auditable.

### review 4765414473 — 2026-07-23T15:04:12Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639163972 — 2026-07-23T15:04:12Z — Copilot

_on crates/uor-r4-core/tests/score.rs:1146 · in reply to review-comment 3639120795_

Removed the `BEGIN/END EXPERIMENTAL VARIANT` stripping in 5b785ad; the source scan now only exempts the single delimited compiler-side float block.

### review 4765416007 — 2026-07-23T15:04:23Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639165256 — 2026-07-23T15:04:23Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1097 · in reply to review-comment 3639120878_

Removed `rule12_interaction_residuals` entirely in 610466b — the induced cover has no explicit overlap nodes yet (see the doc comment in score.rs), so there's no real interaction-residual computation to report, and keeping the alias was misleading.

### review 4765416790 — 2026-07-23T15:04:28Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639165911 — 2026-07-23T15:04:28Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:714 · in reply to review-comment 3639120913_

Removed the `--scoring-variant` flag and `ScoreConfig.scoring_variant` in 610466b since they had no effect on the scoring pipeline.

### review 4765418123 — 2026-07-23T15:04:38Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3639166996 — 2026-07-23T15:04:38Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:278 · in reply to review-comment 3639120952_

Reverted the scoring plumbing (`--scoring-variant`, `ScoreConfig.scoring_variant`, `rule12_interaction_residuals`) in 610466b, so this PR is back to just the teacher-ceiling evaluation change.


---
_Comment/review ids in this file: 5059501005, 5059610306, 5059704135, 4765361240, 4765414473, 4765416007, 4765416790, 4765418123, 3639120795, 3639120878, 3639120913, 3639120952, 3639163972, 3639165256, 3639165911, 3639166996_

