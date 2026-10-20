# #351 — perf: compact one-term dot SIMD layout

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T21:41:17Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-02T21:55:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/351
- **Merged:** 2026-08-02T21:55:07Z
- **Merge commit:** fc6a7c75dc7d4f9c8d81b0149fc9a5d457183ccd
- **Base / Head:** main / UOR-Foundation:issue-330-dot-layout
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add a compact 64-byte load-time representation for one-term TLA6/TLA7 dot vectors
- detect the active packed term in either byte, matching the high-byte-first compiler encoding
- retain the 192-byte legacy representation when entries contain two active terms
- preserve scalar/reference behavior, on-disk u16 ABI, integer-only runtime semantics, and operation census accounting
- add exact layout/legacy tests and record the benchmark evidence

## Measurement

Local arm64 release benchmark on the TLA6 fixture
`smollm2-135m-instruct-tla6`, 5,000 iterations:

| layout | assignment |
| --- | ---: |
| previous load layout | 159,189 ns/token |
| compact one-term layout | 58,058 ns/token |

That is approximately 2.74x faster end-to-end on the measured fixture. Checksums are identical. The original isolated 3x target remains a separate question; the end-to-end >=2x target is met.

## Validation

- `cargo fmt --all -- --check`
- `cargo test -p uor-r4-core --lib --offline` — 27 passed
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- graph-format no-std checks with and without `alloc`
- allocation census: transformerless path remains allocation-free; the suite still fails at the pre-existing R4G1 census (2 allocations / 594 bytes)

Refs #330.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

