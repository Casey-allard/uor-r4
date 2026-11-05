# The web client

The web client is meant to be a language model that lives in a browser tab. You open a web page once, it downloads the model file, and from then on the model can run on your own computer, inside the browser, in a sandboxed form of machine code called WebAssembly. In local WASM mode your prompts and the model's replies stay on the device. An optional user-configured OpenAI-compatible API instead sends prompts to the endpoint the user chooses. The page you would use is the Studio, an editor-and-chat interface already in the repository. The live `/studio/` page now offers the publicly released M1 model as a top-8 next-token predictor after an explicit download click, alongside a clearly labelled synthetic demo and local-file, URL and API model sources. **M1 is complete, but the M1 model is a language base, not a chat assistant; until M7 and M8 pass the web client is a research explorer and demonstration, not a product.**

**30 October 2026 update — the Studio is fixed and published live.** The source Studio at `web/studio` is now a clean, responsive page with an honest current-status header and a Model Source panel: the bundled synthetic demo stack, a local `.lut`/`.bin` file loader, a user-initiated URL loader, and an OpenAI-compatible API manager (presets, add/edit/remove/test, keys kept in localStorage only). It runs the integer engine built from `crates/uor-r4-integer` (`--features serving-wasm`) in a Web Worker and accepts the engine's served stack schemas `uor-r4.lut-stack/1-7` (the `/7` turn memory is served on synthetic fixtures; trained-artifact parity and memory-session restore remain open) and the trained discrete family `uor-r4.lut-discrete/1` / `uor-r4.lut-discrete-trunk/1` as next-token prediction. The build and publish path is `scripts/studio/build-and-publish.sh` with `scripts/stage-studio-site.sh`; the initial live `/studio/` page was published by that script on 30 October 2026, with `/1`–`/6` served before #3446 later added `/7`, (gh-pages d8e67c66; owner decision: publish live), and the current live publish is gh-pages eb832431a with the M1-capable bundle verified below. The `.github/workflows/docs-site.yml` change that would make the build automatic remains a proposal in DeepSeek's non-repository deploy report, not applied or included in this documentation PR; the existing workflow preserves a `studio/` directory, so a manual publish persists across later book deploys. The trained M1 model is now released publicly: [caseyallard/uor-r4-m1-model](https://huggingface.co/caseyallard/uor-r4-m1-model) holds the M1-closing 19-table discrete trunk pair (86.5 MB / 84.9 MB, MIT) with its stack-bound tokenizers, and its primary card in the Studio loads seed 1 from the Hugging Face resolve URL after an explicit click, verifies the sha256 and tokenizer, and switches the page to a top-8 next-token predictor. The model is a language base, not a chat assistant: greedy continuation repeats tokens, so the page says so and refuses generation on the discrete engine. Two further public releases hold the 4-table Stage-T base exports ([uor-r4-m1-discrete](https://huggingface.co/caseyallard/uor-r4-m1-discrete)) and the trained FBS trunk pair ([uor-r4-fbs-dual-c131](https://huggingface.co/caseyallard/uor-r4-fbs-dual-c131), research: no token writer, so it loads but does not generate). The bundled demo stays the synthetic stand-in; local-file, URL and API sources are unchanged. The page itself was then redesigned (DeepSeek's #3447) into an agent-console shell closer to Claude Code / Codex desktop: command bar, icon rail, sessions sidebar, transcript with a terminal-frame composer, full-width model providers page, compact context inspector and a mono status bar, in a warm graphite + ember palette with a designed light theme, a Cmd/Ctrl+K command palette and persisted theme/chrome; the engine, worker and API client are unchanged; later provider/catalog/storage behavior is covered by focused tests. See [#3440](https://github.com/UOR-Foundation/uor-r4/issues/3440) and [the item](../roadmap/items/m7-studio-live-model-sources.md).

**2 November 2026 Studio source refinement:** the source page now has source-specific Agent welcome panels, a safe discoverable demo workspace, and a labeled mobile scrim with Escape and destination dismissal. No-source, research-only, and discrete-M1 chat are disabled, the supported synthetic non-discrete stack remains labeled synthetic continuation, and valid HTTP(S) API configuration enables the external-baseline API send path without proving reachability. Demo replacement refuses dirty work and demo state is labeled after restore. Desktop 1440×900 and iPhone 393×852 dark/light views and the demo/edit/attachment workflow were checked in the built-in browser; this is visual/non-model evidence, not a model, API, real-disk, typed-library cutover, or new publication claim.

**2 November 2026 native source follow-up:** the source now uses system typography, opaque surfaces, softer control/card radii, minimum 24px targets, and visible keyboard focus in both themes. Connections centers **UOR Chat — experimental** on the existing synthetic WASM path and **Custom API providers**, while M1 prediction, research exports, and local-file/URL tools remain available in a closed **Artifact diagnostics** region. Existing listeners, capabilities, readiness checks, and privacy boundaries are preserved. No trained-chat service, MCP connectivity, real API/disk trajectory, typed-library cutover, or new publication is claimed; remote MCP is a separate protocol unit and desktop stdio is a later host bridge.

**4 November 2026 unified harness follow-up:** the shared source now also includes bounded remote Streamable HTTP MCP initialize/tools list/call with JSON/SSE normalization and inert text rendering, plus native remote/stdio MCP behind host confirmation. A Claude Code desktop-inspired session toolbar exposes working sidebar, section-history, session, Files, Changes, Terminal, New chat and view-option controls. Native same-named files retain separate buffers and granted roots, recent entries carry their project identity, and concurrent opens cannot duplicate a tab. MCP replacement connects first and preserves the working session on failure or cancellation. macOS navigation is restricted to `tauri://localhost`; trained-manifest-format Pages fixtures and bundled `pkg/` artifacts pass catalog-bound path, hash and symlink checks. Real native GUI checks passed for both MCP transports, same-file save isolation, cancellation preserving the active session and normal-exit cleanup at development scope. Replaying the synthetic assistant's literal `<|bos|>` output in a subsequent turn returns HTTP 400 for special-token injection; the empty state explicitly labels synthetic output, not useful conversation. Protected delivery and hosted publication are separate pending steps.

**3 November 2026 native conversational serving:** the merged native loopback service (#3606) and conversational composer now let the plain-JS source page send real `uor-chat` requests on this device, with Enter/Shift+Enter, IME-safe submission, ArrowUp/Down prompt history with draft restoration, long text pastes, text/source attachments, and exact native usage/TTFT/tokens-per-second metrics. The same-origin temperature-zero bootstrap token and provider ID stay in memory rather than localStorage or exported sessions. Two real browser turns, text-file upload, reload-safe sessions, context-overflow errors, and the embeddings 501 were exercised; this is local protocol/UX evidence, not useful conversation, production deployment, or visual understanding. The delivered R4 image path is recorded separately below.

**3 November 2026 native R4 image ingestion:** the integer loopback now exposes
bounded `POST /v1/images/decompose` and accepts authenticated `uor_r4_image`
content parts with exact canonical and source-hash validation. The browser
Studio accepts pasted, dropped and selected raster images, downsamples them to
at most 256 by 256, previews/removes them and keeps the image data memory-only.
Native Send adds R4 patch context only after the explicit action; custom
providers receive ordinary `image_url`. Native parts also carry a server-issued
decomposition ID and must match one of at most 64 exact records, so altered or
expired content is refused. This is experimental data plumbing, not a trained
visual reader; native ordinary `image_url` and image generation/edit families
still return 501.

**4 November 2026 conversation-first Studio and native runner, published:** the source and live Studio now default to a single dark chat column with an automatic bundled synthetic demo, rounded messages and a floating composer; the old navigation, inspector and status panels no longer dominate chat. Connections, Workspace, sessions and diagnostics remain secondary. Desktop and phone browser checks, offline Enter generation and ArrowUp recall passed. GitHub Pages cannot host an API, so the [native runner prerelease](https://github.com/UOR-Foundation/uor-r4/releases/tag/studio-native-runner-v0.1.0) serves the runtime-only Studio and authenticated `uor-chat` API together on `127.0.0.1:8000` with pinned downloads and an in-memory token. Fresh public downloads and an empty-cache launch passed; a native browser stream returned exact `19/16/35` usage and 16 completion tokens. A 69-position turn against the 64-position synthetic context returned an explicit HTTP 413 without truncation. No public tunnel, CORS-only workaround, useful-chat or visual-understanding claim is made. Source PR #3651 merged as `a877ef916`; `gh-pages` advanced by fast-forward `233b97fb9 → eb832431a`, the Pages API reports built, and hosted desktop generation and mobile layout inspection, Native setup, all five model cards and development-file 404s were verified.

**Initial Studio publish and interface history (superseded).** The initial 30 October publish at gh-pages `d8e67c66` established the clean responsive page, self-hosted integer wasm engine, local-file/URL/API model sources, `/1`–`/6` serving at the initial publish, with `/7` added by #3446, and discrete next-token prediction, followed by the public M1, Stage-T and FBS releases described above. DeepSeek's #3447 agent-console shell then superseded that interface before the historical v2 below; the engine, worker and API layer remained intact.

**Historical v2 interface (#3448).** DeepSeek then rebuilt the Studio into a workable AI studio around one labeled navigation sidebar: Agent / Workspace / Connections / Guide plus a permanent Sessions section, a File System Access workspace with editor tabs and save, agent context attachments/build templates/message actions, organized connection sections, Settings and Guide views, an eight-group command palette and a labeled mobile drawer; the same engine/API layer and verifier contracts are preserved. See [#3440](https://github.com/UOR-Foundation/uor-r4/issues/3440) and [the item](../roadmap/items/m7-studio-live-model-sources.md).

**Live follow-up publish:** the current gh-pages `eb832431a` publish serves the 700,756-byte M1-capable bundle (sha256 `df368909…`), and the live `models.json` remains byte-identical with both M1 seeds and both research FBS seeds. Kimi published after the owner's explicit request; the Pages API reports built and hosted file/browser checks passed.

Labels: **proof**, **measurement**, **hypothesis**. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md) and the [glossary](../start/glossary.md).

![A browser window with three layers: the Studio interface, a Web Worker holding the integer engine, and browser storage split between verified synthetic session snapshots and a dashed, planned model cache. Local WASM mode keeps prompts on the device; an optional user-configured API sends prompts to the chosen endpoint.](../figures/products/web-client-architecture.svg)

*Current model downloads require an explicit load; the download-once persistent cache is planned. Session save/restore is verified on the synthetic stand-in.*

## How it is meant to work

1. **Load the static Studio.** The charter places it at `/studio` on the documentation site.
2. **Download one `UORLUT01` file and verify its SHA-256.** A release pin identifies the model. A
   download-once browser-storage cache is planned, not claimed as delivered.
3. **Start the integer engine in a Web Worker.** The worker loads `pkg/uor_r4_integer.js` and calls
   `init` and `loadStack` on the model bytes.
4. **Generate.** Each token calls `step`. The D11 engine uses integer arithmetic; integer multiply
   and divide are allowed, while floats, transcendentals and dense matmul stay banned. Accepted-model
   cross-browser parity is an M7 measurement, not an assumption.
5. **Save and resume.** `saveSession` emits a `UR4SESS1` snapshot and `restoreSession` loads it.

The real exports are `init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`,
`weightsPerToken`, `allocEvents` and `leakedModels`
([`wasm_bindgen_exports.rs`](../../crates/uor-r4-integer/src/wasm_bindgen_exports.rs)). Serving an
exported D11 artifact on a client device is allowed; the GPU-first research-compute rule does not
apply to product serving ([charter](../../CHARTER.md), section 3.6). D84's CPU training program is
development research and does not qualify the browser runtime.

## What exists today

| Piece | Current state | Source |
|---|---|---|
| Studio interface | Editor, file tree, chat UI, model manager and local folder access, with self-hosted vendor shims | [`web/studio/index.html`](../../web/studio/index.html) |
| Integer worker connection | `loadStack`, `step`, save/restore, header/engine-derived capabilities and a stack-bound ByteLevel BPE tokenizer, verified on a synthetic stand-in | [#2868 record](../labs/m7-studio-connect-wasm-engine/README.md) |
| Offline assets and tokenizer | Main page and router demo make zero external page-load requests; the tokenizer artifact replaces the old byte-mod fallback on bound stacks | [#3075 record](../labs/m7-studio-self-host-page-assets/README.md) |
| WebAssembly integer engine | `serving-wasm` build and 7-of-7 serving-function audit; save/restore identity on the synthetic model | [runtime record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| Parity harness | Eight-prompt native/CLI/browser bit-identity on the seed-5 synthetic stack, with fresh-process restore | [M7 parity record](../labs/m7-parity-library-cli-browser/README.md) |
| Status text | Pre-alpha, 1 of 8 milestones complete; M1 decode measured separately from the synthetic stand-in; no qualified model bundled | [`index.html`](../../web/studio/index.html) |
| Public deploy | Kimi's conversation-first Studio is live at `/studio/`; #3444 recorded the initial publish before the M1 model bundle, and the current gh-pages `eb832431a` publish serves the verified M1-capable `df368909…` bundle. The mdBook workflow publishes preview on main and production only by manual dispatch | [`deploy.yml`](../../.github/workflows/deploy.yml), [`docs-site.yml`](../../.github/workflows/docs-site.yml), [#3440](https://github.com/UOR-Foundation/uor-r4/issues/3440) |

The SIMD kernels in the WebAssembly build are fixtures, not the hot path; the engine currently uses
its scalar table kernel. These implementation limits do not negate the M1 model's native results,
but they are not a browser qualification of that model.

## What is still missing

- **A qualified model bundle for the web product.** M1's native artifacts do not by themselves prove
  browser parity or M7 distribution.
- **The accepted M4 model and its browser export.** Full quality, session and audit bars come before
  M7's accepted-model parity check.
- **A download-once cache and release loader.**
- **Bit-identical tokens and sessions on the real model across library, CLI and browser.** The
  synthetic harness passing is an instrument result, not M7 completion.
- **Automatic build/publish remains owner-only.** DeepSeek's manual production publish is already
  live; this documentation refresh performs no deployment.

## Privacy and integrity

- **Local WASM mode computes locally.** No prompt or reply needs a model server. Native loopback
  uses the `uor-chat` process on this device, with an ephemeral in-memory token. An optional
  user-configured OpenAI-compatible API instead sends prompts to the endpoint the user chooses.
- **Page-load resources are self-hosted.** The static and headless-Chrome checks report zero external
  requests on both Studio pages. Loading the public M1 model from Hugging Face is a separate,
  explicit click that downloads the artifact without sending prompts. User-initiated GitHub features
  are also explicit actions and are refused in Tauri/offline mode.
- **A checksum proves integrity, not secrecy or quality.** The model is public; a hash identifies
  the released bytes, not who made them or whether they answer well.

Until M7 and M8 pass, the page remains a research next-token explorer and synthetic-engine
demonstration, not a working assistant. M1's language-base result and browser prediction are real,
but they do not change that product boundary.

## Open questions

- How large will the accepted model file be, and is a one-time download acceptable on a phone?
- How fast is WebAssembly against native on the same file? The M1 pair's native 10,876/10,566 tok/s
  does not answer that. Older FBS native rates and older evaluation throughput are not browser rates.
- Which remaining served-memory/readout parts are needed for the accepted model's full browser path?

## Go deeper

- [Desktop client](./desktop-client.md), which shares the interface.
- [Serving and export](../model/serving-and-export.md) and the [integer serving dossier](../mechanisms/lut-serving.md).
- [WebAssembly runtime record](../labs/wasm-serving-runtime-2026-10-18/README.md).
- [D36](../integration/DECISIONS.md#d36--github-pages-keep-the-last-good-deploy-live-deploy-job-skipped-until-a-d11-served-wasm-engine-lands) and [charter](../../CHARTER.md), section 3.6.
