# #345 — perf: optimize CPU teacher compilation

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T05:48:10Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-08-02T20:23:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/345
- **Merged:** 2026-08-02T20:23:56Z
- **Merge commit:** fe8067b7fc537a963593c90260a5d6b3f12c8317
- **Base / Head:** main / UOR-Foundation:issue-320-cpu-teacher-optimization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- cache RoPE sine/cosine angles once per position and head dimension, removing repeated `powf`/`sin`/`cos` work from every layer
- use macOS Accelerate vForce CPU exponentiation for the noncanonical teacher path by default, with `TLESS_TEACHER_VFORCE_EXP=0` as a scalar fallback
- keep `TLESS_CANONICAL_DETERMINISTIC=1` on the portable deterministic path
- document the CPU-only backend and local throughput measurement

The deployed transformerless runtime is unchanged and remains multiplication-free. No CUDA, Metal, OpenCL, or GPU backend is added or selected.

## Measurement

On the local SmolLM2 360M source, a fresh 5,000-token compile measured 74.5 tokens/s with both changes enabled versus 54.5 tokens/s in the earlier scalar-exp / uncached-RoPE run.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- focused model-source, softmax, and cache angle-bit tests pass
- `cargo test --workspace --offline` reaches the existing R4G1 allocation census failure: 2 allocations / 594 bytes in steady state; changed files do not touch runtime code

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

