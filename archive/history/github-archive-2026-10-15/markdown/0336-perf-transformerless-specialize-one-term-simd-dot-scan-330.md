# #336 — perf(transformerless): specialize one-term SIMD dot scan (#330)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T21:41:32Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T22:32:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/336
- **Merged:** 2026-08-01T22:32:09Z
- **Merge commit:** c1fecf9ff8d5b1a664f320f847670238c37b879d
- **Base / Head:** main / UOR-Foundation:issue-330-simd-adapter
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Follow-up to #334 for #330.

The checked-in TLA6 compiler emits one active power-of-two term per packed dot entry. The SIMD adapter was still decoding and executing the permanently inactive second term, including its metadata loads and active-mask blend.

This change:

- Detects one-term stages at load time.
- Uses a dedicated AVX2/NEON one-term scan.
- Retains the two-term SIMD path for legacy artifacts.
- Uses a shift-count sentinel for inactive lanes so the one-term adapter can omit an extra mask load and blend.
- Preserves scalar fallback, artifact bytes, operation census accounting, and exact outputs.

## Verification

- `cargo fmt --all`
- `cargo test -p uor-r4-core --lib --offline` — 25 passed
- `cargo clippy -p uor-r4-core --all-targets --offline -- -D warnings`
- `cargo check -p uor-r4-core --all-targets --target x86_64-apple-darwin --offline`
- TLA6 benchmark checksum unchanged: `[35, 216, 124, 191]`

The benchmark is host-noisy at roughly 0.24–0.32 ms per assignment locally; the original #330 3x isolated / 2x end-to-end target remains unclaimed pending stable cross-host and TLA7 measurements.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

