# #600 — compiler: make source and compiled geometry explicit and version the 288-dimensional projection

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T21:58:34Z
- **Updated:** 2026-08-12T10:44:37Z
- **Closed:** 2026-08-12T10:44:37Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/600
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Open-weight source models have native widths that differ from the table-native R4 geometry. The current compiler already makes a lossy projection, but the projection is hidden behind the generic `TeacherOracle::dim()` contract and is not independently identified in the artifact provenance.

# Current evidence

- `crates/uor-r4-core/src/transformerless/compiler.rs` defines the legacy compiler geometry `D = 288` and uses it throughout cover/signature construction.
- `crates/uor-r4-model-source/src/lib.rs:1255-1332` reports `source_dimension()` from the HF config but returns `288` from `TeacherOracle::dim()` and averages source embedding buckets into the requested output width in `embedding`.
- The pinned 135M descriptor records `hidden_size: 576`, while the R4 compiler consumes 288 dimensions; the adapter therefore changes geometry before cover induction.
- The hidden-state lane in merged PR #95 rejected non-288 source widths rather than silently projecting them, demonstrating that geometry affects existing measurement semantics.
- `crates/uor-r4-core/tests/kappa_reproduction.rs` and era-sensitive fixture tests make artifact reproducibility a compatibility concern.

# Problem

The native source width, compiled width, projection algorithm, and projection identity are not an explicit stable boundary. A geometry change can silently alter observations and artifact κs.

# Goal

Represent source and compiled geometry explicitly, version the projection, and bind its implementation/configuration to the compiled provenance while preserving historical artifact readers.

# Scope

- Introduce repository-native geometry/projection metadata at the compiler boundary; do not duplicate `TeacherOracle` with a second source interface.
- Make the current 576→288 bucket-average projection an explicitly named/versioned implementation.
- Record source width, compiled width, projection ID/version, parameters, and implementation digest in compile/observation/artifact provenance.
- Add deterministic projection tests, non-divisible-width tests, and migration documentation.
- Provide a compatibility path for historical 288-dimensional artifact eras.

# Non-goals

- Do not select a new target width or re-pin artifacts in this issue.
- Do not change runtime arithmetic, packed layout, or source-model family support.
- Do not claim that projection preserves transformer semantics; measure it as a compiler transform.

# Dependencies and blockers

- Depends on #597 for source snapshot identity and #599 for source-adapter conformance.
- Existing artifact era/reproducibility fixtures must be reviewed before changing defaults.
- #515/#531 remain orthogonal owners; #589 blocks upstream integration.

# Proposed implementation shape

Keep `source_dimension()` and compiled geometry as separate typed metadata. Have the compiler request an explicit projection from a versioned geometry registry, and include that record in observation/compile provenance. Historical readers accept absent legacy metadata under an explicit legacy-era interpretation.

# Acceptance criteria

- [ ] A compile report distinguishes native source width from compiled R4 width.
- [ ] The current projection is named, versioned, deterministic, and tested for widths divisible and non-divisible by the target.
- [ ] Projection ID/version and parameters affect the content identity of new artifacts.
- [ ] Historical artifacts remain readable and their existing κ fixtures are unchanged.
- [ ] A double compile with the same source snapshot and projection produces identical bytes.

# Verification

- Unit tests for projection boundaries, empty/short buckets, non-divisible dimensions, and canonical metadata serialization.
- Deterministic rebuild and artifact provenance tests; run the existing κ/rebuild suites without re-pinning.
- `cargo fmt --check`, workspace clippy/tests, graph-format no-std checks, and the claim-wording checker.

# Compatibility and migration

This is a provenance and boundary change with possible artifact-κ impact. New metadata must be additive or version-gated; any default projection change requires a separately reviewed migration and maintainer-approved fixture re-pin with era notes.

# Claim status

Definition and implementation, with deterministic empirical criteria. It is not a quality or equivalence guarantee.


## Comments (3)

### comment 5264815337 — 2026-08-12T09:21:27Z — Casey-allard

## Plan

1. **Typed geometry metadata at the compiler boundary** (no second TeacherOracle): `source_width` (native, e.g. 576) and `compiled_width` (R4 288) as separate typed values; the current bucket-average becomes the named, versioned projection `bucket-average/1` with parameters (source→target widths, bucket layout for non-divisible cases) + an implementation digest.
2. **Versioned projection registry** the compiler requests from; deterministic; tested for divisible (576→288) and non-divisible widths (incl. empty/short trailing buckets).
3. **Provenance binding**: projection ID/version/params/digest recorded in compile report + observation manifest (serde(default) additive fields) and into NEW artifacts' content identity via version-gating — while historical artifacts remain readable under an explicit legacy-era interpretation and **existing κ/rebuild fixtures stay green without re-pin** (the issue's own compatibility constraint; era fixtures reviewed before any default change — and no default changes in this issue).
4. Deterministic double-compile test (same snapshot + projection ⇒ identical bytes); migration doc section.

The 'affects content identity' vs 'fixtures unchanged' tension is resolved by version-gating, not by mutating the legacy era — if implementation shows both can't hold simultaneously on some surface, that lands here as an honest decision point rather than a silent choice. Six gates + full suite before the PR.

### comment 5265286815 — 2026-08-12T10:07:47Z — Casey-allard

## Delivery record (PR #636 in queue)

Acceptance: **(1)** compile report distinguishes native source width (576) from compiled R4 width (288) via the typed `GeometryProjection` record. **(2)** Projection named+versioned (`bucket-average/1`), deterministic, factored into a free function with arithmetic order untouched, tested for divisible (576→288), non-divisible (577/700→288, remainder policy pinned), empty/short buckets, plus double-projection bit determinism. **(3)** Projection ID/version/params/digest affect the content identity of the provenance sidecars (report + observation manifest); **container-byte binding measured impossible additively** — filed as https://github.com/UOR-Foundation/uor-r4/issues/637 (era decision, maintainer re-pin per this issue's own compatibility clause) rather than chosen silently. **(4)** Historical artifacts readable; **all κ/rebuild/era fixtures green without re-pin** — deterministic_rebuild --include-ignored reproduces the pinned TLA container byte-identically. **(5)** Double compile identical bytes ✓. Registry refuses unknown (id,version) via sanctioned error surface (R5); implementation digest = blake3 over the pinned parameter declaration, not source text (rationale in module docs); migration note: absent metadata = legacy-era *interpretation*, never relabeling. Six gates green. Closes on #636 merge; #601 (tokenizer adapters) next.

### comment 5265656732 — 2026-08-12T10:44:37Z — Casey-allard

PR #636 merged — delivered per the record (comment 5265286815). The era/container-bytes decision is tracked in #637 (Alex/Ari governance + Casey re-pin). Closing; #601 begins now.


---
_Comment/review ids in this file: 5264815337, 5265286815, 5265656732_

