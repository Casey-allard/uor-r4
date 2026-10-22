# The web client

The web client is meant to be a language model that lives in a browser tab. You open a web page once, it downloads the model file, and from then on the model runs on your own computer, inside the browser, in a sandboxed form of machine code called WebAssembly. Your prompts and the model's replies never leave the device, because there is no model server to send them to. The page you would use is the Studio, an editor-and-chat interface already in the repository. Today the pieces exist separately and are not joined: the browser engine works on a synthetic test model, the Studio still points at an older engine that was retired, the public site is frozen, and there is no trained model to download. **Until milestones M7 and M8 pass, the web client is a geometry explorer and demonstration, not a product.**

Labels used below: **proof** (derived from definitions or code), **measurement** (a recorded run with its artifact), **hypothesis** (untested). See [`docs/formal_vocabulary.md`](../formal_vocabulary.md) and the [glossary](../start/glossary.md).

![A browser window with three layers. Top: the Studio page (editor, chat panel, model manager). Middle: a Web Worker box holding the WebAssembly engine with its calls loadStack, step, saveSession and restoreSession. Bottom: browser storage holding the downloaded UORLUT01 model file and saved sessions (UR4SESS1). One arrow, drawn only once and labelled "first visit: download model, check SHA-256", comes in from a static file host. A crossed-out arrow to a "model server" shows that no prompt leaves the device.](../figures/products/web-client-architecture.svg)

*One download, then everything runs in the tab. The only network traffic is fetching the static page and the model file.*

## How it is meant to work

1. **Load the page.** The Studio is static HTML, JavaScript and CSS. The charter places it at `/studio` on the project's documentation site ([charter](../../CHARTER.md), section 2).
2. **Download the model once.** The page fetches one `UORLUT01` file and checks its SHA-256 against the hash published with the release ([releases](./releases-and-artifacts.md)). It then keeps the file in browser storage so later visits work offline. This cache is planned, not built (from reading the repository).
3. **Start the engine in a Web Worker.** A worker is a background thread of the page, so typing stays smooth while the model computes. The worker loads the WebAssembly build of `crates/uor-r4-integer` and calls `init`, then `loadStack` on the model bytes.
4. **Generate.** For each token the worker calls `step`. The engine uses only integer adds, shifts and table look-ups (the D11 contract), so the same file gives the same tokens in every browser. That is a design property; M7 has to measure it.
5. **Save and resume.** `saveSession` writes the conversation state as a `UR4SESS1` snapshot, and `restoreSession` brings it back byte for byte.

These function names are the real exports of the WebAssembly build: `init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`, `weightsPerToken`, `allocEvents`, `leakedModels` ([record](../labs/wasm-serving-runtime-2026-10-18/README.md); **proof** by reading [`wasm_bindgen_exports.rs`](../../crates/uor-r4-integer/src/wasm_bindgen_exports.rs)).

**Why serving in the browser is allowed.** Research compute must run on a GPU (D24). On 15 October that rule paused the browser engine, because serving in a browser runs on the user's processor ([D36](../integration/DECISIONS.md#d36--github-pages-keep-the-last-good-deploy-live-deploy-job-skipped-until-a-d11-served-wasm-engine-lands)). The owner resolved this on 19 October: serving the exported D11 file on client devices, including browser WebAssembly, is the product, and D24 does not apply to it ([charter](../../CHARTER.md), section 3.6).

## What exists today

