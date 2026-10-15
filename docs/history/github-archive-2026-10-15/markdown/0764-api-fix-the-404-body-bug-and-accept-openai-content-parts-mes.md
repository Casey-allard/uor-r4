# #764 — api: fix the 404-body bug and accept OpenAI content-parts messages (#740)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T13:42:49Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-08-17T14:04:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/764
- **Merged:** 2026-08-17T14:04:08Z
- **Merge commit:** 99362064aff149cee5dc1984341fb12b316787c5
- **Base / Head:** main / UOR-Foundation:i740-openai-surface
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

api: fix the 404-body bug and accept OpenAI content-parts messages (#740)

#740 audited the `/v1/*` OpenAI-compatible surface and asked for a
deliberate per-gap decision, not a default-implement-everything pass.
This closes it by fixing the two genuine correctness/interop bugs it
found and recording an explicit decision on everything else, so the
issue doesn't stay open against work nobody decided to do.

Fixed (real bugs, not scope questions):
- The catch-all for an unmatched route returned a bare
  `HTTP/1.1 404 NOT FOUND` with an empty body and no `Content-Type`,
  falling through to the dashboard's static-file-serving path. Any
  `/v1/*` request to a route that doesn't exist (a typo, the legacy
  `/v1/completions`, `/v1/embeddings`) now gets the same
  `{"error":{...}}` envelope every other API error on this server
  uses, with `Content-Type: application/json` -- a client that tries
  to JSON-parse the error no longer gets a parse failure instead of a
  readable one. Non-`/v1/` paths are unaffected: the dashboard's own
  static assets and its existing empty-body 404 behavior are
  untouched (verified live, not just by the new unit tests -- see
  below).
- `messages[].content` accepted a plain string only; the
  content-parts array shape (`[{"type":"text","text":"..."}]`) --
  increasingly a default some SDKs send even for plain-text-only
  messages -- failed to deserialize, breaking otherwise-compliant
  clients for no capability reason. Text parts are now concatenated
  in order; a part of any other `type` (`image_url`, `input_audio`,
  ...) still fails closed with a clear `invalid_request_error`
  instead of being silently dropped, consistent with this file's
  existing "support is never implied by omission" convention
  (`deny_unknown_fields` on the outer request types).

Decided (per-gap, not implemented -- recorded so this isn't silent):
- `top_p`, `n`, `stop`, `presence_penalty`, `frequency_penalty`,
  `logit_bias`, `logprobs`/`top_logprobs`, `seed`: continue to reject
  with 400 (already the existing `deny_unknown_fields` behavior,
  unchanged). Implementing any of these for real is a generation-
  engine change, not an HTTP-surface one -- #740's own non-goals
  section already draws that line ("any change to sampling/generation
  semantics... is the #653/generation-coherence track"), and #762
  (issue-#759 follow-up, merging as #763) is the concrete evidence:
  it added a real seeded-sampling primitive, but scoped deliberately
  to the CLI/chat-engine surface, not this server's per-request
  handling, for exactly this reason.
- `tools`/`tool_choice` (function calling), `response_format` (JSON
  mode): not implemented. Both are substantial standalone engine
  capabilities (a tool-call protocol; constrained/structured decoding)
  disproportionate to an HTTP-surface issue; tracked as an explicit
  non-goal here rather than reopened piecemeal.
- `user`: not implemented -- there is no per-user state on this server
  to attach it to.
- `/v1/completions` (legacy) and `/v1/embeddings`: explicit non-goals,
  not silence. `/v1/completions` would duplicate `/v1/chat/completions`
  semantics under a different wire shape with no concrete client
  demand; a table-native transformerless graph has no embedding-model
  shape to serve `/v1/embeddings` from today.
- Multi-model routing: already an explicit non-goal in #740 itself
  (tracked under #655 if ever).

Verification:
- cargo fmt --check
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo run -p xtask -- audit-limits
- cargo test --workspace --offline (full suite, no failures)
- Live verification against the real server binary (`r4 serve`), not
  just unit tests: `curl` against an unmatched `/v1/*` route confirms
  the JSON envelope + `Content-Type: application/json`; a non-`/v1/`
  unmatched path confirms the old empty-body static-file 404 is
  unchanged; a content-parts-array chat request reaches the model
  layer (503 model-not-ready in this environment, not a 400 parse
  error); a non-text content part correctly 400s.

New tests (src/server.rs):
- chat_message_content_still_accepts_a_plain_string
- chat_message_content_accepts_the_openai_content_parts_array_shape
- chat_message_content_rejects_non_text_parts_instead_of_dropping_them

Closes #740

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

