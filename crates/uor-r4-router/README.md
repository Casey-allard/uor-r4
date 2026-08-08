# uor-r4-router

The R⁴ geometric text router and manifold web dashboard backend.

`UorR4Router` embeds words as 512-dimensional zeta-zero vectors, routes a
rolling "brain state" to one of 16 scale windows by norm, indexes sentences
into per-identity corpus manifolds, retrieves by prime-overlap + cosine
resonance, and generates with geometric Markov chains (bigram/trigram
transitions). It ships thought-stream physics for the browser dashboard
(`index.html`, `geometric_prime_router_webapp.html` at the repo root).

## ⚠ Retrieval ranking: read this before touching `retrieve_geometric_resonance`

**As deployed, retrieval on this path ranks by word overlap, not geometry.**
Relevance is `shared_count * DEFAULT_LEXICAL_WEIGHT + sim * slice_norm +
scope_boost`, and the geometric term's measured dynamic range is ~0.37 — so any
lexical weight above ~0.4 already yields strict lexicographic order with the
cosine as a within-bucket tie-break (#484).

The cosine contributed nothing at all until #486, which found the cause: the
query vector was built from the **routing** path while `index_sentence_internal`
stores a **content** vector. Different objects; their cosine was at chance
(self-similarity at the 0.4938 percentile, where 0.5 is chance — measured by
querying with the *exact stored sentence*).

`set_content_query_vector(true)` builds the query with the same
`content_state_vector` construction the stored side uses, which takes retrieval
from 0.7179 to **0.8542** MRR with the weight unchanged (0.8763 at weight zero),
recall 0.9720 → 0.9900. **It ships default OFF**: changing it changes retrieval
ordering, which moves the pinned #421 anchor-accuracy rows, so adoption is gated
(issue #490).

Measurement knobs, all default-off and non-serialized:
`set_lexical_weight`, `set_unscaled_geometric_term`, `set_content_query_vector`,
`set_full_width_query`, `set_banded_storage`.

Records: [`docs/geometry_selfmatch_486.md`](../../docs/geometry_selfmatch_486.md),
[`docs/lexical_weight_484.md`](../../docs/lexical_weight_484.md),
[`docs/query_projection_480.md`](../../docs/query_projection_480.md).

**Gotchas that have cost real measurements.** Retrieval is *identity-scoped* —
query under the same identity `index_corpus` used, or the index is empty and
every arm scores zero. `ResonanceResult::window_index` is a routing-window id
shared by several stored sentences, **not** a store index; match ground truth on
the stored sentence. `slice_norm` is per-window-bucket, so zeroing the lexical
weight alone does *not* give a cosine ranking.

## Status and relationship to the graph compiler

This crate is **f64, floating-point, and allocates freely by design** — it is
the exploratory geometric system, not the proof-carrying one. The R⁴
holographic graph compiler plan (`docs/r4_graph_compiler_implementation_plan.md`
§3.3) deliberately leaves it untouched: the transformerless engine
(`uor-r4-core::transformerless`) and the R4G1 graph artifacts are the path to
the mul-free, allocation-free runtime contract, and this router's word-Markov
generator survives only as a documented fallback for `r4 chat` when no
compiled store is bound.

- **FallbackRouter Pipeline**: `FallbackRouter` (`src/fallback.rs`) manages dynamic engine cascades from primary `r4g1-graph` to secondary `transformerless-tla5` fallback upon encountering `EngineStatus::UnmappedRegion` or `EngineStatus::Pathological` statuses, returning valid response streams without dropping HTTP/WS payloads.

## API surface

- **wasm-bindgen API** (`#[wasm_bindgen]`, for the dashboard): `new`,
  `index_default_corpus`, `calculate_resonance`, `route_query_to_manifold`,
  `index_sentence`/`index_corpus`, `generate_geometric_response`,
  `get_top_resonances`, `compile_thought`/`inject_thought_stream`,
  `update_drift_physics`/`execute_zkp_phase_reset`,
  `route_query_to_manifold_uor` (UOR trace steps + κ payload),
  `export_state`/`import_state`.
- **Native mirror API** (`*_native`): same operations for the local server
  (`src/server.rs` in the root package) without wasm overhead.
- **UOR witness layer**: `R4Axis` (640-byte packed query → 28-byte metrics
  output, via `uor-foundation-sdk::axis!`), `R4RoutingInput/Output`,
  `UorR4RouterModel` (`PrismModel` → `Grounded` certificates with derivation
  replay), thread-local `ACTIVE_ROUTER`, `R4HostBounds`.
- **State**: serde-serializable `UorR4Router` (streams, vocabulary,
  word_primes, transitions, corpus_index_by_identity, session_brain_states);
  `manifold_cache_rust.json` at the repo root is a state dump.

## Key types

`ThoughtStream`, `CorpusItem`, `GeometricResponse`, `RoutingData`,
`RoutedResult`, `MetricsResult`, `QimcResult`, `HopfResult`,
`TrajectoryStep`, `QuantumMetrics`, `ResonanceInfo`.

## Dependencies

`uor-r4-core` (R⁴ math), `uor-addr` + `uor-foundation` + `uor-foundation-sdk`
(pinned git deps — content addressing and the proof substrate), wasm-bindgen +
serde-wasm-bindgen for the browser build, serde/serde_json, sha2, blake3.
