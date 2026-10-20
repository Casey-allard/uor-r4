# #319 — Add κ-addressed region objects and resolver

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T00:45:49Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T00:50:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/319
- **Merged:** 2026-08-01T00:50:30Z
- **Merge commit:** 80b794893f01317365697354713280dbbc62f4b4
- **Base / Head:** issue-264-r4g1-realization / UOR-Foundation:issue-263-region-objects
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements #263 on top of #317.

- Exports each TLS1 `(depth, prefix)` subtree as a canonical CBOR region object.
- Derives stable UOR κ-labels for region objects and a manifest skeleton.
- Adds strict canonical decoding and κ/length/depth validation.
- Adds `ModelStore` CAS read/write APIs and a `RegionResolver` adapter.
- Adds resolver-backed prediction conformance against the monolithic witness path.
- Leaves the deployed prediction kernel and TLS1 wire format unchanged.

## Validation

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo test --workspace --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`

Closes #263.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

