# REPORT: mount the real typed Studio application, kimi-52

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `e9bfd7751e3eb31736178d23289e8d99480bb0d0` (original branch base `ab4a5b328`). The exact final HEAD/tree are captured in `identity.log` and will be posted on PR #3835 after push, not embedded as a self-referential REPORT hash. Before push they are local evidence.

## What did not work

- The typed tree was only an exported library. New bootstrap and built-browser checks failed because there was no `bootstrap.ts`, no typed root, and Vite used library mode.
- The first application build emitted absolute `/assets/...` URLs, so the browser never loaded the bundle beneath the Studio's hosted subdirectory. `viteConfig.test.ts` failed before `base: './'` was added.
- The next mount exposed a readiness race: `getSettings` and workspace tabs were not initialized when the typed shell called them. The bootstrap now waits for those methods and refuses incomplete initialization with a timeout test.
- After the mount worked, the bundled model failed with HTTP 404 because `dist/pkg` was not staged. The build now copies generated model/tokenizer/catalog artifacts there. A final instrument-order issue recorded MCP after model loading had returned to Agent; it now records MCP before that transition.
- Two test-only typing fixes were needed: a structural model-source mock and the optional Vite `build` access. No product behavior was changed to silence a test.

## What changed

- `src/main.ts` invokes `mountTypedStudio` only for `?typed=1`; default `index.html` retains the legacy application. `src/bootstrap.ts` waits for workflow methods, creates the lifecycle store/model adapter, reuses the existing `window.__studioApp.worker` when present, and mounts `StudioApp` through Preact.
- Vite builds `index.html` as an application with relative asset URLs. `src/stage-dist.mjs` copies generated artifacts to `dist/pkg` without adding dependencies or committing binaries.
- A narrow preview-only guard in `mcp_client.js` suppresses legacy auto-boot when `?typed=1`, so only the typed MCP panel mounts and no request is made by the legacy mount. The shared client otherwise retains its behavior.
- Additive browser checks prove a visible typed shell, hidden legacy shell, zero legacy MCP mounts, one typed MCP panel, bundled model readiness, and zero external requests. Existing default-page checks are unchanged and green.

## Measurements

- Bootstrap/config checks at that stage: **7/7 tests pass** (four bootstrap, two main, one Vite config). The final cumulative suite is **384/384 tests across 46 files**.
- Complete typed suite: **384/384 tests across 46 files**.
- Plain-JS suite: **112/112 tests pass**.
- Historical kimi-52 build measurements, not the final code-split head: TypeScript and application build PASS; main JS **284.83 kB** (gzip **86.39 kB**), CSS **118.84 kB** (gzip **20.47 kB**), worker **27.70 kB**, generated WASM **794.67 kB**, uncommitted. Final build dimensions are in the kimi-56 report.
- Historical kimi-52 built-browser preview, not the final request count: `shell=true`, `active=true`, `legacyHidden=true`, `legacyMcp=0`, `typedMcp=1`, `artifactReady=true`, `visible=true`, `typedText=true`, **7 requests, zero external**. Final browser evidence is in the kimi-56 report.
- Default static page, runtime page-load, desktop-client, and worker checks: all PASS; worker save/restore is bit-identical and desktop/worker remote request counts are zero.
- At the final committed repair state, harness build, `loop report --check`, `loop pr-check --local`, and all 11 `loop gates --full` checks pass.

## Checks run at the head

Red evidence is retained under `local/kimi/studio-mcp-connections/superpowers/checks/`, including the missing bootstrap, readiness, base-path, 404, and computed-visibility failures. `kimi-52-precheck/` contains green pre-commit evidence. The intentional `mcp_client.js` D76 scope advisory is disclosed; the shared notebook/FILEMAP advisories are unchanged. Agent-31 returned CHANGES for hidden-root visibility, stale evidence, base identity, and publication wording; the follow-up fresh exact-head review, PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The PR uses `References #3500`; the issue remains open.

## Skills used

Working discipline, scope guard, solution integrity, red-first TDD, systematic bootstrap/build diagnosis, modern web application guidance, and verification before completion. A fresh independent exact-head delivery review follows the committed repair. No model compute, paid resource, credential, real user data, MCP endpoint, or production publish was used.

## Still open

This is a mounted development preview, not final lifecycle migration, production publication, desktop acceptance, or an autonomous/model capability claim. Legacy workflow services remain temporarily bridged during the preview. Later Phase A units replace those owners, finish native/shared packaging and acceptance, and publish only after review. The current runtime allowlist must stage `dist` before any typed publication.

## Plain summary

The typed Studio now mounts visibly and loads the bundled UOR demo. The default source UI remains legacy and no production publish has run. All declared product checks pass; no real endpoint, final migration, packaged-GUI, useful-chat, M7/M8, or accessibility-certification claim is made.
