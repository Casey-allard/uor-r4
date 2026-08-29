# uor-r4-core

The shared mathematics and route-native substrate for UOR-R4’s research toward
a transformerless local AI engine.

**Current status.** `prime_route_attention`,
`prime_route_geometric_attention`, and `spiralcore_operator` provide canonical
prime/semiprime routes, spin and torsion state, Hopf projection with a retained
fiber, golden-radial structure, exact transport controls, and bounded route
operators. They are the retained address/frame/transport substrate, not a
semantic language model. The bounded `GeometricGatedDeltaRetentionR4V1` core
now implements separate learned K/V/Q, H4-frame transport, four retained banks,
and candidate-relative readout, but its sealed synthetic smoke was weaker than
plain delta. `DirectCausalGeometricAttentionR4V1` now implements the literal
offline Q/K/V/O, tangent-projected causal-softmax operator. Its V2 result is
non-promotable because of a raw-manifold-parameter mismatch. Fresh equal-manifold-budget V3
returned full H4 3/12, matched plain 12/12, current-only 6/12, and an
inference-time coherent alternative-connection swap 10/12. The active build in
[ADR-0005](../../docs/adr/0005-predictive-geometric-connection-memory.md) is
now `HELM-D-R4`: V4 preserved construction covariance but failed held-out
functional binding, so #973 pins the HELM-D architecture, freezes an ordinary
decoder donor, and builds full-decoder gauge-equivalent softmax in exact
cumulative R4/Spin frames. Learned
Q/K/V, value aggregation, and `W_o` remain unchanged; K/V transport to the
query frame and map-back now retain bounded numerical and real-language behavior,
with exact replay, zero future reads, and a live frame-permutation control.
A trained intrinsic R4 distance/centroid operator is now the active rung, followed
conditionally by the multi-resonance sieve and bounded recurrence. None is yet
a serving or transformerless language mechanism. The
authoritative sequence is the
[Geometric Intelligence Programme](../../docs/geometric_intelligence_programme.md).
HELM-D pinned-source provenance, donor reproduction, and transported-R4 parity
are `PASS`; see
[`docs/helm_d_r4_softmax_decoder_973.md`](../../docs/helm_d_r4_softmax_decoder_973.md).
Upstream checkpoint parity, the intrinsic arm, resonance replacement, and
recurrence/lowering evidence remain `NOT_RUN`.

This crate also preserves the earlier teacher-compiled TLA/R4G1 engine as a
research lane. That engine established useful artifact, deterministic-runtime,
and multiplication-free kernel work, but it is not the current route-native
intelligence architecture and is not evidence of transformerless chat.

The crate therefore hosts three related bodies of work:

1. **The R⁴ math layer** (`src/lib.rs`, `src/zeta_zeros.rs`): zeta-zero
   embeddings, Hopf coordinates, prime/QIMC identity, and state metrics used by
   the geometric router (`uor-r4-router`).
2. **The prime-route and geometric-attention substrate**
   (`src/prime_route_attention.rs`,
   `src/prime_route_geometric_attention.rs`, `src/spiralcore_operator.rs`):
   canonical route identities, recursive geometric state operators, direct
   attention reference, bounded gated-delta candidate, and the construction
   seams reused by the active #973 programme.
3. **The historical transformerless engine** (`src/transformerless/`): cross-compiles a
   pinned Hugging Face teacher into a multiplication-free, table-native
   inference artifact (TLA3-TLA7 containers; TLA7 is the default emission), plus the integer-only
   runtime that serves it. This is the system the graph compiler
   (`uor-r4-graph-format` and friends) generalizes — see
   `docs/r4_graph_compiler_implementation_plan.md`.

## Historical TLA/R4G1 module map (selected)

