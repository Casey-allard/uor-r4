# #686 — api: POST /v1/responses via the shared generation adapter (#654 phase E)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T01:37:36Z
- **Updated:** 2026-10-09T14:17:08Z
- **Closed:** 2026-08-14T01:57:06Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/686
- **Merged:** 2026-08-14T01:57:06Z
- **Merge commit:** 783a8d2366eaa281eee700cc0ae270fac5be2e75
- **Base / Head:** main / UOR-Foundation:i654e-openai-responses
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #654; closes #685 (phase E). Builds on phase D (#684, merged). Adds the Responses API, all in `src/server.rs`.

## Extract the shared generation core
The routing → #248 serving cascade → R4 audit block moves out of the chat handler into `generate_serving_completion` (returning `GeneratedCompletion` or an honest declined-by-all terminal). Both `/v1/chat/completions` and `/v1/responses` call it, so the two wire surfaces **run the identical cascade and never diverge** in what they actually execute. The chat path stays bit-identical — the existing chat tests are unchanged and pass.

## Add non-streaming POST /v1/responses
- **Request subset** `{model, input, instructions, max_output_tokens, temperature, engine}` with `deny_unknown_fields` → tools, reasoning, structured-output formats, and streaming fail closed with the error envelope.
- **`input`** accepts a bare string or an array of message items (content is a string or an array of text parts). `flatten_responses_input` reuses the chat role-flattening so both surfaces treat roles identically; `instructions` is a system preamble; unsupported roles/shapes fail closed.
- **Response** is a real `Response` object: one assistant `message` output item with an `output_text` part, `usage {input,output,total}_tokens` from the serving tokenizer, `status` completed/incomplete (`incomplete_details.reason = max_output_tokens` at the budget), and the R4 audit trail as parity extras.

Streaming for `/v1/responses` (its own event protocol) is deferred to a later increment, mirroring chat (non-stream C → stream D).

## Tests
Input flattening (bare string, instructions+messages, content parts, fail-closed on bad role/shape/type/empty); response body shape (completed and incomplete); request parse + deny of unknown params.

## Gates
fmt, clippy `--all-targets --all-features -D warnings`, wasm32 lib build, server lib 47 + bin 6 tests, claim-wording — all green.

Later: F official Python/JS SDK smoke tests + README pin, G `/v1` control-route split.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

