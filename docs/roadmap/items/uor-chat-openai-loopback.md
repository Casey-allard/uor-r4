---
id: uor-chat-openai-loopback
title: Serve the native uor-chat engine through a secure OpenAI-compatible loopback API
milestone: M7
area: serving
status: done
open_to: lab
needs_gpu: false
depends_on: []
done_class: host
scope: [crates/uor-r4-integer/src/openai.rs, crates/uor-r4-integer/src/loopback.rs, crates/uor-r4-integer/src/serving_root.rs, crates/uor-r4-integer/src/stack/chat.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/bin/uor-chat.rs, crates/uor-r4-integer/tests/openai_loopback.rs, crates/uor-r4-integer/README.md, web/studio/verify_studio_worker.mjs, docs/loop/tasks/uor-chat-openai-loopback/**, docs/products/web-client.md, docs/roadmap/items/uor-chat-openai-loopback.md, docs/roadmap/issues.json, PLAN.md]
---

## Why it matters

The owner wants the AI Webstudio to use the live native `uor-chat` engine through a real conversational HTTP interface, with live token metrics and truthful OpenAI-compatible failures. The native stack chat currently has only a CLI and no supported local HTTP transport.

## What is missing

`IntegerStackModel` + `StackChat` has no shared per-token callback or OpenAI-compatible loopback service. The Studio cannot discover the native artifact, send multi-turn JSON/SSE requests, read exact usage/timing, or distinguish implemented routes from unsupported OpenAI families.

## Done when

- One shared `StackChat` stepping path drives existing batch replies and per-token streaming with identical IDs/text and unchanged stop behavior.
- Loopback-only HTTP serves health, model list/detail, chat completions, legacy completions, Responses, and an exact capability matrix. JSON and SSE expose exact usage plus integer elapsed/TTFT/token-rate metrics; unsupported routes, parameters, and content parts fail with OpenAI-shaped 400/404/413/429/501 errors.
- The host enforces exact Host/Origin/token checks, bounded requests, sequential generation, disconnect aborts, and safe optional same-origin Studio assets. A per-launch token is never logged or persisted. No public bind or production publish occurs.
- `uor-chat --stack … --tokenizer … --serve 127.0.0.1:<port>` loads the same artifact as the REPL without spawning the CLI or linking the retired router. Existing CLI behavior and native/WASM parity remain intact.
- Synthetic protocol tests and a labeled already-local non-discrete stack smoke run report exact artifact identity, hash, measured metrics, and limitations without a useful-chat claim. Fresh exact-head review, green GitHub checks, protected queue delivery, and fresh-main verification pass.

## How to start

Use `docs/loop/tasks/uor-chat-openai-loopback/kimi-1/ORDER.md`. Write batch-vs-stream parity first, then the schema/transport tests. Use standard-library TCP and existing serde crates only. Keep the Studio composer and image decomposition in their own follow-up units.

## Rules that apply

- No new external runtime dependency, model/training change, transformer path, CLI stdout scraping, or retired-router linkage.
- No fabricated usage, float timing, estimated tokens, placeholder success, silent truncation, ignored content parts, or unsupported image capability.
- Serving stays on the user's device; no GPU/FIT sheet is required for finite host tests or product tok/s measurement. Model-quality qualification is not claimed.
- Production publication, #3500 typed migration, #3569 MCP behavior, and later multimodal qualification remain separate.

## Sources

- The owner's 2026-11-02 Studio conversational-serving request in `PLAN.md`.
- `crates/uor-r4-integer/src/stack/chat.rs`, `src/bin/uor-chat.rs`, and `crates/uor-r4-workbench/src/http.rs`.
- `docs/superpowers/specs/2026-11-02-studio-ide-harness-design.md` and the delivered Studio baseline in #3593.
