# uor-r4-core

The shared mathematics and route-native substrate for UOR-R4’s research toward
a transformerless local AI engine.

**Current status.** `prime_route_attention`,
`prime_route_geometric_attention`, and `spiralcore_operator` provide canonical
prime/semiprime routes, spin and torsion state, Hopf projection with a retained
fiber, golden-radial structure, exact transport controls, and bounded route
operators. They are the retained address/frame/transport substrate, not a
semantic language model. #973's subsequently qualified
[`R4RetainedLanguagePathV1`](../../docs/r4_retained_language_path_v1_973.md)
establishes one bounded, source-free, causally load-bearing retained-attention
path, not H4 superiority or prompt-coherent generation. Its direct and sole
layerwise-normalized parameter-free readout successors each improved prompt and
fresh-language metrics but missed both frozen capacity-gain floors. The latest
terminal is
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`, result CID
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`;
all `13/13` fresh-process verifier comparisons passed. The parameter-free
readout ladder is closed, and #973 must next freshly freeze a learned
associative binding/readout. Generation, reasoning, and lowering remain
`NOT_RUN`; #954 remains blocked. The bounded
`GeometricGatedDeltaRetentionR4V1` core
now implements separate learned K/V/Q, H4-frame transport, four retained banks,
and candidate-relative readout, but its sealed synthetic smoke was weaker than
plain delta. `DirectCausalGeometricAttentionR4V1` now implements the literal
offline Q/K/V/O, tangent-projected causal-softmax operator. Its V2 result is
non-promotable because of a raw-manifold-parameter mismatch. Fresh equal-manifold-budget V3
returned full H4 3/12, matched plain 12/12, current-only 6/12, and an
inference-time coherent alternative-connection swap 10/12. The current
reference in
[ADR-0005](../../docs/adr/0005-predictive-geometric-connection-memory.md) is
`HELM-D-R4`: V4 preserved construction covariance but failed held-out
functional binding, so #973 pins the HELM-D architecture, freezes an ordinary
decoder donor, and builds full-decoder gauge-equivalent softmax in exact
cumulative R4/Spin frames. Learned
Q/K/V, value aggregation, and `W_o` remain unchanged; K/V transport to the
query frame and map-back now retain bounded numerical and real-language behavior,
with exact replay, zero future reads, and a live frame-permutation control.
This ordinary softmax reference is `PASS`. Intrinsic Lorentz V1 attempt 02 then
stopped `UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` because its
construction barycenter covariance was `9.121400701417315e-08` against the
frozen `1e-08` ceiling; diagnostic curved NLL was worse than donor and flat, and
D3 remained sealed. Source-faithful learned-manifold V2 then failed donor
retention and matched Euclidean parity, and the 8/8-contract localization
attempt stopped at its two-document preflight and rejected tangent readout.
Ordinary dot-product/stable-softmax causal attention
in coherent R4/Spin frames is the accepted baseline. Provider-free autonomous
`R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) native CLI generation now passes
using the credited HELM attention seam and UOR's pinned SmolLM2
`HuggingFaceLlamaOracle` for embeddings, RoPE, residual/RMSNorm, MLP, final
normalization, and the language-model head. Its explicit opt-in native
HTTP/dashboard bridge is a completed source-backed reference surface, not the
active source-free research rung.
It remains transformer-compatible and `f32`/multiply/alloc/source-weight
backed—not yet a table-native, multiply-free, or transformerless serving
mechanism. Intrinsic/readout, resonance, replacement, recurrence, and lowering
are parked.
The
authoritative sequence is the
[Geometric Intelligence Programme](../../docs/geometric_intelligence_programme.md).
HELM-D pinned-source provenance, donor reproduction, and transported-R4 parity
are `PASS`; see
[`docs/helm_d_r4_softmax_decoder_973.md`](../../docs/helm_d_r4_softmax_decoder_973.md).
The intrinsic V1 record is
[`docs/intrinsic_lorentz_r4_attention_973.md`](../../docs/intrinsic_lorentz_r4_attention_973.md).
The localization preflight-result record is
[`docs/helm_d_score_centroid_localization_973.md`](../../docs/helm_d_score_centroid_localization_973.md).
The binding generator result and compact aggregate are
[`docs/r4_softmax_reference_generation_973.md`](../../docs/r4_softmax_reference_generation_973.md)
and
[`docs/r4_softmax_reference_generation_attempt_01_result_973.json`](../../docs/r4_softmax_reference_generation_attempt_01_result_973.json).
Upstream checkpoint parity remains `NOT_RUN`; intrinsic/readout, resonance
replacement, recurrence, and exact lowering are parked; #954 remains blocked.

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
   attention reference, the historical bounded gated-delta comparator, and the
   group-addressed geometry/export seams retained from #973's terminal cell.
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
