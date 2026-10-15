# #657 — model-source: dispatch the compile path over TeacherOracle by architecture (enable GPT-2 compile)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-13T02:49:14Z
- **Updated:** 2026-08-14T11:07:52Z
- **Closed:** 2026-08-14T11:07:52Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/657
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #607 (PR #656). #656 landed the GPT-2 source adapter (`HuggingFaceGpt2Oracle`) behind the `TeacherOracle` two-surface trait, validated against an independent numpy reference (tiny ~1e-8, real 124M ~3e-5), all family logic confined to `uor-r4-model-source`. This issue tracks **every surface the adapter deliberately left absent** so a compiled, certified GPT-2 artifact becomes possible. None of these are silent gaps in #656 — each is enumerated here and referenced from the code/docs.

## 1. Compile-path dispatch over `TeacherOracle` (the enabler)

The compile entry points bind the **concrete** `HuggingFaceLlamaOracle` and its Llama-only inherent methods, not the trait:

- `crates/uor-r4-graph-cli/src/lib.rs`: ~234, 428, 476, 509, 2177, 2645
- `crates/uor-r4-graph-compiler/src/lib.rs`: ~394
- `src/server.rs`: ~248, 269

**Recommendation:** an architecture-keyed teacher factory in `uor-r4-model-source` (read `model_type`/`architectures`, return the right oracle keyed by the #599 declarations), plus generalizing the ~9 sites to `Box<dyn TeacherOracle>` or an enum. The currently-inherent capture/attention methods that callers use (`set_r4_attention`, `step_with_layer_capture`, `cfg`, `last_logits`) must be reconciled onto the trait or the factory enum. SmolLM2 byte-parity is the regression gate.

## 2. GPT-2 #603 trace-capture surface

`HuggingFaceGpt2Oracle` returns the safe trait defaults for `trace_capture_geometry` (`None`) and `step_with_trace_capture` (`false`) — so richer #603 trace profiles are **refused, never zero-filled**. GPT-2 needs a `forward_capturing_trace` equivalent (post-layer residual, q/k/v at pos, per-head attention weights) so observation can produce traces. Implement it **with** the compile pipeline (item 1) so the taps are exercised end-to-end rather than shipped untested.

## 3. GPT-2 #602 attention operator record (learned-absolute)

`attention_operator_spec` is deliberately `None`: the only registered operator, `standard-source-attention/1`, declares `positional_action = "rope-rotation-of-q-and-k-before-scoring"`, which GPT-2 does **not** do (learned absolute positions). Reusing it would be a false record. **Recommendation:** register a new versioned operator (e.g. `standard-source-attention` with a learned-absolute positional action, or a new id) in `attention.rs`, with the registry refusing unknown `(id, version)` as today, and wire `attention_operator_spec` to it. This feeds the #606 certificate's operator row.

## 4. GPT-2 #601 tokenizer identity

The oracle reports the generic `tokenizer_address = "huggingface-tokenizer"` (same as the Llama oracle). The #601 versioned surface should record the GPT-2 tokenizer identity distinctly — `hf-byte-bpe/1` family with the pinned `vocab.json` + `merges.txt` CID — so a compiled bundle binds the exact tokenizer, not just the id family.

## 5. Full compile→certify canary

Once items 1–4 exist, extend the presence-gated GPT-2 canary to the full pipeline: source conformance (#599) → observation (#603 traces) → compile → artifact load → teacher-forced Gate C parity → runtime contract → #606 certificate rows (source-parity family populated; operator/geometry/tokenizer identity carried). Deterministic double-run of observation/artifact bytes; serialize the heavy run.

## Acceptance

A pinned GPT-2 source compiles to an R4G1 artifact end-to-end under its declared features; SmolLM2 is byte-unchanged; the #606 certificate carries the GPT-2 source rows with a truthful operator record. Claim discipline unchanged: end-to-end pipeline success, not architecture universality.


## Comments (2)

### comment 5284357283 — 2026-08-13T17:49:29Z — Casey-allard

Item 1 (the architecture dispatch enabler) implemented in PR #671: a `Teacher` enum keyed on config.json model_type, with SmolLM2 byte-parity proven (Teacher::Llama bit-identical to HuggingFaceLlamaOracle — κ + all 49152 forward logits). Remaining items filed as sub-issues: #667 (item 2, #603 trace-capture + GPT-2 batched executor), #668 (item 3, learned-absolute attention operator), #669 (item 4, tokenizer identity), #670 (item 5, full compile→certify canary). Note: the batched observe path (observe_text_batched_command) stays Llama-only until #667 gives GPT-2 a batched executor (BatchedTeacher's State is the Llama batched state).

### comment 5292606415 — 2026-08-14T11:07:51Z — Casey-allard

All five enumerated items have landed; closing the tracker.

1. Compile-path dispatch over TeacherOracle (GPT-2 compiles end-to-end) — #671.
2. GPT-2 #603 trace-capture surface (`forward_capturing_trace` + real `trace_capture_geometry`/`step_with_trace_capture`) — landed.
3. GPT-2 #602 learned-absolute attention operator record — #672.
4. GPT-2 #601 tokenizer identity (`hf-byte-bpe/1`, distinct from the generic HF address) — #696 / PR #697.
5. Full compile→certify canary (compile → cover → R4G1 + #606 source-parity rows, byte-reproducible) — #670 (increments #693), hardened by #694/#695.

Acceptance met: a pinned GPT-2 source compiles to an R4G1 artifact end-to-end under its declared features; SmolLM2 stays byte-unchanged; the certificate carries truthful GPT-2 source rows (operator/geometry/κ) and the tokenizer identity is distinct.


---
_Comment/review ids in this file: 5284357283, 5292606415_

