# #245 — Geometric store is content-free: reconnect sentence content to angular storage (two-edit revival)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:52:04Z
- **Updated:** 2026-07-28T04:17:30Z
- **Closed:** 2026-07-28T04:17:30Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/245
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The geometric router's angular **storage** is content-free while its angular **machinery** runs fine. Two call sites sever text from geometry:

1. `SpectralGeometry::ground` (`crates/uor-r4-router/src/geometry.rs:65-79`) ignores `object.content` entirely — it copies the session brain state.
2. `index_sentence_internal` (`crates/uor-r4-router/src/lib.rs:1473-1494`) routes every sentence with `state_vector=None` → the identity's *current* brain state, so every stored `CorpusItem.state_vector` encodes whatever the session state was at index time, independent of the sentence's words.

In the Python original (prime-router `server.py:1361`), each sentence was routed through the projection of **its own text** — stored vectors encoded content. The Rust port kept the scoring formula (`retrieve_geometric_resonance`, `lib.rs:1559-1652`, faithful to `server.py:1263-1322`) but the cosine term is degenerate on content-free vectors; retrieval effectively runs on shared-prime word overlap alone.

### Fix (two edits; downstream is already wired)
1. In `ground()`: when content is non-empty, build the state from the sentence's own tokens — the vocab-vector summation loop already exists at `lib.rs:1226-1245` (zeta-seeded word vectors, `uor-r4-core/src/lib.rs:356`).
2. In `index_sentence_internal`: pass that content-derived vector as `Some(state_vector)`.

Cosine resonance, per-sentence Hopf coordinates (χ/δ/α), and the semantic map all light up immediately — they are ported and connected, currently fed noise.

### DoD
- Test: two different sentences indexed under an identical session state store different `state_vector`s; same sentence indexed twice stores the same vector.
- Retrieval fixture: content-relevant sentence ranks above content-irrelevant one for a matching query (fails today).
- No graph-path impact: `uor-r4-router` is outside the graph migration path (AGENTS.md).

*Prerequisite for the session-signature lane (geometric state must carry content to be worth injecting into the graph engine). From the prime-router → uor-r4 lineage audit.*


## Comments (1)

### comment 5099805973 — 2026-07-28T04:04:57Z — Casey-allard

Taking this (assigned). Implementation deviates slightly from the issue's literal "edit ground()" — same effect, cleaner layering:

**Where the fix lives**: the vocabulary (zeta-seeded word vectors) lives on the router, not on the geometry structs, so content-derivation happens at the router level and flows through the *existing* `state_vector: Option<&[f64]>` parameter of `route_query_to_manifold_internal` — which `SpectralGeometry::ground` already consumes via `active_state`. No geometry-trait change needed.

- New `content_state_vector(&self, text) -> Option<Vec<f64>>`: L2-normalized sum of the text's known vocab vectors — the exact construction `evolve_brain_state` uses for queries, now applied to the sentence being indexed.
- `index_sentence_internal` routes each sentence through its own content state instead of `None` (= session-state capture). Falls back to session state only when no word is in vocab (rare — words are added just above).
- New `pub fn corpus_items_for(identity)` read-only accessor (test/inspection surface).

**Tests** (all impossible to pass before the fix): different sentences store different vectors; same sentence stores an identical vector across routers with different session histories (content-determinism); a query with sentence A's content scores strictly closer (cosine) to A's stored vector than to unrelated B's.

Branch `issue-245-content-bearing-store`. Router crate only — outside the graph migration path per AGENTS.md.



---
_Comment/review ids in this file: 5099805973_

