# REPORT: m7-studio-connect-wasm-engine, antigravity-21

Order: [ORDER.md](ORDER.md). Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md) (References #2868).

## What did not work, first

1. **Unsourced `"35.1% tok/s vs 50% bar"` status figure (`antigravity-20` finding 1, HIGH):** `web/studio/index.html` carried `"35.1% tok/s vs 50% bar"` at lines 2326 and 2498 and `"35.1% tok/s"` on the model card (`:2773`). Nothing on `main` reports a `tok/s` percentage; `35.1%` is the A8 preflight checkpoint-0 `P_succ` admission rate (`docs/labs/address-cells-preflight-2026-10-19/README.md:71`), and D11 decode `tok/s` on a qualified model is unmeasured. All occurrences in `web/studio/index.html`, `docs/products/web-client.md`, `docs/labs/m7-studio-connect-wasm-engine/README.md`, and `web/studio/verify_studio_worker.mjs` now cite `P_succ = 35.1%` (`docs/labs/address-cells-preflight-2026-10-19/README.md:71`) and state that D11 decode `tok/s` is unmeasured.
2. **Mid-generation `close_session` / `reset_session` race (`antigravity-20` finding 2, MEDIUM):** `web/studio/assets/js/uor_model_worker.js` lacked an `activeJob` guard on `close_session` and `reset_session`. Because `generate` yields every 8 tokens (`setTimeout(..., 0)`), dispatching `close_session` or `reset_session` mid-stream called `restoreSession(activeHandle, initialSessionBytes)` while `generate` was paused and then re-inserted the closed conversation via `sessions.set(convId, ...)` at completion. Both `close_session` and `reset_session` now enforce the `activeJob` guard (`Stop generation before closing or resetting a session.`), `generate` checks per-conversation `sessionEpochs` + `sessions.has(convId)` on every step and before saving, `deleteConversation` and `resetNativeSession` in `web/studio/index.html` refuse while `window.isGenerating` is true, and `web/studio/verify_studio_worker.mjs` tests mid-generation `close_session` and `reset_session` at the step-8 yield boundary.
3. **Page-level third-party UI CDN scripts remain external (`antigravity-20` finding 3, MEDIUM):** While the model Web Worker (`uor_model_worker.js`) makes zero remote calls and external Wikipedia/DuckDuckGo prompt search is disabled, `web/studio/index.html` still loads UI fonts and scripts from external CDNs (`Google Fonts`, `marked`, `KaTeX`, `pdf.js`, `three.js`, and `Monaco` at lines 11–25, 3711, and 5164). Accordingly, `docs/roadmap/items/m7-studio-connect-wasm-engine.md` is kept at `status: ready` (`References #2868`), and all client-side claims are scoped to the **model worker path only**.
4. **Incomplete network traps in `verify_studio_worker.mjs` (`antigravity-20` finding 4, MEDIUM):** The initial verifier stubbed only `fetch`, `XMLHttpRequest`, and `WebSocket`, which would not catch `navigator.sendBeacon` or `EventSource`. `web/studio/verify_studio_worker.mjs` now traps `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `navigator.sendBeacon`, `importScripts`, `Worker`, and `SharedWorker` on both `globalThis` and `globalThis.self`, self-tests all 6 network/script traps (`trapSelfTestHits: 6`) before importing the worker, and statically scans `uor_model_worker.js` for forbidden network tokens.
5. **Stand-in prompt tokenizer (`antigravity-20` finding 5, LOW):** `encodePromptTokens` folds UTF-8 bytes into the vocabulary with `rawBytes[i] % vocabSize` and emits raw integer token IDs. Both `uor_model_worker.js` (`tokenizer_label`) and `web/studio/index.html` now explicitly label the prompt tokenizer as `"UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)"`.
6. **Earlier `antigravity-20` findings:** Retired `uor_r4_wasm_router.js` failed on `prewarm` (`ERR_MODULE_NOT_FOUND`); `tickClock` required `BigInt` (`1n`) and `step` returned `Uint32Array(3)` (`[nextTok, marginQ16, stepCount]`); `UORSTATE` checkpoint byte size scales with sequence length (`5,366 B` at 16 steps, `9,782 B` at 32 steps).

## What changed

- [`web/studio/assets/js/uor_model_worker.js`](../../../../../web/studio/assets/js/uor_model_worker.js): Added `activeJob` guard and per-conversation `sessionEpochs` tracking on `close_session` / `reset_session` and inside `generate`, and added `tokenizer_label: "UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)"` to `capabilities` and `generate_complete`.
- [`web/studio/index.html`](../../../../../web/studio/index.html): Replaced unsourced `"35.1% tok/s vs 50% bar"` / `"35.1% tok/s"` strings with the sourced A8 preflight checkpoint-0 `P_succ = 35.1%` citation (`docs/labs/address-cells-preflight-2026-10-19/README.md:71`) and explicit statement that D11 decode `tok/s` is unmeasured; scoped client-side descriptions to the model worker path only; guarded `deleteConversation` and `resetNativeSession` when `window.isGenerating` is true; surfaced `tokenizer_label` in `nativeCapabilitiesPanel`.
- [`web/studio/verify_studio_worker.mjs`](../../../../../web/studio/verify_studio_worker.mjs): Added traps and 6-probe self-test for `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `navigator.sendBeacon`, and `importScripts` (`trapSelfTestHits: 6`), added the step-8 mid-generation `close_session` / `reset_session` race test (`midGenCloseResetGuarded: true`), and verified static source invariants on `uor_model_worker.js` and `index.html`.
- [`docs/roadmap/items/m7-studio-connect-wasm-engine.md`](../../../../roadmap/items/m7-studio-connect-wasm-engine.md), [`docs/roadmap/README.md`](../../../../roadmap/README.md), [`docs/products/web-client.md`](../../../../products/web-client.md), and [`docs/labs/m7-studio-connect-wasm-engine/README.md`](../../../../labs/m7-studio-connect-wasm-engine/README.md): Kept item status `ready` (`References #2868`) and scoped completion to the model worker path on the synthetic stand-in.

## Measurements

- **Reference harness (`docs/wasm-serving/verify.mjs`):** `handle=0`, `wpt=15968`, `run1=[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30]`, `stateBytes=9782`, `reloadIdentical=true`, `run1MatchesRef=true`.
- **Studio worker verification (`node web/studio/verify_studio_worker.mjs`):** `verdict="ALL STUDIO WORKER CHECKS PASSED"`, `engine="uor-r4-integer/serving-wasm"`, `statusLabel="synthetic stand-in"`, `tokenizerLabel="UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)"`, `weightsPerToken=15968`, `checkpointBytes32=9782`, `checkpointBytes16=5366`, `saveRestoreBitIdentical=true`, `midGenCloseResetGuarded=true`, `trapSelfTestHits=6`, `remoteRequests=0`.
- **Static WASM instruction audit (`python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap`):** `7/7 PASS` (`mul=0`, `div=0`, `float=0`, `unclassified=0`, `381,734 B`, `sha256:656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356`).

## Checks run at the head

- `node web/studio/verify_studio_worker.mjs` (`ALL STUDIO WORKER CHECKS PASSED`, `midGenCloseResetGuarded=true`, `trapSelfTestHits=6`, `remoteRequests=0`)
- `python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap` (`7/7 PASS`)
- `~/.cargo/bin/cargo test --locked -p uor-r4-integer --features serving-wasm` (`165 passed`)
- `~/.cargo/bin/cargo run --locked -p xtask -- roadmap check` (`88 items OK`)
- `uor-harness loop gates --full` (`8/8 GREEN`)
- `uor-harness loop report --check` (`PASS`)
- `uor-harness loop pr-check --local` (`verdict: PASS`)

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`): Applied truthful capability labeling, sourced status citations, and D11 serving verification.
- `solution-integrity` (`~/.gemini/config/skills/solution-integrity/SKILL.md`): Verified the mid-generation session race and the 6-probe network trap self-test before running the worker check.
- `scope-guard` (`~/.gemini/config/skills/scope-guard/SKILL.md`): Kept all edits strictly within the item's declared scope.

## Still open

- Self-hosting the remaining page-level UI CDN scripts and fonts in `web/studio/index.html` (`Google Fonts`, `marked`, `KaTeX`, `pdf.js`, `three.js`, `Monaco`) and bundling a stack-bound tokenizer artifact are still open on `#2868`.
- GitHub Pages deployment (`.github/workflows/deploy.yml`) remains disabled per D36 until a qualified M4 `UORLUT01` model artifact exists and Casey authorizes unfreezing.
- Appending the six-column row to `docs/research/notebook/m4-serving.md` remains open until `m7-studio-connect-wasm-engine.md` includes `notebook:` or `docs/research/notebook/m4-serving.md` in its scope.

## Plain summary

This repair (`antigravity-21`) resolves all four required findings and one low finding from Claude's review of PR `#2978`: it removes the unsourced `"35.1% tok/s vs 50% bar"` text and cites the A8 preflight checkpoint-0 `P_succ = 35.1%` record, guards `close_session` and `reset_session` against mid-generation races (verified at the step-8 yield boundary), keeps `#2868` at `status: ready` (`References #2868`) with client-side claims scoped to the model worker path while UI CDN scripts remain external, traps and self-tests `navigator.sendBeacon`, `EventSource`, `importScripts`, `fetch`, `XMLHttpRequest`, and `WebSocket` in `verify_studio_worker.mjs`, and labels the prompt tokenizer as a UTF-8 byte-mod stand-in.
