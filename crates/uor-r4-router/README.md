# uor-r4-router

The R⁴ geometric memory/router and manifold dashboard backend.

**Research component, not a complete chat engine.** Its stored trajectories,
identity-scoped memory, prime-overlap retrieval, resonance, Hopf state, and
session geometry are inputs to the active route-native architecture. Its
historical Markov generator, retrieval ranker, and dashboard demonstrations do
not implement candidate-conditioned language attention or establish source-free
coherent generation. The #973 offline `HELM-D-R4` reference preserves a frozen
ordinary donor's learned Q/K/V, causal softmax, value aggregation, and `W_o`
while representing head blocks in exact cumulative R4/Spin frames and
transporting K/V into the query frame. Numerical and real-language behavioral
parity now passes in `uor-r4-core`, establishing ordinary softmax attention in
R4/Spin frames. Intrinsic Lorentz V1 attempt 02 stopped unavailable before D3
on its construction covariance audit. Learned-manifold V2 later completed as a
valid non-D3 construction-validation negative: learned Lorentz failed donor
retention and matched Euclidean parity although all destructive controls
separated. The active successor is the frozen 8/8 score-by-readout localization;
this router
supplies substrate and
visualization only; it is not the donor decoder or attention implementation. See
[ADR-0005](../../docs/adr/0005-predictive-geometric-connection-memory.md) and the
[Geometric Intelligence Programme](../../docs/geometric_intelligence_programme.md).
The binding parity record is
[`docs/helm_d_r4_softmax_decoder_973.md`](../../docs/helm_d_r4_softmax_decoder_973.md).
The V2 result is recorded in
[`docs/helm_d_learned_manifold_r4_construction_973.md`](../../docs/helm_d_learned_manifold_r4_construction_973.md).
This router contributes no result to either record; resonance, recurrence,
exact lowering, #954, and serving work remain blocked here.

`UorR4Router` embeds words as 512-dimensional zeta-zero vectors, routes a
rolling "brain state" to one of 16 scale windows by norm, indexes sentences
into per-identity corpus manifolds, retrieves by prime-overlap + cosine
resonance, and retains a historical geometric Markov baseline (bigram/trigram
transitions). It ships thought-stream visualization for the browser dashboard
(`index.html`, `geometric_prime_router_webapp.html` at the repo root).

The active [geometric intelligence programme](../../docs/geometric_intelligence_programme.md)
may reuse the identity state, content-bearing memory, retrieval, R⁴/Hopf math,
persistence, and turn writeback as substrate after its owning gates. It does not
promote the Markov generator or
existing retrieval score as attention, and it removes hash-derived thought
streams from intelligence-critical token selection.

## ⚠ Retrieval ranking: read this before touching `retrieve_geometric_resonance`

The deployed content-query path defaults to geometric cosine ranking with
lexical weight zero. The older routing-query path retains its historical
lexical weighting for comparison.

The cosine contributed nothing at all until #486, which found the cause: the
query vector was built from the **routing** path while `index_sentence_internal`
stores a **content** vector. Different objects; their cosine was at chance
(self-similarity at the 0.4938 percentile, where 0.5 is chance — measured by
querying with the *exact stored sentence*).

`set_content_query_vector(true)` builds the query with the same
`content_state_vector` construction the stored side uses, which takes retrieval
from 0.7179 to **0.8542** MRR with the weight unchanged (0.8763 at weight zero),
recall 0.9720 → 0.9900. It is the current content-query default (#490/#502).

Measurement overrides are non-serialized. `content_query_vector` defaults on;
the pre-#490 routing-query path is selected explicitly with
`set_content_query_vector(false)`. The other experimental projection/storage
overrides default off:
`set_lexical_weight`, `set_unscaled_geometric_term`, `set_full_width_query`,
and `set_banded_storage`.

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

This crate is **f64, floating-point, and allocates freely by design**. That is
allowed in the experimental compiler/research lane. Its memory/router state is
an input candidate for the new connection-memory prototype; it is not yet a
promoted product input. Its word-Markov generator remains a historical
baseline being replaced.
The historical TLA/R4G1 crates retain their separate multiplication-free and
allocation-free runtime contracts.

- **Historical `FallbackRouter`:** `src/fallback.rs` retains cascade
  machinery for explicit research use, but current production has no silent
  TLA fallback and the geometric decoder does not route through this cascade.

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
  word primes, corpus entries, session brain states, and persistence fields).
  Transition maps are rebuilt from stored corpus sentences on import;
  `manifold_cache_rust.json` at the repo root is a state dump.

## Key types

`ThoughtStream`, `CorpusItem`, `GeometricResponse`, `RoutingData`,
`RoutedResult`, `MetricsResult`, `QimcResult`, `HopfResult`,
`TrajectoryStep`, `QuantumMetrics`, `ResonanceInfo`.

## Dependencies

`uor-r4-core` (R⁴ math), `uor-addr` + `uor-foundation` + `uor-foundation-sdk`
(pinned git deps — content addressing and the proof substrate), wasm-bindgen +
serde-wasm-bindgen for the browser build, serde/serde_json, sha2, blake3.
