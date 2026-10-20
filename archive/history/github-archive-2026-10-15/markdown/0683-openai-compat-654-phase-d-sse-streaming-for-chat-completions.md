# #683 — OpenAI-compat #654 phase D: SSE streaming for chat completions

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T00:34:53Z
- **Updated:** 2026-08-14T00:56:22Z
- **Closed:** 2026-08-14T00:56:22Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/683
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Sub-issue of #654 (phase D). Adds the `stream: true` surface to `POST /v1/chat/completions`.

The serving cascade produces the whole completion up front (no incremental token generator on this path), so streaming re-frames the finished text as ordered `chat.completion.chunk` Server-Sent Events — the wire format is streaming, the generation is not. This is honest and SDK-conformant.

Scope:
- `stream: Option<bool>` + `stream_options: {include_usage}` added to the request DTO (deny_unknown_fields preserved — unknown stream options fail closed).
- `text/event-stream` response: role chunk → content deltas (concatenation byte-exactly reconstructs the completion) → terminal chunk with truthful `finish_reason` (reuses phase-C `completion_finish_reason`) → optional usage-only chunk when `include_usage` → `data: [DONE]`.
- Each chunk carries the shared id/created/model/system_fingerprint; the R4 audit trail rides the terminal chunk for parity with the single-JSON body.
- Client disconnect stops the write loop cleanly (no panic).
- Pure-function unit tests for delta reconstruction, frame ordering, usage chunk, and request parsing/deny.

Follow-ups (later phases): E `POST /v1/responses`, F official-SDK smoke tests + README pin, G `/v1` control-route split.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

