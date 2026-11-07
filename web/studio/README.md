# Studio (`web/studio`) — the browser client

**Live:** https://uor-foundation.github.io/uor-r4/studio/ — the current gh-pages `eb832431a` publish serves the conversation-first interface and 700,756-byte wasm bundle (sha256 `df368909…`) and preserves the model catalog with both M1 seeds and both research FBS seeds. Source PR #3651 merged as `a877ef916`; the Pages API reports built at `eb832431a`, and hosted desktop generation and mobile layout inspection, Native setup, all five model cards, runtime bytes and development-file 404s were verified. The previous `233b97fb9` publish served the 700,628-byte `b4ab83f1…` bundle.

The Studio is the project's in-browser interface: editor, chat panel and model manager, with the
integer serving engine (`crates/uor-r4-integer`, `--features serving-wasm`) running in a Web
Worker. It is served from the project's documentation site at **`/studio/`**
(<https://uor-foundation.github.io/uor-r4/studio/>). The current live publish is gh-pages `eb832431a`; `233b97fb9` was the previous publish.

### Conversation-first Studio and native runner (4 November 2026)

The source now opens into one quiet dark conversation column instead of the former panel-heavy IDE. First run automatically loads the bundled synthetic demo; the empty state offers working offline-demo, model-choice and native-setup actions. Connections, Workspace, sessions and diagnostics remain available outside the main chat flow. System typography, rounded message bubbles, a floating composer, mobile-safe 16px input text and compact phone controls are covered by browser inspection at 1440×900 and 390×852 plus focused regressions. Enter/Shift+Enter, IME-safe submission, ArrowUp/Down history, long pastes, attachments, image decomposition and exact metrics retain their existing paths.

Static GitHub Pages still cannot host an API, and HTTPS Pages must not call insecure HTTP loopback. The new [studio-native-runner-v0.1.0 prerelease](https://github.com/UOR-Foundation/uor-r4/releases/tag/studio-native-runner-v0.1.0) supplies a hash-pinned Apple silicon `uor-chat`, a runtime-only Studio archive, `native-runner.sh` and `SHA256SUMS`. The runner serves the same UI and authenticated native API together on `127.0.0.1:8000`, with an in-memory launch token and no public tunnel or token output. Fresh public downloads and an empty-cache launch passed. A real native short-prompt stream returned 16 completion tokens with exact `19/16/35` usage, measured tok/s and TTFT; a longer prompt correctly returned HTTP 413 at 69 of 64 context positions without truncation. These are synthetic protocol results, not useful conversation, visual understanding, M7/M8 completion or production readiness. The live-Pages publish is `eb832431a`, with hosted desktop generation and mobile layout inspection and Native setup verified.

### First console redesign (superseded by v2)

*Historical interface: the workable v2 Workspace/Agent design below; the conversation-first section above is the current source.*

An agent-console shell: command bar (wordmark, workspace breadcrumb, project chip, command search,
view switch, theme/new), icon rail, collapsible sessions sidebar with a runtime environment card, a
central transcript with a quiet console hero and a terminal-frame composer, a full-width Model
providers page, a compact context inspector (active model, runtime, shortcuts), and a mono status
bar (engine/model/privacy/storage/clock). Warm graphite + ember palette with a designed light theme,
toggled from the rail or palette and persisted in localStorage (`uor.studio.theme`); chrome toggles
persist in `uor.studio.ui.v1`; `prefers-reduced-motion` disables animation. The command palette
opens with Cmd/Ctrl+K (also `#command-palette`), groups actions (new chat, load demo/M1/local/URL/API,
switch views, theme and chrome) and supports arrow/Enter navigation, focus trap and Escape. QA deep
links `#view=chat|model|editor` and `?theme=dark|light` exist for screenshots and tests.

The shell is presentation only: every existing element id/handler, the data-studio-ready boot
contract, the status markers and the engine/worker/model-source/API layer are unchanged, so
`verify_studio_page.mjs`, `verify_studio_page_load.mjs`, `verify_studio_worker.mjs` and
`verify_desktop_client.mjs` pass at the redesign head.

### Workable AI studio (31 October 2026, second redesign)

The console shell above was superseded after owner feedback that it was "janky" with menus that
were "not organized or labeled very well". The page is now a workable AI studio:

- **One labeled navigation sidebar** with six destinations: **Chat** (conversation and
  build tasks; ⌘1), **Files** (file tree, editor tabs and save; ⌘2), **Changes** (bounded Git status/diff; ⌘4), **Terminal** (host-approved commands; ⌘5), **Connections** (UOR Chat,
  MCP servers, custom API providers and artifact diagnostics; ⌘3) and **Settings** (appearance, data and About; ⌘,). Guide remains available for workflows, shortcuts and limits. A
  permanent **Sessions** section lists title/preview/relative time with a `⋯` menu (Rename,
  Duplicate, Export JSON, Delete with an in-app confirm). Settings (⌘,) and Help (?) live in the
  sidebar footer. The old icon rail, segmented view switch and duplicate controls are gone; their
  element ids remain as hidden compatibility controls for scripts and verifiers.
- **Workspace** opens a local folder with the File System Access API (tree, open files as editor
  tabs, active tab drives content/name/language/status, dirty dots, ⌘S save back to disk,
  ⇧⌘S save-as download). Browsers without the API use the import-files fallback and download save;
  directory access is memory-only and never uploaded.
- **Agent** takes context attachments (current editor file, workspace files, pasted text) as chips
  with sizes and a token hint; six Build task templates fill structured prompts; assistant messages
  can be copied, inserted into the editor, saved as a file or retried; the composer states whether
  the active mode is local WASM (stays on device), native loopback (stays on device), or a custom OpenAI-compatible API (prompts leave the
  device). The native OpenAI-compatible service and plain-JS client are in this repository.
- **Connections** presents UOR Chat — experimental, custom API providers, and a closed Artifact
  diagnostics region for M1/research/local-file/URL tools. **Settings** has Appearance (theme, reduce motion), Editor (font, tab size, wrap), Data
  (sessions/draft storage and clearing) and About. **Guide** is the product documentation.
- **Command palette** (⌘K, also `#command-palette`) has eight groups - Navigation, Sessions,
  Workspace, Agent, Models, Build, Interface, Help - with label, one-line description and only
  implemented shortcuts; the list scrolls with the footer pinned. The implemented shortcut map is:
  ⌘K, ⌘, , ⌘1-6, ⌘B, ⌘I, ⌘S, ⇧⌘S, Enter/Shift+Enter, Esc and ?. QA deep links `#view=`,
  `?theme=` and `#command-palette` exist for screenshots/tests.
- **Responsive:** at 390px the navigation is a labeled drawer, the inspector is hidden, the composer
  is sticky and the status bar is two labeled rows; measured `scrollWidth == innerWidth`.
- **Contract:** every pre-existing element id and handler is preserved (hidden compatibility block
  where the UI moved), the two status markers and the data-studio-ready boot contract are intact,
  the engine/worker/API client are unchanged, model-source changes are limited to tested provider/catalog/storage behavior, no external assets or requests are added,
  and the strict static checks pass with assertions intact. Real-browser/worker parity checks remain later work. The page remains offline-first: no
  external request at load; remote model/API contacts are user-initiated only.

### Native IDE and simplified connections (2 November 2026)

The source page uses a Mac-inspired application treatment: system typography, opaque panels, softer radii, compact controls with at least 24px targets, and a 2px keyboard-focus outline in dark and light themes. Dark mode uses neutral black/graphite layers with a measured system-blue accent; light-theme layout and controls are unchanged, with text-accent contrast corrected to `#8f3f14`. Connections presents **UOR Chat — experimental** as the primary local source through the existing synthetic WASM path, with **Custom API providers** beside it and M1/research/local-file/URL tools inside a closed **Artifact diagnostics** region. The original source buttons, listeners, `data-action` values, storage, readiness checks, and capability labels are preserved; configuration still does not prove reachability. These are source changes, not a production publish or trained-chat claim. Remote Streamable HTTP MCP and native remote/stdio MCP are now implemented behind explicit Connect/Call actions and host confirmation; native GUI acceptance remains separate work.

### IDE experience in the Studio source (2 November 2026)

The Studio source page now presents a coherent IDE-style welcome panel with source-specific states and quick actions. Users can explore a demo workspace before a serving model exists: open two tabs, edit buffers, and attach files as context. Demo replacement refuses dirty work, restored tabs are labeled demo, and no automatic dirty edit is fabricated. No-source, research-only, and discrete-M1 chat are disabled; positively synthetic non-discrete continuation and valid HTTP(S) API configuration remain available. Mobile navigation persists destination dismissal, closes through the labeled scrim or destination, and handles Escape after any open modal or palette. Desktop 1440×900 and iPhone 393×852 dark/light views and the demo/attachment workflow were exercised in the built-in browser; no model/API/disk trajectory is claimed.

### Conversational native serving and composer (3 November 2026)

The live plain-JS Studio now serves the native `uor-chat` engine through the merged loopback API (#3606). Enter sends, Shift+Enter adds a newline, and IME confirmation does not send. ArrowUp/Down recalls complete submitted prompts and restores the current draft, including multiline prompts. Long text pastes are not silently truncated. Text/source files can be attached by paste, drop, or the multiple-file picker with explicit size/binary errors; attachment context is included only after an explicit Send and is ignored in local WASM mode.

The native same-origin source uses temperature zero and an ephemeral launch token. That token and its provider ID are not written to localStorage or exported sessions. Native JSON/SSE responses display live completion tokens and measured tok/s, then exact prompt/completion/total usage, TTFT, elapsed time, and finish reason; other providers show character-only or unavailable usage honestly. `uor-chat --stack … --tokenizer … --serve 127.0.0.1:8765 --studio-root web/studio` starts the local service. This is protocol and UX evidence, not useful conversation, production deployment, or visual understanding. R4 image decomposition is a separate follow-up; standard `image_url` and embeddings return 501.

### Deterministic R4 image input (3 November 2026)

Raster images can be attached by paste, drop, or the multiple-file picker. Browser Canvas downsamples proportionally to at most 256×256 and sends raw RGBA to `POST /v1/images/decompose` only when the user sends. The internal `uor-r4-vision` crate uses checked integer math to produce positioned mean-RGB/luma/chroma/edge patches and signed Q12 R4 vectors in canonical `R4IMG001` bytes with a Blake3 source hash. Native chat accepts only a flat `uor_r4_image` content part with schema `uor-r4.image-decomposition/1`, canonical hex, source hash, dimensions, grid, and a server-issued decomposition ID. The server retains at most 64 issued decompositions and requires the complete part to match its record; ordinary native `image_url`, embeddings, and image generation remain unimplemented.

Image attachments show previews, names/sizes, removal, and `R4 patch input — experimental; not visual understanding`. Custom OpenAI providers may use ordinary `image_url` content parts with provider errors shown verbatim; local WASM refuses image context. Image bytes and previews remain memory-only. Reload shows zero attached image files and a memory-only notice; text-only sends still work, Clear dismisses the notice, and sending another image requires reattachment. Context overflow is rejected without truncation. This is deterministic decomposition and protocol evidence only, not captioning, recognition, visual question answering, or a trained multimodal artifact.

### Typed workflow components (development library)

The `src/` library now contains the lifecycle store and all eight approved core components:
AppShell, ModelPicker, WorkspaceEditor, AgentComposer, SessionList, Inspector, Guide, and Settings.
`StudioApp` composes them without mounting or replacing the live page. AgentComposer, WorkspaceEditor,
ModelPicker, SessionList, and Inspector subscribe to one store; AppShell, Guide, and Settings are
coordinated through props and callbacks.
The components call the existing file/session/UI/model-source APIs through typed bridges. Small public
hooks forward editor content, generation completion, settings, and attachment changes; the engine,
worker, tokenizer, and `studio_api.js` remain unchanged. Existing provider credentials are not
rendered or copied into component state. Newly entered keys are held only in the modal draft until saved, then persisted through the existing `modelSources.v1` writer; the draft is cleared after a successful save. Failed storage writes roll back and remain visible in the dialog.
Narrow live-page bridge corrections correlate chat completion, include attachments in API prompts, and return per-session clear results without creating a replacement session; the engine, worker, tokenizer, and `studio_api.js` are unchanged.

`AgentComposer` now mounts the exported TypeScript `createComposerInput` controller, preserving Enter/Shift+Enter, IME refusal, bounded prompt history with exact draft restoration, reset/destroy, paste/drop/picker channels, and the existing bounded text-file reader. Its `/` menu filters the six build-task templates and its `@` menu filters the active file/current attachments without duplicating retained names. Both menus expose listbox/option state and keyboard focus through the prompt textarea. These are typed-component checks only: the legacy plain-JS controller and live page remain unchanged.

`RichMessage` renders common Markdown headings, lists, quotes, fenced code, inline code, strong text, and HTTP(S) links through Preact nodes rather than HTML strings. Raw HTML and unsafe/relative destinations stay literal, rendering performs no fetch, and copy/insert/save/retry keep the exact original message. Tables, math, raw HTML, images, and syntax highlighting are outside this component's current scope. The live transcript remains unchanged.

`StudioApp` now exposes Agent, Workspace, Changes, Terminal, Models, and Guide through navigation and the command palette. `ChangesPanel` refreshes bounded Git status/diff through the existing native adapter and copies the exact raw diff. `TerminalPanel` requires app-session enablement and retains per-command native approval. Unavailable hosts, failures, cancellation, and stale async results are explicit; repository/terminal output is inert. These are typed component/adapter fixtures, not real repository, command, or packaged-GUI acceptance.

The Models view also includes `McpConnections` through the existing `connectRemote` contract. Explicit Connect lists only valid tool metadata, nested object/array arguments round-trip exactly, Connect is disabled while a call is active, and Call is refused while Connect is active. **Attach result** adds exact result JSON through the existing attachment API with a sanitized `mcp-<tool>.json` name and verifies retention before reporting success. Existing `@` options include the attachment. Credentials are memory-only and cleared after connection/unmount; active sessions supplying `disconnect` are released on unmount, while the direct-browser client's missing hook is an explicit no-op. Failed previous-session cleanup preserves the old session and tools, attempts candidate release, and surfaces previous and candidate cleanup errors. No autonomous loop, second protocol client, stdio expansion, real endpoint, or packaged-GUI claim is made. Future cutover must also avoid the legacy MCP auto-boot mounting beside this component.

Run `npm ci`, `npm test`, `npm run typecheck`, and `npm run build` in this directory. This unit passes **377 typed tests across 44 files**, **112 plain-JS tests**, and all four legacy checks. The library
build emits uncommitted `dist/` output; `src/studio.css` does not alter the legacy `index.css`.
Native progress and dialog elements preserve unknown totals and indeterminate finalization; dialogs
use native focus semantics, while browser focus-return verification remains in the parity record.
M1 discrete artifacts are labeled next-token prediction, not conversational chat.

This is core-workflow evidence only. Full real-browser/worker parity, mobile/accessibility checks,
one-time browser-data migration, publisher changes, and production cutover remain later units.

This directory holds the page source. **The wasm bundle, the demo model and the tokenizer are
generated, never committed** (CHARTER §3.13 forbids committed binaries and corpora); the generated
files live in the gitignored `pkg/` and are produced by the build below or by
`scripts/studio/build-and-publish.sh`.

## What is published and what is dev-only

Published (copied by `scripts/stage-studio-site.sh`):

- `index.html`, `index.css`, `geometric_prime_router_webapp.html`, `coi-serviceworker.js`;
- everything under `assets/` (JS shims, tokenizer artifact, CSS, fonts);
- `pkg/` — the wasm-pack output plus `synthetic-stack.lut` and `tokenizer.json`;
- `models.json` — generated at publish time by the staging helper (see below).

Dev-only (never published): `tools/`, `verify_*.mjs`, `*.sh` and this README.

## Build the bundle and run the checks locally

```sh
export PATH="$HOME/.cargo/bin:$PATH"          # the PATH cargo may be a non-Rust shim
# 1. wasm engine (wasm-pack 0.15.0 used locally; the proposed CI workflow pins 0.13.1; scalar table kernel, no wasm SIMD)
RUSTFLAGS='-C target-feature=-simd128' wasm-pack build crates/uor-r4-integer \
  --target web --no-opt --out-dir ../../web/studio/pkg -- --locked --features serving-wasm
# 2. deterministic synthetic demo stack (seed 5, schema uor-r4.lut-stack/1) + its tokenizer
~/.cargo/bin/cargo run -q --locked -p uor-r4-integer --features serving-wasm \
  --bin mk-synthetic-stack -- 5 web/studio/pkg/synthetic-stack.lut
node web/studio/tools/make_tokenizer.mjs web/studio/pkg/synthetic-stack.lut \
  --out web/studio/pkg/tokenizer.json
node web/studio/tools/make_tokenizer.mjs web/studio/pkg/synthetic-stack.lut \
  --check web/studio/pkg/tokenizer.json
# 3. the checks the publisher runs at the same tree
node web/studio/verify_studio_worker.mjs
node web/studio/verify_studio_page.mjs --strict-router
node web/studio/verify_studio_page_load.mjs --strict-router   # needs local Chrome/Chromium
python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm \
  --serving --call-graph --tap
# 4. serve it (from the repository root)
ruby -run -ehttpd web/studio -p 8000            # open http://localhost:8000/
```

The page-load harnesses enforce the offline bar: **no external HTTP request at page load**
(same-origin requests only). The runtime trap fails closed when no Chrome/Chromium is available;
the publisher skips it with a warning only when no browser exists on the machine.

## Publishing (`scripts/studio/build-and-publish.sh`)

**How the docs site works today.** `.github/workflows/docs-site.yml` builds the mdBook and
replaces the root of the target branch, but it deliberately keeps a directory named `studio/`:

```sh
find site -mindepth 1 -maxdepth 1 ! -name .git ! -name studio -exec rm -rf {} +
```

So a built Studio published once to `gh-pages` / `gh-pages-preview` **persists across every later
book deploy**. The current live app at `/studio/` is gh-pages `454bb30fc`, with the M1-capable
integer wasm bundle and both M1 seeds in `models.json` verified by HTTP and hash checks. The earlier
`d8e67c66` publish was pre-M1. Publishing a newer page means replacing that directory once.

**The publisher.** `scripts/studio/build-and-publish.sh` does the whole unit:

1. builds the wasm engine, the seed-5 synthetic demo stack and its stack-bound tokenizer into the
   gitignored `web/studio/pkg/`;
2. runs the worker, static page-load, runtime page-load and wasm-instruction checks;
3. stages the runtime files with `scripts/stage-studio-site.sh` (replacing `studio/` wholesale so
   no file from the retired app survives) and writes `models.json`;
4. makes a publish commit in a throwaway git worktree of `origin/gh-pages` or
   `origin/gh-pages-preview`, checks the GitHub Pages size limits, prints the staged tree, the
   previous head and the rollback line;
5. pushes only when `--push --yes` is given. **The default is a dry run, and `--push` without
   `--yes` refuses.** Live (`--target gh-pages --push --yes`) is refused unless the checkout is
   on an attached `main` branch whose HEAD is `origin/main`.

```sh
# dry run against the preview branch: builds, stages, commits locally, pushes nothing
scripts/studio/build-and-publish.sh --target gh-pages-preview --stage-dir /tmp/studio-staged

# after review, publish the live site from a clean, attached main checkout (owner/Lead only)
scripts/studio/build-and-publish.sh --target gh-pages --push --yes
```

**Why not in CI yet.** Making the Studio build part of the workflow needs
`.github/workflows/docs-site.yml` changed, and `.github/**` is an **owner-only path**
(`docs/harness/owner-paths.json`): a lab PR touching it fails the `owner-paths` gate. DeepSeek's non-repository
deploy report carries the proposed workflow diff (add the wasm/demo/verify build and a
`studio/` replacement to the existing publish step, plus path triggers for `web/**` and
`crates/uor-r4-integer/**`) for the owner to land. Until then this script is the manual publisher;
the proposed workflow reuses the same `stage-studio-site.sh` helper, so the staged output cannot
drift from the script.

### `models.json` (schema `uor-r4.studio-models/1`)

Generated by `scripts/stage-studio-site.sh`; `models[]` entries become cards in the Model sources
panel. `kind: "bundled"` entries must stay same-origin and override the bundled demo; every other
kind is a **published** card: its `url` may be cross-origin (for example a Hugging Face resolve
URL), and it is fetched only after the user clicks that card's Load button. Optional pins are
`sha256`, `size_bytes`, `format` and `schema`; `tokenizer_url` names the stack-bound tokenizer
artifact that is validated against the downloaded stack before the worker is asked to load it:

```json
{
  "schema": "uor-r4.studio-models/1",
  "sources": {
    "bundled": true,
    "local_file": true,
    "url": {"available": false, "note": "set public_model_url at dispatch to publish a public model URL slot"},
    "openai_compatible_api": {"configured_in_browser": true, "storage_key": "uor.studio.modelSources.v1"}
  },
  "models": [
    {"id": "synthetic-seed-5", "kind": "bundled", "label": "...", "url": "pkg/synthetic-stack.lut",
     "tokenizer_url": "pkg/tokenizer.json", "format": "UORLUT01", "sha256": "...", "size_bytes": 469252},
    {"id": "fbs-dual-c131-seed1", "kind": "published", "label": "... - this site",
     "url": "./models/fbs-dual-c131-seed1.uorlut", "tokenizer_url": "./models/fbs-dual-c131-seed1.tokenizer.json",
     "format": "UORLUT01", "schema": "uor-r4.lut-stack/5", "sha256": "a9df...", "size_bytes": 2144960},
    {"id": "fbs-dual-c131-seed1-hf", "kind": "published", "label": "... - Hugging Face",
     "url": "https://huggingface.co/<org>/<repo>/resolve/main/fbs-dual-c131-seed1.uorlut",
     "tokenizer_url": "./models/fbs-dual-c131-seed1.tokenizer.json", "format": "UORLUT01",
     "schema": "uor-r4.lut-stack/5", "sha256": "a9df...", "size_bytes": 2144960}
  ]
}
```

The model decision is: ship the bundled synthetic demo, a **local-file** loader, a **URL** loader,
an optional **published** trained-model card (same-origin copy and/or a public Hugging Face URL),
and an **OpenAI-compatible API** manager with add-API, all in the Studio page; **no private HF
artifact is published**. The manifest's bundled entry keeps the page usable with no network, the
optional `url` slot (only when a publish supplies `--public-model-url`) is the legacy public-URL
form, and API configuration (base URL / model / key) lives in `localStorage` under
`uor.studio.modelSources.v1`, never in this manifest.

`assets/js/model_sources.js` reads `./models.json` once on init (same-origin; a missing or invalid
file silently falls back to the hardcoded `./pkg/synthetic-stack.lut` demo). It verifies a
published entry's `sha256` and `size_bytes` after download, then validates `tokenizer_url` against
the stack (stack sha256 + vocab) before posting `bytes`, `expectedSha256` and `tokenizerArtifact`
to the worker; the worker re-verifies the hash before loading. A mismatch refuses the load with
the reason shown in the Model sources status line. The page makes no external request at page
load; only a user click on a published card or on Load URL can fetch a remote host.

### Shipping the trained FBS model (cross-origin + tokenizer)

Stage one or two trained `.uorlut` artifacts with `--trained-manifest`. The manifest is local
JSON; nothing is fetched from the network:

```json
[
  {"id": "fbs-dual-c131-seed1",
   "label": "FBS trunk dual-c131 seed 1 (uor-r4.lut-stack/5, DEV scope)",
   "file": "/tmp/.../dual-c131-seed-1-raw_both.uorlut",
   "hf_url": "https://huggingface.co/<org>/<repo>/resolve/main/fbs-dual-c131-seed1.uorlut",
   "tokenizer": "/tmp/.../fbs-dual-c131-seed1.tokenizer.json",
   "note": "Trained FBS trunk export; served for the Studio demo, not a milestone qualification."},
  {"id": "fbs-dual-c131-seed2", "...": "second seed"}
]
```

```sh
# build the stack-bound tokenizer per artifact (BPE vocab 4096 comes from the M1 saved-model inputs)
node web/studio/tools/make_tokenizer.mjs <seed1.uorlut> \
  --source-tokenizer /tmp/hf-m1b/.../inputs/driver-data/tokenizer.json --out <seed1.tokenizer.json>
node web/studio/tools/make_tokenizer.mjs <seed1.uorlut> --check <seed1.tokenizer.json>
# stage (adds <dest>/models/<id>.uorlut + <id>.tokenizer.json and two published cards per model)
scripts/stage-studio-site.sh web/studio /tmp/studio-staged --trained-manifest /tmp/trained.json
```

The helper validates each tokenizer against its artifact (`stack_sha256` + `vocab_size`) and
refuses a file over 100 MB before copying. The bundled synthetic demo and the legacy
`public_model_url` positional slot keep working exactly as before.

### Serving the trained M1 model (Hugging Face)

The trained M1 model is the M1-closing 19-table discrete trunk (`uor-r4.lut-discrete-trunk/1`,
86,536,132 B seed 1 / 84,943,328 B seed 2). It is released publicly at
<https://huggingface.co/caseyallard/uor-r4-m1-model> with both stack-bound tokenizers; the 4-table
Stage-T base exports are at <https://huggingface.co/caseyallard/uor-r4-m1-discrete> and the research
FBS trunk pair at <https://huggingface.co/caseyallard/uor-r4-fbs-dual-c131> (no token writer, so it
loads but cannot generate). The primary Model Source card is **remote-only**: the page makes no
external request at load, and the 86.5 MB artifact is downloaded only after an explicit Load click,
with sha256 and tokenizer validation, then loaded into the discrete wasm runtime and served as
**top-8 next-token prediction** (greedy continuation repeats tokens; the page says so and refuses
chat generation on the discrete engine). The page/worker artifact cap is 128 MB.

```json
[{"id": "m1-trunk-seed1", "label": "M1 model (19-table trunk, seed 1)", "primary": true,
  "hf_url": "https://huggingface.co/caseyallard/uor-r4-m1-model/resolve/main/t16-frozen-zero-seed1-trunk.d11",
  "hf_tokenizer_url": "https://huggingface.co/caseyallard/uor-r4-m1-model/resolve/main/tokenizer-seed-1.json",
  "sha256": "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
  "size_bytes": 86536132, "schema": "uor-r4.lut-discrete-trunk/1", "note": "..."}]
```

```sh
scripts/studio/build-and-publish.sh --target gh-pages --push --yes --trained-manifest /tmp/m1-trained.json
```

### The /7 turn memory

The integer engine serves `uor-r4.lut-stack/7` turn-memory stacks (units 2-3 of
`turn-memory-integer`): both memory tables, the caller's reply start (`markReplyStart`), the
opening/reply keys, and the moves at the readout. The worker reports `memoryInfo` in the loaded
capabilities and calls `markReplyStart` before the first generated token. A memory session's
state is not saved yet, so the worker refuses restore/export/import for it (reload the model to
start again); units 4 (DEV parity under a pre-registered sheet) and 5 (cost) of the item remain open.

### Limits, rollback, and why no artifact is committed

- GitHub Pages limits: **100 MB per file** and **1 GB per published site**; the publisher enforces
  both and prints the file count and sizes, and the proposed workflow does the same.
- **Rollback:** the publisher prints the previous target head and an exact
  `git push --force-with-lease origin <previous-sha>:<branch>` line. Force-pushing a build branch
  to the known-good commit is the rollback; nothing is deleted.
- Build outputs are generated locally/runner-side and land only on the build branch. Nothing is
  committed to `main`: no wasm, no `.lut`, no tokenizer, no manifest.
- The bundled demo is a synthetic **stand-in**; the trained M1 model is loaded from its public
  Hugging Face release (remote-only primary card) and served by the discrete runtime as next-token
  prediction. The page labels both.
