# REPORT: prove destination-divergent migration reads, kimi-69

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `c41d1a870127022f538c4a0febb760b372ab0147`, tree `7e1ad67a601fc32df525d6dc4db29c8bed46da16`; original merge base: `a3ac3f326fa95ffe38159cc3c589100cea627664`. Current identity is in `identity.log` and `merge-base.log`. Agent-48 returned CHANGES at the base and blocked ready/queue.

## What did not work

- Theme/current-UI checks used dark/collapsed defaults and an uncontrolled viewport, so they could pass without migrated reads. Other domains were read while source and destination matched, allowing an old-source read route to pass unnoticed.
- Deleted recent-files destination was checked only in storage, not through the rendered recent-files read path.
- Initial marker read failure had no direct no-write test.

## What changed

- Browser migration now runs at 1440×900 with light theme and expanded navigation. Typed controls change theme to dark, navigation to collapsed, add an attachment, and edit workspace content. Every value differs from its old raw before reload.
- Reload asserts the renamed session, font size, dark theme, collapsed navigation, new attachment, and edited workspace are read from destinations while every old migratable raw remains unchanged. The recent-files view must render “No files opened yet” and no recent-file button after destination deletion.
- Fresh UI and workspace module loads re-read divergent destinations in Vitest, rather than relying on initial source-identical values. Initial marker read failure now has an explicit no-write/no-remove source-preservation test.
- Documentation names destination-divergent read evidence. No product behavior change was needed.

## Measurements

- Focused migration/singleton/legacy-routing result: **27/27**. Complete suite: **434/434 across 49 files** with zero unhandled errors. Plain-JS **112/112**, typecheck/build, all four legacy checks, and 11/11 full gates pass.
- Browser first success is `migrated`; marked reload is `current`. Asserted session/transcript/settings/attachments/workspace/tabs/theme/current-UI all restore. Edited values and the empty recent view are asserted after reload, not only in storage.
- One-shot injected browser quota remains `quota-exceeded` with no second activation. Malformed/conflict cases remain fail-closed with no unexpected destinations. Default **24**, typed **13**, mount-error **5**, migration success **23**, each migration failure **11** requests; all have zero external/raw-TypeScript requests and zero runtime errors.
- The activation-disable and old-source-read-fallback mutations both fail the browser assertions. Identity-tagged build/browser logs, real one-line patches, byte-identical restoration, and green restored runs are preserved. No real profile, actual system quota, credential, or external endpoint was used.

## Checks run at the head

Evidence is under `local/kimi/studio-lifecycle-migration/` and the exact-head subdirectory: identity/base/diff, full product suite, typecheck/build, four legacy checks, expanded browser matrix, harness/report/local-PR/full-gate logs, activation-disable and read-fallback mutation evidence, and empty restoration diffs. Accepted advisories remain the notebook, lifecycle runtime asset allowlist entries, and existing FILEMAP rows. Fresh exact-head review, PR creation/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. The PR uses `References #3500`; the issue remains open. No production publisher was invoked.

## Skills used

Receiving code review with evidence verification, red-first TDD, systematic root-cause diagnosis, project research/delivery rules, and verification before completion. No dependency, real-data migration, native/package work, production cutover, or Phase B compute was added.

## Still open

This completes destination-divergent mounted-preview migration evidence, not Phase A. First activation remains single-tab only with no cross-tab atomicity claim. `ui.v1` is archival only. Quota is injected browser/unit evidence, not actual system-quota qualification. Final lifecycle ownership for other subsystems, native/shared packaging, packaged macOS workflows, final visual/accessibility acceptance, reviewed production publication, and packaged-app launch remain. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

Migration checks now prove reads after values diverge from old sources. Theme, navigation, attachments, workspace, sessions, and settings persist after reload, recent files are actually empty after deletion, and initial marker read failure performs no writes. Fresh review and protected delivery remain pending.
