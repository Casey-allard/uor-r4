# Foundry Web Studio (`web/studio`) — UOR-R4 + RV64IMAC MVM

The [public Studio](https://uor-foundation.github.io/uor-r4/studio/) hosts the `UOR-Foundation/foundry` browser Studio GUI (`web/studio/foundry/`, imported from upstream commit `ea917b8d92974c3d78ebd213eaa1c21fd47666bf`), powered by:

1. **Real UOR-R4 Integer Serving (`crates/uor-r4-integer`, `--features serving-wasm`)** via [`uor_model_worker.js`](assets/js/uor_model_worker.js) and [`uor-r4-runtime.mjs`](foundry/studio/shell/uor-r4-runtime.mjs):
   - Supports the bundled `synthetic-seed-5` stack (`pkg/synthetic-stack.lut`), the pinned public M1 discrete trunk (`m1-discrete-trunk-seed-1`) with user-initiated Hugging Face download + SHA-256 verification + persistent OPFS cache (zero off-origin network requests before user selection), and local `.uorlut` / `.lut` / `.d11` file uploads from the Model Picker.
   - Foundry's compiled `IdeaReader` / LexLean / `.holo` workflow (`foundry:reader`) remains available in the Model Picker.
   - Honest status everywhere: `Pre-alpha · M1 complete (1 of 8 milestones) · Integer WASM`.
2. **RV64IMAC MVM Micro-Linux Interactive Terminal (`crates/uor-r4-mvm`, `--features wasm`)** via [`mvm-worker.mjs`](foundry/studio/shell/mvm-worker.mjs) and [`mvm-host.mjs`](foundry/studio/shell/mvm-host.mjs):
   - Boots `WasmMvmSandbox` inside a dedicated Web Worker with persistent Copy-on-Write filesystem snapshots stored in OPFS (IndexedDB fallback), wired to Foundry's **Terminal** tab (`PreviewToolbar` / `PreviewPanel` / `TerminalPanel`) and `/mvm` + `<tool_call>`/`<tool_response>` harness in [`r4-agent-loop.mjs`](foundry/studio/shell/r4-agent-loop.mjs).
3. **Locked Upstream Assets & Zero Committed Binaries:**
   - Only text Foundry sources are committed under `web/studio/foundry/`. The 51 upstream locked assets (fonts, backdrops, wallpapers, `kernel.holo`, Lean kernel, `holospaces_web.js` + `holospaces_web_bg.wasm`) are pinned by SHA-256 in [`foundry-assets.lock.json`](foundry-assets.lock.json) and fetched on build by [`fetch-foundry-assets.mjs`](../../scripts/studio/fetch-foundry-assets.mjs).
   - Retired Preact/desktop-scaffold frontend files from the previous `web/studio/` UI are preserved byte-identical under [`archive/web-studio-preact-2026-11/`](../../archive/web-studio-preact-2026-11/README.md). The Tauri desktop client (`src-tauri/`) stays paused while the Foundry web workbench is stabilized.

## Build, Verify, and Publish

```sh
# Build UOR-R4 WASM, MVM WASM, Foundry core.wasm, Vite renderer, host bundle & dist/
bash web/studio/build_foundry_studio.sh

# Run worker traps and real headless Chrome CDP E2E tests
node web/studio/verify_studio_worker.mjs
node web/studio/verify_foundry_studio.mjs web/studio/dist

# Dry-run or publish to gh-pages /studio/ (from clean main after review)
bash scripts/studio/build-and-publish.sh --target gh-pages --trained-manifest scripts/studio/public-m1.json
```
