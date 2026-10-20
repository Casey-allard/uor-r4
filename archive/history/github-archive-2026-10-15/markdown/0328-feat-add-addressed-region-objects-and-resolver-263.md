# #328 — feat: add κ-addressed region objects and resolver (#263)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T03:58:45Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T04:12:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/328
- **Merged:** 2026-08-01T04:12:08Z
- **Merge commit:** ca695ae260549c4547bf6931950fe2d228f93c49
- **Base / Head:** main / UOR-Foundation:issue-263-region-objects-main
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #263

This lands the existing κ-addressed region-object implementation onto main.

What changed:
- Adds canonical CBOR `RegionObject`, `RegionManifest`, and export types with deterministic ordering and κ validation.
- Adds `MemoryRegionResolver` and manifest-driven prediction witnesses.
- Adds ModelStore CAS put/get APIs and the RegionResolver implementation.
- Adds deterministic region/manifest conformance tests and implementation documentation.
- Keeps the deployed runtime kernel unchanged; this is compiler/storage/resolver-side work.

Validation:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- graph-format no-std checks (default and `alloc`)
- `cargo test -p uor-r4-core --lib --offline`
- `cargo test --workspace --offline --exclude uor-r4-core`
- claim-wording and diff checks

The full workspace suite reaches the pre-existing R4G1 allocation census failure (2 allocations / 594 bytes, expected 0); the transformerless census and all #263 tests pass.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

