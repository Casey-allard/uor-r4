# #528 — feat(#510): R5 tranche 3c — core runtime store/code layer to total ops

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T04:54:23Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-09T05:12:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/528
- **Merged:** 2026-08-09T05:12:55Z
- **Merge commit:** 88e3c441bfe224e785e9068094ae2424aa6bf7e3
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t3c-runtime
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Continues **core** (issue #510): the store-parse and parallel code/store generation in `uor-r4-core::transformerless::runtime`. Follows region_store (#526) and the TransitionGraph cluster (#527).

- `parse_store_strict_u32` → `Option<Store>` (None = not a valid u32 TLS1 store, whether malformed or a deprecated legacy 16-bit encoding). `StoreParseError` removed; the legacy detector `parse_store_legacy_u16` is **retained** (R4) for callers that want to distinguish the migration case.
- `codes_with_threads` → `Vec<[u8; STAGES]>` and `build_store_with_threads` → `(Store, Vec<[u8; STAGES]>)`: a worker thread panicking is a bug, not a data condition, so the panic is propagated (there is no partial code sequence to report) and the functions are total.
- `purge_legacy_store_cache` → `usize`: total, best-effort — an unenumerable directory contributes zero and an unremovable file is skipped.

Callers adapted (drop `?`/`.map_err`/`.expect` on the now-total returns): `code_sidecar::build_store_cached`, graph-cli `runtime_corpus` + recorded-store build, and the code_sidecar / store_legacy tests. `build_store_cached` keeps its own `String` `Result` (its own code_sidecar tranche).

`audit-limits`: core `runtime.rs` reports **zero** unsanctioned errors. Build / clippy (`-D warnings`) / fmt clean; all workspace test targets compile; store_legacy green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

