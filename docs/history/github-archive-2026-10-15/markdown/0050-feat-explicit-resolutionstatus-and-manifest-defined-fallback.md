# #50 — feat: explicit ResolutionStatus and manifest-defined fallback policy (closes #25)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T02:54:53Z
- **Updated:** 2026-07-22T03:18:55Z
- **Closed:** 2026-07-22T03:18:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/50
- **Merged:** 2026-07-22T03:18:52Z
- **Merge commit:** 70ba83f386793b6765b062458cf5be4180fe943f
- **Base / Head:** main / UOR-Foundation:feature/issue-25-resolution-status-fallback-policy
- **Reviews / inline review comments:** 3 / 6
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #25: ResolutionStatus classification (Supported, Boundary, BackedOff, Novel, Contradictory) from calibrated integer features (Theorem 12) and manifest-defined fallback policy engine (Decision D4).

## Timeline (10 entries; chronological: conversation comments, reviews, review comments)

### comment 5041290721 — 2026-07-22T02:54:57Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3627114418 — 2026-07-22T02:59:26Z — Copilot

_on crates/uor-r4-core/src/transformerless/resolution_status.rs:37_

`classify()` currently has a couple of overflow/panic hazards and an ordering pitfall:
- `self.calibrated_radius * 2` can overflow `u32` (panic in debug, wrap in release), changing Novel classification.
- `self.score_margin.abs()` can overflow/panic for `i32::MIN`.
- `is_backed_off` is checked after the Novel test, so a feature set with `is_backed_off=true` and large `hamming_dist` will never classify as `BackedOff`.
Consider using saturating/checked arithmetic and a non-overflowing abs, and moving the `is_backed_off` check ahead of Novel/Boundary so the boolean is respected.

### review 4750603830 — 2026-07-22T02:59:27Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds an explicit `ResolutionStatus` classification type and a (serde-serializable) fallback policy model to the transformerless subsystem, with initial tests and module wiring. This aligns with the Phase 5 plan to make resolution outcomes explicit and drive per-status runtime behavior from a manifest-defined policy.

**Changes:**
- Introduces `ResolutionStatus`, `CalibratedFeatures::classify()`, and `FallbackPolicy`/`FallbackAction` types under `transformerless::resolution_status`.
- Adds unit tests covering basic status classification and default/custom fallback-policy behavior.
- Exposes the new module via `crates/uor-r4-core/src/transformerless/mod.rs`.

### Reviewed changes

Copilot reviewed 3 out of 3 changed files in this pull request and generated 4 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/resolution_status.rs | New ResolutionStatus + fallback policy types and classification logic. |
| crates/uor-r4-core/tests/resolution_status_test.rs | New unit tests for classification and fallback policy defaults/customization. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exports the new `resolution_status` module. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3627114444 — 2026-07-22T02:59:27Z — Copilot

_on crates/uor-r4-core/src/transformerless/resolution_status.rs:66_

`FallbackPolicy::default()` claims to implement Decision D4, but the repository docs + Issue #25 describe the D4 default as “consult EXCT exact-residual evidence, then abstain” (no mention of `BasePrior`, and it’s described primarily for Novel/Contradictory cases). The current default maps `Supported`/`Boundary`→ConsultExact and `BackedOff`→BasePrior, which doesn’t match the documented default behavior; please align the default mapping with D4 (or update the docs/issue text if the intended semantics differ).

### review-comment 3627114465 — 2026-07-22T02:59:27Z — Copilot

_on crates/uor-r4-core/src/transformerless/resolution_status.rs:50_

The R4G1 spec describes fallback policy behavior codes as “continue / widen / consult EXCT / certified fallback / abstain” (Decision D4), but `FallbackAction` currently only models `ConsultExact`, `Abstain`, `BasePrior`, and `FallbackToken`. If this type is meant to represent the manifest policy engine, it likely needs to cover the spec’s behavior codes (and/or use the same naming), otherwise manifest policy can’t be represented faithfully.

### review-comment 3627114478 — 2026-07-22T02:59:27Z — Copilot

_on crates/uor-r4-core/tests/resolution_status_test.rs:26_

`CalibratedFeatures::classify()` has an additional rule (`score_margin.abs() < 10`) that isn’t exercised by these tests, and it also currently risks panicking on `i32::MIN` (because `abs()` overflows). Adding test cases for “Boundary via low score margin” and a non-panicking extreme negative margin would lock in the intended behavior and catch regressions.

### review 4750662724 — 2026-07-22T03:14:17Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3627165500 — 2026-07-22T03:14:17Z — Copilot

_on crates/uor-r4-core/src/transformerless/resolution_status.rs:66 · in reply to review-comment 3627114444_

Fixed in a76855e.

### review 4750666343 — 2026-07-22T03:15:11Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3627168776 — 2026-07-22T03:15:11Z — Copilot

_on crates/uor-r4-core/src/transformerless/resolution_status.rs:50 · in reply to review-comment 3627114465_

Implemented in 0278f06: `FallbackAction` now includes Continue, Widen, and CertifiedFallback, with serialization coverage for all D4 behavior codes.


---
_Comment/review ids in this file: 5041290721, 4750603830, 4750662724, 4750666343, 3627114418, 3627114444, 3627114465, 3627114478, 3627165500, 3627168776_

