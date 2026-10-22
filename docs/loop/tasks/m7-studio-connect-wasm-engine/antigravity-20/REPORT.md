# REPORT: m7-studio-connect-wasm-engine, antigravity-20

Order: [ORDER.md](ORDER.md). Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md) (closes #2868).

## What did not work, first

1. **Retired `uor_r4_wasm_router.js` worker bindings failed on `prewarm`:** Running `node web/studio/verify_studio_worker.mjs` before rewiring [`web/studio/assets/js/uor_model_worker.js`](../../../../../web/studio/assets/js/uor_model_worker.js) failed immediately on `prewarm` with `ERR_MODULE_NOT_FOUND: Cannot find module '.../web/studio/pkg/uor_r4_wasm_router.js'`, because the worker still imported the retired 15 October router bundle (`native_geometric_init`, `native_geometric_create_session`, `native_geometric_generate_step`).
2. **`tickClock` `BigInt` signature and `step` `Uint32Array(3)` return value:** On the first draft of the rewired worker, passing `1` instead of `1n` to `tickClock(us: u64)` threw `TypeError: Cannot convert 1 to a BigInt`, and assigning `tok = wasmModule.step(activeHandle, tok)` without destructuring the `Uint32Array(3)` return tuple `[nextTok, marginQ16, stepCount]` coerced the array object to `0` on subsequent steps. Switching to `wasmModule.tickClock(1n)` and `const [nextTok, marginQ16, stepCount] = wasmModule.step(activeHandle, tok)` resolved both issues.
3. **Checkpoint byte length scales with sequence position:** Asserting `9,782` checkpoint bytes after a 16-token run failed (`got 5366`), because `UORSTATE` serializes only the populated KV cache entries (`276` bytes per step across heads/layers: `5,366 B` after 16 steps and `9,782 B` after 32 steps). Both checkpoint sizes (`checkpointBytes16: 5366`, `checkpointBytes32: 9782`) are now verified in [`web/studio/verify_studio_worker.mjs`](../../../../../web/studio/verify_studio_worker.mjs).
4. **Notebook scope boundary:** Step 7 of [`m7-studio-connect-wasm-engine.md`](../../../../roadmap/items/m7-studio-connect-wasm-engine.md) mentions adding a row to `docs/research/notebook/m4-serving.md`, which lies outside the item's declared `scope: [web/**, crates/uor-r4-integer/**, docs/products/**, docs/labs/m7-studio-connect-wasm-engine/**]`. To satisfy `uor-harness loop pr-check` (`rule_scope`), the six-column research notebook row is recorded in [`docs/labs/m7-studio-connect-wasm-engine/README.md`](../../../../labs/m7-studio-connect-wasm-engine/README.md) and [`docs/products/web-client.md`](../../../../products/web-client.md).

## What changed

- [`web/studio/assets/js/uor_model_worker.js`](../../../../../web/studio/assets/js/uor_model_worker.js): Rewired the Studio Web Worker from the retired `uor_r4_wasm_router.js` interface to `../../pkg/uor_r4_integer.js` (`init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `allocEvents`, `leakedModels`), added `parseUorLutHeader` for `UORLUT01` (`uor-r4.lut-stack/1`) header inspection and `"synthetic stand-in"` labeling, implemented multi-conversation `saveSession`/`restoreSession` checkpoint management, and confirmed zero steady-state heap allocations during `step`.
- [`web/studio/r4_worker.js`](../../../../../web/studio/r4_worker.js): Replaced the retired worker implementation with a forwarding entry point to `./assets/js/uor_model_worker.js`.
- [`web/studio/index.html`](../../../../../web/studio/index.html): Updated `d36EngineStatusBanner` and `sidecarD36Status` (`0/8` milestones, `35.1%` tok/s, Pages deploy frozen per D36), added `#nativeCapabilitiesPanel` displaying only header- and engine-derived capability fields, updated file pickers for `.lut`/`.bin` stack artifacts and checkpoints, and disabled external Wikipedia/DuckDuckGo prompt search (`fetchLiveWebKnowledge` / `toggleWebSearch`).
- [`web/studio/verify_studio_worker.mjs`](../../../../../web/studio/verify_studio_worker.mjs): Added the automated Node verification test that loads `synthetic-stack.lut` through `uor_model_worker.js`, verifies the 32-token trajectory against `docs/wasm-serving/verify.mjs`, verifies 16+16 `saveSession`/`restoreSession` continuation bit-identity, and enforces zero remote network calls.
- [`web/README.md`](../../../../../web/README.md), [`docs/products/web-client.md`](../../../../products/web-client.md), [`docs/labs/m7-studio-connect-wasm-engine/README.md`](../../../../labs/m7-studio-connect-wasm-engine/README.md), [`docs/roadmap/items/m7-studio-connect-wasm-engine.md`](../../../../roadmap/items/m7-studio-connect-wasm-engine.md), and [`docs/roadmap/README.md`](../../../../roadmap/README.md): Recorded build/verification commands, capability and D11 audit receipts, and updated roadmap item status to `done`.

## Measurements

- **`docs/wasm-serving/verify.mjs` reference run (`synthetic-stack.lut`, `469,252 B`, seed `5`):** `handle=0`, `wpt=15968`, `run1=[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30]`, `stateBytes=9782`, `reloadIdentical=true`, `run1MatchesRef=true`.
- **`node web/studio/verify_studio_worker.mjs`:** `verdict="ALL STUDIO WORKER CHECKS PASSED"`, `engine="uor-r4-integer/serving-wasm"`, `statusLabel="synthetic stand-in"`, `weightsPerToken=15968`, `tokens32` bit-identical to `verify.mjs`, `checkpointBytes32=9782`, `checkpointBytes16=5366`, `saveRestoreBitIdentical=true`, `allocDelta=0`, `remoteRequests=0`.
- **`python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap`:** `7/7 PASS` (`loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `stack::IntegerStackSession::step`, `stack::stack_argmax`), `mul=0`, `div=0`, `float=0`, `unclassified=0`, artifact `381,734 B` (`sha256:656fe88cdd5740d8c82da91bbd30e872c6a25122943641105ea07651e526d356`).

## Checks run at the head

- `node web/studio/verify_studio_worker.mjs` (`ALL STUDIO WORKER CHECKS PASSED`)
- `python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap` (`7/7 PASS`)
- `uor-harness loop gates --full` (`8/8 GREEN`, recorded in [`receipts.jsonl`](receipts.jsonl))
- `uor-harness loop report --check` (`PASS`)
- `uor-harness loop pr-check --local` (`PASS`)

## Skills used

- `uor-research-architect`: verified zero-matmul/zero-float WASM serving boundaries and D36 public-status invariants.
- `solution-integrity` & `scope-guard`: kept all changes strictly within `m7-studio-connect-wasm-engine` scope and verified end-to-end worker behavior against `docs/wasm-serving/verify.mjs`.

## Still open

- GitHub Pages deployment (`.github/workflows/deploy.yml`) remains disabled per D36 until a qualified M4 `UORLUT01` model artifact exists and Casey authorizes unfreezing.
- Self-hosting the remaining third-party UI CDN scripts/fonts in `web/studio/index.html` and three-host parity on a qualified model remain tracked under M7 (`#2035`).

## Plain summary

Connected the Studio Web Worker (`web/studio/assets/js/uor_model_worker.js`, `web/studio/r4_worker.js`, and `web/studio/index.html`) to the `crates/uor-r4-integer` (`serving-wasm`) browser engine (`loadStack`, `step`, `saveSession`, `restoreSession`), disabled external prompt search calls, verified 32-token and mid-stream `saveSession`/`restoreSession` parity against `docs/wasm-serving/verify.mjs` via `web/studio/verify_studio_worker.mjs` with zero remote requests, and verified `7/7 PASS` (`mul=0, div=0, float=0`) on `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`.
