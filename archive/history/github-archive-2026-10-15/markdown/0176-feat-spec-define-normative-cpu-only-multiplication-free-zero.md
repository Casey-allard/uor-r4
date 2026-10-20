# #176 — feat(spec): define normative CPU-only, multiplication-free, zero-allocation inference contract (#157)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T20:16:09Z
- **Updated:** 2026-10-09T14:16:52Z
- **Closed:** 2026-07-24T21:06:35Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/176
- **Merged:** 2026-07-24T21:06:35Z
- **Merge commit:** 843d9a6910c181f5b63689ac1f93cf5f59d0a83a
- **Base / Head:** main / UOR-Foundation:issue-157-inference-contract
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #157. Establishes the normative execution contract for the production R⁴/R4G1 transformerless graph inference engine.

## Key Changes
1. **Normative Contract Specification ()**: Complete 10-section document formalizing scope, boundaries, permitted/forbidden operation classes, zero-allocation steady-state lifecycle, CPU-only target platform specification, and typed execution interfaces.
2. **Machine-Readable Contract Module ()**: `no_std`, `alloc`-free module defining `InferenceContractVersion` (v1.0.0), `BoundaryActivity`, `OperationClass`, `LifecyclePhase`, and `InferenceContractVerifier`.
3. **Formal Vocabulary & Proof Model Integration**: Updated `docs/formal_vocabulary.md` (v0.1.2) and `crates/uor-r4-proof-model/src/structural_guarantees.rs`.
4. **Cucumber BDD Suite**: Added `features/suites/inference_contract.feature` and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All 18 feature suites (68 scenarios, 232 steps) passed
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (7 entries; chronological: conversation comments, reviews, review comments)

### comment 5073976844 — 2026-07-24T20:16:14Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5074013008 — 2026-07-24T20:20:27Z — Casey-allard

@copilot review

### review-comment 3647948589 — 2026-07-24T20:26:48Z — Copilot

_on docs/inference_contract.md:28_

The referenced inference-boundary entrypoints don’t match the current codebase: `R4G1Runtime::predict_step` and `routing::evaluate_program` don’t exist (graph runtime exposes `predict_token`/`predict_distribution`/`predict_candidates` and routing exposes `evaluate_route`). Also `transformerless::runtime::predict` is a method on `Runtime`, not a free function. Please update the boundary list to point at the actual entrypoints.

This issue also appears in the following locations of the same file:
- line 67
- line 85

### review-comment 3647948653 — 2026-07-24T20:26:49Z — Copilot

_on crates/uor-r4-graph-format/src/inference_contract.rs:41_

This doc comment references `infer_step`/`predict_step`, but the graph runtime hot-path methods are `R4G1Runtime::predict_token`/`predict_distribution`/`predict_candidates` (and transformerless uses `Runtime::predict`). Updating the comment avoids pointing readers at non-existent APIs.

### review-comment 3647948709 — 2026-07-24T20:26:49Z — Copilot

_on crates/uor-r4-graph-format/src/inference_contract.rs:174_

`InferenceContractVerifier::audit_contract_compliance()` currently returns a fully “certified” report unconditionally (hardcoded booleans), so it will report compliance even when the actual runtime violates the contract (e.g., `crates/uor-r4-graph-runtime/src/cayley_dickson.rs` uses `wrapping_mul`/`saturating_mul`). This makes the audit/certification signal unreliable and can cause downstream proof checks to incorrectly report Verified. Consider either implementing a real audit (and failing/uncertified when violations exist) or renaming/reframing this API to represent the contract requirements rather than an audited compliance report.

### review 4776468305 — 2026-07-24T20:26:50Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds a new normative “inference contract” specification and exposes it as a machine-readable module, then wires it into the proof model and the existing Cucumber BDD harness so the contract can be referenced and (eventually) enforced across the workspace.

**Changes:**
- Added a new `docs/inference_contract.md` spec document and referenced it from the formal vocabulary changelog.
- Introduced `uor_r4_graph_format::inference_contract` with versioned contract enums/types plus a verifier/audit surface.
- Added a new BDD feature suite and step definitions, and integrated a proof-model verifier hook for contract compliance.

### Reviewed changes

Copilot reviewed 7 out of 7 changed files in this pull request and generated 4 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| `tests/bdd.rs` | Adds Cucumber step definitions that call the inference contract verifier and assert expected contract properties. |
| `features/suites/inference_contract.feature` | New BDD feature suite describing expected contract verification and operation-class acceptance/rejection. |
| `docs/inference_contract.md` | New normative contract document defining scope, permitted/forbidden operations, lifecycle, and CPU-only target. |
| `docs/formal_vocabulary.md` | Changelog entry referencing the new inference contract definitions and module. |
| `crates/uor-r4-proof-model/src/structural_guarantees.rs` | Adds a structural guarantee verifier function that calls into the contract verifier and reports compliance. |
| `crates/uor-r4-graph-format/src/lib.rs` | Exposes the new `inference_contract` module publicly from `uor-r4-graph-format`. |
| `crates/uor-r4-graph-format/src/inference_contract.rs` | New no_std contract representation (version/enums/report) plus verifier/audit APIs. |
</details>





<details>
<summary>Comments suppressed due to low confidence (2)</summary>

**docs/inference_contract.md:67**
* This line references `infer_step`/`predict_step`, but those names don’t correspond to the current runtime APIs. Using the actual hot-path entrypoints here avoids implying a non-existent interface.
```
   Repeated execution steps (`infer_step`, `predict_step`) operate strictly over pre-allocated fixed-capacity memory buffers (`RuntimeState`, `StepOutput`). **Heap allocation is 0 bytes.**
```
**docs/inference_contract.md:92**
* The `infer_step(...)` Rust signature shown here doesn’t exist in the repository (and types like `StepOutput`/`RuntimeError` aren’t defined in this context). Consider replacing this with either pseudocode explicitly labeled as such, or pointers to the real hot-path entrypoints/signatures so the contract doesn’t look like it’s specifying an API that doesn’t exist.
```
```rust
pub fn infer_step(
    graph: &GraphView<'_>,
    state: &mut RuntimeState,
    token: u32,
    output: &mut StepOutput,
) -> Result<(), RuntimeError>;
```
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3647948751 — 2026-07-24T20:26:50Z — Copilot

_on crates/uor-r4-proof-model/src/structural_guarantees.rs:418_

`verify_inference_contract_compliance` maps any contract-audit failure into `ResourceBoundExceeded` (with `actual: 1, limit: 0`), which doesn’t reflect what went wrong, and it always sets `status: ProofStatus::Verified` even when `contract_report.is_certified` is false. This can produce internally inconsistent reports (status “Verified” but `verified: false`) and hides the underlying audit failure reason.


---
_Comment/review ids in this file: 5073976844, 5074013008, 4776468305, 3647948589, 3647948653, 3647948709, 3647948751_

