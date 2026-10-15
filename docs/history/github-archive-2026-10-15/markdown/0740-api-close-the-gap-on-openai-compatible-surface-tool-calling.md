# #740 — api: close the gap on OpenAI-compatible surface (tool calling, sampling params, embeddings, 404 body)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-16T20:23:30Z
- **Updated:** 2026-08-17T14:04:09Z
- **Closed:** 2026-08-17T14:04:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/740
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

#655 set out to "ship a ready-by-default R4 model on the uor-matmul-backed
transformerless runtime," and one of its implicit goals was making sure the
server's `/v1/*` surface actually behaves like the OpenAI API standard, so
existing OpenAI-SDK-based clients/tools work against it without special
casing. An audit of `src/server.rs`'s OpenAI-compatible routes (against
`origin/main` `f3580343`, 2026-08-16) found the core path is solid but real
gaps remain in the wider surface. This issue exists to scope those gaps —
it is explicitly **not** being picked up as the next slice; the team is
prioritizing #653 (GNAF) first to establish a solid baseline of coherent,
syntactically-correct transformerless generation before investing further
in API-surface breadth.

# Current evidence (audit findings, 2026-08-16)

**Routes present:** `GET /v1/models`, `GET /v1/models/{model}`,
`POST /v1/chat/completions`, `POST /v1/responses`.

**What `/v1/chat/completions` gets right:** multi-turn `system`/`user`/
`assistant` messages, `temperature`, `max_tokens` (server-capped at 256),
`stream` (correct SSE framing: `chat.completion.chunk` deltas terminated by
`data: [DONE]\n\n`), a correct non-streaming `chat.completion` response
envelope (`id`/`object`/`created`/`model`/`choices[].message`/
`finish_reason`/`usage`), and OpenAI-shaped error envelopes
(`{"error":{"message","type","param","code"}}`) with sensible HTTP status
codes for malformed requests, unsupported roles, model-not-found, and
model-not-ready. Requests use `#[serde(deny_unknown_fields)]`, so unknown
parameters are refused with a 400 rather than silently ignored — a
deliberate fail-closed design choice, not an oversight, and one this issue
should preserve rather than paper over.

**Gaps found:**
- No `/v1/completions` (legacy text completions) or `/v1/embeddings` — not
  implemented at all.
- `POST /v1/chat/completions` rejects (400, not silently ignores):
  `top_p`, `n`, `stop`, `presence_penalty`, `frequency_penalty`,
  `logit_bias`, `logprobs`/`top_logprobs`, `tools`/`tool_choice` (no
  function/tool-calling support), `response_format` (no JSON mode),
  `seed`, `user`.
- `messages[].content` accepts a plain string only — the array-of-content-
  parts shape (`[{"type":"text","text":"..."}]`, increasingly a default in
  some SDKs even for plain text) fails to deserialize.
- The server only ever serves one active model; the `model` field is
  effectively decorative validation against that single model, not real
  multi-model routing.
- **Real bug, not a scope gap:** the catch-all 404 for an unknown route
  returns an empty body (`Content-Length: 0`), not the OpenAI error
  envelope — a client that tries to JSON-parse a 404 response will get a
  parse error instead of a readable error.
- No `Authorization: Bearer <token>` handling at all (harmless for a local
  server today, but means the server can't distinguish/reject
  unauthenticated traffic the way real OpenAI-API-compatible auth probes
  expect).

# Goal

Decide, deliberately, how much of the OpenAI API surface this project
actually wants to replicate — not by default-implementing everything, but
by choosing which gaps matter for the clients/tools this project actually
needs to interoperate with — and close the gaps that are chosen, plus the
404-body bug regardless (that one is a straightforward correctness fix, not
a scope question).

# Scope (to be refined when this issue is picked up)

- Fix the empty-body 404 to return the OpenAI error envelope for any
  unmatched route.
- Decide per-parameter whether to implement, accept-and-apply, or continue
  to reject-with-400 each of: `top_p`, `n`, `stop`, `presence_penalty`,
  `frequency_penalty`, `logit_bias`, `logprobs`, `tools`/`tool_choice`,
  `response_format`, `seed`, `user`, array-of-content-parts messages.
- Decide whether `/v1/completions` and `/v1/embeddings` are ever in scope
  for this project (a table-native transformerless graph doesn't obviously
  have an embeddings-model shape yet — worth an explicit non-goal rather
  than silence if the answer is "not planned").

# Non-goals (for now)

- Multi-model serving / real dynamic `model` routing — out of scope while
  the server serves exactly one active model (tracked separately, if ever,
  under #655's own decomposition).
- Any change to sampling/generation semantics themselves — this issue is
  about the HTTP surface, not the transformerless engine's output quality
  (that's the #653/generation-coherence track, which is being prioritized
  ahead of this one).

# Verification

Whatever is implemented gets exercised against a real OpenAI SDK client
(not just hand-rolled JSON request tests), plus the existing unit-test
convention in `server.rs` for "rejected, not ignored" parameter checks
(update or remove those tests for any parameter this issue decides to
start accepting).


## Comments (1)

### comment 5316817088 — 2026-08-17T13:43:11Z — Casey-allard

PR incoming: #764.

This issue asked for a deliberate decision on the OpenAI-surface gaps, not a default-implement-everything pass, so here's the decision, per gap:

**Fixed (real bugs, not scope questions):**
- The 404-body bug: an unmatched `/v1/*` route now returns the standard `{"error":{...}}` envelope with `Content-Type: application/json`, not an empty body. Non-`/v1/` paths (the dashboard's static assets) are unaffected.
- `messages[].content` now accepts the OpenAI content-parts array shape (`[{"type":"text","text":"..."}]`), not just a plain string -- several SDKs default to this even for plain text, so rejecting it was breaking real, otherwise-compliant clients. A non-text part (`image_url`, etc.) still fails closed with a clear error rather than being silently dropped.

**Decided, not implemented (recorded so this isn't silent):**
- `top_p`, `n`, `stop`, `presence_penalty`, `frequency_penalty`, `logit_bias`, `logprobs`/`top_logprobs`, `seed`: continue to reject (400), unchanged. This issue's own non-goals section already draws the line here -- "any change to sampling/generation semantics... is the #653/generation-coherence track." #762 (merging as #763) is the concrete evidence for where that line falls: it built real seeded sampling, deliberately scoped to the CLI/chat-engine surface, not this server's per-request handling.
- `tools`/`tool_choice`, `response_format`: not implemented -- both are substantial standalone engine capabilities out of proportion to an HTTP-surface issue.
- `user`: not implemented -- no per-user state on this server to attach it to.
- `/v1/completions`, `/v1/embeddings`: explicit non-goals. `/v1/completions` would duplicate `/v1/chat/completions` under a different wire shape with no concrete demand; a table-native transformerless graph has no embedding-model shape to serve `/v1/embeddings` from today.
- Multi-model routing: already an explicit non-goal in this issue (tracked under #655 if ever).

Full verification (fmt/clippy/audit-limits/full workspace test, plus live `curl` against the real server binary for both fixes) is in the PR description.



---
_Comment/review ids in this file: 5316817088_

