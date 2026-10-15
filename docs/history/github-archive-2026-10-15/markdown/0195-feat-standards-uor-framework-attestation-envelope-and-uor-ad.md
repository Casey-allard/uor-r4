# #195 — feat(standards): UOR-Framework attestation envelope and uor-addr CID serialization across server endpoints

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T01:31:47Z
- **Updated:** 2026-07-26T03:41:17Z
- **Closed:** 2026-07-26T03:41:17Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/195
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: Master Roadmap Epic
Relates to: #160 (normative runtime), #184 (typed integration facade).
Source: `crates/uor-r4-core/src/tless_uor.rs`; `Cargo.toml` (pinned `uor-addr` / `UOR-Framework`).

## Motivation

The `uor-r4` codebase includes pinned git dependencies for `uor-addr` and `UOR-Framework`, and defines `TlessAxis` in `crates/uor-r4-core/src/tless_uor.rs`. However, the HTTP server endpoints currently return un-attested JSON payloads.

To integrate with Hologram infrastructure, every inference and state output must be wrapped in a verifiable `UorAttestationResult` envelope carrying BLAKE3 CIDs for the input, artifact container, state vector, and response payload.

## Scope

- **Attestation Envelope Integration**: Connect `src/server.rs` response pipeline to `tless_uor::generate_uor_attestation`.
- **Content-Addressed Headers & Payloads**: Include `uor_address`, `artifact_cid`, `store_cid`, and `attestation_cid` in every server response.
- **Verification API**: Expose `/api/uor/verify` endpoint to allow external systems (like Hologram) to verify attestation CIDs against disk artifacts.

## Non-Goals

- Modifying the external `uor-addr` or `UOR-Framework` standards crates.

## Affected Crates / Modules / Files

- `crates/uor-r4-core/src/tless_uor.rs`
- `src/server.rs`
- `crates/uor-r4-api`

## Deliverables

1. Server endpoint integration wrapping all synthesis outputs in `UorAttestationResult`.
2. Verifiable BLAKE3 CID headers and fields on `/api/chat` and `/api/sysinfo`.
3. Integration unit tests verifying CID recalculation and verification against `uor-addr` specifications.

## Acceptance Criteria

- Every response from `/api/chat` contains a valid `uor_attestation` payload with verified BLAKE3 CIDs.
- External validation call `tless_uor::generate_uor_attestation` returns `UorAttestationResult::Verified`.
- Four gates green (`cargo test`, `cargo clippy`, `cargo fmt`, `no_std`).

## Formal Claim Classification

- "All synthesis outputs carry a content-addressed UOR Attestation CID": **Guarantee**, status **Witnessed** (CI integration test).

## Test Requirements

- **Unit:** Merkle proof and attestation CID validation tests.
- **BDD / Cucumber:** Feature scenario validating end-to-end attestation verification.


## Comments (1)

### comment 5081405906 — 2026-07-26T01:32:02Z — Casey-allard

Parent Master Epic: #198


---
_Comment/review ids in this file: 5081405906_

