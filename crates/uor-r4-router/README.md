# uor-r4-router

The R⁴ geometric text router and manifold web dashboard backend.

`UorR4Router` embeds words as 512-dimensional zeta-zero vectors, routes a
rolling "brain state" to one of 16 scale windows by norm, indexes sentences
into per-identity corpus manifolds, retrieves by prime-overlap + cosine
resonance, and generates with geometric Markov chains (bigram/trigram
transitions). It ships thought-stream physics for the browser dashboard
(`index.html`, `geometric_prime_router_webapp.html` at the repo root).

## Status and relationship to the graph compiler

This crate is **f64, floating-point, and allocates freely by design** — it is
the exploratory geometric system, not the proof-carrying one. The R⁴
holographic graph compiler plan (`docs/r4_graph_compiler_implementation_plan.md`
§3.3) deliberately leaves it untouched: the transformerless engine
(`uor-r4-core::transformerless`) and the R4G1 graph artifacts are the path to
the mul-free, allocation-free runtime contract, and this router's word-Markov
generator survives only as a documented fallback for `r4 chat` when no
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
