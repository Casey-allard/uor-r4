# #252 — fix(router): store content-derived state vectors in the angular corpus index (#245)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T04:07:37Z
- **Updated:** 2026-10-09T13:09:12Z
- **Closed:** 2026-07-28T04:17:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/252
- **Merged:** 2026-07-28T04:17:29Z
- **Merge commit:** ec7b4a8f80a1c0d6437892cbbf64e479067c523b
- **Base / Head:** main / UOR-Foundation:issue-245-content-bearing-store
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #245.

## What

Stored `CorpusItem.state_vector`s now derive from sentence **content** instead of capturing the session brain state at index time. The fix flows through the existing `state_vector: Option<&[f64]>` parameter of `route_query_to_manifold_internal` (the vocabulary lives on the router, not the geometry structs — so no geometry-trait change): a new `content_state_vector()` builds the L2-normalized sum of the sentence's known zeta-seeded vocab vectors, exactly as `evolve_brain_state` does for queries.

This revives the cosine term of `retrieve_geometric_resonance` (previously degenerate — retrieval was effectively shared-prime overlap only), per-sentence Hopf coordinates, and the semantic map — all already wired downstream.

## Tests (all impossible before the fix)

- Different sentences store different vectors.
- Same sentence stores an identical vector before/after unrelated session activity (session-independence within a router). A recorded caveat: vectors are **vocabulary-relative across routers** — word→prime assignment is arrival-ordered — noted for the future UOR-canonicalization discussion of this address space.
- A query with sentence A's content scores strictly closer (cosine) to A's stored vector than to unrelated B's.

## Scope

`uor-r4-router` only — exploratory crate, outside the graph migration path (AGENTS.md). Gates: all four green locally (macOS arm64, pinned 1.97.1, offline).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

