# #256 — Attestation envelope bypasses uor-addr and carries placeholder/fabricated CIDs (conformance finding)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T11:50:24Z
- **Updated:** 2026-07-28T13:20:57Z
- **Closed:** 2026-07-28T13:20:57Z
- **Labels:** bug, conformance
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/256
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The `uor_attestation` envelope added in PR #195/#214 does not use the uor-addr crate and carries fabricated identifiers:

- `src/server.rs` computes `uor_address` with raw `blake3::hash` on the response JSON rather than uor-addr's canonicalize-then-hash pipeline (byte-different-but-equivalent encodings get different addresses — the exact failure uor-addr exists to prevent).
- `artifact_cid` / `store_cid` are hardcoded placeholder strings (`"blake3:score.r4g1.header"`).
- `src/server.rs:1974` fabricates a `uor:cbor:blake3:{:016x}` label from a **curvature float** — a syntactically-valid, semantically-meaningless address.

### Classification
Conformance finding, not a plain bug: these are textbook **negative vectors** for a UOR-ATTESTATION suite (an envelope must not advertise standards conformance it does not have). Cheap to fix; embarrassing at demo time if not.

### DoD
- Envelope addresses computed via uor-addr's canonical pipeline; artifact/store CIDs real (from `ModelStore` / actual artifact bytes) or omitted, never placeholders.
- The fabricated-from-float label removed.
- The before/after pair recorded as seed vectors (negative: placeholder CID accepted; positive: canonical envelope verifies) for the conformance suites.


## Comments (1)

### comment 5104519599 — 2026-07-28T13:06:07Z — Casey-allard

Taking this together with the verify-endpoint sibling (assigned). **One branch/PR for both** (`issue-256-uor-conformance`) — deliberate deviation from one-branch-per-issue: the envelope (producer) and `/api/uor/verify` (checker) must move to the canonical pipeline atomically, or every round-trip between the two PRs is broken by construction.

Plan:
- `canonical_json_address_blake3()`: envelope + verifier both address via `uor_addr::json::address_blake3` (JCS canonicalization → blake3) — encoding-independent labels, same axis as today's fields.
- Envelope: real `artifact_cid` (mtime-cached blake3 of the artifact the serving cascade resolves — candidate list factored to a shared const) or **omitted**; `store_cid` dropped entirely (it named a section of the same file — structurally bogus as a separate CID); self-asserted `verify_result: "Verified"` dropped (the verify endpoint is the verifier).
- Audit trace: the `uor:cbor:blake3:{curvature-float}` fabrication replaced with a canonical label over the audit content (mode, κ, deficit angle, γ, temperature).
- Verifier: typed syntax gate (`empty_address` / `unsupported_axis` / `digest_length_invalid` / `digest_not_lowercase_hex` → HTTP 400), exact-equality comparison only (the old code passed empty addresses AND any string merely containing the digest).
- Tests double as first conformance vectors: encoding-independence of the canonical label, the negative-vector table (empty subject first), substring-attack regression, and a producer/verifier round-trip under re-encoded bytes.



---
_Comment/review ids in this file: 5104519599_

