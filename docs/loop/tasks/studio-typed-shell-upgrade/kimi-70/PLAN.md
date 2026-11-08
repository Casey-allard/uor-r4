# Shared Mounted Studio Runtime Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship one generated Vite `dist` layout for the static preview, native runner, and macOS Tauri app, with a local-only native-runner path and actual packaged-window evidence.

**Architecture:** The existing publisher catalog generator becomes a shared module. Typed runtime allowlisting stages generated hashed assets and pkg/catalog artifacts without legacy/development files. `native-runner.sh --local` builds the existing WASM/pkg stack and local `uor-chat`, then serves the same dist at `/?typed=1`. Tauri stages those bytes and loads `index.html?typed=1` through its existing App URL. Resource checks are supplemented by native-runner CDP and macOS accessibility/UI automation.

**Tech Stack:** Existing Bash/Node/Python/Rust/Tauri/Vite/Preact tools and the existing CDP/macOS accessibility verifiers. No new dependency.

**Spec:** `docs/superpowers/specs/2026-11-08-studio-shared-shell-design.md` and `docs/loop/tasks/studio-typed-shell-upgrade/kimi-70/ORDER.md`.

## Global constraints

- Preserve release defaults, `src-tauri/src/main.rs`, engine/worker/tokenizer/`studio_api.js`, host approval, and unsigned-development boundaries. No production publication or Phase B.
- No real browser data, credentials, external test endpoint, second desktop UI, extra service, or hidden prerequisite.
- `index.html?typed=1` must activate early migration and the mounted application before legacy readers. Any fallback requires a real packaged-window failure first.
- Generated outputs stay uncommitted; typed allowlisting rejects source/test/docs/maps/unexpected files.

### Task 1: Shared publisher catalog and dist catalog

**Files:** create `scripts/studio/catalog.mjs`; modify `scripts/stage-studio-site.sh`, `web/studio/src/stage-dist.mjs`, and focused catalog tests.

- [ ] Add a failing check that `web/studio/dist/models.json` is absent and compare its synthetic-only output with the actual pre-change publisher on identical input. Preserve the Tauri resource catalog's separate release pin `96e296b74530e8488a6c525c59b4dfbd9f980f519ce39c1c338ed7a5cd12554a`; do not confuse that five-model catalog with the local synthetic dist catalog.
- [ ] Extract the current catalog implementation into `catalog.mjs`, preserving public-model/trained-manifest behavior and receipt validation. Delegate both the publisher and `stage-dist.mjs` to it.
- [ ] Rerun publisher/catalog checks and the Vite build; require byte identity with the pre-change publisher for identical input and no behavior change for optional trained/public entries. Commit the shared catalog path.

### Task 2: Typed runtime allowlist and local native runner

**Files:** modify `scripts/studio/runtime-files.sh`, `prepare-tauri-runtime.sh`, `native-runner.sh`, `src-tauri/src/lib.rs`, and focused runtime tests.

- [ ] Add failing tests for typed `index.html`, hashed JS/CSS/WASM, early migration runtime and four pkg files; rejection of source files, maps, tests, docs and unexpected outputs; manifest generation; local runner `/?typed=1` and no-download behavior; and the Tauri App typed URL.
- [ ] Implement an explicit typed copy/verify mode while preserving legacy mode. `prepare-tauri-runtime.sh` stages `web/studio/dist`; `native-runner.sh --local` builds existing WASM/pkg artifacts and local `uor-chat`, then serves that same dist without release downloads. Load `index.html?typed=1` in Tauri.
- [ ] Run focused runtime tests and real native-runner CDP checks: mounted Preact shell, workspace/session/context, actual UOR arithmetic or explicit unsupported state, activity/change inspection, approved command/edit/save where supported, reopened state, and zero external page-load requests. Commit the shared runtime.

### Task 3: Packaged verification and actual desktop evidence

**Files:** modify `scripts/studio/verify-tauri-app.sh` and focused runtime/package tests; add only task-owned verification outputs under `local/kimi/studio-shared-shell/`.

- [ ] Add failing package assertions comparing staged typed files to `web/studio/dist`, rejecting stale resources and requiring the early migration runtime and embedded typed frontend.
- [ ] Update the verifier without weakening embedded-resource, sidecar/hash/icon/development-file boundaries. Build the unsigned app with dedicated target output and run default/GUI Rust suites plus package verification.
- [ ] Exercise the packaged window through macOS accessibility/UI automation: mounted Preact shell, workspace/session/context, actual UOR arithmetic or explicit unsupported state, change/activity inspection, approved command/edit/save where supported, and reopened state. Capture screenshots/AX text and native-host evidence, not only a launch or hash assertion. If `index.html?typed=1` fails, use the smallest reviewed activation fallback and prove it with the same window checks.
- [ ] Commit package verification and native evidence paths.

### Task 4: Records and complete acceptance

**Files:** modify `web/studio/README.md`, `docs/products/web-client.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/research/notebook/m4-serving.md`, and this task's `REPORT.md` only with measured behavior.

- [ ] Record exact shared-runtime behavior, local-only prerequisites, packaged evidence limits, capability differences, and remaining final acceptance/publication work.
- [ ] At the committed head run the complete matrix: `npm test`; plain-JS; typecheck; Vite build; typed native-runner browser; all four legacy checks; default/GUI desktop Rust suites; packaged app verification; harness build; `loop report --check --order docs/loop/tasks/studio-typed-shell-upgrade/kimi-70/ORDER.md`; `loop pr-check --local`; `loop gates --full`.
- [ ] Capture exact HEAD/tree/base/diff/status and task-owned logs under `local/kimi/studio-shared-shell/<HEAD>/`, including restored-source mutation evidence for any deliberate verifier mutation.
- [ ] Obtain fresh `uor-delivery-reviewer` approval; repair findings before pushing. Post `uor-review v1`, keep `References #3500`, require exact-head GitHub checks, use protected queue, verify actual MERGED state, rerun required checks on detached fresh `main`, and clean only this unit's generated outputs. Do not publish production or start another unit in this plan.
