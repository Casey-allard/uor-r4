# Studio Shell Cleanup per D36 — Dead WASM Export Removal, `native_geometric_finish_generation` Guard, Engine-Status Banner, and Provenance Note

- **Date:** 2026-10-15
- **Lab:** Antigravity (`antigravity`, `antigravity/ag1-studio-shell-d36`)
- **Decision / Track:** [D36](../../integration/DECISIONS.md#d36--webassembly-and-github-pages-scope-deployment-stays-frozen-cleanup-only) (`WebAssembly and GitHub Pages scope: deployment stays frozen, cleanup only`) — source-only track, no pods, no model compute, no WASM rebuild
- **Line:** no model line — D36 Studio shell cleanup · count 0/3 unchanged · M1/M4 headlines unchanged
- **References:** `#2028`, [D36](../../integration/DECISIONS.md#d36--webassembly-and-github-pages-scope-deployment-stays-frozen-cleanup-only), [`docs/labs/decisions-d35-d36-2026-10-15/README.md`](../decisions-d35-d36-2026-10-15/README.md)

---

## 1. Scope and Authorization

Per [D36](../../integration/DECISIONS.md#d36--webassembly-and-github-pages-scope-deployment-stays-frozen-cleanup-only) and Kimi PM assignment, this source-only unit executes the four scoped Studio shell cleanups on `index.html` and `assets/js/uor_model_worker.js`:

1. **Remove the five permanently-dead `window.wasmModule.wasm_*` branches in [`index.html`](../../../index.html):**
   - Removed `window.indexWorkspaceFilesGeometric` and `window.queryWorkspaceGeometric` plus their callers in `openLocalDirectory`, `loadTauriGitWorkspace`, `sendChat`, `saveActiveFileLocal`, `executeEditorAgentPrompt`, and `showMonacoAgentDiff`, eliminating all references to the five non-existent exports:
     - `wasm_index_file`
     - `wasm_clear_workspace_index`
     - `wasm_query_workspace`
     - `wasm_get_workspace_stats`
     - `wasm_myers_diff`
   - None of those five symbols exist anywhere in the repository's Rust source (`src/native_wasm.rs`, `src/lib.rs`, or `crates/`), and `window.wasmModule` is never assigned on the main window thread.
2. **Guard `native_geometric_finish_generation` in [`assets/js/uor_model_worker.js`](../../../assets/js/uor_model_worker.js):**
   - At the post-loop completion/cancellation site (`if (job.cancelled || stoppedBy === 'length')`), guarded `wasmModule.native_geometric_finish_generation` (and `wasmModule.native_geometric_cancel` in `cancelActive()`) with `typeof wasmModule?.native_geometric_finish_generation === 'function'`.
   - When the shipped WASM bundle lacks `native_geometric_finish_generation` (as in the frozen 13 October 2026 GitHub Pages bundle built prior to `#1610`), the worker no longer throws `TypeError: wasmModule.native_geometric_finish_generation is not a function` into `generate_error`; instead, it completes cleanly and surfaces an explicit `finishStatus` (`native_geometric_finish_generation absent in shipped wasm bundle (D36: no D11 served wasm engine yet; deploy frozen)`) appended to `stoppedBy` in the `generate_complete` message and rendered in `#nativeStopReason`.
3. **Add explicit D36 engine-status banner in [`index.html`](../../../index.html):**
   - Added `#d36EngineStatusBanner` directly below `<header class="top-navbar">` (plus matching status notes in `.sidebar-footer` and `#sidecarD36Status` inside `.telemetry-sidecar`) stating plainly:
     > **Engine Status (D36):** No D11 served wasm engine exists yet — GitHub Pages deploy is frozen per D36, and the live site is the last good deploy (13 Oct 2026, pre-#2345).
4. **Add explicit provenance note in [`index.html`](../../../index.html) and [`assets/js/uor_model_worker.js`](../../../assets/js/uor_model_worker.js):**
   - Added `#studioProvenanceNote` in the top banner, `.sidebar-footer`, `.telemetry-sidecar`, and the file header comments of `index.html` and `assets/js/uor_model_worker.js` stating plainly that the Studio source is an import of the companion repository `Casey-allard/uor-r4-wasm-chat`.

---

## 2. Hard Prohibitions Respected (D36)

| Prohibition | Verification |
|---|---|
| Do **not** touch `.github/workflows/deploy.yml` (stays disabled: `if: false`) | `git diff origin/main -- .github/workflows/deploy.yml` is empty (`0` lines changed) |
| Do **not** rebuild or replace `pkg/` WASM artifacts | `git diff origin/main -- pkg/` is empty (`0` lines changed) |
| Zero pods, zero GPU/CPU model compute | `$0.00` compute, `0` pod leases, `0` corpus or model updates |
| Preserve `active_studio_worker_has_no_implicit_model_or_provider_fallback` static contract (`src/lib.rs:307-313`) | All four prohibited worker strings (`native_geometric_init(new Uint8Array())`, `UorR4Router`, `@huggingface/transformers`, `computeHopfTelemetry`) remain absent (`0` matches); `node --check assets/js/uor_model_worker.js` passes (note: root `uor-r4-wasm-router` crate remains uncompilable post-`#2345` per D36) |

---

## 3. Verification Receipts

1. **Dead `wasm_*` branch absence in `index.html`:**
   - `rg -n "wasm_index_file|wasm_clear_workspace_index|wasm_query_workspace|wasm_get_workspace_stats|wasm_myers_diff|window\.wasmModule|indexWorkspaceFilesGeometric|queryWorkspaceGeometric" index.html` returns `0` matches.
2. **Worker `native_geometric_finish_generation` guard & syntax in `assets/js/uor_model_worker.js`:**
   - `typeof wasmModule?.native_geometric_finish_generation === 'function'` guards the completion/cancellation call and emits `finishStatus` + honest `stoppedBy` on `generate_complete` instead of throwing.
   - `node --check assets/js/uor_model_worker.js`: `0` syntax errors.
   - `rg -n "native_geometric_init\(new Uint8Array\(\)\)|UorR4Router|@huggingface/transformers|computeHopfTelemetry" assets/js/uor_model_worker.js`: `0` matches (preserving `src/lib.rs:307-313`).
3. **Static & Policy Ratchets:**
   - `python3 scripts/check_claim_wording.py`: `Claim-wording gate passed`
   - `uor-harness ratchet d23`: `OK: no softmax added`
   - `uor-harness ratchet d25`: `OK: no dense matrix product call sites added`
   - `uor-harness check-formats` & `uor-harness shard-check`: `PASS`
