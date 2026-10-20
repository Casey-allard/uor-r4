# #656 — feat(#607): first non-Llama source adapter — faithful GPT-2 behind the neutral boundary

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T02:48:41Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-13T05:34:18Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/656
- **Merged:** 2026-08-13T05:34:18Z
- **Merge commit:** 775206bae6269f216259334889dbb08d9c0d0248
- **Base / Head:** main / UOR-Foundation:i607-gpt2-adapter
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #607. Adds the first genuinely non-Llama source adapter, proving the architecture-neutral compiler boundary generalizes. (Compile-path dispatch is a scoped follow-up — see below — so this does not auto-close #607.)

## Summary

Pinned source `openai-community/gpt2` (revision `607a30d783dfa663caf39e06633721c8d4cfcd7e`, MIT, source κ `blake3:3bca1b7f…`). GPT-2 is non-Llama on every axis the boundary cares about: LayerNorm **with a learned bias** (not RMSNorm), **learned absolute** positions (`wpe`, no RoPE), `gelu_new`, a **fused `c_attn`** Conv1D QKV projection, bare `wte`/`wpe`/`h.<l>.*`/`ln_f` tensor names, and tied embeddings.

## What's included

- **Generalized #599 conformance surface** — additive `NormKind` (RMSNorm vs LayerNorm), `PositionKind` (RoPE vs learned-absolute), and `ConfigKeys` (per-family `config.json` spellings), plus `AdapterFeatures::huggingface_gpt2()`. The Llama declaration and behaviour are **byte-unchanged**.
- **`models/gpt2-124m.json`** descriptor with authoritative source identity.
- **Faithful GPT-2 executor** (`crates/uor-r4-model-source/src/gpt2.rs`) loading through the existing #598 ingest boundary + #599 gate, and **`HuggingFaceGpt2Oracle`** on the neutral `TeacherOracle` two-surface trait (source width 768, compiled 288 via the #600 `bucket-average/1` projection). All family logic confined to the adapter crate.
- **Boundary leak test** — 0 GPT-2 tokens across 147 downstream source files.
- **Docs** — `MODEL_LIFECYCLE.md` architecture-family section.

## Verification (measured, against an INDEPENDENT numpy GPT-2 reference)

- **Tiny generated fixture** (`scripts/gen_gpt2_tiny_fixture.py`): every per-layer residual, the final hidden state, and the full logit vector reproduced to worst |Δ| ≈ 1e-8…1e-6 (tolerances 2e-3/3e-3) across 3 prompts.
- **Real 124M canary** (`scripts/gen_gpt2_real_golden.py`), **presence-gated**: with the 548 MB snapshot present, final-hidden |Δ| ≈ 3e-5 and exact argmax + top-5 token-set agreement; absent (CI) it reports UNAVAILABLE and is a no-op success — never a silent skip, never a download.
- **Full workspace suite: 1218 tests, 0 failed.** Register gates R1/R2/R3/R4 pass (R6/`cargo deny` is CI-only and dependency-neutral — no deps added). Workspace `clippy` clean.

## Breaking changes

None. Existing SmolLM2 fixtures and the entire suite pass unmodified.

## Claim discipline

Established: the pinned GPT-2 source runs the adapter → oracle → `TeacherOracle` boundary end-to-end under its declared features, faithfully to an independent reference, with all family logic confined to the adapter crate. This is **not** a claim of architecture universality.

## Follow-up (not in this PR)

Emitting a *compiled* R4 artifact from a GPT-2 source: the `graph-cli`/`graph-compiler`/`server` compile entry points currently bind the concrete `HuggingFaceLlamaOracle` type (~9 sites) and its Llama-only inherent methods, rather than dispatching over `TeacherOracle` by architecture. Wiring GPT-2 through `compile`/`observe`/`score` needs generalizing those call sites to the trait plus an architecture-keyed teacher factory. Documented in `MODEL_LIFECYCLE.md`; recommend a tracked follow-up issue.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

