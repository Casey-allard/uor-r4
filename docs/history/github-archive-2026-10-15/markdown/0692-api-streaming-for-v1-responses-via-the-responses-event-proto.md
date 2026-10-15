# #692 — api: streaming for /v1/responses via the Responses event protocol (#654)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T02:55:29Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-08-14T03:15:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/692
- **Merged:** 2026-08-14T03:15:13Z
- **Merge commit:** ed64e0a191f51a701cda0f46eff67b1ee68c15f3
- **Base / Head:** main / UOR-Foundation:i654h-responses-streaming
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #691 — the deferred streaming increment from the #654 A–G work. Builds on phase G (#690, merged). Adds `stream: true` to `POST /v1/responses`.

## What
Like chat streaming (phase D), the serving cascade produces the whole completion up front, so this **re-frames the finished text as the spec's typed Responses event sequence** — the wire format is streaming, not the generation.

**Validated empirically against the official OpenAI Python SDK (3.0.0)** (the phase-F method): the exact event sequence and SSE framing (`event: <type>` / `data: <json>`) is parsed by the real SDK, which reconstructs the text from `response.output_text.delta` events and reads the final `response.completed` (status, `output_text`, usage) — including our `build_responses_body` completed shape with the R4 audit extras.

- `VendorResponsesRequest` gains `stream: Option<bool>` (`deny_unknown_fields` preserved — unsupported params still fail closed).
- Event sequence, each with a monotonic `sequence_number`: `response.created` → `.in_progress` → `.output_item.added` → `.content_part.added` → `.output_text.delta`* (deltas rejoin byte-exact via `split_stream_deltas`) → `.output_text.done` → `.content_part.done` → `.output_item.done` → `.completed`. **No `[DONE]` sentinel** — the Responses stream terminates on `response.completed` (unlike chat).
- Helpers `sse_event_frame` + `build_responses_stream_frames`; reuses `send_sse_stream` (phase D) and `build_responses_body` (phase E) for the terminal Response.
- 2 pure-function tests (full event sequence + stream request parse).

## Gates
fmt, clippy `-D warnings`, wasm32 lib build, server lib 56 + bin 6 tests (2 new), claim-wording — all green.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

