# #12 — Fixed-width newtypes; eliminate semantic usize at serialization boundaries

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-21T21:24:04Z
- **Updated:** 2026-07-22T19:46:47Z
- **Closed:** 2026-07-22T19:46:02Z
- **Labels:** graph-compiler
- **Milestone:** Phase 1: Packed graph format and trusted parser
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/12
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Introduce NodeId, SectionOffset, TokenId, ScoreQ, Depth, Radius, CID newtypes (PDF §20-21). Known violations: Prediction.depth (crates/uor-r4-core/src/transformerless/runtime.rs:369), router result structs. Plan §9.2

## Comments (2)

### comment 5040373440 — 2026-07-22T00:12:55Z — auser

Progress (commit e8a0367, local main pending push):

Done:
- Newtypes landed in crates/uor-r4-graph-format (NodeId, SectionOffset, TokenId, ScoreQ, Depth, Radius, ArtifactCid) per the RFC.
- Prediction.depth is now fixed-width u8 at the deployed serialization boundary (was usize at runtime.rs:369); construction sites, TlessAxis record, and consumers updated. 24 suites green, clippy -D warnings clean.

Remaining before close:
- Router result structs (RoutedResult.active_range, QimcResult.prime/index, HopfResult.sector_id, expert_active_counts) still carry usize across the wasm/JSON boundary. Deliberately deferred: that boundary is shared with the web dashboard and needs a dashboard-side check, not just a type change.

### comment 5050719248 — 2026-07-22T19:46:47Z — auser

Completed in e3ddd7d (pushed to main). All serde-serialized fields now fixed-width: router result structs, router state maps (word_primes, max_prime, corpus_index*, expert_active_counts), facet store, sparse wire repr, and UorAttestationResult.hash_algorithm_id — all u64. JSON wire shape byte-identical with exact-JSON regression tests; pre-change manifold_cache_rust.json imports fine. get_expert_counts deliberately keeps its usize wasm ABI (Vec<u64> would switch the JS dashboard to BigInt elements). Follow-ups filed: #62 (export_state facet-store bug), #63 (wasm32 build break).


---
_Comment/review ids in this file: 5040373440, 5050719248_

