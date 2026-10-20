# #200 — feat(api/server): Unified InferenceRequest & InferenceResponse payload protocol (#194)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-26T01:59:07Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-07-26T02:10:48Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/200
- **Merged:** 2026-07-26T02:10:48Z
- **Merge commit:** 0cc6bb3e6ee2ec427351bd8ffc605df66dac6aeb
- **Base / Head:** main / UOR-Foundation:issue-194-unified-inference-protocol
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent Epic: #198
Closes #194

## Summary of Changes

- **Unified API Payload Contract (`crates/uor-r4-api`)**: Implemented strongly-typed `InferenceRequest` and `InferenceResponse` structs in `crates/uor-r4-api/src/engine.rs` with `Serialize` and `Deserialize` derivations and re-exported them in `crates/uor-r4-api/src/lib.rs`.
- **Server Endpoint Integration (`src/server.rs`)**: Updated server payload handlers to consume `InferenceRequest` and emit `InferenceResponse`, establishing schema parity across HTTP REST, WebSocket, and WASM interfaces.
- **Unit Testing & Serde Roundtrip Verification**: Added unit tests in `crates/uor-r4-api/src/engine.rs` verifying roundtrip JSON serialization and deserialization for `InferenceRequest` and `InferenceResponse`.

## Acceptance & Gate Verification

- [x] **Local Workspace Tests**: `cargo test --workspace --offline` passed (all 94 BDD scenarios green).
- [x] **Clippy Check**: `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` passed with 0 warnings.
- [x] **Format Check**: `cargo fmt --check` passed cleanly.
- [x] **no_std Ladder**: `cargo check -p uor-r4-graph-format --no-default-features` & `cargo check -p uor-r4-graph-format --no-default-features --features alloc` passed cleanly.
- [x] **Schema Parity**: REST `/api/chat`, WS streams, and client UI adhere to unified `InferenceRequest` / `InferenceResponse` JSON schemas.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

