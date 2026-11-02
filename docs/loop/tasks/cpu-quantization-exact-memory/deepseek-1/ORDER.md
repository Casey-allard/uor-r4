# ORDER: exact Hamming-plus-memory session restart, deepseek-1

Lab: deepseek. Item: [cpu-quantization-exact-memory](../../../../roadmap/items/cpu-quantization-exact-memory.md) (#3503). Base: `454a90031`.

## Goal

Persist and restore the complete dynamic CPU FBS session for the supported Hamming-read and turn-memory path, then prove that continuation and reset after restore match an uninterrupted session exactly.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-quantization-exact-memory/deepseek-1/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-quantization-exact-memory.md`
- `docs/harness/claims.json`

No serving-format or serving-behavior edit, corpus run, GPU run, quantization-aware gradient, float-shadow comparison or language claim. Those remaining #3503 requirements stay explicit.

## Cancel probe

Stop if the CPU forward's dynamic state cannot be reconstructed from its immutable artifact plus an explicit session-state payload. Every mutable field is local to `CpuFbsForward` or its memory anchors, so the unit stands without changing serving code.

## Done when

- A versioned checkpoint binds the exact artifact SHA-256, shape, compute substrate and qualification label.
- It serializes every mutable forward field needed for continuation: position, recurrence state, convolution history, token history, Hamming payload caches, opening/reply memory anchors and reply-start position.
- Restore reparses the exact artifact, validates every nested array length, token, position and anchor against shape/context, and returns no partially constructed forward on refusal.
- A child-process restore on a combined Hamming-plus-memory artifact continues with the same next complete query, token, kept-cell vector and checkpoint bytes as the uninterrupted session.
- A restored session followed by `reset` matches a fresh session at every subsequent step, including reply signalling and the point-key fallback.
- Truncated, trailing, stale-artifact, wrong-digest, malformed shape, out-of-range token/anchor and inconsistent reply-start controls fail closed.
- Focused tests and the full release suite pass; clippy, formatting, D23/D25/D32 ratchets, local PR check and all loop gates pass.
- One REPORT and notebook row state that this closes only the saved-session restart part of #3503; quantization-aware updates and the float-shadow comparison remain open.
- A fresh exact-head adversarial review approves the change before merge-queue delivery.
