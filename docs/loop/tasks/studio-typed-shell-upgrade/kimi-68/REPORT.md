# REPORT: repair migration activation and evidence, kimi-68

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `dd8957d0a0a9e5a1b13330d681d9a474c393b448`, tree `e8983d839ba59928a19362c298788e234320b83b`; original merge base: `859621f3373708fdd0b8f78421a494f5f0ee88c4`. Current identity is in `identity.log` and `merge-base.log`. Agent-47 returned CHANGES at the base and blocked ready/queue.

## What did not work

- Classic and bundled runtime evaluation both activated migration. A one-shot first failure could therefore be retried into a successful typed mount, and first success was mislabeled as a second `current` activation.
- The activation-disable patch was empty because the diff path was wrong, and mutation build/browser logs lacked exact identity.
- Marker read-back/read failure and rollback-failure paths lacked explicit tests; marker-write coverage did not assert source preservation.
- The design called for browser quota evidence but only unit injection existed. README/notebook also said “every seeded domain” without checking theme/current-UI.

## What changed

- Global runtime initialization is singleton/idempotent. A second evaluation cannot activate or replace the existing instance. First success remains `migrated`; marked reload is `current`; first failure stays visible. Independent test factories retain direct transaction coverage.
- Added singleton success/failure tests and marker mismatch/read-failure/rollback-failure coverage with source and identical-destination assertions. Marker-write coverage now checks every source.
- Added a real-browser one-shot injected quota fixture. It remains `quota-exceeded` with no shell or marker, all old raws intact, and no second activation. Success now requires exact first `migrated` status; reload requires `current`.
- Browser restoration now asserts theme and current-UI as well as sessions/messages/settings/attachments/workspace/tabs. Documentation names measured domains and injected—not actual system—quota scope.
- Mutation evidence is preserved as a real one-line patch with identity-tagged build/browser red and restored-green logs and byte-identical restoration.

## Measurements

- Red first: singleton success/failure **2 failed / 21 passed** before the guard; after fixture correction and singleton implementation, **23/23** focused tests pass. The marker/rollback additions pass and total **433/433 across 49 files** with zero unhandled errors in the complete suite. Plain-JS **112/112**, typecheck/build, all four legacy checks, and 11/11 full gates pass.
- Build: typed main **92.99 kB** (gzip **29.87 kB**), index JS **152.33 kB** (gzip **44.08 kB**), typed CSS **5.19 kB**, index CSS **113.65 kB**, worker **27.70 kB**, generated WASM **794.67 kB**, HTML **55.68 kB**. Vite continues to warn that the early classic script is not bundled; staging copies it explicitly.
- Browser success requires first `migrated`, restores asserted session/active transcript/settings/attachments/workspace/tabs/theme/current-UI, completes typed rename/font edits, preserves all old raws and provider isolation, and proves marked reload persistence and deleted-destination non-resurrection.
- One-shot browser quota stays `quota-exceeded` with no shell/marker or second activation and no unexpected destinations. Malformed/conflict cases remain fail-closed. Default **24**, typed **13**, mount-error **5**, migration success **23**, each migration failure **11** requests; all have zero external/raw-TypeScript requests and zero runtime errors.
- Actual one-line activation-disable patch fails browser migration assertions. Byte-identical source restoration yields an empty diff and a full green rerun. Marker mismatch/read failure and rollback failure have explicit source-preservation assertions. No actual system quota, real profile, credential, or external endpoint was used.

## Checks run at the head

Evidence is under `local/kimi/studio-lifecycle-migration/` and the exact-head subdirectory: identity/base/diff, full product suite, typecheck/build, four legacy checks, built-browser including quota, harness/report/local-PR/full-gate logs, and identity-tagged activation-disable red/restored-green evidence with an empty restoration diff. Accepted advisories remain the notebook, `lifecycle_persistence.js`/`.d.ts` asset allowlist entries, and existing FILEMAP rows; read-path and hard checks pass. Agent-48 returned CHANGES for non-discriminating theme/current-UI and read-route evidence, plus the recent-view and initial-marker-read gaps. Those findings and final delivery continue in the [kimi-69 repair REPORT](../kimi-69/REPORT.md). The PR uses `References #3500`; the issue remains open. No production publisher was invoked.

## Skills used

Receiving code review with evidence verification, red-first TDD, systematic root-cause diagnosis, project research/delivery rules, and verification before completion. No dependency, real-data migration, native/package work, production cutover, or Phase B compute was added.

## Still open

This repairs mounted-preview migration activation and evidence, not Phase A. First activation remains single-tab only with no cross-tab atomicity claim. `ui.v1` is archival only. Quota is injected browser/unit evidence, not an actual system-quota qualification. Final lifecycle ownership for other subsystems, native/shared packaging, packaged macOS workflows, final visual/accessibility acceptance, reviewed production publication, and packaged-app launch remain. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

The runtime now latches its first result: a second evaluation cannot hide failure or duplicate success. Browser and unit evidence cover first status, one-shot quota, marker verification, rollback failure, and source preservation. The real mutation patch and restored-green chain are preserved. Fresh review and protected delivery remain pending.