| Module | Role |
|---|---|
| `compiler` | Corpus pipeline, deterministic projection + sampled RVQ codebooks, thresholds, class signatures, TLA container emit/parse, span/byte-anchored observation records |
| `runtime` | Mul-free integer kernel (`OpKernel` with op census, no multiply method), sign signatures, Hamming assignment, graded evidence store (TLS1), bounded top‑M membership, allocation-free generation |
| `reference_state` | Reference `ActiveFrontier` + checked packed edge-range resolvers |
| `transitions` | Forward semantic transitions + reverse indexes (Theorem 7 consistency) |
| `convert_r4g1` | Migration converter: TLA/TLS1 artifacts → canonical R4G1 containers |
| `score_q` | `ScoreQ` Q16.16 fixed-point log-domain scores (mul-free add/sub) |
| `resolution_status` | Supported / Boundary / BackedOff / Novel / Contradictory status |
| `graph_patch` | Immutable content-addressed patch epochs and route translation |
| `scenarios` | ChatML prompt wrappers (`format_instruct_chat_prompt`, `encode_chat_prompt`), historical and tagged decode-only runtime-tokenizer export/parse, scenario suite |
| `cd_space` / `endomorphism` / `lie_jordan` / `bott_fock` | Dormant Furey-plan substrate modules (see `docs/r4_furey_quantum_geometric_plan.md`, dated deferral record) |

Modules that started here and moved out in the crate split:

| Former module | Current home |
|---|---|
| `teacher` (`TeacherOracle`, llama-family adapters) | `uor-r4-model-source` |
| `runtime_state` (fixed-capacity multi-timescale state) | `uor-r4-graph-runtime::runtime_state` |
| `observe` / `cover` (observation pipeline, cover induction) | `uor-r4-graph-compiler` (`observation`, `induction`) |
| `score` (Phase-4 compiler + Gate C harness), `score_runtime` (integer-only reference scorer), `certify` / `compare`, `certificate` / `performance_certificate`, `anti_degeneracy`, `predictive_sufficiency`, `fairness_provenance`, `shortlist_evaluator` | `uor-r4-graph-certify` |
| `command` (`r4 transformerless …` CLI dispatch) | `uor-r4-graph-cli` |

## Historical TLA/R4G1 runtime contract

Per-token inference uses only XOR/AND/OR/shift/popcount/integer add/compare/
table reads. No multiplication or division exists in the runtime kernel
(machine-checked source scan in `transformerless/mod.rs` witnesses P-1…P-4).
The prediction hot path is allocation-free in steady state (asserted by
`tests/allocation_census.rs`). Context sliding beyond `WINDOW = 8` performs zero-allocation
slice truncation `[t-7..t]` and emits `tracing::warn!`. Store parsing is strictly 32-bit (`parse_store_strict_u32`). Compiler and certifier are offline and may use
floats, matmul, and allocation; the runtime may not.
The boundary and allowed/forbidden operation classes are normatively versioned
in `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`.

## Dormant verification inventory

These suites document the earlier engine’s contracts. Repository-wide QA is
dormant during mechanism development and is run only when an active issue names
the smallest check required to make its next decision; release QA returns at the
release stage.

- `tests/window_paths.rs` — container round-trips, window/corpus path equality,
  prediction witnesses (byte-identity gates)
- `tests/kappa_reproduction.rs` — full-compile κ-reproduction (ignored by
  default; needs the stories15M checkpoint; re-pin helper `dump_baseline_kappa`)
- `tests/allocation_census.rs` — allocation + op census on real artifacts
- `tests/deterministic_rebuild_test.rs` — Gate E deterministic rebuild slice
- `tests/convert_r4g1.rs`, `tests/graph_patch_test.rs`,
  `tests/transitions_test.rs`, … — feature suites (the observation-pipeline
  suites moved to `uor-r4-graph-compiler` with the crate split)

## Layout notes

`std` throughout; `wasm32` cfgs gate the native-only modules (fs, teacher,
compiler CLI). External UOR standards come in as pinned **git** dependencies
(see the root `Cargo.toml`); no code under `uor_standards/` is required to
build.
