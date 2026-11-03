# DESIGN: Studio conversational composer and native metrics

## Boundaries

The live page stays plain HTML/CSS/JavaScript. The separate Preact library remains unmounted and #3500 remains its migration track. This unit changes browser input, API transport, and presentation only; the merged native Rust service remains the engine boundary unless an integration test proves a concrete defect.

## Composer controller

`createComposerInput({ textarea, onSend, onFiles, getHistory, onHistoryChange })` attaches keyboard, paste, drop, and picker behavior without owning generation state. Enter prevents its default and calls the existing guarded Send path; Shift+Enter keeps native newline insertion. Enter is ignored while `isComposing` or keyCode 229 is active, preserving IME confirmation. A bounded session history stores complete raw prompts, never content-truncated. ArrowUp at the first line or empty input recalls older entries; ArrowDown at the last line recalls newer entries and restores the draft. Recalled text replaces the buffer only through the controller, with explicit caret placement and a return-to-draft path.

Long text paste uses native textarea behavior and never sends automatically. Text/source files from clipboard, drop, or picker are read fully with `File.text()` under explicit per-file/total limits. Binary, oversize, duplicate, and read failures produce visible errors and no attachment. Chips and removal remain owned by the existing `window.__studioUi` attachment API. Local WASM displays that attachment context is ignored; native/custom APIs serialize accepted text context only when Send is pressed.

## API and metrics

The API parser accepts JSON and SSE independently of transport chunking, including CRLF, split `data:` lines, multiple events per chunk, final usage, additive native `uor` timing, and `[DONE]`. It propagates aborts and preserves OpenAI-shaped and non-JSON errors. No estimated token counts are substituted.

`createStreamMetrics()` consumes `{usage?, uor?, elapsedNs?, chars?}` and exposes one formatter/state for live and final rendering. Native completion IDs and integer nanoseconds yield exact completion count, elapsed, TTFT, and tokens/s-milli; final usage supplies prompt/completion/total. Other providers show elapsed/characters until exact usage arrives, then exact usage, or `usage unavailable`. Finish/stop and failures remain visible without pretending successful usage.

## Native source and privacy

`window.__UOR_NATIVE_BOOTSTRAP__` registers a same-origin OpenAI-compatible source with `temperature: 0` and `ephemeral: true`. The bootstrap token stays in memory only. Ephemeral credentials are excluded from localStorage, exported sessions, logs, screenshots, and diagnostics. Session text and attachments metadata reload only within the existing privacy contract; image blobs and image input are deferred to the later R4 decomposition unit.

## Verification

Write failing tests before implementation for keyboard/IME/history/caret behavior, long paste and explicit file failures, duplicate/removal, SSE boundaries/usage/errors/abort, metrics, native bootstrap persistence, and the removed 120,000-character silent slice. Run all existing Studio tests, then real built-in-browser flows against the merged loopback server: two turns, recall/edit/recall, multiline/long paste, text attachment, streaming metrics, Stop, excessive-context error, embeddings 501, and reload-safe text with no unexpected external request. Deliver through exact-head review and the protected queue, then verify fresh main.
