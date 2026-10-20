# #684 — api: SSE streaming for chat completions (#654 phase D)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T00:35:49Z
- **Updated:** 2026-10-09T14:17:08Z
- **Closed:** 2026-08-14T00:56:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/684
- **Merged:** 2026-08-14T00:56:21Z
- **Merge commit:** 218b0d60ec02d302d07ceed9d374b9fafc2b4720
- **Base / Head:** main / UOR-Foundation:i654d-openai-sse-streaming
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #654; closes #683 (phase D). Builds on phase C (#682, merged). Adds the `stream: true` surface to `POST /v1/chat/completions`, all in `src/server.rs`.

## What
The serving cascade produces the whole completion up front — there is no incremental token generator on this path — so streaming **re-frames the finished text as ordered `chat.completion.chunk` Server-Sent Events**. The wire format is streaming; the generation is not. This is honest and SDK-conformant (SSE does not require chunks to arrive as they are computed).

- **Request DTO**: `stream: Option<bool>` + `stream_options { include_usage }`. `deny_unknown_fields` is preserved, so an unsupported stream option fails closed with the error envelope rather than being silently ignored.
- **`text/event-stream` body**: role chunk → one chunk per content delta → terminal chunk → optional usage-only chunk → `data: [DONE]`.
  - `split_stream_deltas` guarantees the content deltas **rejoin byte-for-byte** to the completion (streaming never alters content).
  - Terminal chunk carries the truthful `finish_reason` (reuses phase-C `completion_finish_reason`) and the R4 audit trail, for parity with the single-JSON body (SDKs ignore the extra fields).
  - Usage-only chunk (empty `choices`) is emitted only when `stream_options.include_usage` is set.
  - Every chunk carries the shared `id`/`created`/`model`/`system_fingerprint`.
- **Disconnect**: a write error stops the frame loop cleanly (no panic); the body ends at connection close (no `Content-Length`).

## Tests (pure, in `src/server.rs` mod tests)
Delta reconstruction across whitespace/unicode edge cases; frame ordering (role first, content rejoins, terminal `[DONE]`); the usage-only chunk under `include_usage`; request parse + deny of unknown stream options.

## Gates
fmt --check, clippy `--all-targets --all-features -D warnings`, wasm32 lib build, server lib 41 + bin 6 tests, claim-wording — all green locally.

Later phases: E `POST /v1/responses`, F official-SDK smoke tests + README pin, G `/v1` control-route split.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

