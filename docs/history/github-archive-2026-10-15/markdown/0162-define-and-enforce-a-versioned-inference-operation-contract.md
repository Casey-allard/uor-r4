# #162 — Define and enforce a versioned inference operation contract across runtime boundary modules

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-24T18:28:52Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-07-24T21:43:31Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/162
- **Merged:** 2026-07-24T21:43:31Z
- **Merge commit:** 0a35e4e66aaf342b6b4260630883b9088ef17736
- **Base / Head:** main / UOR-Foundation:copilot/define-normative-inference-contract
- **Reviews / inline review comments:** 2 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR turns the multiplication-free runtime rule from scattered prose/tests into a single versioned contract with explicit inference boundary, operation classes, exclusions, and ownership mappings. It also extends enforcement to all contract-owned runtime modules and removes current in-boundary violations.

- **Normative contract definition (versioned)**
  - Added `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md` (v0.1.0) as the authoritative **Definition** for:
    - in-scope runtime activities,
    - allowed/forbidden operation lists,
    - explicit exclusions (compiler/training/offline flows unrestricted),
    - compiler-generated arithmetic policy,
    - traceability/status language (Witnessed now, Structural after disassembly audit).

- **Machine-readable contract module (`no_std`)**
  - Added `crates/uor-r4-graph-format/src/inference_contract.rs` and exported it from `lib.rs`.
  - Encodes:
    - `INFERENCE_OPERATION_CONTRACT_VERSION`,
    - boundary activity registry,
    - allowed/forbidden operation classes,
    - explicit exclusions,
    - activity→owner mapping with focused error type.
  - Added version encode/decode and coverage tests for mapping completeness.

- **Invariant/proof integration**
  - Added `crates/uor-r4-graph-format/src/invariant_ownership.rs` with an explicit `Operation-Set Conformance` row and exported it from `lib.rs`.
  - Extended `crates/uor-r4-proof-model/src/proof_matrix.rs` with an `Operation-Set Conformance` entry (witnessed source-scan evidence path).
  - Updated proof-matrix audit test to assert the new entry.

- **Boundary enforcement expansion**
  - Refactored the P-4 scanner in `crates/uor-r4-core/src/transformerless/mod.rs` into a reusable detector.
  - Added scan coverage for all contract-owned graph-runtime modules:
    - `engine.rs`, `routing.rs`, `runtime_state.rs`, `status.rs`, `cayley_dickson.rs`.
  - Scanner now catches operator and method-form mul/div/rem usage (e.g. `wrapping_mul`, `saturating_mul`).

- **In-scope runtime violations addressed**
  - Reworked `crates/uor-r4-graph-runtime/src/cayley_dickson.rs` to remove live multiplication:
    - replaced multiplicative hash step,
    - replaced scalar product multiplication with shift/add saturating multiply routine.
  - Replaced legacy hot-path `unreachable!` terminals in `crates/uor-r4-core/src/transformerless/runtime.rs` prediction methods with deterministic fallback returns.

- **Contract/doc synchronization hooks**
  - Cross-linked contract from:
    - `docs/r4_graph_compiler_implementation_plan.md` (§6),
    - `docs/transformerless/GLOSSARY.md` (“Normative kernel”),
    - `crates/uor-r4-core/README.md`,
    - `docs/formal_vocabulary.md` (guarantee table status row).
  - Added BDD scenario + step bindings to assert doc version matches module version:
    - `features/suites/inference_operation_contract.feature`
    - `tests/bdd.rs`

Example of the new machine-readable contract surface:

```rust
pub const INFERENCE_OPERATION_CONTRACT_VERSION: ContractVersion = ContractVersion {
    major: 0,
    minor: 1,
    patch: 0,
};

pub const BOUNDARY_ACTIVITIES: [BoundaryActivity; 9] = [
    BoundaryActivity::IncrementalContextSignatureUpdate,
    BoundaryActivity::SemanticRegionRouting,
    BoundaryActivity::CandidateVerification,
    BoundaryActivity::ActiveFrontierUpdate,
    BoundaryActivity::TransitionScoring,
    BoundaryActivity::GoalConstraintScoring,
    BoundaryActivity::TokenCandidateScoringAndShortlist,
    BoundaryActivity::FixedWidthPlanning,
    BoundaryActivity::ScoreQDescriptorDecode,
];
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #157

## Timeline (8 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3647538903 — 2026-07-24T18:55:49Z — Copilot

_on crates/uor-r4-graph-format/src/inference_contract.rs:38_

`ContractVersion::encode_packed` silently truncates `major/minor/patch` if they exceed the bit widths implied by the packed format (major: 12 bits, minor/patch: 10 bits). That would make the packed representation non-invertible and could hide a future version bump bug.

### review-comment 3647538943 — 2026-07-24T18:55:49Z — Copilot

_on crates/uor-r4-core/src/transformerless/mod.rs:68_

`scan_for_forbidden_arith` only skips lines that *start* with `//`, but it still scans trailing `// ...` comments. That can produce false positives if a line has an inline comment mentioning `mul(` / `div(` / etc, and it also contradicts the doc comment claim that comments are stripped.

