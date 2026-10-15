# #40 — Add reference ActiveFrontier and checked packed edge-range model

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-21T23:13:13Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-07-21T23:57:28Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/40
- **Merged:** 2026-07-21T23:57:28Z
- **Merge commit:** 011c9406bffe116495d23e720f0df335721cbcb4
- **Base / Head:** main / UOR-Foundation:copilot/reference-activefrontier-packed-edge-ranges
- **Reviews / inline review comments:** 1 / 1
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This change introduces a reference runtime state model for bounded frontier tracking and packed section-relative edge ranges, aligning the implementation with the issue’s Phase 2/5 graph-runtime contract. It adds explicit fixed-capacity frontier semantics and checked refinement/overlap range resolution primitives.

- **Reference frontier model**
  - Added `ScoreQ` (`i32`) as the fixed-point score type for frontier entries.
  - Added `ActiveFrontierEntry { region_id, score_q, margin, depth }`.
  - Added `ActiveFrontier<const CAP: usize>` with fixed-capacity push semantics (`push` returns `false` when full), plus `len/is_empty/as_slice/clear`.

- **Packed edge range model**
  - Added `PackedEdgeRanges` with section-relative fields:
    - `refinement_start/refinement_len`
    - `overlap_start/overlap_len`
  - Added checked resolvers:
    - `refinement_range(edge_count) -> Option<Range<usize>>`
    - `overlap_range(edge_count) -> Option<Range<usize>>`
  - Range resolution uses checked conversion/addition and rejects out-of-bounds spans.

- **Module surface**
  - Exposed new reference types from `transformerless::mod`:
    - `ActiveFrontier`, `ActiveFrontierEntry`, `PackedEdgeRanges`, `ScoreQ`.

- **Focused unit coverage**
  - Added tests for:
    - frontier capacity enforcement and field preservation
    - valid vs invalid packed range resolution

```rust
let mut frontier = ActiveFrontier::<2>::default();
assert!(frontier.push(ActiveFrontierEntry { region_id: 3, score_q: 42, margin: 7, depth: 1 }));
assert!(frontier.push(ActiveFrontierEntry { region_id: 5, score_q: 21, margin: -3, depth: 2 }));
assert!(!frontier.push(ActiveFrontierEntry::default())); // capacity reached

let ranges = PackedEdgeRanges { refinement_start: 2, refinement_len: 3, overlap_start: 7, overlap_len: 2 };
assert_eq!(ranges.refinement_range(10), Some(2..5));
assert_eq!(ranges.overlap_range(10), Some(7..9));
```

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 4749751285 — 2026-07-21T23:22:28Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR adds a small “reference state” model under `transformerless` to formalize (a) a fixed-capacity active frontier for bounded refinement, and (b) packed, section-relative edge subranges with checked resolution into `Range<usize>`.

**Changes:**
- Added `reference_state` module defining `ScoreQ`, `ActiveFrontierEntry`, and `ActiveFrontier<const CAP: usize>` with fixed-capacity push semantics.
- Added `PackedEdgeRanges` with checked `refinement_range` / `overlap_range` resolvers plus unit tests for capacity enforcement and range validation.
- Re-exported these reference types from `transformerless::mod`.

### Reviewed changes

Copilot reviewed 2 out of 2 changed files in this pull request and generated 1 comment.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/src/transformerless/reference_state.rs | Introduces reference frontier + packed edge-range model, including unit tests. |
| crates/uor-r4-core/src/transformerless/mod.rs | Exposes the new reference types and registers the new module. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626354967 — 2026-07-21T23:22:28Z — Copilot

_on crates/uor-r4-core/src/transformerless/reference_state.rs:49_

`ActiveFrontier::clear` only resets `len`, leaving previously-pushed entries intact. Because the type derives `PartialEq`/`Eq` (and `Debug`), stale data in the unused tail can make two logically-empty frontiers compare unequal and show confusing debug output after `clear()`.

Consider zeroing the previously-used prefix when clearing so the struct’s derived traits reflect the logical state.


---
_Comment/review ids in this file: 4749751285, 3626354967_

