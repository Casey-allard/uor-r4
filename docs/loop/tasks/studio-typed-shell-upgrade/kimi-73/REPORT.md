# REPORT: verify the merged Studio lifecycle migration, kimi-73

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `bfa5de6ce73dbd459f1d2dca1e4e677ccc2bae54`, tree `b5dea93d0dde75f3134cc8576f279d57f08232df`.

## What did not work

- A bare `npm run build` in the fresh worktree could not resolve generated `pkg/uor_r4_integer.js`. The standard local native-runner path generated the uncommitted WASM/pkg inputs and then built successfully; no source import was changed.
- The first complete Task 4 matrix passed product, typed-browser, and four legacy checks, then `loop pr-check --local` rejected the zero-diff branch because it added no ORDER/REPORT. Full gates did not run in that attempt. This documentation unit supplies the required delivery record rather than bypassing the check.
- No migration behavior failed. The original red-first implementation steps are historical and already delivered through PR #3859; they were not reenacted against working code.

## What changed

- Added the requested verification/gap-only plan and this bounded verification ORDER/REPORT. No product or verifier code changed.
- The inline execution ledger records the no-reimplementation ruling, exact commands, and empty implementation commit ranges. PR #3859 and merge commit `000f4b457ee020013654ffbc65f9da5061c2317a` are confirmed ancestors of the base.

## Measurements

- Focused migration transaction: **22/22** plus typecheck. Focused activation/routing/singleton/bootstrap: **10/10** plus typecheck and **113/113** plain-JS tests.
- Complete product suite at the base: **435/435 across 49 files**, **113/113** plain-JS, typecheck and Vite build PASS.
- `UOR_STUDIO_TYPED=1 node verify_studio_page_load.mjs --strict-router`: **RUNTIME PAGE-LOAD TRAP PASSED**. Default 24 requests, typed 13, mount-error 5, migration success 23, each migration failure 11; zero external/raw-TypeScript requests and zero runtime errors.
- Browser success restores all declared domains, preserves old raws after edits, isolates the provider sentinel, changes to `current` after marked reload, renders the empty recent-files view, and does not resurrect the deleted destination. Malformed/conflict/quota modes show the migration error with no shell or marker and preserve sources.
- The isolated local native-runner also returned **NATIVE TYPED RUNTIME PASSED** with the same migration guarantees and zero external/raw-TypeScript requests/runtime errors. Its server was stopped afterward.
- Legacy static page, runtime page-load, desktop client, and worker with bit-identical save/restore all PASS. Harness build and `loop report --check --order docs/loop/tasks/studio-typed-shell-upgrade/kimi-67/ORDER.md` PASS.
- At the corrected documentation head, `loop pr-check --local` is PASS with only the declared scope/FILEMAP advisories, and all **11/11 full gates are GREEN**. Fresh exact-head review, protected delivery, and fresh-main verification are **NOT_RUN** at this report commit.

## Checks run at the head

Evidence is under `local/kimi/studio-lifecycle-migration/<HEAD>/`, including identity/base/diff, complete product and legacy checks, typed browser, harness/report/local-PR/full-gate logs, and final status. Base-head logs are under `local/kimi/studio-lifecycle-migration/bfa5de6ce73dbd459f1d2dca1e4e677ccc2bae54/`. The inline ledger is under this worktree's ignored `.superpowers/sdd/PLAN/`. No deliberate source/verifier mutation was made in this execution, so there is no new mutation-restoration claim. No actual profile, real credential, external endpoint, actual system-quota condition, or production path was used.

The empty-diff PR-check failure and the subsequent missing-heading failure were corrected without changing evidence or weakening checks. The corrected documentation head passes local PR checks and all 11 full gates. Fresh exact-head review, protected delivery, and fresh-main verification are **NOT_RUN** at this report commit.

## Skills used

Inline plan execution with a persistent ledger, red-first TDD rules without reenacting historical failures, systematic root-cause diagnosis, solution-integrity review, worktree isolation, project research/delivery rules, and verification before completion. No dependency, real-data migration, native/package work, production cutover, or Phase B compute was added.

## Still open

First activation remains single-tab only; `ui.v1` is archival only and quota evidence is injected. This unit establishes no production cutover/publication, packaged-app acceptance, useful-chat, M7/M8, accessibility-certification, or Phase B claim. The plan path produces the same scope advisory already accepted for prior Studio planning records; the item's product scope is unchanged.

## Plain summary

The migration was already merged and works at the verified head. This unit delivers only the plan and verification record needed to close the execution loop without duplicating the implementation.
