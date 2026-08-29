# uor-r4-core

The shared mathematics and route-native substrate for UOR-R4’s research toward
a transformerless local AI engine.

**Current status.** `prime_route_attention`,
`prime_route_geometric_attention`, and `spiralcore_operator` provide canonical
prime/semiprime routes, spin and torsion state, Hopf projection with a retained
fiber, golden-radial structure, exact transport controls, and bounded route
operators. They are the retained address/frame/transport substrate, not a
semantic language model. The active build is the stop-first
`PredictiveConnectionRetentionGate0V1` in
[ADR-0005](../../docs/adr/0005-predictive-geometric-connection-memory.md):
construction-validation of candidate-discriminative current/previous/last-two/
full-prefix exact-route relations. A positive authorizes learned keys/values/
queries, exact R4/Spin transport, multiscale retention, key-specific delta
updates, and candidate-relative readout. Neither is yet qualified. The
authoritative sequence is the
[Geometric Intelligence Programme](../../docs/geometric_intelligence_programme.md).

This crate also preserves the earlier teacher-compiled TLA/R4G1 engine as a
research lane. That engine established useful artifact, deterministic-runtime,
and multiplication-free kernel work, but it is not the current route-native
intelligence architecture and is not evidence of transformerless chat.

The crate therefore hosts three related bodies of work:

1. **The R⁴ math layer** (`src/lib.rs`, `src/zeta_zeros.rs`): zeta-zero
   embeddings, Hopf coordinates, prime/QIMC identity, and state metrics used by
   the geometric router (`uor-r4-router`).
2. **The prime-route and predictive-memory substrate**
   (`src/prime_route_attention.rs`,
   `src/prime_route_geometric_attention.rs`, `src/spiralcore_operator.rs`):
   canonical route identities, recursive geometric state operators, and the
   construction seams reused by the active Gate 0 and connection-memory
   programme.
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
