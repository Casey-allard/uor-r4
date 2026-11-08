# Shared mounted Studio runtime design — 8 November 2026

## Intent and why

The mounted Preact Studio now has real session and migration behavior, but the native runner and Tauri still ship a pinned legacy interface. This unit makes the same reviewed Vite output the runtime for the static web preview, native web runner, and macOS Tauri app. The goal is one mounted application and its existing UOR/GLLM integer backend, not another desktop-specific implementation.

## Goals and non-goals

### Goals

- One generated `web/studio/dist/` asset set and one typed allowlist for all three runtimes.
- Native runner and Tauri use `index.html?typed=1`, activating the early singleton migration before legacy readers.
- Native runner has an explicit local-only path using the same local build and local `uor-chat` binary; no release download or external test endpoint is used in that path.
- Packaged app and native-host smoke checks prove actual mounted UI and backend contracts, while static-WASM/native-runner/Tauri capability differences remain explicit.
- Existing release defaults and unsigned-development boundaries remain intact.

### Non-goals and anti-goals

- No second desktop UI, extra loopback service, new dependency, release/publication, or production cutover.
- No model, engine, worker, tokenizer, `studio_api.js`, real browser-data, credential, or external test-endpoint changes.
- No claiming a packaged app is signed, production-ready, conversational, M7/M8-complete, or accessible merely because it launches.
- No replacing current runtime checks with a file existence assertion or a build-only claim.

## Constraints and preservation boundaries

- Work in one isolated worktree and deliver through a protected PR. Keep `.github/**`, generated `dist`, `node_modules`, `pkg`, `target`, and runtime outputs uncommitted.
- Preserve `uor_model_worker.js`, tokenizer/artifact code, native sidecar/RPC behavior, host approval boundaries, and existing release defaults. Local native-runner builds are development evidence only.
- The typed allowlist must accept generated hashed JS/CSS/WASM and the early classic migration runtime, while rejecting source files, tests, docs, maps, and unexpected outputs.
- The early `?typed=1` query is required on native paths so migration and mounted bootstrap run before legacy initialization. If Tauri's App URL does not preserve the query, a narrow tested native activation fallback is required rather than silently shipping legacy UI.

## Approaches

### A. Shared Vite `dist` runtime — selected

Build once, stage the same generated files for all three runtimes, and use a typed allowlist. `native-runner.sh --local` builds local artifacts and serves that output. Tauri stages the same output and loads `index.html?typed=1` through its existing `WebviewUrl::App`.

This is the smallest architecture that reuses the reviewed app, migration, worker, and host adapters. Its main risks are query preservation and generated-file allowlisting, so both require direct evidence.

### B. Loopback-only desktop

Embed the native runner in Tauri and point the window at loopback HTTP. This could reuse the runner URL but adds a second service lifecycle, token/launch coordination, and failure modes without improving UI consistency. Rejected for this unit.

### C. Separate native components

Rewrite a desktop-specific UI. This duplicates state and behavior and contradicts the shared-shell objective. Rejected.

## Selected implementation contract

1. `runtime-files.sh` retains its legacy mode and gains an explicit typed mode. Typed copy/verify includes `index.html`, `models.json`, generated `assets/*.js`, `assets/*.css`, `assets/*.wasm`, and the four declared `pkg` files. Other files are rejected. SHA256SUMS is generated from the staged files.
2. `prepare-tauri-runtime.sh` builds/stages `web/studio/dist` and verifies the same catalog-bound artifacts and manifest. `tauri.conf.json` keeps the runtime frontendDist/resources boundaries.
3. `src-tauri` loads `index.html?typed=1` through the existing App URL. Packaged verification compares staged typed files to `web/studio/dist`, retains embedded-frontend/resource boundaries, and rejects stale/development files.
4. `native-runner.sh --local` builds the local `uor-chat` and Vite artifacts, uses `web/studio/dist` as Studio root, prints `http://127.0.0.1:8000/?typed=1`, and performs no release download. Existing default release behavior is unchanged.
5. Real verification includes the mounted native-runner workflow and packaged Tauri smoke checks: workspace/session/context, actual UOR arithmetic or explicit unsupported state, change/activity inspection, approved command/edit/save where supported, and reopened state. Unsupported host capabilities are shown explicitly rather than simulated. Production publication and final full acceptance are later units.

## Builder latitude

The builder may choose test file layout and exact generated-file filtering implementation, but may not weaken existing assertions, bypass typed mounting, add dependencies/services, change engine boundaries, use real data/credentials, or publish. Any App URL fallback must be narrow, reviewed, and covered by a real packaged-window check.

## Builder-check decisions

1. **Catalog provenance.** Extract the existing publisher catalog generation into `scripts/studio/catalog.mjs` rather than duplicating it. `scripts/stage-studio-site.sh` delegates to the same code, and `web/studio/src/stage-dist.mjs` uses it to create `dist/models.json`. For identical synthetic-only input, the local catalog must match the actual pre-change publisher byte-for-byte. The Tauri resource catalog separately preserves the release's five-model pins at `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`; that is not the synthetic dist catalog's hash. No archive supplies local UI assets.
2. **Artifact build.** `native-runner.sh --local` runs the existing `build_serve_wasm.sh --no-verify` before `npm run build`, producing the WASM/pkg/stack/tokenizer set on a clean checkout. Existing pinned wasm-pack/Rust/npm tools are prerequisites, not new dependencies or test endpoints.
3. **Packaged UI evidence.** Resource/byte identity is necessary but insufficient. The unsigned packaged window is exercised through macOS native accessibility/UI automation, with screenshots/AX evidence for the mounted Preact shell, session/context interactions, and actual UOR arithmetic or explicit unsupported state. No in-app test hook or CDP dependency is added. Zero external page-load requests are asserted in the web/native-runner harness; packaged evidence is shared-UI/native-host smoke, not a browser-network certification.
4. **Host scope.** `src-tauri/src/main.rs` remains unchanged. The mounted native-runner workflow uses `uor-chat` against `web/studio/dist`; packaged checks cover the embedded frontend directly. No separate desktop host is invented.

## Required verification

- Red-first typed allowlist, native-runner local-path, App URL, and package verifier checks.
- Full Vitest/plain-JS/typecheck/Vite build and all four legacy checks.
- Real native-runner mounted workflow with zero external page-load requests.
- Desktop default/GUI Rust suites and packaged app/shared-UI/native-host smoke checks, with exact executable/resource hashes and no development files.
- Exact HEAD/tree/base evidence, fresh independent approval, protected merge, and fresh-main verification. No production publication.
