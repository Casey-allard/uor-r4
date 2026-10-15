# #48 — feat: semantic anti-degeneracy corpus transformations and evaluation harness (closes #23)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T02:15:44Z
- **Updated:** 2026-07-22T02:33:52Z
- **Closed:** 2026-07-22T02:33:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/48
- **Merged:** 2026-07-22T02:33:20Z
- **Merge commit:** ea4fc3408cc7f55aec83d27d7bd306cbaa3833c6
- **Base / Head:** main / UOR-Foundation:feature/issue-23-anti-degeneracy-transformations
- **Reviews / inline review comments:** 3 / 7
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #23: Perturbation suites (masking, span substitution, truncation, reordering, counterfactuals), paraphrase/polysemy evaluation, MDL objective J(C) calculation, and SemanticCoherenceCertificate harness.

## Timeline (11 entries; chronological: conversation comments, reviews, review comments)

### comment 5041059775 — 2026-07-22T02:15:48Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3626958162 — 2026-07-22T02:20:44Z — Copilot

_on crates/uor-r4-core/src/transformerless/anti_degeneracy.rs:33_

`mask_rate` is used directly without any validation/clamping. Unlike `keep_fraction` (which is clamped), a negative `mask_rate` will never mask and a `mask_rate > 1.0` will always mask, which is surprising for a public API. Clamping to `[0.0, 1.0]` keeps behavior consistent and avoids accidental misuse.

### review-comment 3626958178 — 2026-07-22T02:20:44Z — Copilot

_on crates/uor-r4-core/tests/anti_degeneracy_test.rs:20_

The test suite exercises Masking/Truncation/Reorder/Counterfactual, but does not cover the new `SpanSubstitution` perturbation. Adding a simple test case here would prevent regressions (e.g. rate handling, determinism, wrapping behavior).

### review-comment 3626958193 — 2026-07-22T02:20:44Z — Copilot

_on crates/uor-r4-core/src/transformerless/anti_degeneracy.rs:26_

`Masking` uses `mask_token = 0`, but token id 0 appears to be a valid vocabulary token (the tokenizer assigns ids starting at 0). That means masking can be indistinguishable from a genuine token and silently reduce the effectiveness of this perturbation. Consider using a reserved/sentinel id (or making the mask token configurable in `PerturbationKind::Masking`).

### review-comment 3626958209 — 2026-07-22T02:20:44Z — Copilot

_on crates/uor-r4-core/src/transformerless/anti_degeneracy.rs:45_

In `SpanSubstitution`, `(i * 17) as u64` multiplies in `usize` before casting. For extremely large inputs this can overflow and panic in debug builds (and wrap in release), changing determinism and potentially crashing. Also, `substitute_rate` is not clamped like `keep_fraction`, so values outside `[0,1]` can produce surprising behavior. Compute the offset in `u64` and clamp the rate.

### review 4750426272 — 2026-07-22T02:20:45Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds a new “semantic anti-degeneracy” component under `transformerless`, including deterministic token perturbation utilities, simple paraphrase/polysemy evaluators, an MDL objective helper, and a `SemanticCoherenceCertificate` verification harness to support Issue #23’s evaluation workflow.

**Changes:**
- Introduces `transformerless::anti_degeneracy` with perturbation kinds (masking/span substitution/truncation/reorder/counterfactual), evaluators, and certificate verification.
- Exposes the new module in `transformerless/mod.rs` (non-wasm builds).
- Adds unit tests covering most perturbations plus MDL objective and certificate verification.

### Reviewed changes

Copilot reviewed 3 out of 3 changed files in this pull request and generated 5 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/anti_degeneracy.rs | New anti-degeneracy perturbation/evaluation utilities and certificate type. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new `anti_degeneracy` module for non-wasm targets. |
| crates/uor-r4-core/tests/anti_degeneracy_test.rs | Adds unit tests for the new anti-degeneracy module behaviors. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626958215 — 2026-07-22T02:20:45Z — Copilot

_on crates/uor-r4-core/src/transformerless/anti_degeneracy.rs:139_

`SemanticCoherenceCertificate` includes `mdl_cost_j_c`, but `verify()` never checks it. This makes it easy for callers to assume MDL cost is part of certificate validity when it currently isn’t. Either include an explicit MDL threshold/constraint in `verify()`, or document (and/or rename) the field to make it clear it’s informational only.

### review 4750455256 — 2026-07-22T02:28:26Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3626984782 — 2026-07-22T02:28:26Z — Copilot

_on crates/uor-r4-core/src/transformerless/anti_degeneracy.rs:26 · in reply to review-comment 3626958193_

Made the mask token a configurable field on `PerturbationKind::Masking { mask_rate, mask_token }` so callers can supply a reserved/sentinel id instead of the hardcoded 0. Test updated to use `u32::MAX`. Fixed in 706e634.

### review 4750470179 — 2026-07-22T02:31:58Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3626998232 — 2026-07-22T02:31:58Z — Copilot

_on crates/uor-r4-core/src/transformerless/anti_degeneracy.rs:139 · in reply to review-comment 3626958215_

Added a doc comment to `mdl_cost_j_c` making it explicit that the field is informational only and is not evaluated by `verify()`. Fixed in the latest commit.


---
_Comment/review ids in this file: 5041059775, 4750426272, 4750455256, 4750470179, 3626958162, 3626958178, 3626958193, 3626958209, 3626958215, 3626984782, 3626998232_