### review-comment 3647538969 — 2026-07-24T18:55:49Z — Copilot

_on crates/uor-r4-graph-format/src/lib.rs:71_

The contract doc advertises the machine-readable binding as `uor-r4-graph-format::inference_contract`, but `inference_contract` is declared as a private module here. External users can’t actually refer to that module path (only to the re-exported items).

### review 4775957860 — 2026-07-24T18:55:50Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR formalizes the multiplication-free inference/runtime boundary as a single versioned “Inference Operation Contract”, provides a machine-readable contract surface in `uor-r4-graph-format`, and extends source-scan enforcement to additional contract-owned runtime modules while removing known in-boundary arithmetic violations.

**Changes:**
- Add a normative, versioned contract document and sync it with a machine-readable contract version constant (plus BDD coverage).
- Introduce machine-readable contract registries (activities, allow/forbid classes, exclusions, ownership mapping) and an invariant-ownership row tied to the contract version.
- Expand P-4 source-scan enforcement and remove runtime-bound multiplication in `cayley_dickson.rs`; replace hot-path `unreachable!` with deterministic fallbacks.

### Reviewed changes

Copilot reviewed 15 out of 15 changed files in this pull request and generated 4 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| tests/bdd.rs | Adds Cucumber steps to assert contract doc version matches the module constant. |
| features/suites/inference_operation_contract.feature | Adds BDD scenario for doc/module contract version agreement. |
| docs/transformerless/INFERENCE_OPERATION_CONTRACT.md | New normative contract definition (v0.1.0) including scope, allow/forbid lists, exclusions, and ownership map. |
| docs/transformerless/GLOSSARY.md | Cross-links the “Normative kernel” definition to the new contract document. |
| docs/r4_graph_compiler_implementation_plan.md | References the new normative contract and its machine-readable binding. |
| docs/formal_vocabulary.md | Adds an “Inference operation-set conformance” guarantee row/status reference. |
| crates/uor-r4-proof-model/src/proof_matrix.rs | Adds proof-matrix entry for “Operation-Set Conformance”. |
| crates/uor-r4-proof-model/tests/proof_matrix_audit.rs | Extends audit test to assert the new proof-matrix entry exists with expected status. |
| crates/uor-r4-graph-format/src/inference_contract.rs | Adds the machine-readable contract surface (version, registries, ownership mapping, errors, tests). |
| crates/uor-r4-graph-format/src/invariant_ownership.rs | Adds an invariant-ownership row referencing the contract version. |
| crates/uor-r4-graph-format/src/lib.rs | Wires new contract/ownership modules and re-exports their surfaces. |
| crates/uor-r4-graph-runtime/src/cayley_dickson.rs | Removes runtime multiplication by replacing scalar-product mul with shift/add saturating routine and updates hashing. |
| crates/uor-r4-core/src/transformerless/mod.rs | Refactors and expands the P-4 arithmetic source scanner to cover additional runtime-owned modules. |
| crates/uor-r4-core/src/transformerless/runtime.rs | Replaces `unreachable!` terminals with deterministic fallback returns in prediction methods. |
| crates/uor-r4-core/README.md | Documents the contract as the normative source for the boundary and operation classes. |
</details>





<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**crates/uor-r4-core/src/transformerless/runtime.rs:483**
* Same as `predict_witness_plain`: this fallback now silently returns token=0 on invariant violation. Adding a debug-only assertion keeps release determinism while still catching unexpected empty stores during development.
```
    Prediction {
        token: 0,
        depth: 0,
        count: 0,
    }
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3647538998 — 2026-07-24T18:55:50Z — Copilot

_on crates/uor-r4-core/src/transformerless/runtime.rs:448_

This fallback silently returns token=0 when the "level 0 is always populated" invariant is violated. That makes bugs harder to detect in debug/test runs and the comment above is no longer accurate.

This issue also appears on line 479 of the same file.

### comment 5074489419 — 2026-07-24T21:17:08Z — Casey-allard

@copilot resolve the merge conflicts in this pull request

### comment 5074565518 — 2026-07-24T21:25:01Z — Copilot


> @copilot resolve the merge conflicts in this pull request

Resolved the merge conflicts in this PR and pushed the fix in commit `7cbe472`.

### review 4776927291 — 2026-07-24T21:38:24Z — Casey-allard — APPROVED


---
_Comment/review ids in this file: 5074489419, 5074565518, 4775957860, 4776927291, 3647538903, 3647538943, 3647538969, 3647538998_

