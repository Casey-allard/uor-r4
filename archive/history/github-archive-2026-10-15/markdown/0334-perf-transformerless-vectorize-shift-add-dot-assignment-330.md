# #334 — perf(transformerless): vectorize shift-add dot assignment (#330)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T21:27:46Z
- **Updated:** 2026-08-01T21:38:29Z
- **Closed:** 2026-08-01T21:38:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/334
- **Merged:** 2026-08-01T21:38:29Z
- **Merge commit:** f2adaaea07a3b21dba6abcf00308fc01d0932e28
- **Base / Head:** main / UOR-Foundation:issue-330-simd-adapter
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Part 2 of #330: adds the contract-legal SIMD adapter behind the scalar shift-add dot assignment.

- Expands packed dot terms into load-time transposed metadata; artifact bytes and format are unchanged.
- Adds AVX2 four-lane and NEON two-pair shift/add adapters.
- Keeps scalar fallback and exact scalar semantics as the equality oracle.
- Preserves operation census accounting and the no-multiply runtime contract.
- Applies to both the TLA6 dot path and TLA7 residual path; residual subtraction remains scalar and bounded.

## Measurement

On the checked-in TLA6 fixture, arm64 macOS:

- Scalar baseline: 294.0 µs per complete assignment, 3,401 assignments/sec.
- NEON adapter: 187.9 µs per complete assignment, 5,322 assignments/sec.
- Improvement: approximately 1.56x.
- Runtime checksum unchanged: `[35, 216, 124, 191]`.

The 3x isolated / 2x end-to-end issue criterion is not claimed yet; this is the first adapter data point. A pinned TLA7 artifact is still needed for the second matrix row.

## Verification

- `cargo fmt --all -- --check`
- `cargo clippy -p uor-r4-core --all-targets --offline -- -D warnings`
- `cargo check -p uor-r4-core --all-targets --target x86_64-apple-darwin --offline`
- `cargo test -p uor-r4-core --lib --offline` (25 passed, including P-4 and equality witnesses)
- `cargo bench -p uor-r4-core --bench transformerless_dot --offline -- tests/fixtures/tless_artifacts.bin 100`

Depends on #333.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

