# Studio Web Worker Connection to `uor-r4-integer` (`serving-wasm`) (`m7-studio-connect-wasm-engine`, Issue `#2868`)

- **Date:** 2026-10-22
- **Lab:** `antigravity`
- **Issue:** `#2868` (parent milestone `#2035` — M7 Distribution: capability API, WASM runtime, Pages Studio)
- **Status:** Studio model worker wired and verified on the `mk-synthetic-stack` (`UORLUT01`) stand-in; item `#2868` remains `status: ready` until page-level UI CDN scripts/fonts (`Google Fonts`, `marked`, `KaTeX`, `pdf.js`, `three.js`, `Monaco`) are self-hosted and a qualified M4 `UORLUT01` model artifact exists; GitHub Pages deploy remains frozen per D36.

## 1. Summary

Connects [`web/studio/assets/js/uor_model_worker.js`](../../../web/studio/assets/js/uor_model_worker.js), [`web/studio/r4_worker.js`](../../../web/studio/r4_worker.js), and [`web/studio/index.html`](../../../web/studio/index.html) to the `crates/uor-r4-integer` (`--features serving-wasm`) WebAssembly exports (`init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `allocEvents`, `leakedModels`), replacing the retired `uor_r4_wasm_router.js` worker bindings.

1. **Worker Protocol (`uor_model_worker.js`):**
   - `prewarm` -> initializes `../../pkg/uor_r4_integer.js` (`init()`) and emits `{ action: "runtime_loaded", engine: "uor-r4-integer/serving-wasm", ready, capabilities }`.
   - `load_model` -> validates the 16-byte `UORLUT01` magic + length prefix and embedded JSON header (`uor-r4.lut-stack/1`), calls `loadStack(bytes)` and `saveSession(handle)` for the initial blank state, queries `weightsPerToken(handle)`, `allocEvents()`, and `leakedModels()`, and emits `{ action: "model_loaded", capabilities, bytes }` with `status_label: "synthetic stand-in"` whenever `header.source.synthetic === true` and `tokenizer_label: "UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)"`.
   - `generate` -> restores the conversation's `UORSTATE` checkpoint via `restoreSession(handle, bytes)`, steps prompt tokens via `step(handle, tok)`, generates up to `max_new_tokens` via `step(handle, tok)` + `tickClock(1n)`, verifies zero steady-state heap allocations (`allocDelta === 0`), guards against mid-generation session mutation via `activeJob` and per-conversation epoch counters, snapshots the updated session via `saveSession(handle)`, and emits `generate_complete`.
   - `export_session` / `import_session` / `reset_session` / `close_session` -> maps directly onto `saveSession(handle)` and `restoreSession(handle, bytes)`, with `activeJob` guards refusing session close/reset/export/import while a generation job is running.
2. **Truthful Capability & D36 Status Reporting (`web/studio/index.html`):**
   - `d36EngineStatusBanner` and `sidecarD36Status` state `Pre-alpha (0/8 milestones complete; A8 preflight checkpoint-0 admission P_succ = 35.1% in docs/labs/address-cells-preflight-2026-10-19/README.md:71; D11 decode tok/s on a qualified model is unmeasured)`, removing the unsourced `"35.1% tok/s vs 50% bar"` text, and keep GitHub Pages deployment frozen per D36.
   - `nativeCapabilitiesPanel` displays only fields present in the `UORLUT01` header and WASM exports (`status_label`, `schema`, `vocab`, `width`, `heads`, `context`, `tokenizer_label`, `weights_per_token`, `alloc_events`, `leaked_models`).
   - External Wikipedia and DuckDuckGo search calls (`fetchLiveWebKnowledge` / `toggleWebSearch`) are disabled so no user prompt or model input leaves the browser. Page-level UI scripts and fonts (`Google Fonts`, `marked`, `KaTeX`, `pdf.js`, `three.js`, `Monaco`) still load from external CDNs and are scoped outside the model worker path.

## 2. Verification Receipts

### 2.1 Reference Harness (`docs/wasm-serving/verify.mjs`)

```json
{"handle":0,"wpt":15968,"run1":[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"stateBytes":9782,"cont":[30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"refTail":[30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"reloadIdentical":true,"run1MatchesRef":true}
```

### 2.2 Automated Studio Worker Verification (`node web/studio/verify_studio_worker.mjs`)

```json
{"verdict":"ALL STUDIO WORKER CHECKS PASSED","engine":"uor-r4-integer/serving-wasm","statusLabel":"synthetic stand-in","tokenizerLabel":"UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)","weightsPerToken":15968,"tokens32":[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30],"checkpointBytes32":9782,"checkpointBytes16":5366,"saveRestoreBitIdentical":true,"midGenCloseResetGuarded":true,"trapSelfTestHits":6,"remoteRequests":0}
```

### 2.3 Static WASM Instruction Audit (`scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`)

```text
TAP version 13
1..7
# Artifact: web/studio/pkg/uor_r4_integer_bg.wasm
# Artifact SHA-256: 656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356
# Artifact bytes: 381734
# Scope: direct calls followed transitively; allowlisted callees, imported callees and indirect call targets are not audited
ok 1 - loadStack: mul=0, div=0, float=0, unclassified=0 (102 instructions in 4 fn(s); 3 allowlisted and 1 imported callee(s) not audited; 0 indirect call site(s))
ok 2 - step: mul=0, div=0, float=0, unclassified=0 (23962 instructions in 59 fn(s); 19 allowlisted and 1 imported callee(s) not audited; 2 indirect call site(s))
ok 3 - saveSession: mul=0, div=0, float=0, unclassified=0 (146 instructions in 5 fn(s); 5 allowlisted and 1 imported callee(s) not audited; 0 indirect call site(s))
ok 4 - restoreSession: mul=0, div=0, float=0, unclassified=0 (128 instructions in 6 fn(s); 3 allowlisted and 1 imported callee(s) not audited; 0 indirect call site(s))
ok 5 - weightsPerToken: mul=0, div=0, float=0, unclassified=0 (122 instructions in 6 fn(s); 3 allowlisted and 1 imported callee(s) not audited; 0 indirect call site(s))
ok 6 - stack::IntegerStackSession::step: mul=0, div=0, float=0, unclassified=0 (22924 instructions in 48 fn(s); 12 allowlisted and 0 imported callee(s) not audited; 0 indirect call site(s))
ok 7 - stack::stack_argmax: mul=0, div=0, float=0, unclassified=0 (161 instructions in 1 fn(s); 0 allowlisted and 0 imported callee(s) not audited; 0 indirect call site(s))
```

## 3. Research Notebook Entry (Six-Column Record)

| Date | Mechanism / Line | Question / Hypothesis | Artifact & Seed | Result | Next Step |
|---|---|---|---|---|---|
| 2026-10-22 | M7 Studio WASM worker wiring (`#2868`, `#2035`) | Does wiring `web/studio/assets/js/uor_model_worker.js` to `crates/uor-r4-integer` (`serving-wasm`: `loadStack`, `step`, `saveSession`, `restoreSession`) reproduce the exact 32-token trajectory and mid-stream `saveSession`/`restoreSession` continuation of `docs/wasm-serving/verify.mjs` with zero remote model calls, mid-generation session guards, and `7/7 PASS` on `audit_zero_matmul_wasm.py`? | `uor_r4_integer_bg.wasm` (`381,734 B`, `sha256:656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356`), `synthetic-stack.lut` (`469,252 B`, seed `5`), `web/studio/verify_studio_worker.mjs` | **Confirmed (model worker path on synthetic stand-in):** `tokens32` (`[38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, ...]` from token `0`), `weightsPerToken=15968`, `checkpointBytes32=9782`, `checkpointBytes16=5366`, 16+16 split continuation bit-identical (`saveRestoreBitIdentical: true`), `midGenCloseResetGuarded: true`, `trapSelfTestHits: 6`, `allocDelta=0`, `remoteRequests=0`, and `7/7 PASS` (`mul=0, div=0, float=0, unclassified=0`) on `audit_zero_matmul_wasm.py --serving --call-graph --tap`. | Self-host page-level UI CDN scripts/fonts, verify three-host parity (`uor-r4-integer`, `uor-chat`, Studio worker) on a qualified M4 `UORLUT01` model artifact, and keep GitHub Pages deploy frozen per D36 until then. |
