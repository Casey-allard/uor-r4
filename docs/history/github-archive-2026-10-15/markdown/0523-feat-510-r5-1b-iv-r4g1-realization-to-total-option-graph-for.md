# #523 — feat(#510): R5 1b(iv) — r4g1 realization to total Option (graph-format done)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T02:02:05Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-09T02:40:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/523
- **Merged:** 2026-08-09T02:40:15Z
- **Merge commit:** 696bb776e75982507df672920e6ebfb560b379b5
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t1b-r4g1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The realization layer composite RealizationError (parse→NotAProduct, CID→KappaError, uor-addr rejection) cannot be one sanctioned error per Result. Per the approved total-ops direction, r4g1 address/artifact_kappa/section_kappa/address_cbor become total Option (None = not addressable); the reason stays recoverable via GraphView::parse/verify_cids. RealizationError removed. Callers (api engine, graph-cli emission, root server cid cache) and the realization test updated. Last graph-format R5 file — with #520/#521/#522 it takes graph-format to 0. Full workspace builds, std+no_std, clippy clean, tests pass. Refs #510.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

