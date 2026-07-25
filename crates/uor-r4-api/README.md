# uor-r4-api

Typed library façade over the uor-r4 transformerless graph stack, for
downstream consumers (e.g. hologram-ai) that embed uor-r4 as a **library**
rather than driving its CLI.

## What it exposes

- **Typed compile** (`compile::compile`): one call from a verified local
  Hugging Face-style source directory (`config.json`, `tokenizer.json`,
  `*.safetensors`) to a `CompiledModel` — the scored deployable R4G1
  graph, the signature artifact, tokenizer, and both reports, all as
  owned bytes, plus provenance (options, format/contract versions,
  blake3 digests). Orchestrates the three existing compiler stages
  in-process: teacher bundle → multiresolution cover → scoring.
  Resumable: an incomplete teacher corpus returns
  `CompileOutcome::Incomplete` and re-calling with the same request
  resumes from the work-directory checkpoint (detected structurally from
  the corpus metadata done byte — the same gate the corpus loader
  applies — never by parsing stage stdout).
- **Typed engine** (`engine::R4Engine`): loads from byte slices only
  (`EngineParts`) — no filesystem access. The graph passes the format
  crate's two-stage structural validation and CID verification before
  any scorer state is built. The D4 manifest status policy is data
  (defaults + `score_report.json` override), abstention is a typed
  outcome (never a fabricated token), and the steady-state
  predict/generate step is allocation-free (scratch allocated once at
  `load`). Text helpers (`encode_text_into` / `decode_tokens_into`) ride
  the same engine via the bytes-based tokenizer.
- **Tokenizer from bytes** (`Tokenizer::from_bytes`, re-exported from
  `uor-r4-core`): parses the binary tokenizer.bin format in memory.

## Downstream contract

- Inputs are byte slices exactly as the compiler emitted them; outputs
  are typed structs/enums with focused error enums (`CompileError`,
  `LoadError`, `InferenceError`, all `std::error::Error`).
- `AbiVersion` (`R4Engine::abi_version`) reports the R4G1 format
  version, the normative inference operation contract version, and this
  crate's version — pin against it when persisting artifacts.
- Nothing here changes algorithms, scoring, or the kernel contract; this
  crate only wraps existing stages and the existing deployed adapter
  (moved from the root package's `src/r4g1.rs`, which is now a thin
  path-based wrapper over `engine`).

## Known shim

The stage entry points predate this crate and take CLI flag strings.
Building those flags from the typed request is a **private, temporary
compat shim** inside `compile` (the `stage_*_flags` functions); it goes
away when the stages grow typed entry points. No stage stdout is parsed.

## Testing

Unit tests run with the workspace (`cargo test --workspace --offline`).
An ignored end-to-end compile + load test exists for local use:

```sh
UOR_R4_API_E2E_SOURCE=/path/to/local/hf-source \
  cargo test -p uor-r4-api --release -- --ignored e2e --nocapture
```