| Piece | Where | State | Source |
|---|---|---|---|
| Studio interface | [`index.html`](../../web/studio/index.html), `index.css`, [`assets/js/uor_model_worker.js`](../../web/studio/assets/js/uor_model_worker.js), [`r4_worker.js`](../../web/studio/r4_worker.js), [`verify_studio_worker.mjs`](../../web/studio/verify_studio_worker.mjs) | Editor (Monaco, loaded from a CDN), file tree, chat, model manager, local folder access, GitHub chooser. The model worker imports `pkg/uor_r4_integer.js` (`serving-wasm`: `loadStack`, `step`, `saveSession`, `restoreSession`), reports only header- and engine-derived capability fields (`"synthetic stand-in"` for the synthetic fixture, plus `"UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)"` for the prompt tokenizer), and shows the D36 banner (`0/8` milestones complete, A8 preflight checkpoint-0 `P_succ = 35.1%`, D11 decode `tok/s` unmeasured, Pages deploy frozen); page-level UI CDN scripts remain external | [`index.html`](../../web/studio/index.html), banner `d36EngineStatusBanner`, and [Studio WASM wiring record](../labs/m7-studio-connect-wasm-engine/README.md) (`#2868`) |
| WebAssembly integer engine | [`crates/uor-r4-integer/src/wasm_serve.rs`](../../crates/uor-r4-integer/src/wasm_serve.rs), `wasm_bindgen_exports.rs`, `session_codec.rs` | Built behind the `serving-wasm` feature; merged as `UOR-Foundation/uor-r4#2654` | [record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| Its checks | same record | 165 crate tests pass; 7 of 7 serving functions with 0 multiply, 0 divide and 0 float instructions; a Node.js run of 32 steps, save, restore and 16 more steps matches a fresh 48-step run (`reloadIdentical: true`) on a synthetic model (**measurement**) | [record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| Browser and Node harnesses | `docs/wasm-serving/wasm_serve_demo.html`, `verify.mjs` | Source only: they import a build folder that is not committed, so they do not run from a clean clone | maintainer annotation in the [record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| Public deploy | [`.github/workflows/deploy.yml`](../../.github/workflows/deploy.yml) | Job disabled (`if: false`) by owner decision on 15 October; the live site is the last good build, 13 October 18:28Z | [D36](../integration/DECISIONS.md#d36--github-pages-keep-the-last-good-deploy-live-deploy-job-skipped-until-a-d11-served-wasm-engine-lands) |
| Geometry explorer | `web/studio/geometric_prime_router_webapp.html` | A standalone demo page, separate from the Studio; not verified in this audit | [`index.html`](../../web/studio/index.html) and the `web/studio/` files |
| Cross-origin isolation helper | `web/studio/coi-serviceworker.js` | Vendored third-party file (v0.1.7, MIT) | [`index.html`](../../web/studio/index.html) and the `web/studio/` files |

Three limits from the engine's own record matter for a reader. The WebAssembly SIMD kernels are test fixtures that nothing calls yet; the engine still runs its scalar table kernel. The demo needs a local build first. And the `serving-wasm` build compiles some modules unconditionally, so its compliance with the "presence counts" rule (D29) rests on an interpretation that `DECISIONS.md` does not record ([record](../labs/wasm-serving-runtime-2026-10-18/README.md), maintainer annotation).

## What is missing

| Missing piece | Needed for |
|---|---|
| A trained model exported to `UORLUT01` | Anything to download (M4, `#2032`) |
| Studio rewired from the retired engine to the integer engine's exports, through a Web Worker | Model worker path rewired and verified on the synthetic stand-in (`#2868`, [record](../labs/m7-studio-connect-wasm-engine/README.md)); item stays `ready` until UI CDN assets are self-hosted and a qualified model artifact exists |
| A download-once loader with a checksum check and a browser-storage cache | Offline use after the first visit |
| Bit-identical token and session parity across the Rust library, `uor-chat` and the browser worker, on the real model | M7 |
| Truthful capability reporting in the Studio and zero remote model calls | Model worker path verified on synthetic stand-in via `web/studio/verify_studio_worker.mjs` (`#2868`); awaits real model |
| Re-enabled Pages deploy (D36 says remove the `if` in the PR that lands a D11 served engine) | Public access |
| Self-hosted copies of every CDN asset (Monaco, marked, KaTeX, pdf.js, three.js and the Google fonts) | Fully offline use and the no-outside-traffic claim; external Wikipedia/DuckDuckGo prompt search is disabled (`#2868`), while UI scripts/fonts still load from CDNs |
| Honest screenshots taken after behaviour checks | Documentation |

The M7 bar sums it up: the qualified model "produces bit-identical token completions and session snapshots across the Rust library, the `uor-chat` CLI and the browser `serving-wasm` worker", and the Studio "runs entirely client-side in a Web Worker with truthful capability reporting and zero remote model calls" ([project track](../integration/project-track.md), M7). The `.wasm` audit criterion is green and will be re-run on the shipped bundle; the rest of M7 (rewiring the Studio, three-host parity, client-side Studio) is open.

## Until then: an explorer, honestly labelled

Until M7 and M8 pass, the web page should present itself as what it is: a geometry explorer and a demonstration of the serving engine on a synthetic model. It must not offer a chat box that implies a working assistant. The Studio's engine-status banner already says so. Showing the engine's real exports (`weightsPerToken`, `allocEvents`) on a synthetic model is fair. Showing chat replies is not, until there is a trained model behind them.

## Privacy model

- **The model computes only on the device.** No prompt or reply is sent anywhere by the engine. That is the M7 bar, and today it holds trivially, because there is no remote model to call.
- **Network use is not yet limited to static files.** The target is the page and the model file only. Today the Studio also loads third-party code and fonts from CDNs (Monaco, pdf.js and three.js from cdnjs; marked, KaTeX and the three.js controls from jsDelivr; fonts from Google Fonts), and its search feature sends the typed query to `en.wikipedia.org` and `api.duckduckgo.com`. Features that open files from GitHub talk to GitHub. None of that is model compute, but a privacy claim must not hide it: the requirement is to self-host every asset and make web search opt-in, or remove it, before M7 ([`index.html`](../../web/studio/index.html), lines 11 to 25 and 5845 to 5906).
- **Integrity, not secrecy, of the model file.** The SHA-256 check proves the file is the released one. The model is public, so there is nothing to hide in it.

## Open questions

- How large will the model file be, and is a one-time download of that size acceptable on a phone? Unknown until a model exists.
- How fast is WebAssembly against native on the same file? No tokens-per-second figure for the browser has been recorded. The only native figures are from older float models: 68 to 80 tokens per second on 8 threads for a 29M model, which is evaluation throughput over 64 scored windows, not free-running decode ([D11 serving record](../../archive/labs/d11-serving-2026-10-10/README.md)). No decode tokens per second exists for any D11 artifact.
- Should the Studio's provenance be settled first? Its source is an import from a companion repository, `Casey-allard/uor-r4-wasm-chat`, which this audit did not inspect.

## Go deeper

- [Desktop client](./desktop-client.md), which shares this interface.
- [Serving and export](../model/serving-and-export.md) and the [integer serving dossier](../mechanisms/lut-serving.md).
- The [WebAssembly runtime record](../labs/wasm-serving-runtime-2026-10-18/README.md) (`UOR-Foundation/uor-r4#2654`, tracking issue `#1172`).
- [D36](../integration/DECISIONS.md#d36--github-pages-keep-the-last-good-deploy-live-deploy-job-skipped-until-a-d11-served-wasm-engine-lands) and the [charter](../../CHARTER.md), section 3.6.
