---
id: m7-studio-connect-wasm-engine
title: Connect the Studio to the browser WASM engine
milestone: M7
area: serving
status: done
open_to: anyone
needs_gpu: false
depends_on: []
scope: [web/**, crates/uor-r4-integer/**, docs/products/**, docs/labs/m7-studio-connect-wasm-engine/**]
---

## Why it matters

This delivers the web client product as soon as a model exists: the Studio page runs the model on your own device with no server. Delivered by PR #2978 (`24f9d8ab6`, merged 2026-10-22, verified) with its antigravity-21 repair: the Studio worker now loads the integer `serving-wasm` engine (`pkg/uor_r4_integer.js`; `loadStack`, `step`, `saveSession`, `restoreSession`), no `uor_r4_wasm_router` reference remains under `web/`, and the worker verification records zero remote requests. The Pages deploy stays paused (D36) until a qualified M4 artifact exists. See [the web client](../../products/web-client.md).

## Resolution (2026-10-23, PM flip by kimi-26)

The done-when below was met on the model worker path: client-side Web Worker execution, truthful capability reporting (unsourced figures replaced with sourced citations, tokenizer labelled a stand-in, "synthetic stand-in" status), zero remote model calls (`remoteRequests=0` with six trapped channels), and the static WASM audit at 7/7 PASS (mul=0, div=0, float=0). Evidence: PR #2978 report and `docs/labs/m7-studio-connect-wasm-engine/README.md`. The residual the author deliberately scoped out — verifying the offline-shim claim at page load, the router demo page's remaining CDN loads, and the byte-mod stand-in prompt tokenizer — carries no registered bar on this item (its text always said so) and moved to [m7-studio-self-host-page-assets](m7-studio-self-host-page-assets.md); #2868 closes via the post-merge roadmap sync and the residual's own issue tracks the rest.

## What is missing

Nothing for this item — delivered by PR #2978 (see the resolution note above). The page-level CDN assets and stand-in tokenizer are a separate unregistered bar, carried by [m7-studio-self-host-page-assets](m7-studio-self-host-page-assets.md).

## Done when

M7 criterion 3: the Studio runs entirely client-side in a browser Web Worker with truthful capability reporting and zero remote model calls; the `.wasm` module passes `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` (7/7 PASS, mul=0, div=0, float=0). The 18 October record counts 7 serving functions, not whole modules.

## How to start

1. Read `docs/products/web-client.md` (what exists and what is missing), then `crates/uor-r4-integer/src/wasm_bindgen_exports.rs` and `crates/uor-r4-integer/src/wasm_serve.rs` (the exports and their return types), then `web/studio/assets/js/uor_model_worker.js` and `web/studio/r4_worker.js`. Find where `web/studio/index.html` creates the worker (`initModelWorker`) and the banner `d36EngineStatusBanner` near line 2323.
2. Post a design note on the issue. It must list: the message protocol between page and worker mapped onto `loadStack`, `step`, `saveSession` and `restoreSession`; how the tokenizer reaches the page; which capability fields the Studio shows and where each value comes from, so nothing is claimed that the engine does not report; and every network request the page makes, with a plan to remove or make opt-in each one that carries model input. State that the page adds no softmax, no matmul and no floating-point sampling.
3. Build the engine bundle: `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm`. Generate a synthetic stack with `~/.cargo/bin/cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack`. `docs/wasm-serving/verify.mjs` reads `./uor_r4_integer_bg.wasm`, `./uor_r4_integer.js` and `./synthetic-stack.lut` from its working directory, so copy the wasm-pack `pkg/` output and the synthetic stack next to it, then run it with Node to confirm the bundle works before touching the Studio.
4. Write the first deliverable: a failing browser or Node test that loads the synthetic stack through the Studio worker and gets the same tokens as `verify.mjs`. Then rewire the worker to the new exports and remove the retired engine path.
5. Make the capability panel truthful: it reports only what the loaded engine returns, and says "synthetic stand-in" when a synthetic stack is loaded. Add a check, for example an automated page test, that records zero requests to remote model endpoints.
6. Run the audit on the built module: `python3 scripts/audit_zero_matmul_wasm.py <wasm> --serving --call-graph --tap`. This is an existing script; add no new Python. Post the 7/7 output.
7. Open a PR that says `Closes #<this issue>`. Do not re-enable the Pages deploy in it unless the owner asks (D36 ties that to landing a served engine). Add one row to `docs/research/notebook/m4-serving.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`STATUS.md:22`](../../../STATUS.md)
- [`docs/start/where-were-going.md:74`](../../../docs/start/where-were-going.md)
- issue#2035
- [`docs/products/web-client.md`](../../products/web-client.md)
- [`docs/labs/wasm-serving-runtime-2026-10-18/README.md`](../../labs/wasm-serving-runtime-2026-10-18/README.md)
