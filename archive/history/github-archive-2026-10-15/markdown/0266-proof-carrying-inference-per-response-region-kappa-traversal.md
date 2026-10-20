# #266 — Proof-carrying inference: per-response region-kappa + traversal witness, verifiable against the artifact

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T15:06:32Z
- **Updated:** 2026-08-01T21:37:40Z
- **Closed:** 2026-08-01T21:37:40Z
- **Labels:** enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/266
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
#256 made the attestation envelope honest about *what* was said. The full proof-carrying-inference vision (the semantic-routing proposal) is attestation of *how*: a response optionally carries the **region κ that answered** plus the **traversal witness** (prefix depth / resolution status / beam or ROUT path taken), verifiable against the artifact's section addresses.

Most of the substrate exists: `PrismModel` impls minting `Grounded` certificates (`src/tless_uor.rs`), per-step `ResolutionStatus`, and (once the R4G1 realization lands) per-section κs to anchor the witness to. Nobody has filed the issue connecting them to the serving path.

### Sketch
- Opt-in (request flag): response envelope gains `witness: { region_kappa, depth, resolution_status, engine }` per generated span.
- Verification: `/api/uor/verify` extended (or a sibling endpoint) to check a witness against a held artifact — the region κ exists in the artifact manifest and the claimed depth/status is consistent.
- Cost target: witness assembly server-side only; zero runtime-kernel changes.

### DoD
A response produced with the flag verifies end-to-end against the serving artifact; a tampered witness (wrong region κ, inflated depth) is rejected with a typed reason; conformance vectors recorded alongside the #256/#257 set.

*Depends on #264 (section addresses); rides on #263's region identity. From the "UOR in the loop" gap analysis.*


## Comments (1)

### comment 5153559823 — 2026-08-01T21:37:39Z — auser

Closing as complete via PR #326 (merge commit fe6f6fc, on main). DoD verified on origin/main: (1) POST /api/r4g1/generate accepts opt-in include_witness and the response verifies end-to-end against the loaded artifact via the extended POST /api/uor/verify (src/server.rs:3286-3447, crates/uor-r4-api/src/engine.rs::verify_witnesses); (2) tampered witnesses — inflated depth and replaced region κ — are rejected with typed WitnessVerificationError reasons (engine.rs:245-283), asserted in tests/status_policy.rs::proof_witness_roundtrips_and_rejects_tampering; (3) conformance vectors recorded in the same tests-as-vectors convention as the #256/#257 set, with a vector table in docs/proof_carrying_inference_266.md. Runtime kernel and allocation-free hot path unchanged.


---
_Comment/review ids in this file: 5153559823_

