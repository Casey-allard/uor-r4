# ORDER: Native uor-chat OpenAI-compatible loopback service

Lab: kimi. Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md) (#2868), with follow-up [uor-chat-openai-loopback](../../../../roadmap/items/uor-chat-openai-loopback.md) (#3595). Base: `de21884d104391b3dfc29e569ca9c33e85f61057`. The follow-up issue remains open until the full unit is delivered.

## Goal

Serve the existing non-discrete `IntegerStackModel` + `StackChat` path through a truthful local OpenAI-compatible API so the next Studio unit can provide a real native chat workflow, exact usage and tokens/s, and explicit failures for unsupported endpoints.

## Design

[DESIGN.md](DESIGN.md). One shared generation implementation drives CLI batch, JSON and SSE. Standard-library TCP and existing serde crates only; no retired router, CLI subprocess, external runtime, or production publication.

## Files in scope

- `crates/uor-r4-integer/src/openai.rs`, `src/loopback.rs`, `src/serving_root.rs`, `src/stack/chat.rs`, `src/stack/tests.rs`, `src/bin/uor-chat.rs`, `tests/openai_loopback.rs`, and `README.md`.
- This task folder, `docs/roadmap/items/uor-chat-openai-loopback.md`, generated roadmap data, `PLAN.md`, and narrowly scoped `docs/products/web-client.md` updates if the delivered behavior requires them.
- Test-only `web/studio/verify_studio_worker.mjs`: accept the already-shipped `normalizeBaseUrl` API import so the native/WASM worker verification can run; no Studio runtime edit.

## Done when

- Existing `reply_history` remains byte-identical while a callback-based shared path emits every generated ID and its tokenizer bytes. Multi-turn and EOS/cycle/max-cap parity is tested.
- `GET /healthz`, `GET /v1/models`, `GET /v1/models/{id}`, `POST /v1/chat/completions`, `POST /v1/completions`, `POST /v1/responses`, and `GET /v1/capabilities` work against a loaded native artifact with JSON and real SSE where applicable.
- Exact usage, integer elapsed/TTFT/token-rate metrics and stop mappings are returned. Unsupported models/routes/parameters/content parts return truthful OpenAI-shaped 400/404/413/429/501 errors; unsupported OpenAI families are never placeholder successes.
- Loopback-only binding, exact Host/Origin/token admission, bounded headers/bodies, sequential generation, disconnect aborts, and safe optional same-origin Studio serving are exercised. The per-launch token is not logged or persisted.
- `uor-chat --stack … --tokenizer … --serve 127.0.0.1:<port> [--studio-root web/studio]` preserves existing REPL behavior. Synthetic protocol tests, existing integer/CLI/discrete/WASM checks, native/WASM audits, claim/policy gates, fresh exact-head review, green GitHub checks, protected queue and fresh-main verification pass.
- An already-local non-discrete stack + tokenizer smoke run records schema/hash, measured metrics and limitations without claiming useful conversation or M4/M7 completion.

## Stop and report conditions

Stop before adding a runtime dependency, changing model/training/tokenizer semantics, spawning/scraping the CLI, linking the retired router, widening the discrete path, adding floats/matmul/squashing, binding non-loopback interfaces, weakening Host/Origin/token/body limits, publishing production, persisting/logging tokens, editing `.github/**` or the owner checkout, implementing the Studio composer or image decomposition in this unit, or claiming trained chat/visual understanding/M4/M7 completion. Merge forward from fresh `main`; never force-push. Both #3500 and #3569 remain open; this PR says `References #3595` and, where appropriate, references rather than closes them.
