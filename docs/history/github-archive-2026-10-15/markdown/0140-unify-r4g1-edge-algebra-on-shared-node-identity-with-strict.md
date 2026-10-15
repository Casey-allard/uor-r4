# #140 — Unify R4G1 edge algebra on shared node identity with strict kind/index validation

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-24T14:10:20Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-07-24T17:00:01Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/140
- **Merged:** 2026-07-24T17:00:01Z
- **Merge commit:** 54d13f54fdc24f7e7157639ef5f2cc47d2e77022
- **Base / Head:** main / UOR-Foundation:copilot/unify-edges-over-one-node-space
- **Reviews / inline review comments:** 1 / 2
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This change extends R4G1 so one canonical node space can carry multiple edge algebras (semantic/causal/temporal/constraint/provenance-style relations) without duplicating node identity. It also hardens loader invariants so edge kinds, payload semantics, and declared node indexes are validated coherently.

- **Edge algebra: stable discriminants + shared-node semantics**
  - Added stable `EdgeKind` discriminants for:
    - refinement/abstraction, similarity, predictive-forward, causal, temporal, constraint, goal-progress, evidence/provenance, evidential-predecessor
  - Added optional-kind namespace (`kind & 0x80`) so unknown mandatory kinds reject, unknown optional kinds can be skipped.

- **R4G1 mapping + feature gating (no packed-width changes)**
  - Introduced `FEATURE_EDGE_ALGEBRA_V1` in `HEAD.feature_bits_required`.
  - Kept `PackedEdge` layout unchanged; `reserved` now carries contribution IDs when edge-algebra-v1 is declared.

- **Stage-2 loader validation hardening**
  - Rejects unknown mandatory edge kinds and invalid payload encodings.
  - Enforces canonical edge ordering for stable contribution accounting.
  - Adds contribution-ID collision checks to prevent duplicate evidence counting on the same `(src,dst)`.
  - Enforces declared acyclic-kind constraints (reject undeclared cycles).
  - Validates node-declared child/forward ranges against actual canonical/reverse index targets (no dangling/mismatched index wiring).

- **Shared-node fixture coverage**
  - Added a round-trip fixture with multiple edge kinds over the same node set (including evidence/provenance-style and predictive edges).
  - Added focused rejection tests for unknown mandatory kinds, invalid payloads, contribution-ID errors, cycle declaration violations, and forward/reverse range inconsistencies.

- **Spec/docs alignment**
  - Updated R4G1 and graph-format docs to reflect stable edge-kind table, feature-bit behavior, contribution-ID semantics, and tightened index/validation rules.

```rust
#[repr(u8)]
pub enum EdgeKind {
    RefinementAbstraction = 0,
    Similarity = 1,
    PredictiveForward = 2,
    Causal = 3,
    Temporal = 4,
    Constraint = 5,
    GoalProgress = 6,
    EvidenceProvenance = 7,
    EvidentialPredecessor = 8,
}
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #125

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3646531100 — 2026-07-24T15:44:18Z — Copilot

_on crates/uor-r4-graph-format/src/stage2.rs:345_

The reverse-index existence scan still requires *unknown optional* edge kinds to appear at least once in the reverse block. That weakens the stated “optional kinds may be skipped” forward-compat story: a producer could reasonably omit optional edges from reverse coverage, but this validator would reject the artifact.

If optional kinds are intended to be skippable by readers, consider excluding unknown optional kinds from the reverse-coverage requirement (similar to how known kinds can opt out via requires_reverse_index()).

### review 4774691444 — 2026-07-24T15:44:19Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR extends the R4G1 packed graph format to support multiple edge algebras over a single shared node identity space (stable `EdgeKind` discriminants + optional-kind namespace), and tightens stage-2 validation to enforce coherent kind/payload semantics and index wiring invariants.

**Changes:**
- Introduces stable edge-kind discriminants (`EdgeKind`) plus optional-kind handling, and a feature bit (`FEATURE_EDGE_ALGEBRA_V1`) to enable contribution-id semantics in `PackedEdge.reserved`.
- Hardens stage-2 validation: canonical edge ordering, unknown-kind rejection rules, edge payload validation (flags/reserved/contribution id), contribution-id collision checks, and stricter child/forward/reverse index wiring checks.
- Expands tests and docs to cover shared-node, multi-kind fixtures and new rejection cases.

### Reviewed changes

Copilot reviewed 9 out of 9 changed files in this pull request and generated 2 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| docs/transformerless/R4G1.md | Updates spec text for stable edge-kind table, optional-kind namespace, feature-bit behavior, and tighter reverse/index wiring expectations. |
| crates/uor-r4-graph-format/src/stage2.rs | Implements stage-2 validation hardening for kind/payload rules, canonical ordering, collision checks, and forward/child/reverse wiring validation. |
| crates/uor-r4-graph-format/src/records.rs | Adds `EdgeKind` enum + helpers (`is_optional_edge_kind`) and documents updated `PackedEdge.reserved` semantics. |
| crates/uor-r4-graph-format/src/head.rs | Adds `FEATURE_EDGE_ALGEBRA_V1` and `KNOWN_FEATURE_BITS_REQUIRED` for required-feature gating. |
| crates/uor-r4-graph-format/src/error.rs | Adds new stage-2 error variants and an `EdgePayloadField` discriminator for clearer failure reporting. |
| crates/uor-r4-graph-format/src/lib.rs | Re-exports new edge algebra types/constants and updates crate-level stage-2 description. |
| crates/uor-r4-graph-format/README.md | Aligns README stage-2 validation description with the strengthened invariants. |
| crates/uor-r4-graph-format/tests/stage2.rs | Adds shared-node multikind round-trip + new rejection tests for kind/payload/order/collision/wiring invariants; updates existing fixtures. |
| crates/uor-r4-graph-format/tests/common/mod.rs | Documents `EdgeFields.reserved` meaning in v0 vs edge-algebra-v1. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3646531131 — 2026-07-24T15:44:19Z — Copilot

_on crates/uor-r4-graph-format/tests/stage2.rs:632_

The test comment doesn’t match what the fixture actually does: the reverse index passed is `[0, 1]`, so the first reverse slot points to edge 0 (not edge 1). This makes the intent of the test harder to follow.


---
_Comment/review ids in this file: 4774691444, 3646531100, 3646531131_

