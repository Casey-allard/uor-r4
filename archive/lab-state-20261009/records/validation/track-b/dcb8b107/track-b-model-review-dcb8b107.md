# Bounded independent Track B dense-path source review

Reviewer: Codex `/root/second_council_review`, delegated by `/root`.
Date: 2026-09-30.
Exact source head: `dcb8b107f499395c65b86404d67e9e83916054fd`.
Worktree: `/Users/casey.allard/.codex/worktrees/track-b-conversion/uor-r4`.
Scope: `track_b/model.rs`, `track_b/conversion.rs`, directly used loader/RoPE functions, model-source caller definitions at this revision, and locally pinned Candle 0.9.2 implementation. No broad history, infrastructure, model-quality or numerical qualification audit.

**Disposition: no concrete defect requiring source correction found in the requested masking, head mapping, RoPE positions or supported-checkpoint loading boundaries.** This is bounded source review, not an all-logit parity PASS or a claim that the 1e-4 gate will pass. Do not relax that threshold or advance B2 based on this receipt.

## Checks and source evidence

1. **Causal mask and layout.** `model.rs:368–388` validates B*T and token bounds, creates the flattened embedding rows and uses absolute query/key ranges `0..time`. `model.rs:552–572` masks exactly `key > query`, retaining the current token and every earlier token. `model.rs:391–395,425–443` consistently maps flattened [B*T,W] through [B,T,H,D] and back, preserving batch/time order. `model.rs:540–549` applies -infinity to future scores before softmax over keys. This agrees with the inclusive model-source causal prefix (`lib.rs:1755–1782`) and Candle's square full-prefill mask (`llama.rs:218–227`). Right padding cannot affect earlier positions under this causal rule; no general left-padding or variable-position API is promised.

2. **GQA mapping.** `model.rs:505–536` checks compatible native KV shapes and integral H/KV grouping. Its expansion is [B,KV,group,T,D] followed by contiguous flattening, so query head h uses KV head floor(h/group). That matches model-source `lib.rs:1615–1617,1759` and Candle `utils.rs:28–37` (concatenate repeated sequences within each KV head, then reshape). Q and O keep full width; K/V use KV_heads*head_dim. No head-swapping or reduced-width source projection was found.

3. **RoPE positions and convention.** `model.rs:380–407,482–499` generates frequencies using the same F32 theta/exponent construction and position-by-frequency matmul as pinned Candle `llama.rs:155–160,198–207`. Both Q and K use positions starting at zero for a fresh full prefix. Shared `kappa_llama.rs:542–553` rotates split halves `(a*cos-b*sin, b*cos+a*sin)`, matching the native Hugging Face oracle branch in model-source `lib.rs:1694–1729` and Candle's non-interleaved rotary path. Interleaved/scaled/partial RoPE configurations are rejected before loading by conversion config parsing. This establishes source convention alignment, not bitwise backend equivalence.

4. **Conversion cache traversal.** `conversion.rs:126–166` clones the empty cache for each call and advances each singleton at its true absolute position. Full prefill starts at zero; split prefill continues only with singleton cached inputs. This avoids Candle 0.9.2's square-mask limitation for a multi-token tail after an existing cache (`llama.rs:300–351`). The upstream model returns only the final input position (`llama.rs:505–514`), and the conversion wrapper collects all positions through singleton stepping. The shared model deliberately has no incremental-cache API and returns all full-prefix positions.

5. **Loader/shape compatibility.** Shared load applies conversion's configuration checks before invoking the Kappa loader (`model.rs:168–181`). Supported configuration is bias-free SiLU Llama, integral/even head geometry, unscaled split-half RoPE, consistent optional head_dim, finite positive RMS epsilon/theta and bounded context (`conversion.rs:182–255`). Both use the expected row-major HF projections and tie the vocabulary head to embedding when configured (`model.rs:218–270`, Candle `llama.rs:517–526`). Kappa's loader enforces exact tensor inventory/shapes and finite BF16/F32 decoding (`kappa_llama.rs:329–445`); BF16 conversion is an exact upper-bit expansion into F32. The conversion loader prevalidates required weights before entering upstream block construction (`conversion.rs:87–107,258–298`), whose pinned implementation otherwise unwraps a load error.

## Important qualification limits

- Shared RMS normalization, RoPE and softmax are differentiable primitive compositions (`model.rs:473–498,547–549`); stock Candle uses fused/inference operations, and the oracle uses its own exact-executor arithmetic. Shared full-prefix/batched matrix reductions differ from the singleton path. Small floating-point differences can accumulate. Whether every logit is within **1e-4** on **both CPU and Metal** remains an executed, artifact-bound question. No failure or pass is inferred from equivalent formulas.
- The strict shared loader supports the declared exact single-file BF16/F32 inventory. A different checkpoint with shards, extra tensors (including a redundantly serialized tied head), or F16 weights may be rejected even if another loader accepts it. This is a declared supported-input restriction, not evidence that the pinned SmolLM2 source is incompatible. I did not read or load the model artifact in this review.
- Full-prefix positions begin at zero; no cached multi-token continuation or arbitrary offset support should be inferred. Hook output shape/dtype checks do not establish arbitrary custom attention's causal correctness.
- The existing parity example retains fixed 49,152-vocabulary windows and per-position all-logit comparison at 1e-4. I inspected it for the caller scope; no report root or model was executed. Tiny deterministic tests and a successful build, including work being run by the parent, are not loaded full-model CPU/Metal parity or language qualification.

## Exact source identities

- `crates/uor-r4-training/src/track_b/model.rs`: `a9f518ca9f8703c96ab0033d768c574a4091aa729807a0af37beebf5d769a672`
- `crates/uor-r4-training/src/track_b/conversion.rs`: `d11e4fe980d67f99589e69ef42f6902d3bd9b7d4a8d1894fac4b4171bc33c3d4`
- `crates/uor-r4-training/src/kappa_llama.rs`: `11679e44690c7ab772ee2675f426061308ad9ea1a2753fa2c14b02f599672792`
- `crates/uor-r4-model-source/src/lib.rs`: `66c39e1ac98c71dd10d42c1f4853f8f18bbcb67d390bf63391ccfde28b3bf11b`
- `crates/uor-r4-training/examples/track-b-parity.rs`: `75628dffe8fd14f27fa86614cfa5468c78405ba233edf4b53965f98ad273cfc3`
- `Cargo.lock`: `5194d70bd8259baf071e56b21b541caafbfdd0e0c3ccc4f8373d516c90ea7c0f`
- Local pinned Candle `candle-transformers-0.9.2/src/models/llama.rs`: `b3f438e3e12449122c27fe6884dbdbe3246a1f551d4cdac10b67d5f11c70c446`. Cargo.lock pins candle-transformers 0.9.2. Also read this package's `utils.rs` and candle-nn 0.9.2 `rotary_emb.rs` for the called operators.

## Work and independence

Read-only source review, no Cargo/builds/tests/model runs, no edits to source, no GitHub writes, no runner work. Only this review artifact was written. Non-author of the reviewed Track B source. Used the UOR project workflow's source/evidence discipline; current owner numerical policy takes precedence over stale stricter wording in that skill. Historical memory was used only to locate the pinned parity context and its cached-tail pitfall; code and gate definitions were checked directly at the reviewed revision.
