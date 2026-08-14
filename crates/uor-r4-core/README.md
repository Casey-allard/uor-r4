# uor-r4-core

R⁴ mathematics and the transformerless compiler/runtime engine for the R⁴
holographic graph compiler.

This crate hosts two things:

1. **The R⁴ math layer** (`src/lib.rs`, `src/zeta_zeros.rs`): zeta-zero
   embeddings, Hopf coordinates, prime/QIMC identity, and state metrics used by
   the geometric router (`uor-r4-router`).
2. **The transformerless engine** (`src/transformerless/`): cross-compiles a
   pinned Hugging Face teacher into a multiplication-free, table-native
   inference artifact (TLA3-TLA7 containers; TLA7 is the default emission), plus the integer-only
   runtime that serves it. This is the system the graph compiler
   (`uor-r4-graph-format` and friends) generalizes — see
   `docs/r4_graph_compiler_implementation_plan.md`.

## Transformerless module map (selected)

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

## Runtime contract (normative)

Per-token inference uses only XOR/AND/OR/shift/popcount/integer add/compare/
table reads. No multiplication or division exists in the runtime kernel
(machine-checked source scan in `transformerless/mod.rs` witnesses P-1…P-4).
The prediction hot path is allocation-free in steady state (asserted by
`tests/allocation_census.rs`). Context sliding beyond `WINDOW = 8` performs zero-allocation
slice truncation `[t-7..t]` and emits `tracing::warn!`. Store parsing is strictly 32-bit (`parse_store_strict_u32`). Compiler and certifier are offline and may use
floats, matmul, and allocation; the runtime may not.
The boundary and allowed/forbidden operation classes are normatively versioned
in `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`.

## Testing

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
