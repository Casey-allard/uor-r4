# uor-r4-core

R⁴ mathematics and the transformerless compiler/runtime engine for the R⁴
holographic graph compiler.

This crate hosts two things:

1. **The R⁴ math layer** (`src/lib.rs`, `src/zeta_zeros.rs`): zeta-zero
   embeddings, Hopf coordinates, prime/QIMC identity, and state metrics used by
   the geometric router (`uor-r4-router`).
2. **The transformerless engine** (`src/transformerless/`): cross-compiles a
   pinned Hugging Face teacher into a multiplication-free, table-native
   inference artifact (TLA3/TLA4/TLA5 containers), plus the integer-only
   runtime that serves it. This is the system the graph compiler
   (`uor-r4-graph-format` and friends) generalizes — see
   `docs/r4_graph_compiler_implementation_plan.md`.

## Transformerless module map (selected)

| Module | Role |
|---|---|
| `teacher` | `TeacherOracle` two-surface trait (embedding + next-token oracle); llama-family safetensors adapter, optional trace surface |
| `compiler` | Corpus pipeline, deterministic projection + sampled RVQ codebooks, thresholds, class signatures, TLA container emit/parse, span/byte-anchored observation records |
| `runtime` | Mul-free integer kernel (`OpKernel` with op census, no multiply method), sign signatures, Hamming assignment, graded evidence store (TLS1), bounded top‑M membership, allocation-free generation |
| `runtime_state` | Fixed-capacity multi-timescale state: live token state + reserved local/segment/session levels with Phase-8 update hooks |
| `reference_state` | Reference `ActiveFrontier` + checked packed edge-range resolvers |
| `transitions` | Forward semantic transitions + reverse indexes (Theorem 7 consistency) |
| `convert_r4g1` | Migration converter: TLA/TLS1 artifacts → canonical R4G1 containers |
| `observe` | Observation pipeline v2: content-addressed sample IDs, deterministic shard spill/resume, `observe` CLI |
| `cover` | Multiresolution cover induction: spherical k-means, entropy-justified splits, calibrated radii, refinement/neighbor edges, R4G1 emission |
| `score` | Phase-4 compiler: E_f transitions + reverse indexes, root priors + parent-relative emission residuals, scored R4G1 emission, Gate C harness |
| `score_runtime` | Integer-only reference scorer (ScoreQ accumulation, no float/mul), bounded witness records + independent replay verifier; portable to wasm32 |
| `certify` / `compare` | Teacher-fidelity certification and runtime comparison |
| `certificate` / `performance_certificate` | Certificate schema (CIDs, claims, attestation) and bytes-read/cache/branch performance certificates |
| `score_q` | `ScoreQ` Q16.16 fixed-point log-domain scores (mul-free add/sub) |
| `resolution_status` | Supported / Boundary / BackedOff / Novel / Contradictory status |
| `anti_degeneracy` | Semantic anti-degeneracy transformations and evaluation harness |
| `predictive_sufficiency` | Rate-distortion / predictive-sufficiency reports by graph depth |
| `fairness_provenance` | Bias amplification, rare-group erasure, provenance-deletion support |
| `graph_patch` | Immutable content-addressed patch epochs and route translation |
| `shortlist_evaluator` | Shortlist top‑M recall measurement vs the reference classifier |
| `scenarios` | ChatML prompt wrappers (`format_instruct_chat_prompt`, `encode_chat_prompt`), byte-level BPE tokenizer export, scenario suite |
| `command` | `r4 transformerless …` CLI dispatch |

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
- `tests/convert_r4g1.rs`, `tests/observation_anchors.rs`, `tests/observe.rs`,
  `tests/transitions_test.rs`, … — feature suites

## Layout notes

`std` throughout; `wasm32` cfgs gate the native-only modules (fs, teacher,
compiler CLI). External UOR standards come in as pinned **git** dependencies
(see the root `Cargo.toml`); no code under `uor_standards/` is required to
build.
