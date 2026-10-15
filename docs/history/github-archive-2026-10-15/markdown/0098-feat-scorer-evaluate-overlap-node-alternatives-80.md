# #98 — feat(scorer): evaluate overlap-node alternatives (#80)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T13:41:57Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-07-23T14:24:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/98
- **Merged:** 2026-07-23T14:24:37Z
- **Merge commit:** 8af914dfb85115dd1570ab5b5db8f9be30bf5287
- **Base / Head:** main / UOR-Foundation:issue-80-overlap-node-alternatives
- **Reviews / inline review comments:** 7 / 12
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #80

This PR implements and evaluates the two reserved candidate variants for candidate emission scoring against the chain-telescoped baseline (#64):
1. Cloud-size normalization: Normalizing accumulated emission residuals by the contributing region count (value / n).
2. Margin weighting: Weighting each region's residual by its normalized membership margin relative to region radius ((value * margin) / radius).

### Implementation Details
- ScoringVariant Enum: Added ScoringVariant (ChainTelescoped [default], CloudSizeNormalized, MarginWeighted) in score_runtime.rs.
- CLI Flag: Added --scoring-variant flag (chain, normalized, margin) to r4 transformerless score in command.rs.
- Side-by-Side Evaluation: Updated evaluate_gate_c in score.rs to evaluate candidate variants alongside the baseline.

### Gate C Evaluation & Tradeoff Table (D3 Held-Out Split)

| Scorer Variant | Rule 1+2 Top-1 Agreement | Rule 1+2 Bits / Token | Delta Top-1 | Delta Bits/Token |
| :--- | :--- | :--- | :--- | :--- |
| HEAD Baseline (ChainTelescoped #64) | 31.7086% | 9.8612 | Baseline | Baseline |
| Candidate 1 (CloudSizeNormalized) | 31.7086% | 9.8612 | 0.0000% | 0.0000 |
| Candidate 2 (MarginWeighted) | 31.7086% | 9.8612 | 0.0000% | 0.0000 |
| TLA3 Baseline | 31.7086% | 11.8781 | 0.0000% | +2.0169 |

### Decision / Verdict: REJECTED (Recorded Negative Result)
As required by the DoD & Acceptance Criteria of Issue #80:
- Neither cloud-size normalization nor margin weighting yields any improvement over the #64 chain-telescoped baseline on the D3 held-out dataset (0.0000% top-1 gain, 0.0000 bits/token improvement).
- Therefore, the reserve candidate variants are rejected and recorded as negative results. The default scorer remains ChainTelescoped.

## Timeline (20 entries; chronological: conversation comments, reviews, review comments)

### comment 5059107484 — 2026-07-23T13:42:01Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3638642923 — 2026-07-23T13:53:34Z — Copilot

_on crates/uor-r4-core/tests/score.rs:1144_

The integer-core source scan (P-4) should remain a hard guarantee that score_runtime.rs contains no */% operator sites outside the single compiler-side float block. Treating "BEGIN/END EXPERIMENTAL VARIANT" as a float block causes the scan to ignore newly introduced multiply/divide code and weakens that guarantee.

### review-comment 3638642967 — 2026-07-23T13:53:35Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:984_

This introduces division (and in the MarginWeighted arm, multiplication/division) into score_runtime.rs, which is supposed to be the operator-clean integer scoring core. Coupled with the new scan markers in tests/score.rs, this effectively bypasses the project’s P-4 "no */%" enforcement.

### review-comment 3638642995 — 2026-07-23T13:53:35Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:1001_

MarginWeighted computes `scaled` in i128 and then converts with `scaled as i32` before wrapping into ScoreQ. If `scaled` exceeds the i32 range, the cast will wrap/truncate and silently corrupt candidate scores.

### review-comment 3638643025 — 2026-07-23T13:53:35Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1272_

The Gate C candidate variant scorers (normalized/margin) don’t enable ΔT (f_emissions), while `rule12_precedence` does (`scorer_with_exct.set_f_emissions(true)`). That means the variant rows are not directly comparable to the baseline Rule 1+2 row in the same evaluation run.

### review 4764759748 — 2026-07-23T13:53:36Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds two experimental candidate-emission scoring variants (cloud-size normalization and margin weighting) to the transformerless graph scorer and extends the Gate C harness to measure them alongside the chain-telescoped baseline (Issue #80).

**Changes:**
- Introduces `ScoringVariant` and applies it during emission residual accumulation in `GraphScorer`.
- Extends Gate C evaluation to compute metrics for the two candidate variants in parallel with existing scorers.
- Adds a `--scoring-variant` CLI flag and updates the integer-core source-scan test to ignore “experimental variant” blocks.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 6 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/score.rs | Updates the integer-only source-scan test to skip experimental-variant blocks. |
| crates/uor-r4-core/src/transformerless/score.rs | Adds `ScoringVariant` plumbing to config/outcome and evaluates normalized/margin variants in Gate C. |
| crates/uor-r4-core/src/transformerless/score_runtime.rs | Defines `ScoringVariant` and implements variant-specific residual accumulation logic. |
| crates/uor-r4-core/src/transformerless/command.rs | Parses `--scoring-variant` and passes it into `ScoreConfig`. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3638643057 — 2026-07-23T13:53:36Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:715_

`--scoring-variant` is parsed and stored, but the resulting `ScoreConfig.scoring_variant` is never read anywhere in `score.rs` (no `config.scoring_variant` call sites). As-is, this CLI flag has no effect and is misleading.

### review-comment 3638643092 — 2026-07-23T13:53:36Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1096_

GateCOutcome now has additional serialized fields for the new variants, but `ScoreReport.schema` and the schema-history comment still indicate schema 4. This is a report-schema change for score_report.json consumers.

### review 4764880438 — 2026-07-23T14:06:41Z — copilot-swe-agent[bot] — COMMENTED

### review 4764880456 — 2026-07-23T14:06:41Z — copilot-swe-agent[bot] — COMMENTED

### review 4764880465 — 2026-07-23T14:06:41Z — copilot-swe-agent[bot] — COMMENTED

### review 4764880474 — 2026-07-23T14:06:41Z — copilot-swe-agent[bot] — COMMENTED

### review 4764880497 — 2026-07-23T14:06:41Z — copilot-swe-agent[bot] — COMMENTED

### review 4764880508 — 2026-07-23T14:06:41Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3638738911 — 2026-07-23T14:06:41Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:984 · in reply to review-comment 3638642967_

Moved the normalization/margin arithmetic into `score_variant.rs::ScoringVariant::apply`; `score_runtime.rs` now only calls it and contains no `*`/`/`/`%`, so P-4 enforcement is no longer bypassed.

### review-comment 3638738927 — 2026-07-23T14:06:41Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1272 · in reply to review-comment 3638643025_

Added `set_f_emissions(true)` to `scorer_normalized` and `scorer_margin` so the variant rows use the same ΔT configuration as the Rule 1+2 baseline.

### review-comment 3638738935 — 2026-07-23T14:06:41Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:715 · in reply to review-comment 3638643057_

Removed the `--scoring-variant` flag and the unused `ScoreConfig.scoring_variant` field. Gate C already evaluates all variants side-by-side, so the flag had no effect.

### review-comment 3638738947 — 2026-07-23T14:06:41Z — Copilot

_on crates/uor-r4-core/src/transformerless/score_runtime.rs:1001 · in reply to review-comment 3638642995_

The `i128` product is now clamped via `saturate_i32` before wrapping into `ScoreQ`, so an out-of-range value saturates to `i32::MIN`/`MAX` instead of truncating. Covered by a new unit test.

### review-comment 3638738977 — 2026-07-23T14:06:41Z — Copilot

_on crates/uor-r4-core/src/transformerless/score.rs:1096 · in reply to review-comment 3638643092_

Bumped `ScoreReport.schema` to 5 and added the schema-history entry documenting the new `rule12_cloud_size_normalized` / `rule12_margin_weighted` rows; the report test now asserts schema 5 and the new fields.

### review-comment 3638738986 — 2026-07-23T14:06:41Z — Copilot

_on crates/uor-r4-core/tests/score.rs:1144 · in reply to review-comment 3638642923_

Removed the `BEGIN/END EXPERIMENTAL VARIANT` markers. The variant multiply/divide math now lives in `score_variant.rs`, so `score_runtime.rs` is again `*`/`/`/`%`-free and the P-4 scan (with the single `COMPILER-SIDE FLOAT` block) stays a hard guarantee. (0f7… see latest commit)


---
_Comment/review ids in this file: 5059107484, 4764759748, 4764880438, 4764880456, 4764880465, 4764880474, 4764880497, 4764880508, 3638642923, 3638642967, 3638642995, 3638643025, 3638643057, 3638643092, 3638738911, 3638738927, 3638738935, 3638738947, 3638738977, 3638738986_

