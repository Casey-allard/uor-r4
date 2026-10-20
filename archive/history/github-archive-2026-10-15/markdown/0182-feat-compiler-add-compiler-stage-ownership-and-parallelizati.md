# #182 — feat(compiler): add compiler stage ownership and parallelization DAG (#166)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-25T00:42:21Z
- **Updated:** 2026-10-09T14:16:54Z
- **Closed:** 2026-07-25T01:17:42Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/182
- **Merged:** 2026-07-25T01:17:42Z
- **Merge commit:** 65eae7c07096b80c35516eed5082f626009eb438
- **Base / Head:** main / UOR-Foundation:issue-166-compiler-stage-dag
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #166. Formally classifies all 28 R4 compiler pipeline stages into four concurrency classes (`Parallel-Safe`, `Parallel with Deterministic Merge`, `Bounded Parallel`, and `Sequential Canonical Finalization`), assigns explicit parallel boundary edge ownership (#170, #171, #172, #173), and provides programmatic stage inventory validation.

## Key Changes
1. **Normative Specification (`docs/compiler_stage_dag.md`)**:
   - Formally classifies 28 pipeline stages into 4 concurrency classes.
   - Maps each stage to its module, inputs, outputs, determinism justification, and boundary owner.
   - Defines and protects the 6-stage Sequential Canonical Finalization Spine (`S12`, `S24`–`S28`).
2. **Implementation Plan Cross-Link (`docs/r4_graph_compiler_implementation_plan.md`)**: Added cross-links in §4.1 and §5.
3. **Programmatic Stage Inventory (`crates/uor-r4-graph-compiler/src/stage_dag.rs`)**:
   - `CompilerStageDag` registry with `all_stages()` and `finalization_spine()`.
   - Unit tests verifying inventory completeness and spine protection.
4. **Proof Model Integration (`crates/uor-r4-proof-model/src/structural_guarantees.rs`)**: Added `verify_compiler_stage_dag_compliance`.
5. **Formal Vocabulary (`docs/formal_vocabulary.md`)**: Updated to v0.1.7.
6. **Cucumber BDD Suite (`features/suites/compiler_stage_dag.feature`)**: Added scenarios for 28-stage classification and canonical finalization spine protection.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: Passed (All compiler tests passed)
- `cargo test --test bdd --offline`: Passed (82/82 BDD scenarios passed)
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`: Clean
- `python3 scripts/check_claim_wording.py`: Passed

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5075894539 — 2026-07-25T00:42:25Z — Casey-allard

@copilot please review this PR

### comment 5075894569 — 2026-07-25T00:42:25Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5075921297 — 2026-07-25T00:48:43Z — Copilot


> @copilot please review this PR

Three issues found and fixed in `structural_guarantees.rs` (commit `fix(proof-model): correct verify_compiler_stage_dag_compliance`):

**Bug — silent false-positive on validation failure**
`verify_compiler_stage_dag_compliance` returned `Ok(ProofVerificationReport { status: Verified, verified: false })` when the stage count or spine was wrong. Every other method in this impl block returns `Err(ProofValidationError::...)` on failure, so callers that `.unwrap()` or check only `report.status` would have been misled. Fixed to `return Err(ProofValidationError::NondeterministicOutput { … })` on `!valid`.

**Semantic misclassification — wrong `StructuralObligationKind`**
The kind was `BoundedResource`, which is for memory/latency limits. This obligation guards the Sequential Canonical Finalization spine — fundamentally a D2 canonical artifact reproducibility concern — so `Determinism` is the correct kind.

**API inconsistency — `&str` vs `impl Into<String>`**
Every other method takes `obligation_id: impl Into<String>`; this one took `&str`. Normalized.

**Missing unit test**
Added `test_verify_compiler_stage_dag_compliance` covering the success path (`verified: true`, `kind == Determinism`).

All 17 proof-model tests pass; `cargo fmt --check` and `cargo clippy -D warnings` are clean.


---
_Comment/review ids in this file: 5075894539, 5075894569, 5075921297_

