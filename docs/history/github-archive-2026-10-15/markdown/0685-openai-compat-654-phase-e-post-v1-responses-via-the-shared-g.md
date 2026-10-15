# #685 — OpenAI-compat #654 phase E: POST /v1/responses via the shared generation adapter

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T01:37:05Z
- **Updated:** 2026-08-14T01:57:07Z
- **Closed:** 2026-08-14T01:57:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/685
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Sub-issue of #654 (phase E). Implements `POST /v1/responses` (createResponse), the matrix's second supported text-generation op, routed through the SAME internal generation adapter as chat completions.

Two parts, one PR (Casey's call):
1. Extract the shared generation core (routing → #248 cascade → R4 audit) out of the chat handler into `generate_serving_completion`, so both wire surfaces run the identical cascade (no duplication). Chat path kept bit-identical.
2. Add non-streaming `/v1/responses`: request subset {model, input, instructions, max_output_tokens, temperature, engine} with deny_unknown_fields (tools/reasoning/stream fail closed); `input` accepts a bare string or an array of message items, flattened via the same role logic as chat; response is a real `Response` object (output[].message → output_text, usage input/output/total from the serving tokenizer, status completed/incomplete + incomplete_details at budget, R4 audit parity extra).

Streaming for /v1/responses (its own event protocol) is deferred to a later increment — mirroring chat (non-stream B/C → stream D). Follow-ups: F official-SDK smoke tests + README pin, G /v1 control-route split.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

