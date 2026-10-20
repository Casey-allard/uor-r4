# #592 — fix(#515): drop region-store-dormant claim — region_store is on the serving path

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T20:29:45Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-10T20:48:44Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/592
- **Merged:** 2026-08-10T20:48:44Z
- **Merge commit:** 27b3ec575942de7f3c950e3a919d0773dae6dafa
- **Base / Head:** main / UOR-Foundation:r5/ledger-fix-regionstore
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 — correct a misclassified register entry

The `region-store-dormant` claim added in #590 is factually wrong: `region_store`
is **on** the serving path, not off it.

`crates/uor-r4-core/src/transformerless/region_store.rs` is used across the root
serving/model path in `src/model.rs` (L245–298): `RegionObject`,
`canonical_region_bytes`, `region_kappa`, `decode_region_bytes`, `RegionManifest`,
`manifest_kappa_for`. Feature-gating it breaks the default workspace build with 13
unresolved references — the build-level proof that it is not dormant.

- Remove the `region-store-dormant` `[[claim]]` from `model/ledger.toml`.
- Regenerate `CONFORMANCE.md` via `xtask check-model --write` (29 ids, model equals doc).
- No code cites the claim id; no deferral markers reference it.

Refs #515.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

