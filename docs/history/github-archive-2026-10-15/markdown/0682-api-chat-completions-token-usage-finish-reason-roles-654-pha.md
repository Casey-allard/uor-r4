# #682 — api: Chat Completions token usage + finish_reason + roles (#654 phase C)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T23:44:59Z
- **Updated:** 2026-10-09T14:17:08Z
- **Closed:** 2026-08-14T00:24:36Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/682
- **Merged:** 2026-08-14T00:24:36Z
- **Merge commit:** acc1425b7a25c2bd85a9c1289fed245dee814cd9
- **Base / Head:** main / UOR-Foundation:i654c-openai-chat-conformance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #681 (phase C of #654). **Stacked on #680** (phase B) — the queue merges #680 first (MERGE strategy preserves the shared commit, so this PR lands as just the phase-C change). Chat Completions conformance, all in `src/server.rs`.

## Changes
- **Token usage from the selected serving tokenizer** — `count_serving_tokens` counts with the loaded compiled-artifact tokenizer (exact, from a byte-sized buffer), replacing the whitespace-word estimate for `prompt_tokens`/`completion_tokens`. On the served path a tokenizer is available (it produced the completion); the defensive fallback never substitutes a word count.
- **Truthful `finish_reason`** — `completion_finish_reason` returns `length` when the served completion reached the effective token budget (the smaller of the requested `max_tokens` and the 256 server cap), otherwise `stop`. Abstentions and hard failures remain the declined/error path (unchanged).
- **Explicit roles** — `flatten_chat_prompt` supports `system`, `developer` (system-equivalent, per the pinned spec), `user`, and `assistant`; an unsupported role (`tool`/`function`) **fails closed** with the error envelope (`param: messages`) instead of being silently accepted.

## Scope boundaries
SSE streaming (phase D), `/v1/responses` (E), official-SDK smoke tests (F), and multimodal/content-part unions + tool calls (out of profile) are later work. The `#248` cascade and native `/api/*` routes are untouched.

## Tests (`src/server.rs` mod tests, pure-function style)
`finish_reason` boundaries including the server cap; role flattening (single-user verbatim, multi-role labels incl. `developer`, unsupported-role fail-closed).

## Verification (local, pre-push)
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clean)
- `cargo build --target wasm32-unknown-unknown --lib` (clean)
- Server unit tests (lib+bin): 36+6 pass incl the 3 new phase-C tests
- R1–R5; `check_claim_wording.py` passes. No new dependencies.

_(Local `bdd.rs` teacher-parity failures are the known stale-weights environment issue, unrelated; those arms vacuous-skip in CI.)_

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

