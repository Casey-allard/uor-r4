# #49 — feat: predictive-sufficiency and rate-distortion reports by graph depth (closes #24)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-22T02:35:52Z
- **Updated:** 2026-07-22T02:52:43Z
- **Closed:** 2026-07-22T02:52:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/49
- **Merged:** 2026-07-22T02:52:39Z
- **Merge commit:** 994ef49fd297d9bc6f8f5627ac9aecd56df468ee
- **Base / Head:** main / UOR-Foundation:feature/issue-24-predictive-sufficiency-reports
- **Reviews / inline review comments:** 2 / 8
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements Issue #24: Divergence D_KL(T(.|c) || T(.|R_d(c))) across graph depths (BroadCloud, IntermediateCloud, FullCloud, ResidualAugmented), operational rate-distortion curves (bytes+ops vs bits retained), and CBOR RateDistortionReport serialization.

## Timeline (11 entries; chronological: conversation comments, reviews, review comments)

### comment 5041181724 — 2026-07-22T02:35:57Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### review-comment 3627033611 — 2026-07-22T02:40:56Z — Copilot

_on crates/uor-r4-core/src/transformerless/predictive_sufficiency.rs:82_

`compute_kl_divergence` clamps `pi` to `eps`, which makes terms with true `P(i)=0` contribute non-zero mass and biases KL (e.g., KL(P||P) can become >0 for distributions containing zeros). It also uses natural-log but later code/docs describe KL in bits.

### review-comment 3627033619 — 2026-07-22T02:40:56Z — Copilot

_on crates/uor-r4-core/src/transformerless/predictive_sufficiency.rs:98_

`compute_cross_entropy` clamps `pi` to `eps`, so events with true `P(i)=0` still contribute to the sum (biasing cross-entropy). If cross-entropy is intended in bits (as suggested by the rate/distortion doc), use log2 and only clamp `q`.

### review 4750515734 — 2026-07-22T02:40:57Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR introduces a new “predictive sufficiency” evaluation module for the transformerless pipeline, enabling divergence (teacher vs depth-reduced graph) and rate–distortion reporting across graph depths, along with CBOR serialization and basic tests.

**Changes:**
- Added `transformerless::predictive_sufficiency` with `GraphDepth`, divergence metrics, and `RateDistortionReport` CBOR round-tripping.
- Extended `OpKernel` to be serializable/clonable/comparable so it can be embedded in reports.
- Added a new test suite covering divergence computations, depth evaluation, and CBOR round-trip.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 7 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/predictive_sufficiency.rs | New module implementing divergence metrics and rate–distortion reporting + CBOR serialization. |
| crates/uor-r4-core/src/transformerless/runtime.rs | Adds derives to `OpKernel` to support report serialization and comparisons. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new `predictive_sufficiency` module (non-wasm). |
| crates/uor-r4-core/tests/predictive_sufficiency_test.rs | New tests for KL/cross-entropy, depth evaluation, and CBOR round-trip. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3627033624 — 2026-07-22T02:40:57Z — Copilot

_on crates/uor-r4-core/src/transformerless/predictive_sufficiency.rs:55_

The `0.1` ops-to-bytes scaling factor is an unexplained magic number. Extract it to a named constant so its meaning is discoverable and easy to tune consistently.

### review-comment 3627033634 — 2026-07-22T02:40:57Z — Copilot

_on crates/uor-r4-core/src/transformerless/predictive_sufficiency.rs:122_

`partial_cmp(...).unwrap()` will panic if any probability is NaN. These inputs are floating-point model outputs, so this should be made robust (e.g., ignore non-finite values) rather than crashing.

### review-comment 3627033643 — 2026-07-22T02:40:57Z — Copilot

_on crates/uor-r4-core/src/transformerless/predictive_sufficiency.rs:175_

`top5_recall` is currently hardcoded to `1.0`, which makes the metric misleading/meaningless in the report output. Either compute it from inputs (define what it means here) or remove the field until it can be populated correctly.

### review-comment 3627033652 — 2026-07-22T02:40:57Z — Copilot

_on crates/uor-r4-core/tests/predictive_sufficiency_test.rs:32_

`intermediate` and `full` are unused, which will trigger unused-variable warnings (and likely fail the repo’s `cargo clippy ... -D warnings` check). Prefix with `_` or remove them.

### review-comment 3627033661 — 2026-07-22T02:40:57Z — Copilot

_on crates/uor-r4-core/tests/predictive_sufficiency_test.rs:36_

The KL/cross-entropy tests don’t cover the important edge case where `P(i)=0` and/or `Q(i)=0`. Adding a zero-mass case catches biases introduced by epsilon-clamping and ensures KL(P||P) stays exactly ~0 when zeros are present.

### review 4750559428 — 2026-07-22T02:51:27Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3627073680 — 2026-07-22T02:51:27Z — Copilot

_on crates/uor-r4-core/src/transformerless/predictive_sufficiency.rs:175 · in reply to review-comment 3627033643_

Implemented top-five recall from the teacher argmax and graph top-five predictions, with hit/miss coverage in 9e6a5d7.


---
_Comment/review ids in this file: 5041181724, 4750515734, 4750559428, 3627033611, 3627033619, 3627033624, 3627033634, 3627033643, 3627033652, 3627033661, 3627073680_

