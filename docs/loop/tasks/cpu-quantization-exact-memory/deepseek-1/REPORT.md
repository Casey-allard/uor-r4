# REPORT: exact Hamming-plus-memory session restart, deepseek-1

Lab: deepseek. Item: [cpu-quantization-exact-memory](../../../../roadmap/items/cpu-quantization-exact-memory.md) (#3503). Base: `454a90031`.

## What did not work, first

- The CPU FBS forward had exact Hamming and turn-memory execution but no saved-session state. A caller could reset or rebuild the tables, but could not leave one process and continue the same recurrence, Hamming cache, opening anchors and reply stream in another.
- The existing checkpoint controls covered trainers and the discrete proposal applier, not `CpuFbsForward`'s mutable arrays.
- A checkpoint that trusted only its own shape booleans could accept stale Hamming caches or impossible memory anchors. The restore path therefore validates every dynamic array against a freshly parsed artifact before construction.
- The checkpoint is canonical JSON for transparent finite fixtures, not a compact production format. It is caller-managed storage and does not provide atomic file replacement or crash recovery.

## What changed

- Added versioned `CpuFbsForward::checkpoint_bytes` and `CpuFbsForward::resume`.
- The checkpoint binds the exact artifact SHA-256 to the complete dynamic state:
  recurrence state, convolution history, token history, Hamming payload caches,
  opening/reply memory anchors and reply-start position.
- `resume` reparses the exact artifact, rejects schema/substrate/qualification or hash changes, validates state and history dimensions, tokens, Hamming-cache shapes, future-cache zeros, derived memory anchors and reply position, then constructs no partial forward on refusal.
- Hamming cache shape validation uses the parsed read inventory rather than trusting serialized lengths.
- Added a real child-process restore control on a combined Hamming-plus-turn-memory artifact with point-key fallback.
- Added a reset control proving that restored and fresh sessions produce identical subsequent steps and checkpoint bytes.
- Added a second-reply control proving that a pending opening round-trips the prior reply's still-inert anchors before the next step overwrites them.
- Added coherently resealed corruption controls for identity, shape, array length, token range, Hamming cache, memory anchors and reply cursor, plus stale-artifact, truncation and trailing-byte refusals.

## Measurements

- FBS module suite: 28 passed.
- Full `uor-r4-cpu-train` release suite: 97 passed, 4 ignored release benchmarks.
- The child process resumed the exact checkpoint and produced the same next token, kept cells, complete Q12 query and checkpoint SHA-256 as the uninterrupted session.
- No corpus run, training run, float-shadow result or language result occurred.

## Checks run at the head

- `CARGO_TARGET_DIR=target/deepseek-hamming-memory CARGO_BUILD_JOBS=1 cargo test --release --locked -p uor-r4-cpu-train`: PASS after the known duplicate-artifact collision retry, 97 passed and 4 ignored benchmarks.
- `CARGO_TARGET_DIR=target/deepseek-hamming-memory CARGO_BUILD_JOBS=1 cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `UOR_LAB=deepseek uor-harness ratchet d23`: PASS, 4,451 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d25`: PASS, 93 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness ratchet d32`: PASS, 31 sites, baseline unchanged.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS, including ORDER-first, scope, report, banned-presence, rule-source and read-path checks.
- `UOR_LAB=deepseek uor-harness loop gates --full`: PASS, all 11 gates green.

## Skills used

No repository skill was invoked. The checkpoint shape and process-restart test follow the exact-head patterns already reviewed in the CPU discrete checkpoint unit.

## Still open

- Quantization-aware update checkpoints that retain admission.
- A float-shadow versus integer-server comparison on the same bound rows.
- A compact checkpoint format and atomic caller-side file replacement if production durability is required.

## Plain summary

The supported Hamming-read and exact turn-memory session can now leave one process and resume in another without losing any dynamic model state. Restore is fail-closed and reset still starts an exact fresh stream. This closes only the saved-session portion of #3503.
