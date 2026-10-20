# #680 — api: OpenAI error envelope + /v1/models fields + fail-closed params (#654 phase B)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T23:34:32Z
- **Updated:** 2026-10-09T14:17:08Z
- **Closed:** 2026-08-13T23:54:22Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/680
- **Merged:** 2026-08-13T23:54:22Z
- **Merge commit:** dc8aece988effb5b797d730677b3452f9ed28e3d
- **Base / Head:** main / UOR-Foundation:i654b-openai-dtos-errors
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #679 (phase B of #654). Builds on the phase-A pin/matrix (#678). Makes the `/v1` OpenAI compatibility surface **truthful** — all in `src/server.rs`; no streaming / `/v1/responses` / usage-accounting yet (phases C–E).

## Changes
- **Standard OpenAI error envelope** `{error:{message,type,param,code}}` (`openai_error_body` / `send_openai_error`), replacing the ad-hoc `{"error":"..."}` string body on the `/v1` routes. `param`/`code` serialize as JSON `null` when absent (present, not omitted) so official SDKs parse errors from the expected shape.
- **`GET /v1/models`** now advertises **only loadable compiled bundles** (`.uor-models/compiled/*/tless_artifacts.bin`), each with every field the pinned `Model` schema requires: `id`, `object`, `created` (bundle mtime), `owned_by`. Empty list when nothing is compiled — an id absent here is not loadable.
- **New `GET /v1/models/{model}`** — agrees with the list; an id absent from the loadable set returns a **404 with the error envelope** (`code: model_not_found`).
- **`VendorChatCompletionsRequest` gains `#[serde(deny_unknown_fields)]`** — an unsupported OpenAI parameter now **fails closed** with the envelope instead of being silently accepted and ignored. Support is never implied by omission. (`/v1` is the OpenAI surface; the dashboard uses native `/api/*`, so this doesn't affect it.)

## Scope boundaries
The model-id ↔ serving-selection scheme and packaged-artifact readiness stay coordinated with **#655**; the **#248** cascade and native **`/api/*`** routes are untouched. Response-body extension namespacing, roles/content-parts, tokenizer usage, SSE, and `/v1/responses` are later phases.

## Tests (`src/server.rs` mod tests, pure-function style)
Error envelope shape (+ `null` param/code), model object required fields, `loadable_models_in` over a temp dir (bundle-only, sorted, mtime-derived `created`), empty-when-absent, and unsupported-parameter rejection.

## Verification (local, pre-push)
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clean)
- `cargo build --target wasm32-unknown-unknown --lib` (clean)
- Server unit tests: lib+bin pass, including the 5 new phase-B tests
- R1–R5; `check_claim_wording.py` gate passes. No new dependencies.

_(The local `bdd.rs` teacher-parity failures are the known stale-weights environment issue on this machine, unrelated to this change; those arms vacuous-skip in CI where there are no weights.)_

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

