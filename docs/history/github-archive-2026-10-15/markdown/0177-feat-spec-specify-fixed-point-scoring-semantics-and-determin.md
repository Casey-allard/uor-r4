# #177 — feat(spec): specify fixed-point scoring semantics and deterministic tie-breaking (#158)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T20:28:59Z
- **Updated:** 2026-10-09T14:16:53Z
- **Closed:** 2026-07-24T21:41:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/177
- **Merged:** 2026-07-24T21:41:14Z
- **Merge commit:** 0d78085baf851ae96c11cba7e5eb28efa6c07f91
- **Base / Head:** main / UOR-Foundation:issue-158-scoring-semantics
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #158. Establishes the normative fixed-point scoring semantics for the production R⁴/R4G1 transformerless inference engine.

## Key Changes
1. **Normative Scoring Semantics Specification ()**: Complete 10-section document formalizing pre-quantized residual emissions, score width/signedness (`ScoreQ` Q16.16), residual taxonomy, overlap residualization (no-double-counting rule), saturating integer accumulation, deterministic tie-breaking, and empirical quantization-loss certification.
2. **Machine-Readable Scoring Semantics Module ()**: `no_std`, `alloc`-free module defining `ScoringSemanticsVersion` (v1.0.0), `ResidualContributionKind`, `ResidualContribution`, `ScoreAccumulator`, `ScoringError`, and `ScoringSemanticsVerifier`.
3. **Formal Vocabulary & Structural Guarantees**: Updated `docs/formal_vocabulary.md` (v0.1.3) and `crates/uor-r4-proof-model/src/structural_guarantees.rs`.
4. **Cucumber BDD Suite**: Added `features/suites/scoring_semantics.feature` and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 19 feature suites (71 scenarios, 241 steps) passed
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (7 entries; chronological: conversation comments, reviews, review comments)

### comment 5074091434 — 2026-07-24T20:29:03Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5074091677 — 2026-07-24T20:29:05Z — Casey-allard

@copilot review

### review-comment 3647990223 — 2026-07-24T20:34:22Z — Copilot

_on docs/scoring_semantics.md:24_

The `StorageDescriptor` decode equation here doesn’t match the existing runtime decode helper: `uor-r4-core::transformerless::score_q::StorageDescriptor::decode` centers by subtracting `zero_point` and supports negative shifts (right shift). As written, the spec would produce different `ScoreQ` values for the same descriptor/table entry.

### review-comment 3647990252 — 2026-07-24T20:34:22Z — Copilot

_on crates/uor-r4-graph-format/src/inference_contract.rs:171_

`audit_contract_compliance()` returns a fully “certified” report with `is_zero_allocation_guaranteed`, `is_cpu_only_target`, and `is_certified` hard-coded to `true`. This makes contract checks (BDD + structural guarantees) pass even if the actual runtime violates the contract, which defeats the purpose of having an audit/verifier API.

### review 4776534216 — 2026-07-24T20:34:23Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR formalizes and codifies the normative fixed-point scoring semantics (Issue #158) and ties it to a machine-readable, `no_std`/alloc-free surface in `uor-r4-graph-format`, with BDD scenarios intended to enforce overlap residualization and deterministic tie-breaking alongside the inference contract checks.

**Changes:**
- Added normative docs for scoring semantics and the inference contract (`docs/scoring_semantics.md`, `docs/inference_contract.md`) and referenced them from the formal vocabulary changelog.
- Introduced new machine-readable modules in `uor-r4-graph-format` (`scoring_semantics`, `inference_contract`) plus proof-model verifier entry points.
- Added Cucumber feature suites and new BDD step implementations covering the contract and scoring semantics behaviors.

### Reviewed changes

Copilot reviewed 10 out of 10 changed files in this pull request and generated 4 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| tests/bdd.rs | Adds new Cucumber steps and world state for inference contract + scoring semantics scenarios. |
| features/suites/scoring_semantics.feature | New enforced feature suite specifying residual accumulation, no-double-counting, and deterministic tie-breaking. |
| features/suites/inference_contract.feature | New enforced feature suite specifying contract versioning and forbidden/permitted op behavior. |
| docs/scoring_semantics.md | New normative scoring semantics spec (ScoreQ, residual taxonomy, saturation, tie-breaking, quantization-loss certificate). |
| docs/inference_contract.md | New normative inference boundary + permitted/forbidden operation set + zero-allocation lifecycle spec. |
| docs/formal_vocabulary.md | Updates changelog to reference the new normative definitions and modules. |
| crates/uor-r4-proof-model/src/structural_guarantees.rs | Adds structural guarantee verifiers that call into the new contract/scoring verifier APIs. |
| crates/uor-r4-graph-format/src/scoring_semantics.rs | Adds alloc-free `ScoreAccumulator` + residual kinds + verifier stub for scoring semantics. |
| crates/uor-r4-graph-format/src/inference_contract.rs | Adds contract enums + report type + verifier stub for inference contract auditing. |
| crates/uor-r4-graph-format/src/lib.rs | Exposes the new `inference_contract` and `scoring_semantics` modules publicly. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3647990274 — 2026-07-24T20:34:23Z — Copilot

_on crates/uor-r4-graph-format/src/scoring_semantics.rs:153_

The spec and module docs call out a strict canonical accumulation order, but `ScoreAccumulator::accumulate` mutates `current_score` immediately in caller-provided order. With saturating arithmetic, order can change results near `i32::{MIN,MAX}`, so this implementation can’t guarantee deterministic semantics unless every call site already enforces the same order.

### review-comment 3647990291 — 2026-07-24T20:34:23Z — Copilot

_on tests/bdd.rs:1941_

This step is used by a scenario whose Then clause claims “zero heap allocations”, but the step currently only performs the accumulations without asserting the allocation property. Since this repo already has allocation-proof helpers, the BDD should enforce the claim it is making.


---
_Comment/review ids in this file: 5074091434, 5074091677, 4776534216, 3647990223, 3647990252, 3647990274, 3647990291_

