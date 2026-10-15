# #557 — feat(#510): R5 tranche — graph-certify CBOR round-trip helpers to total (53→49)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T21:49:05Z
- **Updated:** 2026-10-09T14:17:24Z
- **Closed:** 2026-08-09T22:07:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/557
- **Merged:** 2026-08-09T22:07:49Z
- **Merge commit:** 62e4d2883fa9abeffcb922c08510e3b90119dc4d
- **Base / Head:** main / UOR-Foundation:r5/certify-cbor
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): first graph-certify tranche. Convert the CBOR serialize/deserialize pair on `ShortlistRecallReport` (`shortlist_evaluator.rs`) and `RateDistortionReport` (`predictive_sufficiency.rs`) off `Result<_, String>`. **Both modules are now fully sanctioned (0 audit sites).**

## Surfaces
- `to_cbor_bytes(&self) -> Vec<u8>` (was `Result`) — the reports' fields are plain scalars, so ciborium serialization into an in-memory buffer cannot fail. The `Result` was spurious; a failure would be a serialization defect, so the one impossible error path becomes an `.expect()` invariant and the fn is infallible.
- `from_cbor_bytes(bytes) -> Option<Self>` (was `Result`) — the bytes may not be a valid CBOR encoding of the report: the absence of a product, reported as `None`.

Both types' CBOR fns have only test callers (all in the crate's `tests/`), which use `.expect("serialize/deserialize CBOR")`; the serialize `.expect` is dropped (`to_cbor_bytes` now returns `Vec<u8>` directly) and the deserialize `.expect` is kept (valid on `Option`).

## Verification
audit-limits graph-certify **53 → 49** (shortlist_evaluator.rs and predictive_sufficiency.rs now 0 sites). Full workspace build + wasm32 graph-compiler + clippy + fmt clean; graph-certify test suite green, BDD 100/100, audit-deferral clean.

Part of the #510 bottom-up sweep — graph-certify (graph-compiler is already fully sanctioned).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

