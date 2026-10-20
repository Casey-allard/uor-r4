# #674 — model-source: GPT-2 #603 trace-capture surface (#667 item 2a)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T20:42:49Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-08-13T21:03:12Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/674
- **Merged:** 2026-08-13T21:03:12Z
- **Merge commit:** 20641d04f576b32859a3618e61dff71fe0ff3937
- **Base / Head:** main / UOR-Foundation:i667-gpt2-trace-capture
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #657 (item 2, first half). Refs #667 — does **not** close it (the GPT-2 batched executor, item 2b, is a separate PR).

## Problem
`HuggingFaceGpt2Oracle` returned the safe trait defaults — `trace_capture_geometry() == None`, `step_with_trace_capture() == false` — so the richer #603 trace profiles (post-layer residual, q/k/v, per-head attention weights) were **refused** for GPT-2, never zero-filled. This adds the capture path, mirroring Llama's `forward_capturing_trace`.

## Change (all in `crates/uor-r4-model-source/src/gpt2.rs`)
- **`Gpt2::forward_capturing_trace`** — one forward step that emits, for the `TraceCaptureRequest` layers, the post-block residual stream, the current-position q/k/v, and the per-head softmax attention weights, through the exact executor path (the taps read intermediates, they never recompute).
- **`block_forward`** gained optional `request`/`sinks` params (`None` on the plain hot path). It now normalizes the per-head softmax weights in place — `score * inv` is the exact per-position weight the value aggregation already applied, so the executor arithmetic stays **bit-identical**, and `scores` then holds the weights the attention tap emits. The committed tiny-fixture numpy parity test (`tiny_gpt2_matches_numpy_reference`) is the drift guard.
- The final `ln_f` + tied lm-head are factored into a shared **`finish_forward`** used by both `forward` and `forward_capturing_trace`.
- **`trace_capture_geometry`** → `{layers, heads, kv_heads == heads (plain MHA), residual_width = n_embd}`; **`step_with_trace_capture`** drives `forward_capturing_trace`, copies logits, returns `true`.

## Test
`forward_capturing_trace_matches_plain_forward_and_bounds_taps`: over a teacher-forced sequence, the traced step's logits are bit-identical (`to_bits`) to the plain forward at every position, and the residual/qkv/attention sinks fire **only** for the requested layers with the declared shapes — residual `n_embd`; q/k/v each `n_embd` wide; per-head attention weights spanning `0..=pos` and summing to 1.

## Regression gate
The plain `forward` hot path passes `None`/`None` (no tap overhead) and is bit-identical; the numpy tiny + real parity tests are unchanged and pass. Llama and other oracles are untouched. The trace-capture consumers (`observation.rs`, `route_fit.rs`) now accept GPT-2 (previously refused) — the intended enablement.

## Verification (local, pre-push)
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clean)
- `cargo build --target wasm32-unknown-unknown --lib` (clean)
- `cargo test -p uor-r4-model-source --lib` — 67 pass (incl tiny + real numpy parity + the new trace test)
- Register conformance R1–R5. No new dependencies.

Item 2b (GPT-2 batched executor generalizing `BatchedTeacher` for `observe_text_batched`) follows as its own PR, then #667 closes.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

