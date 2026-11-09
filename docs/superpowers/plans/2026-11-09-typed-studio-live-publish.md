# Typed Studio Live Publication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish the reviewed neutral Vite workbench through the existing guarded Studio publisher without changing legacy defaults or implying native/hosted capability.

**Architecture:** An explicit `--typed` mode selects `web/studio/dist`, copies it with the existing typed allowlist, regenerates the shared catalog, and reuses the existing dry-run/live/rollback publisher. Legacy mode remains the default.

**Tech Stack:** Bash, Python unittest, existing Node/Playwright verification, Git worktrees and gh-pages.

**Spec:** `docs/superpowers/specs/2026-11-09-typed-studio-live-publish-design.md`

## Global Constraints

- No `.github/**`, model/API/engine/worker/tokenizer, dependency, public-service, or native-consent change.
- Keep legacy mode, dry-run default, main-only live guard, public/trained model inputs, Pages limits and rollback behavior unchanged.
- Static web has no native terminal, background agent service or live R4 API claim.

## Review Focus

- Typed mode must select dist rather than merely rename the legacy source: assert bundled neutral entry/assets and absence of `index.css`, sources and maps in Task 1.
- Default behavior must remain legacy: run the existing default staging test in Task 1.
- Publisher help and staging must agree on `--typed`: assert both in Task 2.
- Generated WASM/pkg must precede Vite build: assert the typed branch runs after the existing artifact build in Task 2 and execute the real dry run in Task 3.
- Public output must not imply hosted native capability: check product/PR wording and the real staged browser result in Task 3.

---

### Task 1: Typed staging

**Files:**
- Modify: `scripts/stage-studio-site.sh`
- Test: `scripts/tests/test_typed_runtime.py`

**Interfaces:** `scripts/stage-studio-site.sh [--typed] <studio-src> <dest-dir> [existing arguments]`; typed mode uses `<studio-src>/dist`, typed copy/verify, and the unchanged catalog/Pages checks.

- [ ] Add `test_typed_pages_staging_uses_dist_and_preserves_legacy_default`: run staging with `--typed`; assert `index.html`, `models.json`, `assets/js/lifecycle_persistence.js`, hashed `index-*.js`/`index-*.css`/WASM and all four pkg files exist, while `index.css`, `src`, `tests`, `README.md` and maps are absent. Run the existing default staging test unchanged.
- [ ] Run `python3 -m unittest scripts.tests.test_typed_runtime.TypedRuntimeContractTests.test_typed_pages_staging_uses_dist_and_preserves_legacy_default`; expect FAIL with `stage-studio-site: unknown option --typed`.
- [ ] Implement only `--typed` parsing, dist source selection and typed copy/verify in `scripts/stage-studio-site.sh`; retain every existing argument and default branch.
- [ ] Rerun the focused test and `python3 -m unittest scripts.tests.test_tauri_runtime`; expect PASS.
- [ ] Commit `[kimi] Add typed Studio staging` with the two named files.

### Task 2: Publisher wiring

**Files:**
- Modify: `scripts/studio/build-and-publish.sh`
- Test: `scripts/tests/test_typed_runtime.py`

**Interfaces:** `scripts/studio/build-and-publish.sh [--typed] [existing options]`; typed mode runs `npm ci` and `npm run build` after artifact generation and passes `--typed` to staging.

- [ ] Add `test_publisher_exposes_opt_in_typed_mode`: assert `--help` succeeds and prints `--typed`; inspect the script to assert typed-only Vite build and staging while the default remains legacy.
- [ ] Run the new unittest; expect FAIL because help has no typed mode.
- [ ] Implement/document the flag, `typed=0` default, typed-only Vite build, mode reporting and `stage_args` forwarding in `build-and-publish.sh`. Do not add options to `.github/**` or weaken `--skip-verify`.
- [ ] Rerun the new test and `bash -n scripts/stage-studio-site.sh scripts/studio/build-and-publish.sh`; expect PASS.
- [ ] Commit `[kimi] Wire typed Studio publisher` with the two named files.

### Task 3: Acceptance and delivery records

**Files:**
- Create: `docs/superpowers/specs/2026-11-09-typed-studio-live-publish-design.md`, `docs/superpowers/plans/2026-11-09-typed-studio-live-publish.md`, and this unit's ORDER/REPORT.
- Modify: `docs/products/web-client.md`, `docs/products/harness-parity.md`, and current item/state pointers only where publication status changes.

**Interfaces:** reviewed branch supplies a reproducible typed dry-run and exact staged root for browser and later gh-pages verification.

- [ ] Run `python3 -m unittest scripts.tests.test_typed_runtime scripts.tests.test_tauri_runtime`, frontend typed/plain-JS/Playwright suites, desktop default/GUI tests, `uor-harness loop gates --full`, and `uor-harness loop pr-check --local`; record exact counts and negatives.
- [ ] Run `scripts/studio/build-and-publish.sh --typed --target gh-pages --keep-worktree --stage-dir <evidence>/stage` as a dry run. Assert no remote ref changed and the staged root passes typed allowlist/catalog checks.
- [ ] Serve that exact staged root locally and mount it in a real browser at desktop/mobile sizes. Require the neutral workbench, every navigation surface, no legacy green page, zero external/raw-TypeScript/runtime errors, and explicit static-terminal unavailability. Preserve screenshots and the staged/publish identities.
- [ ] Complete ORDER/REPORT and product records, obtain fresh exact-head review, protected merge and fresh-main verification. Only then repeat the typed dry run from exact `origin/main`, push the reviewed local gh-pages commit through the script's printed path, watch Pages, and hard-reload the public URL. Preserve the previous gh-pages head for rollback before cleaning generated outputs.
