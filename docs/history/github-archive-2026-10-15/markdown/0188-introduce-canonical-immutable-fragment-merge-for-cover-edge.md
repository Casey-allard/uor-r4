# #188 — Introduce canonical immutable-fragment merge for cover edge discovery

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-25T04:22:59Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-07-25T05:13:46Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/188
- **Merged:** 2026-07-25T05:13:46Z
- **Merge commit:** b38606efab7aabf94d32b767b4fd406334f9cf57
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-uor-r4-extend-clustering
- **Reviews / inline review comments:** 1 / 2
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This change addresses the nondeterminism boundary in graph discovery by enforcing the required pattern: worker-local immutable discovery output followed by canonical sequential merge. It focuses the #171 slice on edge discovery topology stability (order/dedup invariance under fragment interleavings).

- **Canonical merge spine for discovery fragments**
  - Added `canonical_merge_edge_fragments` in `induction.rs`.
  - Merge semantics are explicit and deterministic: flatten fragments → sort by canonical key `(src, kind, dst)` → dedup.
  - Keeps topology-affecting canonicalization centralized and scheduling-independent.

- **`build_edges` refactor to fragment-first construction**
  - `build_edges` now accumulates three immutable fragments (`refinement`, `overlap`, `transition`) and merges them through the canonical merge helper.
  - Removes implicit dedup/order coupling from direct set insertion in the construction path.

- **Determinism coverage for merge semantics**
  - Added focused induction tests for:
    - canonical ordering + dedup correctness
    - output invariance under fragment permutation/interleaving
  - Added a BDD scenario to `parallel_reproducibility.feature` pinning fragment-interleaving invariance for edge output.

```rust
pub fn canonical_merge_edge_fragments(fragments: &[Vec<CoverEdge>]) -> Vec<CoverEdge> {
    let mut edges = Vec::with_capacity(fragments.iter().map(Vec::len).sum());
    for fragment in fragments {
        edges.extend(fragment.iter().copied());
    }
    edges.sort_by_key(|edge| (edge.src, edge.kind, edge.dst));
    edges.dedup();
    edges
}
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #171

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3649429860 — 2026-07-25T04:35:03Z — Copilot

_on features/suites/parallel_reproducibility.feature:15_

This new Cucumber scenario introduces three step strings that are not implemented in the step definitions (see `tests/bdd.rs`, which uses `fail_on_skipped()`). As written, the BDD runner will treat these steps as skipped/undefined and fail the suite. Add matching `#[given]`/`#[when]`/`#[then]` step handlers (ideally asserting `canonical_merge_edge_fragments` is invariant under fragment permutation) or remove/rename the scenario to reuse existing steps.

### review 4778419022 — 2026-07-25T04:35:04Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR targets determinism in cover-edge discovery by switching to a “worker-local immutable fragments → canonical sequential merge” pattern, centralizing topology-affecting ordering/dedup in a single helper.

**Changes:**
- Added `canonical_merge_edge_fragments` to flatten, canonically sort by `(src, kind, dst)`, and deduplicate edge fragments.
- Refactored `build_edges` to build three immutable fragments (`refinement`, `overlap`, `transition`) and merge them through the canonical helper.
- Added unit tests for canonical merge ordering/dedup and invariance under fragment permutation; added a new BDD scenario (currently missing step definitions).

### Reviewed changes

Copilot reviewed 3 out of 3 changed files in this pull request and generated 2 comments.

| File | Description |
| ---- | ----------- |
| features/suites/parallel_reproducibility.feature | Adds a new BDD scenario intended to pin fragment-interleaving invariance for cover-edge output. |
| crates/uor-r4-graph-compiler/tests/induction.rs | Adds unit tests validating canonical merge ordering/dedup and permutation invariance. |
| crates/uor-r4-graph-compiler/src/induction.rs | Introduces canonical merge helper and refactors `build_edges` to fragment-first construction + canonical merge. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3649429871 — 2026-07-25T04:35:04Z — Copilot

_on crates/uor-r4-graph-compiler/src/induction.rs:1554_

`peers` is populated symmetrically for each coactive pair (both `a -> b` and `b -> a`), but this loop canonicalizes `(src,dst)` and pushes an edge for both directions, creating duplicate neighbor edges that then have to be sorted and deduped later. Emitting the neighbor edge only once (e.g., only when `a < b`) avoids extra allocations and sort/dedup work while keeping the same canonical output.


---
_Comment/review ids in this file: 4778419022, 3649429860, 3649429871_

