# #333 — bench(transformerless): add scalar dot scan baseline (#330)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T21:19:42Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T21:29:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/333
- **Merged:** 2026-08-01T21:29:55Z
- **Merge commit:** ff58b5cd1e14a0ec526f1a60109a5c8296b006f6
- **Base / Head:** main / UOR-Foundation:issue-330-dot-benchmark
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the phase-0 prerequisite for #330: a no-dependency wall-clock baseline for the transformerless shift-add dot path.

- Adds `cargo bench -p uor-r4-core --bench transformerless_dot`.
- Measures the scalar 1,024-row dot scan and complete runtime assignment.
- Uses the checked-in TLA6 fixture by default and accepts TLA6/TLA7 artifact paths.
- Emits checksums so future SIMD adapters must preserve exact outputs.
- Makes no runtime-semantic or artifact-format changes.

## Baseline

On the checked-in TLA6 fixture, 100 iterations measured:

- Scalar dot scan: 405.1 µs per scan / 395.6 ns per row.
- Complete runtime assignment: 294.0 µs per token.
- Throughput: 3,401 assignments/sec.

A pinned TLA7 artifact is not currently present in the workspace; the harness accepts one for the second required data point.

## Verification

- `cargo fmt --all -- --check`
- `cargo clippy -p uor-r4-core --all-targets --offline -- -D warnings`
- `cargo test -p uor-r4-core --lib --offline` (25 passed)
- `cargo bench -p uor-r4-core --bench transformerless_dot --offline -- tests/fixtures/tless_artifacts.bin 100`


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

