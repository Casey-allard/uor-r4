---
id: studio-conversational-composer
title: Give Studio a conversational composer, text attachments, and native metrics
milestone: M7
area: serving
status: done
open_to: lab
needs_gpu: false
depends_on: [uor-chat-openai-loopback]
done_class: host
scope: [web/studio/index.html, web/studio/index.css, web/studio/README.md, web/studio/assets/js/**, docs/products/web-client.md, docs/loop/tasks/studio-conversational-composer/**, docs/roadmap/items/studio-conversational-composer.md, docs/roadmap/items/uor-chat-openai-loopback.md, docs/roadmap/issues.json, PLAN.md]
---

## Why it matters

The owner wants the AI Webstudio's live chat window to behave like an ordinary LLM chat: Enter sends, Shift+Enter adds a newline, ArrowUp/Down recalls prompts with draft preservation, long text pastes and text/source files are accepted, and real tokens/s and usage are visible. The native `uor-chat` loopback service delivered in #3606 now supplies the actual engine and exact metrics.

## What is missing

The live plain-JS composer does not provide complete keyboard/history behavior, explicit text-file paste/drop/picker handling, or one truthful streaming-metrics path. Its native same-origin bootstrap is not registered as an ephemeral model source, and existing editing/API behavior can lose or silently truncate input. Image decomposition is a separate follow-up; this unit must not imply visual understanding.

## Done when

- Enter sends, Shift+Enter inserts a newline, and IME confirmation never sends. ArrowUp/Down recalls submitted raw prompts with multiline/caret behavior and restores the in-progress draft. Existing send readiness and Escape/Stop behavior remain intact.
- Long text pastes are preserved without silent truncation. Text/source files can be attached by paste, drop, or multiple-file picker with visible per-file/total limits, duplicate/removal handling, and no send-on-paste. Local WASM explicitly ignores attachment context; native/custom APIs receive text context only after explicit Send.
- JSON/SSE parsing handles CRLF and arbitrary chunk boundaries, aborts, non-JSON errors, and OpenAI error envelopes. Native usage and additive integer timing produce live completion tokens, tok/s, TTFT, elapsed time, prompt/completion/total usage, and finish reason; other providers distinguish character-only or unavailable usage.
- The same-origin native bootstrap registers a temperature-zero ephemeral source. Its per-launch token is never written to localStorage, logs, screenshots, or exported sessions. Session text remains reload-safe without persisting that credential.
- Focused unit/Node tests, all previous Studio checks, real loopback browser flows, repository gates, fresh exact-head review, green GitHub checks, protected queue delivery, and fresh-main verification pass. No useful-chat, production, M4/M7, image, or multimodal capability is claimed.

## How to start

Use `docs/loop/tasks/studio-conversational-composer/kimi-1/ORDER.md` and the delivered native API in #3606. Write failing composer and SSE/metrics tests first, then implement the plain-JS controller and native source without adding a runtime dependency or mounting the separate typed library. Keep R4 image decomposition for its own item.

## Rules that apply

- Preserve existing engine, tokenizer, model artifacts, non-model workflows, and the typed-migration track in #3500. #3569 remains open for its full IDE/MCP acceptance scope.
- No new runtime dependency, production publish, paid provider, silent truncation, fake token metrics, persisted launch token, or unsupported image success.
- Mark the delivered `uor-chat-openai-loopback` dependency done when this unit completes; do not reopen or widen its native transport implementation unless a concrete integration defect requires it.

## Sources

- The owner's conversational Studio request in `PLAN.md` and the delivered native service in #3606 / #3595.
- `web/studio/index.html`, `web/studio/assets/js/studio_api.js`, `studio_ui.js`, `model_sources.js`, and `crates/uor-r4-integer/src/openai.rs` / `loopback.rs`.
- #3500 and #3569 remain separate acceptance tracks.
