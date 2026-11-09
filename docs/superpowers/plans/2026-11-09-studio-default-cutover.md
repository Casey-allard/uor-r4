# Studio Default Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the plain hosted Studio URL mount the neutral workbench while keeping explicit legacy access and all existing runtime boundaries.

**Architecture:** Use a narrowly scoped Vite `transform` so only the built `typed_entry.js` defaults to the workbench; the source entry keeps its explicit `typed=1` preview guard and makes no raw-TypeScript request. `typedPreviewRequested` defaults to typed except for `legacy=1` or `typed=0`, and `mountTypedStudio` activates migration only when inactive, preserving first-mount and marked-reload state. The original URL, other query parameters, and hash are preserved without a redirect.

**Tech Stack:** Existing HTML/vanilla JavaScript, Preact/Vite, Playwright, Node test tooling, Bash publication scripts, protected GitHub delivery.

**Spec:** `docs/loop/tasks/studio-ide-harness/kimi-16/ORDER.md`

**Execution:** Inline implementation in one isolated worktree, followed by fresh exact-head adversarial review, protected merge, fresh-main verification, and guarded publication. The user's request for the plain hosted UI to match the native workbench is the cutover authorization.

## Global Constraints

- No model/API/engine/worker/tokenizer, storage migration, Tauri, native-consent, dependency, `.github/**`, or public-service change.
- Keep `?typed=1` mounting and early persistence behavior unchanged. Explicit `?legacy=1` or `?typed=0` selects the legacy interface instead.
- Do not claim full Cowork runtime, useful model behavior, a hosted PTY, or live R4 API integration. Static browser limitations remain explicit.
- Test the real page navigation before editing production HTML; no mock-only acceptance or post-hoc test.

## Review Focus

- Empty query: enter the typed workbench, not the legacy conversation page.
- Explicit `legacy=1` or `typed=0`: retain the old interface without a redirect.
- Additional query parameters and `#theme=light`: remain unchanged during default mounting.
- Existing `typed=1`: do not create a navigation/reload loop or a second model worker.
- Native `index.html?typed=1`: remains unchanged and needs no routing or storage edit.

---

### Task 1: Default-route regression and built-entry cutover

**Files:**
- Create: `docs/loop/tasks/studio-ide-harness/kimi-16/ORDER.md`, `docs/loop/tasks/studio-ide-harness/kimi-16/REPORT.md`, `docs/superpowers/plans/2026-11-09-studio-default-cutover.md`.
- Modify: `web/studio/index.html`, `web/studio/vite.config.ts`, `web/studio/src/bootstrap.ts`, `web/studio/src/bootstrap.test.ts`, `web/studio/src/main.test.ts`, `web/studio/src/viteConfig.test.ts`, `web/studio/tests/e2e/shell.spec.ts`, `web/studio/tests/e2e/visual.spec.ts`, `web/studio/assets/js/mcp_client.js`, `web/studio/verify_studio_page_load.mjs`, `web/studio/README.md`.
- Correct typed-preview-versus-live wording now in `docs/integration/current-state.md`, `docs/products/web-client.md`, `docs/products/harness-parity.md`, `docs/research/notebook/m4-serving.md`, `docs/roadmap/items/studio-ide-harness.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`; add actual default-route deployment facts only after publication.

**Interfaces:**
- Consumes: existing `typedPreviewRequested(search)` and early persistence activation, both of which already recognize `typed=1`; the typed Vite publisher and existing Playwright configuration.
- Produces: plain `/studio/` mounts the neutral workbench without a redirect; `?legacy=1` and `?typed=0` retain the legacy page; other parameters and hash are preserved. No HTML source, model, storage migration, or native API signature changes.

- [x] **Step 1: Write the failing test**

Add a real Playwright case to `web/studio/tests/e2e/shell.spec.ts`:

```ts
await page.goto('/?publish=cutover#theme=light');
await expect(page).toHaveURL('/?publish=cutover#theme=light');
await expect(page.locator('#studio-typed-root')).toHaveAttribute('data-active', 'true');
await expect(page.locator('#app')).toBeHidden();
await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
```

Also cover `?legacy=1` and `?typed=0`: each must keep the legacy `#app` visible and must not activate the typed root.

- [x] **Step 2: Run the test to verify it fails**

Run: `npx playwright test tests/e2e/shell.spec.ts --grep "default URL enters the typed workbench"`.
Expected: FAIL because the plain root retains the legacy URL and `#app` remains visible. Legacy fallback cases pass before and after the fix.

- [x] **Step 3: Implement the built-entry default in `web/studio/vite.config.ts` and `web/studio/src/bootstrap.ts`**

Keep the source `typed_entry.js` preview guard unchanged, and use the scoped `studio-default-typed-entry` Vite transform to replace only the built guard with `get('legacy') !== '1' && new URLSearchParams(globalThis.location?.search || '').get('typed') !== '0'`; missing guards fail the build. `typedPreviewRequested` defaults to typed except for `legacy=1` or `typed=0`. `mountTypedStudio` activates migration only when inactive, preserving an already migrated or current store, and applies both explicit theme queries and hashes. The source redirect is removed, so the original URL and hash remain unchanged and the source page makes no raw-TypeScript request. Legacy screenshot fixtures explicitly use `?legacy=1` so their original coverage remains intact.

Update `web/studio/README.md` to describe default typed mounting and the explicit legacy fallbacks without claiming a full runtime cutover.

- [x] **Step 4: Run focused and full checks**

Rerun the focused case, then `npm test`, `npm run typecheck`, `npm run test:e2e`, and `python3 -m unittest scripts.tests.test_typed_runtime scripts.tests.test_tauri_runtime`. Require every result to pass. Build `web/studio/dist`, verify the typed staged output and direct WASM checks, and inspect real default/legacy desktop and mobile navigation with zero runtime errors.

- [ ] **Step 5: Commit and deliver**

Commit the failing-test evidence separately from the production fix, then commit REPORT and verification evidence references at the final head. Obtain fresh exact-head approval, required checks, protected queue acceptance, actual merge, and fresh-main verification.

- [ ] **Step 6: Publish and verify the actual default page**

From exact fresh main, run the existing typed publisher/allowlist checks, publish only after they pass, preserve the previous gh-pages head for rollback, and verify plain `/studio/` at desktop and mobile sizes. Require the neutral workbench, one typed root, working explicit legacy fallbacks, no raw-TypeScript or runtime errors, and an explicit static-terminal limitation. Update product/current records and evidence; do not claim the shared agent runtime or live R4 API.
