# #671 — model-source: dispatch the compile path over TeacherOracle by architecture (#657 item 1)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T17:49:11Z
- **Updated:** 2026-10-09T14:16:49Z
- **Closed:** 2026-08-13T18:07:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/671
- **Merged:** 2026-08-13T18:07:55Z
- **Merge commit:** a80cea570ff993a331a7f41ab8be7af6b4e5ed53
- **Base / Head:** main / UOR-Foundation:i657-teacher-dispatch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Delivers **item 1** of #657 — the architecture dispatch enabler. The compile and serving paths bound the concrete `HuggingFaceLlamaOracle`, so only the Llama family could be compiled. This introduces `Teacher`, an architecture-keyed enum that carries either family and dispatches on the source's `config.json` `model_type`, so a GPT-2 source is loaded and compiled through the same path — with the Llama/SmolLM2 path **byte-unchanged**.

## Design — enum, not `Box<dyn TeacherOracle>`
`Teacher { Llama(HuggingFaceLlamaOracle), Gpt2(HuggingFaceGpt2Oracle) }` implements the full `TeacherOracle` surface (RepresentationSource + BehaviorSource + TeacherOracle) by delegating to the held variant via `as_oracle()`/`as_oracle_mut()`. An enum is required because the oracles' **inherent** surfaces diverge — `cfg()` returns `&Config` for Llama and `&Gpt2Config` for GPT-2, different types that cannot share one trait method — so a caller matches on the variant for architecture-specific needs while the shared surface is delegated.

- **Factory:** `Teacher::load` / `load_with_sequence_length` read `config.json` `model_type` and construct the right oracle, keyed by the #599 conformance family declarations: absent or `llama` → Llama (the pre-#657 default), `gpt2` → GPT-2, any other label **fails closed** with `SourceUnavailable` rather than being approximated by a family that does not execute it.
- **`set_r4_attention`:** delegates for Llama; a documented no-op for GPT-2 (its learned-absolute operator is the separate #668 registry entry, not the Llama r4 switch).
- Family-specific policy stays confined to `uor-r4-model-source` (the architecture-neutral boundary).

## Migrated (all TeacherOracle trait-surface sites)
graph-cli (`observe_command`, `observe_text_command` + worker pool, `compile_hugging_face_with_progress`), graph-compiler (`observe`), and the server (oracle slot `Arc<Mutex<Option<Teacher>>>`, loader, handler signatures).

**Deliberately not migrated:** `observe_text_batched_command` scores through `BatchedTeacher`, whose per-sequence `State` **is** the Llama batched state; GPT-2 has no batched executor yet (that is **#667**, item 2), so this path stays Llama-only and is documented as such rather than approximated.

## Verification (measured)
- **SmolLM2 byte-parity (the regression gate):** a presence-gated test proves loading SmolLM2 through `Teacher` is behaviorally identical to the concrete oracle — routes to `Teacher::Llama`, same source κ (`blake3:12d2cd8…`), same vocab/dim/seq_len, and a forward step from BOS yields **all 49,152 logits bit-identical**. The compile is deterministic, so identical oracle behavior ⇒ byte-identical artifacts.
- **Routing unit test** (CI, no weights): absent/`llama` → Llama, `gpt2` → GPT-2, unregistered label → error.
- **Full local gate set, all green:** `fmt --check`, `clippy --workspace --all-targets --all-features -D warnings`, **R5 `audit-limits`**, `wasm32 --lib`, `cargo test --workspace`, `check-model` (R1), `audit-deferral` (R4), `repo-conformance` (R2/R3), `cargo deny` (R6).

## Scope / follow-ups
This is the dispatch enabler only; #657's end-to-end GPT-2 compile acceptance also needs items 2–5, now filed as sub-issues: **#667** (GPT-2 #603 trace-capture + batched executor), **#668** (GPT-2 learned-absolute attention operator), **#669** (GPT-2 tokenizer identity), **#670** (full compile→certify canary). #657 stays open as the umbrella.

Refs #657

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

