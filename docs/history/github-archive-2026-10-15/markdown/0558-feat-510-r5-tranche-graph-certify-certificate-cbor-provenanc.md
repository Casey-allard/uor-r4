# #558 — feat(#510): R5 tranche — graph-certify certificate CBOR + provenance-deletion validator to total (49→44)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T22:23:33Z
- **Updated:** 2026-10-09T14:17:24Z
- **Closed:** 2026-08-09T22:41:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/558
- **Merged:** 2026-08-09T22:41:59Z
- **Merge commit:** cffcc19ea89fec7a49d5ad1c74b15cac5bdeb850
- **Base / Head:** main / UOR-Foundation:r5/certify-certs
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): second graph-certify tranche. Convert `performance_certificate.rs` and `fairness_provenance.rs` off `Result<_, String>`. **Both modules are now fully sanctioned (0 audit sites).**

## Surfaces
- `to_cbor_bytes(&self) -> Vec<u8>` on both certificates (was `Result`) — a derive-Serialize certificate serialized into an in-memory buffer cannot fail; the `Result` was spurious, so the impossible error path becomes an `.expect()` invariant and the fn is infallible.
- `from_cbor_bytes(bytes) -> Option<Self>` on both (was `Result`) — `None` when the bytes are not a valid CBOR encoding OR the recomputed self-CID does not match the embedded one. Both mean the bytes are not a valid, self-consistent certificate — the absence of a product. The CID check stays; only its raised `String` error becomes `None`. `KappaError`'s variants are graph-format-specific and do not describe a generic certificate CID, so `Option` is the honest total surface here.
- `FairnessEvaluator::verify_provenance_deletion(...) -> Option<ProvenanceDeletionWitness>` (was `Result<_, String>`) — `Some(witness)` when every tombstoned edge is really zeroed; `None` when a tombstoned edge id is out of bounds or a referenced edge still carries a non-zero weight. The deletion is verified or it is not; an unverified deletion yields no witness.

## Ripple
Callers are all in the crate's `tests/`: the serialize `.expect` is dropped (`to_cbor_bytes` now returns `Vec<u8>`), the deserialize `.expect` is kept (valid on `Option`), and the untombstoned-edge test's two `.expect_err` assertions become `.is_none()` assertions.

## Verification
audit-limits graph-certify **49 → 44** (performance_certificate.rs and fairness_provenance.rs now 0 sites). Full workspace build + wasm32 graph-compiler + clippy + fmt clean; graph-certify suite green, BDD 100/100, audit-deferral clean.

Part of the #510 bottom-up sweep — graph-certify.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

