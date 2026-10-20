# #526 — feat(#510): R5 tranche 3a — core region_store to total ops

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T03:23:53Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-09T03:42:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/526
- **Merged:** 2026-08-09T03:42:31Z
- **Merge commit:** 2f6507e7b4b83ace0d8a3a5c2619918172a101e0
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t3a-region-store
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fifth R5 step (issue #510): begins the **core** crate, converting the content-addressed region store (`uor-r4-core::transformerless::region_store`) to the sanctioned surface. Follows the graph substrate (#518–#524) and the host-ingestion boundary (#525).

## Total by construction
Region serialization, addressing, and resolution are total: a canonical region object or manifest either decodes/addresses into a valid product or it does not. The sole reportable condition — *not a valid product* — is carried by `None`, so both crate-private error enums (`RegionObjectError`, `RegionResolveError`; neither escaped core) are **removed** and all 14 fallible functions return `Option`:

- `canonical_region_bytes` / `canonical_manifest_bytes` → `Option<Vec<u8>>`
- `decode_region_bytes` / `decode_manifest_bytes` → `Option<Region{Object,Manifest}>`
- `region_kappa` / `manifest_kappa_for` → `Option<String>` (uor-addr addressing via `.ok()`)
- `validate_object` / `validate_regions` → `Option<()>`
- `export_store` → `Option<RegionExport>`; `MemoryRegionResolver::{from_export, insert}`
- `RegionResolver::resolve` → `Option<RegionObject>` (total: a miss or an unavailable backend both resolve to `None`)
- `predict_witness_with_resolver` → `Option<Prediction>`

## Caller
The root wasm-router `src/model.rs` `ModelStore` resolver flattens its `Result` to `Option` (backend/IO failure or miss → `None`); the CAS put/get paths map `None` to `ModelError::InvalidRegionObject` via `ok_or_else`, preserving the application-level diagnostic.

## Verification
- `audit-limits`: region_store reports **zero** unsanctioned errors.
- Build / clippy (`-D warnings`) / fmt clean; all workspace test targets compile; region_store + model tests green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

