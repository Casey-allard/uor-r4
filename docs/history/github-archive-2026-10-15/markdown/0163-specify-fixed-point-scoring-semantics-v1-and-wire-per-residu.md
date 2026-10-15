# #163 — Specify fixed-point scoring semantics v1 and wire per-residual quantization error reporting

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-24T18:29:07Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-07-24T20:23:32Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/163
- **Merged:** 2026-07-24T20:23:32Z
- **Merge commit:** 9110205d2aab52988b175d3c30edf93bf0d0e081
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-specify-fixed-point-scoring
- **Reviews / inline review comments:** 1 / 2
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR freezes a single, versioned fixed-point scoring contract for integer runtime accumulation (order, saturation, sentinels, tie-breaking, contribution taxonomy, no-double-counting), and aligns runtime/reporting surfaces to that contract. It also adds compile-time quantization error reporting per residual kind so certification artifacts can track `D(·,·) ≤ ε` evidence without runtime error computation.

- **Normative scoring semantics (versioned)**
  - Added `docs/transformerless/FIXED_POINT_SCORING_SEMANTICS_V1.md`.
  - Defines canonical `ScoreQ` semantics for:
    - typed residual kinds (`transition`, `emission`, `goal`, `constraint`, `uncertainty`)
    - canonical contribution order
    - saturating arithmetic boundary behavior
    - sentinel ordering
    - deterministic selection tie-break (`score desc`, `TokenId asc`)
    - overlap residualization/no-double-counting contract.

- **no_std canonical scoring surface in runtime**
  - Added `crates/uor-r4-graph-runtime/src/scoring.rs` and exported it from `lib.rs`.
  - Introduces:
    - `ResidualKind`
    - `ScoreSentinel` + `OrderedScore` total ordering
    - `TypedContribution`
    - `sort_contributions_canonical(...)`
    - `accumulate_reference(...)` (duplicate evidence-id rejection + saturating accumulation)
    - `select_best(...)` (canonical tie-break).

- **Per-residual-kind quantization error telemetry (compile-time)**
  - Extended `crates/uor-r4-graph-certify/src/score.rs`:
    - Added `QuantizationErrorStats`.
    - Added `compile_transitions_with_quantization(...)` and kept `compile_transitions(...)` as compatibility wrapper.
    - Recorded quantization stats for:
      - transition weights
      - root prior (incl. floor)
      - emission residuals
      - exact-context residuals.
    - Threaded stats through `ScoredGraphInfo` / `ScoredGraphSections`.
  - Extended `ScoreReportQuantization` with `residual_kind_errors` rows (`kind`, `cid`, sample count, max/mean abs error).
  - Bumped report schema to `8`.

- **Call-site wiring updates**
  - Updated graph compilation/reporting call sites to pass transition quantization stats:
    - `crates/uor-r4-graph-cli/src/lib.rs`
    - `crates/uor-r4-core/src/transformerless/cover_sweep.rs`
    - `tests/status_policy_common/mod.rs` fixture construction.

Example of the canonical runtime API introduced:

```rust
let mut contributions = [
    TypedContribution { evidence_id: 1, kind: ResidualKind::Transition, value: ScoreQ::from_raw(12) },
    TypedContribution { evidence_id: 2, kind: ResidualKind::Emission,   value: ScoreQ::from_raw(-3) },
];
sort_contributions_canonical(&mut contributions);
let total = accumulate_reference(ScoreQ::ZERO, &contributions)?;
let (token, score) = select_best(&[
    (7, OrderedScore::Real(total)),
    (9, OrderedScore::Real(total)), // tie -> token 7 wins
]).unwrap();
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #158

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3647542735 — 2026-07-24T18:56:34Z — Copilot

_on crates/uor-r4-graph-runtime/src/scoring.rs:94_

`accumulate_reference` only rejects duplicate `evidence_id` values when they are adjacent in the slice. With the documented canonical sort `(kind, evidence_id)`, the same `evidence_id` can appear multiple times separated by other kinds and will not be rejected, violating the no-double-counting contract.

This issue also appears on line 164 of the same file.

### review 4775962311 — 2026-07-24T18:56:35Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR formalizes a versioned fixed-point scoring contract and aligns the runtime/compiler/reporting surfaces to it by adding a canonical runtime scoring API and emitting compile-time quantization error telemetry per residual kind for score reports/certification.

**Changes:**
- Added a normative scoring-semantics v1 document under `docs/transformerless/`.
- Introduced a `no_std` runtime scoring surface (`ResidualKind`, `OrderedScore`, canonical sorting/accumulation, deterministic tie-break).
- Extended compile-time scoring compilation and score reporting to collect and serialize per-residual-kind quantization error stats, wiring the new fields through key call sites.

### Reviewed changes

Copilot reviewed 7 out of 7 changed files in this pull request and generated 2 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| tests/status_policy_common/mod.rs | Updates score-graph fixture construction to provide default quantization stats fields. |
| docs/transformerless/FIXED_POINT_SCORING_SEMANTICS_V1.md | Adds the normative v1 fixed-point scoring semantics spec. |
| crates/uor-r4-graph-runtime/src/scoring.rs | Adds canonical runtime scoring types and reference sorting/accumulation/selection utilities (with unit tests). |
| crates/uor-r4-graph-runtime/src/lib.rs | Exports the new `scoring` module from the runtime crate. |
| crates/uor-r4-graph-cli/src/lib.rs | Wires transition quantization stats through the CLI scoring command path. |
| crates/uor-r4-graph-certify/src/score.rs | Implements quantization error aggregation and extends report schema to include per-kind quantization rows. |
| crates/uor-r4-core/src/transformerless/cover_sweep.rs | Wires transition quantization stats through the cover sweep scoring path. |
</details>





<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**crates/uor-r4-graph-runtime/src/scoring.rs:176**
* This test only covers adjacent duplicate `evidence_id`s. Add a non-adjacent duplicate case (optionally after `sort_contributions_canonical`) to prevent regressions where duplicates across kinds slip through.
```
    fn duplicate_evidence_is_rejected() {
        let contributions = [
            TypedContribution {
                evidence_id: 3,
                kind: ResidualKind::Emission,
                value: ScoreQ::from_raw(4),
            },
            TypedContribution {
                evidence_id: 3,
                kind: ResidualKind::Transition,
                value: ScoreQ::from_raw(5),
            },
        ];
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3647542767 — 2026-07-24T18:56:35Z — Copilot

_on docs/transformerless/FIXED_POINT_SCORING_SEMANTICS_V1.md:29_

This doc references an absolute CI-specific filesystem path, which won’t be meaningful to readers and will break if the repo is checked out elsewhere. Prefer a repo-relative path (optionally as a Markdown link).


---
_Comment/review ids in this file: 4775962311, 3647542735, 3647542767_

