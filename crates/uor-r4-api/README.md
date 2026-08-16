# uor-r4-api

Typed library façade over the uor-r4 transformerless graph stack, for
downstream consumers (e.g. hologram-ai) that embed uor-r4 as a **library**
rather than driving its CLI.

## What it exposes

- **Typed compile** (`compile::compile`): one call from a verified local
  Hugging Face-style source directory (`config.json`, the selected
  tokenizer definition, and `*.safetensors`) to a `CompiledModel` — the
  scored deployable R4G1 graph, the signature artifact, tokenizer, and
  both reports, all as owned bytes, plus provenance (options,
  format/contract versions, component digests, and the complete tokenizer
  adapter identity). A byte-level BPE selection consumes `tokenizer.json`;
  a SentencePiece/Unigram selection consumes the raw `spiece.model` rather
  than silently substituting wrapper semantics. Orchestrates the three
  existing compiler stages in-process: teacher bundle → multiresolution
  cover → scoring.
  Resumable: an incomplete teacher corpus returns
  `CompileOutcome::Incomplete` and re-calling with the same request
  resumes from the work-directory checkpoint (detected structurally from
  the corpus metadata done byte — the same gate the corpus loader
  applies — never by parsing stage stdout).
  `CompiledModel::source_execution_identity()` registry-validates the
  attention/dense pair persisted in `compile_report.json`;
  `attention_operator()` and `dense_operator()` are nonbreaking convenience
  accessors. Dense absence remains `None` for Llama and historical reports.
- **Typed engine** (`engine::R4Engine`): loads from byte slices only
  (`EngineParts`) — no filesystem access. The graph passes the format
  crate's two-stage structural validation and CID verification before
  any scorer state is built. The D4 manifest status policy is data
  (defaults + `score_report.json` override), abstention is a typed
  outcome (never a fabricated token), and the steady-state
  predict/generate step is allocation-free (scratch allocated once at
  `load`). Text helpers (`encode_text_into` / `decode_tokens_into`) ride
  the same engine via the bytes-based tokenizer. A tagged tokenizer that
  declares itself decode-only returns `None` from `encode_text_into`; its
  decode table remains exact, graph-CID verified, and available through
  `decode_tokens_into`. `tokenizer_adapter_identity` exposes the tagged
  family, version, raw-definition CID, and adapter digest so a caller can
  attach only the exact matching registered host encoder.
- **Tokenizer from bytes** (`Tokenizer::from_bytes`, re-exported from
  `uor-r4-core`): parses both historical and tagged binary tokenizer.bin
  formats in memory.

## Explicit tokenizer selection

`CompileRequest::tokenizer_adapter` is a required
`TokenizerAdapterKey { family, version }`. This is an intentional API break
introduced by issue #718: typed callers must choose the registered adapter as
one atomic pair, even when today's source directory happens to contain only one
supported tokenizer definition. An unsupported or non-unique selection, or a
missing selected definition, fails before any compiler stage is started; there
is no implicit preference or legacy fallback. A source containing both
`tokenizer.json` and `spiece.model` is accepted only for the definition named by
the request.

```rust
use std::path::PathBuf;
use uor_r4_api::{CompileOptions, CompileRequest, TokenizerAdapterKey};

let request = CompileRequest {
    source_dir: PathBuf::from("/models/source"),
    work_dir: PathBuf::from("/models/work"),
    tokenizer_adapter: TokenizerAdapterKey::hf_byte_bpe_v1(),
    options: CompileOptions::default(),
    source_manifest_kappa: None,
};
```

For a raw SentencePiece source, the current reference-correct adapter is
`TokenizerAdapterKey::new("sentencepiece-unigram", 2)`. Version 1 is frozen for
historical, explicit-only reproduction; callers must name it deliberately when
rebuilding an artifact that used those semantics. The typed request forwards
the family and version together to stage A.

On completion, `CompiledModel::provenance.tokenizer_adapter` is the full
validated `TokenizerAdapter` record: family, version, CID of the exact raw
definition, declared policy, and adapter digest. That raw-definition CID is
distinct from `provenance.digests.tokenizer`, which addresses the emitted
runtime `tokenizer.bin` bytes.

Source arithmetic is execution provenance, not a compile knob. The API derives
the teacher's registered `attention_operator` and optional `dense_operator`
from the source adapter, forwards both through Stage A and cover, and does not
add a `CompileOptions` selector that could mislabel model-produced rows.

## Downstream contract

- Inputs are byte slices exactly as the compiler emitted them; outputs
  are typed structs/enums. Source validation, stage, and engine-load failures
  return the re-exported `SourceUnavailable`; inference preserves the graph
  runtime's typed bounds and abstention outcomes.
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
