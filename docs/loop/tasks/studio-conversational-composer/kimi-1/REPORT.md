# REPORT: Studio conversational composer, text attachments, and native metrics

Lab: kimi. Item: [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608), dependent on the delivered [uor-chat-openai-loopback](../../../../roadmap/items/uor-chat-openai-loopback.md) (#3595). Order: [ORDER.md](ORDER.md). Design: [DESIGN.md](DESIGN.md). Original base: `af6c27ad78f8fdde2babbf084bd10bbaeba1a45e`; implementation head: `f2c3b84e4a68fc375ca2e7718dd241e934f4afa0`; current-main integration base is recorded in the final external verification and PR.

## What did not work

- The first RED run confirmed missing composer/metrics/native-registration modules, a real 120,000-character silent attachment slice, missing SSE usage/native timing, and character-only final metrics. The new controller needed its own in-memory history rather than relying on synchronous persistence callbacks. Repeated multiline ArrowUp required continued history navigation while history is active; ArrowDown remains last-line-bound.
- The first metrics integration overrode exact server elapsed time with later client elapsed time. Server `uor` values now win at completion; client nanoseconds are used only for live progress or when server timing is unavailable.
- The page-hook test factory initially lacked the real metrics dependency, and the new native-label regression exposed stale UI type declarations and incomplete mocks. The real formatter was injected, `updateStatus` and attachment metadata were declared, and mocks were completed rather than casting around type errors.
- Native bootstrap initially displayed generic external-provider text. Separate native branches now label the empty state, privacy chip/footer, source-change toast, composer mode, and attachment note as local native loopback.
- An empty-state ternary edit introduced a module parse error before `boot()`. The actual inline module was syntax-checked and corrected; temporary title/payload diagnostics were removed. Native registration then worked normally.
- The generic API default `max_tokens=768` caused exact 780/786/933-position 413 errors against the 192-position artifact even for `Hi`. Ephemeral native configs now omit that field so the server's default of 16 applies; ordinary providers retain their default. This was a client default defect, not accumulated history.
- The Stop-proxy browser attempt did not register its bootstrap and is **NOT_RUN**, not a pass. The API mid-stream abort regression passes, but this REPORT does not claim a completed browser Stop scenario. The proxy and native servers and the browser tab were stopped.
- Independent review found false exact-token metrics for empty ID arrays, composition Enter being default-prevented, remaining native/external label mismatches, malformed SSE and in-band errors completing successfully, retained-attachment overflow, storage-dependent native boot, and draft leakage across sessions. Regressions now cover each: empty IDs and partial usage stay character-only, final rates recompute, composition defaults are preserved, every native surface is local, malformed SSE errors surface, retained totals include duplicate replacement, storage denial does not block native boot, and reset restores the selected session's draft. Empty/exhausted history keys no longer swallow native caret movement. #3608 is now marked done consistently with its ORDER.
- The second review found five narrower defects: partial usage/rate-only payloads could be presented as exact and render `null`; submitted prompts persisted as unsent drafts and history edits could overwrite drafts; some native labels still said WASM/API/custom; ArrowUp at the oldest history entry was swallowed; and SSE errors could call `onError` twice. Repairs now require complete usage before exact labeling, clear submitted drafts while preserving in-progress drafts, restore the initial session draft, use consistent native labels, preserve native caret behavior at the oldest entry, and report each stream error once. The repair pass also hoisted composer reset initialization, removed a later legacy `#envEngine` overwrite, and corrected the group-title test source.
- The third review found that the oldest-entry fix accidentally let ArrowUp recall history from the middle of a multiline draft, and that native active-model/tokenizer fields still described local WASM. Entering history now requires the first-line boundary, while continued navigation and oldest-entry native caret behavior remain intact. Native model, tokenizer, privacy, and active-model card fields now agree. Exact live completion counts remain exact, but partial final usage explicitly says `prompt/total usage unavailable`. Historical browser evidence is labeled as pre-second-repair.
- No useful-chat, production, M4/M7, visual-understanding, embedding, or multimodal capability is claimed. Image decomposition remains the next unit.

## What changed

- `composer_input.js` implements IME-safe Enter (including keyCode 229), native Shift+Enter, complete raw prompt history with bounded count and no content truncation, ArrowUp/Down recall with draft restoration and multiline boundaries, long text paste, and paste/drop/multiple-picker file channels. The existing guarded Send path remains authoritative and records prompts only after a successful start, per session.
- Text/source files use `File.text()` with explicit per-file/total limits, supported text types, NUL/binary refusal, and visible errors. Nothing sends on paste. Existing chips replace duplicates and remove entries without slicing content. Local WASM explicitly ignores context; native/custom APIs receive it only after explicit Send.
- `studio_api.js` parses JSON/SSE across CRLF/LF and arbitrary chunks, requests usage, emits empty EOS token events for exact live counts, returns final usage/native timing/finish reason, preserves partial text on abort, and surfaces OpenAI/non-JSON errors.
- `stream_metrics.js` displays live native completion IDs and measured tok/s, TTFT and elapsed time, then exact prompt/completion/total usage and server timing. Character-only/unavailable states never masquerade as token usage; missing final prompt/total usage is explicit even when live completion IDs are exact.
- `model_sources.js` registers a same-origin temperature-zero native source whose token and provider ID stay out of localStorage. It remains active in memory and is not exported with sessions. `index.html` selects it at boot and wires real streaming metrics through the existing generation lifecycle.
- The separate typed library remains unmounted. #3500 and #3569 remain open. No runtime dependency, production publication, model/training/tokenizer semantic change, or new server implementation was added.

## Measurements and checks

- Focused RED→GREEN: 16 composer-controller tests, 6 streaming tests, 7 metrics tests, 3 native-bootstrap tests, and native privacy/label regressions. The 120,001-character file/prompt cases preserve every character; per-file/total/binary failures are explicit. Empty-ID/non-native usage, malformed SSE, storage-denied boot, draft reset, retained totals, and assistant-role cases are covered.
- Complete Studio suite: **31 files / 266 Vitest tests PASS**. Node protocol/source tests: **28 PASS**. Typecheck and Vite build: PASS. Studio worker, page static/runtime traps, and desktop checks: PASS, with zero unexpected external requests.
- Historical pre-second-repair real native browser flow against the retained dual-c131 artifact (these measurements predate the second repair pass and are not a final-head browser rerun) (`9e80cc6a894ed50c7dc7d0393e2d3b22f74a7230b52d8c635736202a2afa8694`, schema `uor-r4.lut-stack/5`, context 192):
  - First reply `I.`: **3 completion / 18 prompt / 21 total tokens**, 313.6 tok/s, TTFT 8.966 ms, elapsed 9.568 ms.
  - ArrowUp restored `Hi`; Shift+Enter added a second line; the second `I.` reply: **3 / 38 / 41**, 167.7 tok/s, TTFT 17.170 ms, elapsed 17.891 ms.
  - `composer-small.txt` attached through the real file dialog and displayed a local-context chip. The attachment-backed `I.` reply: **3 / 94 / 97**, 88.4 tok/s, TTFT 33.465 ms, elapsed 33.955 ms.
  - A longer prompt returned `I.` with **3 / 153 / 156**, 67.7 tok/s, TTFT 43.726 ms, elapsed 44.293 ms. Exact 120,001-character preservation is covered by unit/DOM tests; no browser 120,001-character metric is claimed.
  - Excess-context requests returned visible 413 errors, and `/v1/embeddings` returned the expected 501. Session text and the native source survived reload. The launch token was not printed, persisted, or included in screenshots.
- Existing native release and runtime were reused through symlinks; no second npm dependency tree or Rust target was created. The real server and browser tab were stopped.

## Checks run at the head

The original implementation and first review repairs are committed through `2b15f8a3c6fa4500db782d6f14396b7cf16e49d5`; the second repair pass is committed separately. The final exact head, repository gates, claim wording, roadmap check, fresh exact-head review, GitHub checks, protected queue, and fresh-main verification are external verification records and will be posted on the PR only after they exist. They are not claimed in this REPORT.

## Skills used

Isolated worktrees, plan execution, test-driven development, systematic debugging, solution integrity, modern web forms/accessibility/IME guidance, plain-JS composition, OpenAI/SSE handling, real built-in-browser verification, and native loopback smoke checks. No GPU, paid compute, production publish, or model-quality evaluation was used.

## Still open

- The delayed-proxy browser Stop scenario is NOT_RUN; only the automated mid-stream abort regression is verified here. A later focused browser check may use a truthful slow-stream fixture without changing the native default.
- R4 integer image decomposition and experimental image input are the next separate unit. Standard `image_url` remains unsupported. Production publication, useful-chat qualification, M4/M7 completion, typed migration, and MCP behavior remain outside this unit.
- #3500 and #3569 remain open. The delivered `uor-chat-openai-loopback` dependency is marked done in this PR.

## Plain summary

The live Studio now behaves like an ordinary chat window: Enter/Shift+Enter, prompt recall with drafts, long text and text-file input, real native streaming metrics, and reload-safe sessions. It serves the actual `uor-chat` engine locally and reports unsupported operations truthfully. No useful-chat or visual-understanding claim is made.
