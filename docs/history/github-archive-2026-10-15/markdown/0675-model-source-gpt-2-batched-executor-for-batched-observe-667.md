# #675 — model-source: GPT-2 batched executor for batched observe (#667 item 2b)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T21:09:23Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-08-13T21:28:56Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/675
- **Merged:** 2026-08-13T21:28:55Z
- **Merge commit:** 4f8fb88d571768e0b3f489415cf1137860aa47d6
- **Base / Head:** main / UOR-Foundation:i667-gpt2-batched
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second half of #657 item 2. **Closes #667** (item 2a landed in #674). Stacked on #674, rebased onto main after it merged.

## Problem
`observe_text_corpus_batched` ran through `BatchedTeacher`, whose `forward_batch_into(&self, states: &mut [State], …)` **hardcoded Llama's concrete `State`**. GPT-2 has its own `Gpt2State`, so batched compile-corpus observation was Llama-only; GPT-2 had no batched executor.

## Change
- **`BatchedTeacher` generalized** with an associated `type State` plus `reset_state` / `logits_mut` accessors, so the driver no longer names a concrete state. The Llama impl is behaviorally unchanged.
- **`Gpt2::forward_batch`** — advances N independent sequences one position each. The four projection Conv1Ds run once over the batch via a new **`conv1d_batched`** (each weight row read once across the batch, with the *same* per-output accumulation order and bit-neutral zero-skip as the serial `conv1d`), and the tied lm-head reuses each `wte` row across the batch (naive sequential dot). The per-sequence attention is the shared **`block_attention`** factored out of `block_forward`, so serial and batched are **bit-identical by construction**.
- **`impl BatchedTeacher for HuggingFaceGpt2Oracle`** (`State = Gpt2State`).
- **`observe_text_corpus_batched` is now generic** over `T: BatchedTeacher`; the graph-cli caller **dispatches Llama vs GPT-2 on the `Teacher` enum**, so a GPT-2 source's compile-corpus observation uses the batched path.

## Regression gates
- `forward_batch_matches_serial_forward` — GPT-2 batched == serial `forward` **bit-for-bit** (`to_bits`), for a lockstep batch (all at the same position) and a divergent batch (each sequence at a different position, own k/v cache).
- `batched_matches_serial_byte_for_byte` (graph-compiler) — the Llama batched **driver** still produces byte-identical records to the serial driver through the generalized trait.
- Tiny + real numpy GPT-2 parity and the #603 trace tests unchanged.

## Verification (local, pre-push)
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clean)
- `cargo build --target wasm32-unknown-unknown --lib` (clean)
- `cargo test -p uor-r4-model-source --lib` (68 pass), `-p uor-r4-graph-compiler` (all pass incl the byte-for-byte batched test), `-p uor-r4-graph-cli`
- Register conformance R1–R5. No new dependencies.

With this, #657's item 2 is complete; only item 5 (the full compile→certify canary, #670) remains before the umbrella #657 closes.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

