# Mounted Studio Browser-State Migration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver one-time, non-destructive browser-state migration for the mounted typed Studio, preserving rollback keys and the live provider credential store while proving post-edit and reopened-state behavior in a real isolated browser.

**Architecture:** A classic JavaScript runtime runs before every legacy storage consumer. It performs a raw-exact copy-forward transaction, writes a verified versioned marker last, and exposes key-routed storage only after success. Existing session/file/UI implementations keep their behavior but call this shared storage API. TypeScript migration exports delegate to the same runtime; they do not fork its logic. `modelSources.v1` bypasses migration and routing. Typed mounting refuses a reported migration error; default legacy behavior remains unchanged.

**Tech Stack:** Existing browser JavaScript, TypeScript, Preact, Vite, Vitest/jsdom, Node's built-in tests, the existing CDP browser verifier. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-31-studio-shell-upgrade-design.md`, including the 8 November 2026 mounted migration addendum.

## Global constraints

- Fixtures only: never inspect or migrate an actual browser profile, use a real credential, or contact an external endpoint.
- Preserve every old migratable key byte-identically. Never copy provider secrets into a second store or include raw values in reports/logs.
- Preflight the complete first transaction before writes. Do not overwrite conflicts, erase malformed data, or hide failures behind defaults.
- A valid current marker short-circuits all later source preflight/copying. Destinations then remain authoritative when they diverge or disappear; never resurrect a missing destination from its old source.
- Roll back only destinations created by the failed attempt. Existing identical destinations and all source keys remain untouched. Quota failure is fail-closed with no selective migration.
- Claim single-tab first activation only; do not claim cross-tab atomicity. `ui.v1` is archival only. Failure recovery requires explicit user repair, never automatic deletion.
- Keep the default legacy page, engine/worker/tokenizer/`studio_api.js`, native/package, publication, and Phase B outside this unit.

### Task 1: Shared migration transaction and router, red first

**Files:** create `web/studio/assets/js/lifecycle_persistence.js`; modify `web/studio/src/persistence/migration.ts` and `migration.test.ts`.

**Contract:** one runtime exports the existing snapshot/plan API plus migration status, raw-exact transaction, and `getItem`/`setItem`/`removeItem` routing. The TypeScript module delegates to it and retains existing exports. The marker is `{"schema":"uor-r4.studio-lifecycle-migration/1"}`.

- [ ] Run `npm ci` in `web/studio`, using only the pinned lockfile.
- [ ] Add failing tests for every key and unknown fields; raw-exact destinations; idempotent marker/no-op repeat; marked edits/reloads and missing-destination non-resurrection; no second provider-secret store or secret diagnostics; post-migration set/remove routing; live provider-key bypass; malformed JSON and invalid markers; destination conflicts; quota, destination/source/marker write and read-back failures; rollback of only new writes; archival-only `ui.v1`; and typed versus default activation. Spy on logs and assert no source values are emitted.
- [ ] Run `npm test -- src/persistence/migration.test.ts`; require the new transaction/routing cases to FAIL against the absent runtime.
- [ ] Implement the shared classic runtime and TypeScript delegation. Preflight all reads/conflicts, write and verify destinations, re-read sources, write/verify the marker last, and roll back only newly created destinations on any failure. On migration failure use identity reads but make migratable keys read-only, and expose only a generic status/error; never throw away source data.
- [ ] Rerun the focused test and `npm run typecheck`; require PASS. Commit the runtime and tests.

### Task 2: Early activation, legacy call sites, and mount refusal

**Files:** modify `web/studio/index.html`, `assets/js/studio_ui.js`, `assets/js/studio_files.js`, `src/bootstrap.ts`, `src/bootstrap.test.ts`, and narrowly related adapter tests.

**Contract:** the classic runtime loads before the inline theme reader. Theme, conversation/editor JSON helpers, settings/UI/attachments, and tabs/recent persistence use the shared API. Storage-size reporting may continue reading aggregate localStorage metadata without values. Model-source/provider code stays untouched. Typed bootstrap refuses a migration error before rendering; absent runtime status in isolated component fixtures retains existing fixture behavior.

