# ORDER: Studio conversational composer, text attachments, and native metrics

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569), with follow-up [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608), dependent on the delivered [uor-chat-openai-loopback](../../../../roadmap/items/uor-chat-openai-loopback.md) (#3595). Base: `af6c27ad78f8fdde2babbf084bd10bbaeba1a45e`.

## Goal

Make the live plain-JS Studio behave like an ordinary LLM chat window while serving the actual native `uor-chat` engine: complete keyboard/history behavior, long text and explicit text-file input, exact streaming usage/timing, and an ephemeral native source. Do not claim useful conversation or visual understanding.

## Design

[DESIGN.md](DESIGN.md). One composer controller owns editing/history; existing Studio state owns generation. One SSE/JSON metrics path distinguishes exact native tokens from character-only or unavailable usage. Browser and server text limits are explicit; image decomposition remains a separate unit.

## Files in scope

- `web/studio/index.html`, `index.css`, `README.md`, and `assets/js/**`, including new `composer_input.js` and `stream_metrics.js` plus focused tests.
- This task folder, `docs/roadmap/items/studio-conversational-composer.md`, generated roadmap data, and narrowly scoped `docs/products/web-client.md` updates if delivered behavior requires them.
- `docs/roadmap/items/uor-chat-openai-loopback.md` only to mark the already-delivered dependency done.

## Done when

- Enter sends, Shift+Enter inserts a newline, and IME confirmation never sends. ArrowUp/Down recalls raw submitted prompts at the correct multiline boundaries and restores the draft. Existing send readiness and Escape/Stop remain intact.
- Long pastes are not silently sliced. Text/source files work through paste, drop, and multiple-file picker with visible per-file/total limits, duplicate/removal behavior, and no send-on-paste. Local WASM explicitly ignores context; APIs receive text context only after explicit Send.
- JSON/SSE handles CRLF/arbitrary chunks, final usage, additive native timing, aborts, non-JSON failures, and OpenAI errors. Native live completion tokens, tok/s, TTFT, elapsed, prompt/completion/total usage and finish reason are truthful; other providers distinguish character-only/unavailable usage.
- The native bootstrap source uses temperature zero and is ephemeral. Its token never reaches localStorage, logs, screenshots, or exported sessions; session text reloads safely.
- Focused and previous Studio tests, real native browser flows, native/WASM checks where touched, repository gates, fresh exact-head review, green GitHub checks, protected queue, and fresh-main verification pass.
- The PR closes #3608 and marks #3595's delivered roadmap item done. #3500 and #3569 remain open. No production publish or M4/M7/useful-chat/vision claim.

## Stop and report conditions

Stop before adding a runtime dependency, mounting the separate typed library, changing model/training/tokenizer semantics, weakening server admission/body limits, silently truncating input, persisting/logging launch tokens, enabling images without the later integer R4 decomposition unit, publishing production, or editing `.github/**` or the owner checkout. Merge forward from fresh `main`; never force-push. Keep the existing Rust and npm runtimes and remove generated output safely.
