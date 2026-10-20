# #691 — OpenAI-compat: streaming for /v1/responses (Responses event protocol)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T02:54:51Z
- **Updated:** 2026-08-14T03:15:14Z
- **Closed:** 2026-08-14T03:15:14Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/691
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #654 (the deferred streaming increment for the Responses API). Adds `stream: true` to POST /v1/responses.

Like chat streaming (phase D), the serving cascade produces the whole completion up front, so this re-frames the finished text as the spec's typed Responses event sequence — the wire format is streaming, not the generation.

Validated EMPIRICALLY against the official OpenAI Python SDK (3.0.0): the exact event sequence + SSE framing (event: <type> / data: <json>) is parsed by the real SDK, which reconstructs the text from response.output_text.delta events and reads the final response.completed (status, output_text, usage) — including our build_responses_body completed shape with the R4 audit extras.

- VendorResponsesRequest gains stream: Option<bool> (deny_unknown_fields preserved).
- Event sequence (each with monotonic sequence_number): response.created → in_progress → output_item.added → content_part.added → output_text.delta* (deltas rejoin byte-exact via split_stream_deltas) → output_text.done → content_part.done → output_item.done → response.completed. NO [DONE] sentinel (terminates on completed, unlike chat).
- Helpers sse_event_frame + build_responses_stream_frames; reuses send_sse_stream (phase D) and build_responses_body (phase E) for the terminal Response.
- 2 pure-fn tests (full event sequence + stream parse).

This closes the last deferred item from the #654 A-G work.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

