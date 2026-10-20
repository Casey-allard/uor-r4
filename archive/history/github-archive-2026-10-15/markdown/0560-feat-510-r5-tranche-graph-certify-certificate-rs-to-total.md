# #560 — feat(#510): R5 tranche — graph-certify certificate.rs to total

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T23:05:04Z
- **Updated:** 2026-10-09T14:17:24Z
- **Closed:** 2026-08-09T23:24:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/560
- **Merged:** 2026-08-09T23:24:04Z
- **Merge commit:** 0a1de7b98fc7b62359b3d3a28a2171b6c6fd84a1
- **Base / Head:** main / UOR-Foundation:r5/certify-cert
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `certificate.rs` off `Result` / `CertificateError`, dropping the `CertificateError` enum entirely. **The module is now fully sanctioned (0 audit sites).**

## Surfaces
- `claims_from_fragments_with_threads -> Vec<EmpiricalClaim>` (was `Result<_, String>`) — the module comment already noted it never reports an error now that the compiler executor is total (the worker closure only clones a fragment; a worker panic propagates). Dropped the spurious `Result`.
- `new_from_claim_fragments -> Self` (was `Result<Self, CertificateError>`) — its only error path was the now-infallible claims helper, so the constructor is infallible.
- `verify_attestation -> Option<String>` (was `Result<(), CertificateError>`) — a Gate-K validator. `None` when the attestation holds; `Some(reason)` naming the first failure — self-CID mismatch or an unverified attestation flag. Mixed CID + flag failures, so `Option<String>` is the total validator surface.
- `to_cbor_bytes -> Vec<u8>` infallible (`.expect`); `from_cbor_bytes -> Option<Self>` (`None` on invalid bytes OR self-CID mismatch), same pattern as the other certificates.

`CertificateError` enum removed (`SerializationError`/`DeserializationError`/`CidMismatch`/`AttestationFailed` all gone).

## Ripple
Callers are all in `certificate_test`: `new_from_claim_fragments` `.unwrap()` dropped; `verify_attestation` `.is_ok()` → `.is_none()` and `.is_err()` → `.is_some()`; `to_cbor_bytes` `.expect`/`.unwrap` dropped; `from_cbor_bytes` `.expect` kept (valid on `Option`).

## Verification
audit-limits graph-certify certificate.rs now 0 sites. Full workspace build + wasm32 graph-compiler + clippy + fmt clean; graph-certify suite green (certificate test 6/6), BDD 100/100, audit-deferral clean.

Part of the #510 bottom-up sweep — graph-certify.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

