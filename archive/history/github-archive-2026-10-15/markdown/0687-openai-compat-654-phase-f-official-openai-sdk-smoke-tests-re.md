# #687 — OpenAI-compat #654 phase F: official OpenAI SDK smoke tests + README

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T02:09:37Z
- **Updated:** 2026-08-14T02:30:32Z
- **Closed:** 2026-08-14T02:30:32Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/687
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Sub-issue of #654 (phase F). Verifies the pinned wire surfaces against the OFFICIAL OpenAI SDKs and documents it.

EVIDENCE (captured empirically against the real SDKs): both the Python SDK (3.0.0) and JS SDK (7.4.0) send ONLY the params explicitly passed — minimal chat = {model, messages}; responses = {model, input}; streaming adds exactly stream:true. So every ordinary call lands inside our supported subset and deny_unknown_fields never trips on an SDK default. Both SDKs also parse our chat + responses response shapes (incl. the phase-D SSE stream) cleanly, ignoring the R4 audit extras. VERDICT: keep deny_unknown_fields strict — a 400 only happens when a caller explicitly passes an unsupported param, which is the intended fail-closed contract.

Deliverables (Casey chose fixtures + script + README):
1. Deterministic CI fixtures in src/server.rs: the exact SDK-emitted request bodies deserialize into our DTOs; our response bodies carry the fields the SDKs read (choices[].message.content/finish_reason/usage; responses output_text/usage/status). No network/model.
2. Runnable end-to-end scripts profiles/openai/smoke_test.{py,mjs} — drive a live server with the real SDK across all three surfaces (non-stream chat, stream chat, responses). Developer-run (need a compiled model), not CI.
3. profiles/openai/README.md 'SDK compatibility' section: verified SDKs/versions, both coverage layers, the deny_unknown_fields rationale.

Follow-up: G /v1 control-route separation. Deferred: streaming for /v1/responses.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

