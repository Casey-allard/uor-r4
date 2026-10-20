# #141 — Define measurable holographic encoding contract with ablation/recovery metrics and degeneracy guards

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-24T14:10:34Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-07-24T17:07:15Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/141
- **Merged:** 2026-07-24T17:07:15Z
- **Merge commit:** 830764993f3fc6b0fb76e488f1a15ad88fbe03b8
- **Base / Head:** main / UOR-Foundation:copilot/specify-holographic-encoding
- **Reviews / inline review comments:** 1 / 3
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR turns `H(x)` from metaphor into an explicit measurement contract: projection-family structure, ablation semantics, reconstruction behavior, and fidelity/divergence reporting are now represented as certifiable data. It also adds deterministic probes that quantify partial recoverability, distributed evidence, and progressive fidelity while separating structural encoding facts from empirical teacher-agreement claims.

- **Holographic encoding certificate schema (new module)**
  - Added `holographic_encoding.rs` with:
    - projection metadata (`projection_id`, `depth`, `membership_ids`)
    - ablation protocol (`baseline_projection_ids`, `ablation_order`, semantics)
    - declared divergence metric (`KL`, `JSD`)
    - sample count, 95% CI, corpus CIDs
    - split fields for `structural_encoding_facts` vs `teacher_agreement_claims`
  - Added self-referential CID generation/verification for certificate integrity.

- **Reconstruction and fidelity operators**
  - Implemented behavioral recovery over active projection subsets.
  - Implemented divergence evaluation against teacher distributions.
  - Implemented deterministic probe outputs for:
    - per-projection partial recovery
    - distributed evidence support
    - progressive fidelity as projections/membership increase
    - ablation degradation curve
    - paraphrase invariance, perturbation stability, cross-context reuse

- **Degenerate encoding detection/reporting**
  - Added explicit rejection errors for:
    - empty projection sets
    - empty projection memberships
    - single-node memorization patterns
    - duplicate projections
    - invalid recovery distributions
    - unknown projection references in ablation/recovery

- **Deterministic fixture tests**
  - Added `holographic_encoding_test.rs` with deterministic projection fixtures validating:
    - partial recovery/progressive-fidelity behavior
    - ablation monotonic degradation expectations
    - probe metrics consistency
    - certificate schema + CID roundtrip invariants
    - degeneracy rejection paths

- **Spec binding update**
  - Updated `docs/formal_vocabulary.md` (`H(x)` row + changelog) to bind the issue-#126 measurement contract to the new certifier module and fixture tests.

```rust
let cert = HolographicEncodingCertificate::new(
    projection_set,
    ablation_protocol,
    DivergenceMetric::JensenShannonDivergence,
    sample_count,
    confidence_interval_95,
    corpus_cids,
    structural_encoding_facts,
    teacher_agreement_claims,
    probe_report,
);
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #126

## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### review 4774691270 — 2026-07-24T15:44:17Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR formalizes the “holographic encoding” concept (`H(x)`) as a measurable, certifiable contract in `uor-r4-graph-certify`, and binds the formal vocabulary to that implementation and its deterministic fixtures (issue #126).

**Changes:**
- Adds a new `holographic_encoding` certifier module defining projection-family schema, divergence metrics, CID integrity, and probe outputs (partial recovery, progressive fidelity, ablation).
- Adds deterministic fixture tests covering progressive-fidelity expectations, ablation monotonicity, CID roundtrips, and degeneracy rejection.
- Updates `docs/formal_vocabulary.md` to bind the `H(x)` definition to the new certifier module and fixtures.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 3 comments.

| File | Description |
| ---- | ----------- |
| docs/formal_vocabulary.md | Updates the formal notation binding for `H(x)` and adds a changelog entry referencing the certifier implementation/tests. |
| crates/uor-r4-graph-certify/src/lib.rs | Exposes the new `holographic_encoding` module from the certifier crate. |
| crates/uor-r4-graph-certify/src/holographic_encoding.rs | Introduces the certificate schema, CID hashing/verification, degeneracy guards, and evaluators for recovery/fidelity/ablation probes. |
| crates/uor-r4-graph-certify/tests/holographic_encoding_test.rs | Adds deterministic fixtures and tests validating the new contract behaviors and rejection paths. |




<details>
<summary>Comments suppressed due to low confidence (2)</summary>

**crates/uor-r4-graph-certify/src/holographic_encoding.rs:193**
* `recover_behavior_distribution` assumes all projection distributions have the same length. If a later projection has a different length, `acc[idx] += *value` can panic with an out-of-bounds index.
```
            if acc.is_empty() {
                acc = vec![0.0; projection.recovered_distribution.len()];
            }
            for (idx, value) in projection.recovered_distribution.iter().enumerate() {
                acc[idx] += *value;
```
**crates/uor-r4-graph-certify/src/holographic_encoding.rs:251**
* `distributed_evidence_mean_support` indexes `projection.recovered_distribution[token_idx]` based on the first projection’s length; if any projection has a shorter distribution this will panic.
```
                .filter(|projection| {
                    projection.recovered_distribution[token_idx] >= support_threshold
                })
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3646530955 — 2026-07-24T15:44:17Z — Copilot

_on crates/uor-r4-graph-certify/src/holographic_encoding.rs:207_

`validate_projection_family` doesn’t reject duplicate `projection_id`s or inconsistent `recovered_distribution` lengths. Duplicate IDs make `recover_behavior_distribution` ambiguous (it takes the first match), and length mismatches can panic later when indexing/accumulating distributions.

This issue also appears in the following locations of the same file:
- line 189
- line 249

### review-comment 3646530984 — 2026-07-24T15:44:17Z — Copilot

_on crates/uor-r4-graph-certify/src/holographic_encoding.rs:349_

`ablation_curve` silently accepts `ablation_order` entries that aren’t present in `baseline_projection_ids` (because `active.remove` is ignored). This makes “unknown projection references in ablation” pass without an error, contrary to the stated degeneracy guard.

### review-comment 3646531004 — 2026-07-24T15:44:17Z — Copilot

_on crates/uor-r4-graph-certify/src/holographic_encoding.rs:272_

`progressive_fidelity` uses `unwrap_or_default()` on `recover_behavior_distribution`, which silently discards `DegeneracyError`s and can yield empty recovered distributions (and thus `NaN` divergences) without signaling failure to callers.


---
_Comment/review ids in this file: 4774691270, 3646530955, 3646530984, 3646531004_

