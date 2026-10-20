# #607 — model-source: add the first non-Llama adapter through the architecture-neutral compiler boundary

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:02:17Z
- **Updated:** 2026-08-13T03:34:04Z
- **Closed:** 2026-08-13T03:34:04Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/607
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The compiler boundary is nominally architecture-neutral (`TeacherOracle` exposes representation and behavior), but the only Hugging Face adapter is `HuggingFaceLlamaOracle`, whose tensor names and configuration assumptions are Llama-family-specific. One small non-Llama fixture is the cleanest test that the open-weight pipeline is not merely a SmolLM2 port.

# Current evidence

- `crates/uor-r4-model-source/src/lib.rs:802-840` defines the two-surface source interface, while `HuggingFaceLlamaOracle` at `:1012` and `append_layers`/`append_tensor` assume `model.layers.*` Llama tensor names.
- `crates/uor-r4-core/tests/smollm2_adapter.rs` and the model descriptors cover only SmolLM2/Llama-family HF sources.
- `crates/uor-r4-core/src/transformerless/hf_bpe.rs` currently supports a byte-level BPE path, so tokenizer compatibility must be checked rather than assumed.
- #599 establishes adapter conformance; #600 makes source/compiled geometry explicit; #601 versions tokenizer adapters. No open issue currently proves a non-Llama adapter through this boundary.

# Problem

There is no pinned, permissively licensed non-Llama decoder fixture proving that model-family tensor/configuration logic stops at the source adapter and does not leak into graph compilation, traces, packing, or runtime code.

# Goal

Add one small non-Llama source adapter and end-to-end canary through the architecture-neutral boundary, after selecting and recording an immutable source revision, license, tensor format, tokenizer, and practical local execution budget.

# Scope

- Select a small non-Llama decoder fixture from a permissively licensed source after a written selection check; do not assume a specific repository/revision in advance.
- Implement its config/tensor mapping, supported feature declarations, tokenizer adapter, source/compiled geometry, and `TeacherOracle` surfaces.
- Run source conformance, observation, compile, artifact load, teacher-forced parity, and runtime contract checks using existing pipeline stages.
- Record any architecture-specific logic inside `uor-r4-model-source`; downstream crates consume only current architecture-neutral interfaces.

# Non-goals

- Do not optimize for parameter scale or add a modern GQA model in the same issue.
- Do not change the current SmolLM2 fixtures or baseline.
- Do not bypass #599/#601 by accepting silent tensor/tokenizer approximations.
- Do not begin #589 upstream integrations.

# Dependencies and blockers

- Depends on #597, #598, #599, #600, and #601; consumes #603 for trace provenance and #606 for the certificate shape.
- #515’s feature/ledger conventions apply to any dormant adapter path; #531 owns corpus-scale selection; #589 remains a future upstream blocker.

# Proposed implementation shape

Use a small generated or downloaded pinned fixture for fast unit tests and a real source snapshot for the canary. Keep family-specific parsing in the model-source crate and expose only `RepresentationSource`/`BehaviorSource`/`TeacherOracle` plus declared metadata to compiler code.

# Acceptance criteria

- [ ] The selected source has a recorded immutable revision, license, complete source manifest, tokenizer identity, and local resource estimate.
- [ ] The adapter passes #599 conformance and rejects unsupported features explicitly.
- [ ] Observation, compile, artifact validation, and teacher-forced parity complete without architecture-specific code outside model-source.
- [ ] The same source/compiled geometry and tokenizer provenance are visible in reports.
- [ ] Existing SmolLM2 and legacy fixture outputs remain unchanged.

# Verification

- Adapter unit fixtures for config/tensor mapping, GQA/bias/activation/RoPE declarations as applicable, and tokenizer parity.
- End-to-end canary with deterministic double-run of observations/artifact and source/runtime certificate rows.
- Workspace fmt/clippy/tests, graph-format no-std checks, and the claim-wording checker.

# Compatibility and migration

This adds a new adapter and source/observation era; it does not change historical artifacts or the default model. The selected source manifest and all output CIDs must be retained with the canary report.

# Claim status

Implementation and empirical architecture-independence criterion. Passing the canary does not prove broad architecture equivalence.


## Comments (3)

### comment 5272768379 — 2026-08-12T21:03:19Z — Casey-allard

## Written selection check + plan (#607 — first non-Llama adapter)

### Selection check (recorded before implementation, per scope)

Requirements: small non-Llama decoder; permissive license; safetensors available; immutable revision; tokenizer compatible with the existing `hf_bpe` byte-level BPE path or explicitly declared unsupported; practical local budget.

| candidate | license | arch | safetensors | tokenizer | non-Llama axes | verdict |
|---|---|---|---|---|---|---|
| openai-community/gpt2 (124M) | MIT | GPT2LMHeadModel | ✅ | byte-level BPE (vocab.json+merges.txt+tokenizer.json) | learned absolute positions (NO RoPE), LayerNorm **with bias** (not RMSNorm), GELU (`gelu_new`, not gated SiLU), **fused Conv1D `c_attn` QKV** (not split q/k/v), `transformer.h.*` naming (not `model.layers.*`), tied embeddings | **SELECTED** |
| EleutherAI/pythia-70m | Apache-2.0 | GPTNeoXForCausalLM | ✅ | tokenizer.json only | partial rotary (rotary_pct 0.25 — still RoPE-family), parallel residual, fused `query_key_value` | runner-up |
| TinyStories/GPT-Neo family | mixed | GPT-Neo | varies | byte-level BPE | local/global attention alternation | rejected (license mix, exotic attention out of scope) |

