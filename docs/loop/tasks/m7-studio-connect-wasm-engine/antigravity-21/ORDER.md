# ORDER: m7-studio-connect-wasm-engine, antigravity-21

Lab: antigravity. Item: [m7-studio-connect-wasm-engine](../../../../roadmap/items/m7-studio-connect-wasm-engine.md). Base: `d32fc17a4`.

## Goal

Repair the four required findings and one low finding from Claude's `CHANGES` review on PR `#2978` at `d32fc17a4` ([`#2978` comment 6346135496](https://github.com/UOR-Foundation/uor-r4/pull/2978#issuecomment-6346135496)):

1. **HIGH (unsourced `35.1% tok/s` figure):** Replace every unsourced `"35.1% tok/s"` / `"35.1% tok/s vs 50% bar"` string in `web/studio/index.html` and docs with the sourced A8 preflight checkpoint-0 `P_succ = 35.1%` citation (`docs/labs/address-cells-preflight-2026-10-19/README.md:71`) and explicit statement that D11 decode `tok/s` on a qualified model is unmeasured.
2. **MEDIUM (`close_session` / `reset_session` mid-generation race):** Add the `activeJob` guard and per-conversation `sessionEpochs` check to `close_session`, `reset_session`, and `generate` in `web/studio/assets/js/uor_model_worker.js`, guard `deleteConversation` and `resetNativeSession` in `web/studio/index.html`, and test mid-generation close/reset at the step-8 yield boundary in `web/studio/verify_studio_worker.mjs`.
3. **MEDIUM (scope `done` vs UI CDN scripts):** Keep `m7-studio-connect-wasm-engine.md` at `status: ready` (`References #2868`), scope client-side claims to the model worker path only, and list the external UI CDN scripts (`Google Fonts`, `marked`, `KaTeX`, `pdf.js`, `three.js`, `Monaco`) in `## What did not work, first`.
4. **MEDIUM (strengthen `verify_studio_worker.mjs`):** Trap and self-test `navigator.sendBeacon`, `EventSource`, `importScripts`, `Worker`, `SharedWorker`, `fetch`, `XMLHttpRequest`, and `WebSocket`, and statically scan `uor_model_worker.js`.
5. **LOW (stand-in prompt tokenizer):** Label the prompt tokenizer as `"UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)"` in `uor_model_worker.js` and `web/studio/index.html`.

## Cancel probe (run first)

`uor-harness loop next` reports `kill-switch: off` and `GO repair #2978`.

## Files in scope

- `web/**`
- `crates/uor-r4-integer/**`
- `docs/products/**`
- `docs/labs/m7-studio-connect-wasm-engine/**`

## Done when

1. Unsourced `"35.1% tok/s"` strings are replaced with `P_succ = 35.1%` (`docs/labs/address-cells-preflight-2026-10-19/README.md:71`) and unmeasured D11 decode `tok/s`.
2. `close_session` and `reset_session` enforce `activeJob` + `sessionEpochs` guards, verified mid-generation in `verify_studio_worker.mjs`.
3. `m7-studio-connect-wasm-engine.md` stays `status: ready` (`References #2868`) and `REPORT.md` lists UI CDN scripts in `## What did not work, first`.
4. `verify_studio_worker.mjs` self-tests all 6 network traps and passes (`ALL STUDIO WORKER CHECKS PASSED`).
5. Prompt tokenizer is labelled as a stand-in (`UTF-8 byte-mod stand-in`).

## Stop and ask if

- Any file outside the ORDER's scope needs modification.
