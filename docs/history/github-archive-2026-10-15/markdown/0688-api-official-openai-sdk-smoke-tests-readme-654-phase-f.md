# #688 — api: official OpenAI SDK smoke tests + README (#654 phase F)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T02:10:07Z
- **Updated:** 2026-10-09T14:17:08Z
- **Closed:** 2026-08-14T02:30:31Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/688
- **Merged:** 2026-08-14T02:30:31Z
- **Merge commit:** 80bba80a40e41328b00440c2f5b66da074b1da5e
- **Base / Head:** main / UOR-Foundation:i654f-openai-sdk-smoke
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #654; closes #687 (phase F). Builds on phase E (#686, merged). Verifies the pinned wire surfaces against the **official OpenAI SDKs** and documents it.

## Evidence (captured empirically against the real SDKs)
The Python SDK (3.0.0) and JS SDK (7.4.0) send **only the params explicitly passed** — minimal chat = `{model, messages}`, responses = `{model, input}`, a streaming call adds exactly `stream: true`. So every ordinary call lands inside our supported subset and `deny_unknown_fields` never trips on an SDK default. Both SDKs also **parse our chat + responses response shapes** (including the phase-D SSE stream) cleanly, ignoring the R4 audit extras.

**Verdict:** keep `deny_unknown_fields` strict. A 400 happens only when a caller explicitly passes an unsupported param — the intended fail-closed contract. No DTO relaxation.

## Two coverage layers
- **Deterministic CI fixtures** (`src/server.rs`): the exact SDK-emitted request bodies deserialize into our request DTOs, and our response bodies carry the fields the SDKs read (`choices[].message.content`, `finish_reason`, `usage.*`; responses `output[].content[].text`, `usage.*`, `status`). No network or model needed.
- **Runnable end-to-end scripts** (`profiles/openai/smoke_test.py`, `smoke_test.mjs`): drive a live server with the real SDK across all three surfaces (non-stream chat, stream chat, responses). Developer-run (need a compiled model loaded), not CI. Verified functional against a uor-r4-shaped mock.

`profiles/openai/README.md` gains an **SDK compatibility** section: verified SDKs/versions, both layers, run instructions, and the `deny_unknown_fields` rationale.

## Gates
fmt, clippy `-D warnings`, wasm32 lib build, server lib 51 + bin 6 tests, profile-pin 3, claim-wording, plus `py_compile` / `node --check` on the scripts — all green.

Later: G `/v1` control-route split. Deferred: streaming for `/v1/responses`.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