**Selected: `openai-community/gpt2` @ revision `607a30d783dfa663caf39e06633721c8d4cfcd7e` (MIT).** Rationale: it differs from the Llama family on every axis the boundary-leak test cares about — positions, norm type+bias, activation, QKV packing, tensor naming, embedding tying — including the strongest case (no rotary anywhere), while pythia's partial-RoPE is closer to the existing family. Its tokenizer exercises the existing byte-level BPE path with the classic vocab/merges pair. Config (pinned): n_layer 12, n_head 12, n_embd 768, n_ctx 1024, vocab 50257, layer_norm_epsilon 1e-05, activation `gelu_new`. Resource estimate: f32 safetensors ≈ 0.5 GB disk, single-CPU teacher-forced observation on short pinned samples — within the 2-core/7GB dev budget; CI never downloads it (presence-gated, UNAVAILABLE with reason when absent, #599 three-state convention).

### Plan

1. **Adapter (all family-specific logic inside `uor-r4-model-source`):** GPT-2 config/tensor mapping (`transformer.h.*`, fused `c_attn` split into q/k/v views at the adapter boundary, Conv1D weight orientation handled at ingest — recorded, not silently transposed downstream), feature declarations through the #599 `AdapterFeatures` fail-closed surface (declares: absolute-learned-positions, layernorm-with-bias, gelu_new, fused-qkv, tied-embeddings; rejects unsupported combinations explicitly), tokenizer adapter through the #601 versioned surface (byte-level BPE, vocab+merges identity recorded), source/compiled geometry per #600, `TeacherOracle` (`RepresentationSource`/`BehaviorSource`) surfaces.
2. **Fixture strategy (per proposed shape):** small GENERATED pinned fixture (few-layer GPT-2-shaped safetensors with real naming/config schema, deterministic bytes) for fast unit tests; the REAL pinned snapshot for the end-to-end canary, presence-gated.
3. **Canary:** source conformance (#599 runner) → observation (#603 traces) → compile → artifact load → teacher-forced parity (existing Gate C/parity surfaces — same bars as the SmolLM2 fixtures, no new thresholds invented) → runtime contract checks → #606 certificate rows (source parity family populated; identity block carries the full source manifest κ, tokenizer identity, adapter, geometry, operator absent-as-absent where not applicable). Deterministic double-run of observations/artifact bytes.
4. **Leak test (the point of the issue):** the canary asserts no architecture-specific code outside model-source — enforced by compilation boundaries (downstream crates consume only the neutral interfaces) plus a source-scan check that `gpt2`/`c_attn`/`transformer.h` tokens appear only in `uor-r4-model-source` (and its tests/fixtures).
5. **Records:** immutable revision, license, complete #597 source manifest, tokenizer identity, resource estimate — in the adapter's descriptor + the delivery record. SmolLM2 fixtures and legacy outputs byte-unchanged (existing suites are the regression gate).
6. Ledger/docs: MODEL_LIFECYCLE #607 section; ledger lane only if a dormant path is added (expected: none — this is an adapter, not an operator; the #515 conventions apply if that changes).

**Claim discipline:** passing the canary is recorded as "this pinned GPT-2 source ran the pipeline end-to-end under the declared features" — never architecture-universality.


### comment 5275350142 — 2026-08-13T02:49:29Z — Casey-allard

PR #656 (branch `i607-gpt2-adapter`) delivers the first non-Llama source adapter.

**Landed:** generalized #599 conformance surface (NormKind/PositionKind/ConfigKeys, Llama byte-identical) + `huggingface_gpt2()`; `models/gpt2-124m.json` (κ `blake3:3bca1b7f…`, MIT); a faithful GPT-2 executor + `HuggingFaceGpt2Oracle` on the neutral `TeacherOracle` trait (LayerNorm-bias, learned positions, gelu_new, fused c_attn Conv1D, tied head; 768→288 #600 projection); a boundary leak test (0 GPT-2 tokens across 147 downstream files); docs.

**Validated against an INDEPENDENT numpy GPT-2 reference:** tiny generated fixture to worst |Δ| ~1e-8…1e-6; presence-gated real 124M canary to final-hidden |Δ| ~3e-5 with exact argmax + top-5 (UNAVAILABLE, never downloaded, in CI). Full workspace suite 1218 tests / 0 failed; register gates R1–R4 pass; workspace clippy clean; SmolLM2 unchanged.

**Claim discipline:** the pinned GPT-2 source runs the adapter→oracle→TeacherOracle boundary end-to-end under declared features, faithful to an independent reference — NOT architecture universality.

**Not in #656 — filed as #657:** emitting a *compiled* R4 artifact needs the compile path (graph-cli/graph-compiler/server, ~9 concrete-`HuggingFaceLlamaOracle` sites) dispatched over `TeacherOracle` by architecture, plus the full compile→#606-cert canary. Leaving #607 open for your call on whether it closes with #656 or tracks through #657.

### comment 5275627810 — 2026-08-13T03:34:03Z — Casey-allard

Closing: the adapter deliverable (PR #656) passed CI and is auto-merging via the queue. #656 lands the first non-Llama source adapter (faithful GPT-2 executor + HuggingFaceGpt2Oracle on the neutral TeacherOracle trait), the generalized #599 conformance surface, the boundary leak test, and docs — validated against an independent numpy reference (tiny ~1e-8, real 124M ~3e-5).

The remaining work to emit a *compiled* GPT-2 artifact — dispatching the graph-cli/compiler/server compile path over TeacherOracle by architecture, plus the #602 learned-absolute operator record, #603 GPT-2 trace capture, #601 tokenizer identity, and the full compile→#606-cert canary — is tracked in #657.


---
_Comment/review ids in this file: 5272768379, 5275350142, 5275627810_

