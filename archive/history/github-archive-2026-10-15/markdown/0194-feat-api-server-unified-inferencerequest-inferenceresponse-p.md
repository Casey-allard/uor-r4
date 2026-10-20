# #194 — feat(api/server): Unified InferenceRequest & InferenceResponse payload protocol across HTTP, WS, and WASM

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T01:31:45Z
- **Updated:** 2026-07-26T02:10:49Z
- **Closed:** 2026-07-26T02:10:49Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/194
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: Master Roadmap Epic
Relates to: #184 (typed integration facade), #161 (status policy).
Source: `src/server.rs`; `index.html`; `crates/uor-r4-api`.

## Motivation

Historical payload structs (`ChatPayload`, `CorpusPayload`, ad-hoc REST fields) created schema drift across HTTP endpoints, WebSocket streams, and frontend clients. This issue establishes a single unified `InferenceRequest` and `InferenceResponse` contract across all server and client interfaces.

## Scope

- **Unified Request Struct**: Implement `InferenceRequest` across HTTP `/api/chat` and WebSocket handlers:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct InferenceRequest {
      pub text: String,
      pub identity: Option<String>,
      pub engine: Option<String>,
      pub max_tokens: Option<usize>,
      pub temperature: Option<f64>,
  }
  ```
- **Unified Response Struct**: Implement `InferenceResponse` with explicit status codes, engine metadata, execution metrics, and attestation CIDs.
- **Client Synchronization**: Update `index.html` and client callers to send and parse the unified JSON schema identically across REST and WebSocket connections.

## Non-Goals

- Changing the underlying synthesis engine logic in `uor-r4-router` or `uor-r4-core`.

## Affected Crates / Modules / Files

- `src/server.rs`
- `crates/uor-r4-api/src/engine.rs`
- `index.html`

## Deliverables

1. `InferenceRequest` and `InferenceResponse` strongly-typed structs in `crates/uor-r4-api` and `src/server.rs`.
2. HTTP POST `/api/chat` and WebSocket stream handlers updated to consume/produce unified payloads.
3. Serialization unit tests verifying backward-compatibility with legacy requests (`ollama`, `transformerless`, `r4g1`).

## Acceptance Criteria

- All REST `/api/chat` and WebSocket endpoints accept `InferenceRequest` and emit `InferenceResponse`.
- Invalid JSON payloads return HTTP 400 with a structured error object.
- Four gates green (`cargo test`, `cargo clippy`, `cargo fmt`, `no_std`).

## Formal Claim Classification

- "HTTP REST and WebSocket streaming interfaces use a single unified payload protocol": **Guarantee**, status **Witnessed**.

## Test Requirements

- **Unit:** Roundtrip JSON serialization/deserialization tests for `InferenceRequest` and `InferenceResponse`.
- **BDD / Cucumber:** Feature scenarios verifying REST and WS payload contract parity.


## Comments (1)

### comment 5081405851 — 2026-07-26T01:32:01Z — Casey-allard

Parent Master Epic: #198


---
_Comment/review ids in this file: 5081405851_

