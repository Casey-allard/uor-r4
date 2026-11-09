---
id: studio-typed-shell-upgrade
title: Migrate the Studio to a typed lifecycle shell without changing its engine
milestone: M7
area: serving
status: ready
open_to: lab
needs_gpu: false
depends_on: [m7-studio-live-model-sources]
done_class: host
scope: [web/studio/src/**, web/studio/assets/js/model_sources.js, web/studio/assets/js/model_sources.d.ts, web/studio/assets/js/model_sources.test.mjs, web/studio/assets/js/studio_files.js, web/studio/assets/js/studio_ui.js, web/studio/assets/js/studio_ui.d.ts, web/studio/package.json, web/studio/package-lock.json, web/studio/tsconfig*.json, web/studio/vite.config.*, web/studio/index.html, web/studio/legacy.html, web/studio/verify_*.mjs, web/studio/index.css, web/studio/README.md, docs/products/web-client.md, scripts/stage-studio-site.sh, scripts/studio/**, docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md, docs/superpowers/plans/2026-11-01-studio-foundation.md, docs/superpowers/plans/2026-11-02-studio-core-workflows.md, docs/superpowers/specs/2026-11-02-studio-ide-experience-design.md, docs/superpowers/plans/2026-11-02-studio-ide-experience.md, docs/loop/tasks/studio-typed-shell-upgrade/**, docs/roadmap/items/studio-typed-shell-upgrade.md, docs/roadmap/issues.json, src-tauri/src/lib.rs, src-tauri/src/main.rs, src-tauri/tests/**, web/studio/assets/js/desktop_actions.js, scripts/tests/test_typed_runtime.py, docs/products/desktop-client.md, docs/integration/current-state.md, docs/superpowers/plans/2026-11-08-studio-final-acceptance.md, docs/superpowers/specs/2026-11-08-studio-final-acceptance-design.md]
---

## Why it matters

DeepSeek's v2 Studio is published and its integer engine, worker, tokenizer, and `studio_api.js` implementation are preserved. The typed shell includes narrow, tested model-source/provider and live-page bridge corrections because application, model, file, session, and privacy state still have separate owners and multiple DOM status mirrors. The owner approved a later Preact + Vite + TypeScript shell to make those states coherent by construction. This is a new scope, not a reopening of the completed plain-JS live-model-sources item.

## What is missing

The typed tree mounts `StudioApp` through a real Vite application build behind `?typed=1`, reusing the existing worker and legacy workflow adapters. A shared agent/session controller owns verified session identity and operations, and early copy-forward migration preserves old migratable keys while marked destinations own later edits/reloads. One generated `dist` now also supplies the mounted preview, `native-runner.sh --local`, and the unsigned macOS Tauri development app; the publisher/dist catalog generator is shared and typed allowlisting rejects source/development/unexpected files. The local native runner builds the existing sidecar and serves `/?typed=1` without a release download. Tauri loads `index.html?typed=1` and cannot overwrite current WASM/JS with the older archive. Native-browser session/UOR/migration checks and actual packaged-window session reopen, attachment, native save, Git diff, and approved-command checks have run. The packaged acceptance path also repaired a missing default Tauri `invoke` in the typed Git/terminal adapter. One packaged Changes result included timeout flags; a follow-up bounded-process repair now has deterministic closed-pipe regressions and a fresh packaged Changes result with the real unborn branch and complete diff without either flag. The existing legacy `/studio/` remains published; the typed shell has not been cut over or published, and this unit performed no typed production publication. Final full browser/desktop acceptance, mobile/accessibility acceptance, protected delivery of this shared-runtime unit and its process repair, and typed production publication remain. First migration is single-tab only and `ui.v1` is archival only.

## Done when

- A foundation PR adds the typed lifecycle store, narrow engine/persistence adapters, and fixture-backed tests without changing the live page, engine/worker/tokenizer/`studio_api.js`, publisher, or real browser data. Narrow, tested model-source/provider and live-page bridge corrections are permitted.
- Later workflow PRs migrate the shell and every current feature with a parity record for each, preserving those same implementation boundaries.
- Source selection, artifact readiness, download bytes, finalization, file dirty/save state, session generation, and endpoint privacy derive from one store, including stale-load rejection and disabled empty-state actions.
- One-time copy-forward migration preserves old keys byte-identically, except the live `modelSources.v1` compatibility key, which is never copied into a second secret store and may keep changing under its existing writer. Migration tolerates unknown fields, never logs credentials, and passes tests before any write or marker.
- Legacy and new checks pass at their respective pages, including worker parity, offline requests, keyboard/focus, mobile, and accessibility. Full feature parity and a fresh exact-head review precede cutover; production publication is a separate owner decision.
- Preact/Vite/TypeScript and test dependencies are pinned; generated output, node_modules, model artifacts, credentials, and `.github/**` are not committed or edited.

## How to start

Read the rebased/rescoped design at `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`. The foundation merged in #3520 and the core workflows are implemented in PR #3541. Continue with the live IDE experience and remaining real-browser parity/migration work rather than reopening the core library. Do not invoke the publisher or migrate real browser data; production publication is a separate owner decision.

## Rules that apply

- **Preservation:** no model, engine, worker, tokenizer, `studio_api.js`, Claude-owned, generated-binary, or production-deployment changes. Narrow, tested model-source/provider and live-page bridge corrections are in scope.
- **Evidence:** no assistant/model capability, browser parity, accessibility, or offline claim without the corresponding executed check; pre-existing failures stay disclosed.
- **Scope:** full cutover is not part of the foundation PR, and this item does not alter the completed live-model-sources item's historical plain-JS constraint.

## Sources

- #3440 and `web/studio/README.md`
- `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`
- `web/studio/assets/js/model_sources.js`, `studio_files.js`, `studio_ui.js`, and `index.html`