- [ ] Add failing bootstrap/call-site tests proving typed edits change destinations but not sources, default activation retains legacy keys, and a migration error reaches the real typed mount error path without a Preact shell.
- [ ] Run the focused tests; require failure on missing routing/refusal behavior.
- [ ] Load the runtime first in `index.html`, route only the identified storage calls, and add the bootstrap guard. Do not alter legacy session/file/settings semantics or add a global localStorage proxy.
- [ ] Rerun focused tests, `npm run typecheck`, and `node --test assets/js/*.test.mjs`; require PASS. Commit the integration.

### Task 3: Real-browser migration and failure fixtures

**Files:** modify `web/studio/verify_studio_page_load.mjs` and its verifier-focused tests/fixtures if needed.

**Contract:** synthetic pre-navigation fixtures seed every old key, including a sentinel provider value that is not a credential. Success proves the migrated session/editor/settings/attachments/tabs render, a representative session edit and settings edit preserve every old migratable raw value, destinations change, the live provider key remains the only secret location, and a real marked reload preserves the edited state without rerunning conflict checks. A deleted destination stays deleted rather than being resurrected. Malformed, destination-conflict, and simulated-quota variants fail closed; browser failures show the typed error with no shell, no marker, no destructive write, and no runtime exception. `ui.v1` receives an archival-copy assertion only. Existing two-session UOR arithmetic flow and mount-timeout fixture remain green.

- [ ] Add the migration fixture assertions and run `UOR_STUDIO_TYPED=1 node verify_studio_page_load.mjs --strict-router`; require migration checks to FAIL before the integrated implementation.
- [ ] Implement only verifier instrumentation and any product correction directly demonstrated by the failure. Keep all existing assertions, including zero external/raw-TypeScript/runtime requests/errors and immediate/delayed external-request rejection.
- [ ] Regenerate artifacts with `CARGO_TARGET_DIR=$PWD/target/kimi-lifecycle-migration web/studio/build_serve_wasm.sh --no-verify`, run `npm run build`, then rerun the typed browser check; require every migration and existing workflow assertion to PASS. Commit the browser evidence path.

### Task 4: Documentation and complete acceptance

**Files:** modify `web/studio/README.md`, `docs/products/web-client.md`, `docs/roadmap/items/studio-typed-shell-upgrade.md`, `docs/research/notebook/m4-serving.md`, and this task's `REPORT.md` only where measured behavior changes.

- [ ] Record the exact migration guarantee, failure scope, fixture-only limits, test counts, build/request measurements, and the fact that default production cutover, native/package acceptance, publication, useful chat, and Phase B remain open.
- [ ] Run the complete matrix at the committed head: `npm test`; `node --test assets/js/*.test.mjs`; `npm run typecheck`; `npm run build`; typed built-browser verification; all four legacy checks; harness build; `loop report --check --order docs/loop/tasks/studio-typed-shell-upgrade/kimi-67/ORDER.md`; `loop pr-check --local`; and `loop gates --full`. Use only `target/kimi-lifecycle-migration` for Cargo output.
- [ ] Capture exact HEAD/tree/base, 0 tracked workspace changes, the exact diff, clean source-restoration evidence for every deliberate verifier mutation, and head-specific logs under `local/kimi/studio-lifecycle-migration/<HEAD>/`.
- [ ] Obtain a fresh `uor-delivery-reviewer` verdict on the exact head. Repair concrete findings and rerun affected checks; do not mark ready on CHANGES.
- [ ] After APPROVE, push without force, post `uor-review v1`, keep `References #3500`, run `loop close --session --pr <PR>`, require exact-head GitHub checks, run remote policy/queue checks, and use the protected queue. Confirm actual MERGED state.
- [ ] Verify squash content and rerun required checks on detached fresh `main`, then perform guarded cleanup of only this unit's generated outputs. Do not invoke production publication or begin another unit in this plan.
