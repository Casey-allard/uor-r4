# REPORT: non-destructive mounted browser-state migration, kimi-67

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `7fea6c761417c0e8b589cb85ed1ea11eb1216067`, tree `ef59d3d3fbd473593c612291e690a723f4325b23`; original merge base is the same. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially.

## What did not work

- The foundation only read fixtures and planned copies. It had no transaction, verified marker, destination conflict handling, rollback, mounted routing, or real-browser migration evidence. Copy-only migration would let typed edits rewrite rollback keys.
- Builder check found that rerunning source/destination equality on marked reloads would reject every legitimate edit. Missing-destination resurrection, quota behavior, cross-tab scope, failure recovery, and `ui.v1` semantics were underspecified.
- Failure identity writes could let ordinary legacy initialization overwrite malformed sources. The first built run also omitted the early classic runtime from `dist`, so conversation initialization fell back to empty sessions before the typed bundle migrated.
- The first browser control fixture submitted controlled inputs before Preact rendered their updates. Safe diagnostics showed buttons/dialogs present and no product error; real input events and actual focus/blur corrected the fixture rather than changing product behavior.

## What changed

- Added one classic `lifecycle_persistence.js` implementation and TypeScript delegation. First activation preflights all migratable reads/conflicts, writes and verifies raw-exact destinations, re-reads sources, and writes/verifies the versioned marker last. Rollback removes only destinations and any marker created by the failed attempt.
- A valid marker short-circuits all later source checks. Destinations own subsequent edits and deletions, and missing destinations never resurrect from sources. Quota failures refuse the entire migration; single-tab first activation is the only claim, `ui.v1` is archival only, and no automatic recovery deletion exists.
- Provider state bypasses migration/routing and remains only in `modelSources.v1`. Migration failure preserves identity reads but makes migratable keys read-only and blocks Preact mounting. Default non-typed behavior remains unchanged.
- Early theme, conversation/editor JSON, settings/UI/attachments, tabs/recent, and bootstrap mount checks use the shared runtime without a global localStorage proxy or rewritten persistence stack. `stage-dist.mjs` now copies the early runtime explicitly.
- The real CDP verifier seeds every key before application code, restores state through the mounted application, performs typed rename/font controls, reloads, and asserts non-resurrection and provider isolation. Malformed/conflict variants fail closed with zero runtime exceptions; existing UOR/session/network/mount-error assertions remain intact.

## Measurements

- Red first: transaction **9 failed / 9 passed** before the runtime; integration **7 failed / 1 passed** before call sites; failure preservation **1 failed / 18 passed** before read-only failure writes. Final focused migration/integration result is **27/27**.
- Complete suite: **429/429 Vitest across 48 files with zero unhandled errors**. Plain-JS **112/112**. Typecheck and Vite build pass. All four legacy static/runtime/desktop/worker checks pass.
- Build: typed main **92.87 kB** (gzip **29.84 kB**), index JS **152.33 kB** (gzip **44.08 kB**), typed CSS **5.19 kB**, index CSS **113.65 kB**, worker **27.70 kB**, generated WASM **794.67 kB**, HTML **55.68 kB**. The early classic runtime is staged separately; Vite warns that classic scripts are not bundled.
- Browser requests: default **24**, typed **13**, mount-error **5**, migration success **23**, each migration failure **11**. All have zero external/raw-TypeScript requests and zero runtime errors.
- Migration success restores session/active transcript/settings/attachments/workspace/tabs, performs real typed rename and font edits, preserves all old migratable raws, keeps provider sentinels out of every lifecycle destination, and preserves edits after marked reload. A deleted recent-files destination stays deleted. Malformed/conflict fixtures preserve source/conflicting data with no marker or shell.
- Injected unit failures cover quota, destination write/read-back, source changes, and marker write/read-back, including preservation of identical pre-existing destinations. No real quota condition, actual browser profile, real credential, or external endpoint was used.

## Checks run at the head

Evidence is under `local/kimi/studio-lifecycle-migration/` and the exact-head subdirectory named by the reviewed commit: identity/base/diff, full product suite, typecheck/build, four legacy checks, built-browser, harness/report/local-PR/full-gate logs, and disabled-migration red/restored-green evidence with an empty restoration diff. Actual advisories are the notebook scope, `web/studio/assets/js/lifecycle_persistence.js` and its declaration outside the item's named asset allowlist, and existing FILEMAP rows; read-path and all hard checks pass. Agent-47 returned CHANGES at this base for dual-activation retry, missing mutation/rollback evidence, quota scope, and an overstated restoration claim. Those findings and final delivery are continued in the [kimi-68 repair REPORT](../kimi-68/REPORT.md). The PR uses `References #3500`; the issue remains open. No production publisher was invoked.

## Skills used

Brainstorming/design builder check, writing/executing plans, red-first TDD, systematic root-cause diagnosis, modern web guidance, project research/delivery rules, and verification before completion. No dependency, real-data migration, native/package work, production cutover, or Phase B compute was added.

## Still open

This completes mounted-preview browser-state migration, not Phase A. First activation is single-tab only with no cross-tab atomicity claim. `ui.v1` is archival only. Quota behavior is injected-test evidence, not a real browser quota qualification. Final lifecycle ownership for other subsystems, native/shared packaging, packaged macOS workflows, final visual/accessibility acceptance, reviewed production publication, and packaged-app launch remain. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

Existing migratable browser data now copies forward once and remains byte-identical under later typed edits. Marked destinations own reloads and deletions, provider credentials stay in their original live store, and malformed/conflict/write failures preserve sources and block the typed application. Real browser success/reload/failure workflows and the full product/legacy matrix pass. Fresh review and protected delivery remain pending.
