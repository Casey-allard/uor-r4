# ORDER: make the neutral Studio the default web UI, kimi-16

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Base: `402b4091eb7f7e7ea174f96ed0bbd06cf58fa78b`, tree `5af32980ffc2f95cc52de8ead8e80493184f1f73`. Plan: [2026-11-09-studio-default-cutover](../../../superpowers/plans/2026-11-09-studio-default-cutover.md). Execution is inline with a fresh exact-head delivery review.

## Goal

Make the plain hosted `/studio/` URL enter the neutral Cowork-style workbench instead of leaving the legacy conversation UI in place. Preserve explicit legacy access and every existing model, storage, native-host, and publishing contract. This is a routing/default-cutover repair, not full agent runtime or R4 API parity.

## Cancel probe (run first)

At the verified live gh-pages `6540e2717d3a606330dbd141e3642f3f13b87ecf`, a real browser opened plain `/studio/` into the legacy conversation UI, while `?typed=1` mounted the neutral workbench. `typedPreviewRequested` and early persistence both require explicit `typed=1`; this proves a default-route mismatch rather than a missing build or stale CDN. If the plain root already mounts the neutral workbench, do not repeat this unit.

The automatic `loop claim` check was unavailable because the item's existing scope includes shared `docs/roadmap/README.md` and `docs/roadmap/issues.json`. Those owner-modified files are outside this task and will not be changed. This ORDER uses the normal isolated-worktree/protected-PR path without altering that metadata.

## Files in scope

- `web/studio/index.html`, `web/studio/vite.config.ts`, `web/studio/src/bootstrap.ts`, `web/studio/src/bootstrap.test.ts`, `web/studio/src/main.test.ts`, `web/studio/src/viteConfig.test.ts`, `web/studio/tests/e2e/shell.spec.ts`, `web/studio/tests/e2e/visual.spec.ts`, `web/studio/assets/js/mcp_client.js`, `web/studio/verify_studio_page_load.mjs`, and `web/studio/README.md`.
- `docs/loop/tasks/studio-ide-harness/kimi-16/**` and `docs/superpowers/plans/2026-11-09-studio-default-cutover.md`.
- `docs/integration/current-state.md`, `docs/products/web-client.md`, `docs/products/harness-parity.md`, `docs/research/notebook/m4-serving.md`, and the two existing Studio item records.
- `scripts/tests/test_typed_runtime.py` only if an existing expectation needs to describe the new default truthfully.
- Local evidence under `local/kimi/default-cutover/`.

No `src-tauri/**`, API/model/engine/worker/tokenizer or storage-migration implementation, dependency, `.github/**`, native-consent change, public terminal service, or new model compute.

## Done when

- A real-browser test of `/` fails at base by retaining the legacy URL/UI, then passes by mounting one neutral shell through the built entry while preserving the original URL, other query parameters and theme hash. `?legacy=1` and `?typed=0` keep the old interface without redirecting. Mounting activates migration only when inactive and preserves already migrated/current state.
- The full typed unit and Playwright suites, generated-artifact build, typed staging/WASM checks, local policy and full gates pass at the exact clean head. Existing migration and Tauri inputs are unchanged.
- Fresh adversarial review approves, required GitHub checks pass, and the protected queue returns an actual MERGED result.
- Fresh-main verification passes, the reviewed typed build is published through the existing guarded path, and plain public `/studio/` mounts the neutral workbench at desktop and mobile sizes with no startup/runtime errors. The explicit legacy routes still work and the live terminal limitation remains truthful.
- ORDER/REPORT, product/current records, evidence and rollback identity are preserved before scoped generated-output cleanup. Full agent runtime and live R4 API remain separate pending work.

## Stop and ask if

- The defect is not the demonstrated default-route opt-in, a change to API/model/host authority is needed, or the fix would require a dependency, second runtime, public service, or real browser profile/credentials.
- A test cannot distinguish default typed mounting from legacy fallback, migration or storage behavior must change, or any file outside the declared scope is needed. Stop and re-scope rather than widening the implementation.
