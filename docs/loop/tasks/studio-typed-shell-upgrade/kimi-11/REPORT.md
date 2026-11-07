# REPORT: restore committed desktop integration evidence, kimi-11

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `a60eceb65b7d38a14dba5384c0085c881e8445a7`. Prior reviewed head: `e718a4090b4f28949e8a837e74742290997bc4cb`, tree `ccfd1841fb1f07caf94a5c4ce9df9d25fc9a5f8a`. The new exact delivery-head commit/tree and final logs are recorded on PR #3819 rather than embedding a self-referential hash here.

## What did not work

- Fresh review found the kimi-10 product repairs correct but returned CHANGES because `StudioApp.test.tsx` was accidentally omitted from the repair commit. The committed test retained `Native Git is available in the desktop build.` while the committed panel rendered the repaired unavailable message, and the available-host integration test was absent.
- The 358-test logs were therefore from a dirty worktree rather than the reviewed commit. The prior exact-head evidence and REPORT overstated that commit's contents. The CHANGES record is posted as comment `6708670020`.
- The working-tree diff contained only the intended unavailable assertion, adapter import, and available-host StudioApp integration test. No additional product change was needed or included.
- No real repository, terminal command, packaged GUI, live-page cutover, production publish, browser-data migration, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- Committed the omitted `StudioApp.test.tsx` repair as `186d63f9d`, with the correct unavailable assertion and available-host integration test. The evidence-repair ORDER was committed separately as `44114c321`.
- This ORDER changes evidence identity only; the independent Enable/Run identities, empty-rootPath invalidation, empty-input guidance, unavailable wording, timeout/unmount handling, and original adapter/approval behavior are unchanged.
- Updated the notebook artifact chain to include this evidence-repair REPORT. New exact-head suite/gate/diff files supersede the invalid prior evidence rather than relabeling it.

No dependency, legacy JavaScript/HTML, `desktop_actions.js`, Tauri host, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real repository/command/browser data, owner-controlled path, or generated roadmap/issue file was changed. Only the omitted test and authorized records were committed.

## Measurements

- Expected clean-head focused workflow/composition checks: **47/47 tests** (`14` desktop tools, `2` AppShell, `31` StudioApp), matching the already-reviewed test contents.
- Expected complete typed suite: **358/358 tests across 43 files**, now including the committed available-host StudioApp integration test.
- Plain-JS suite: **112/112 tests pass** at the prior head and rerun at this repair head before delivery.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **155.21 kB** (gzip **42.98 kB**) and `dist/studio-foundation.css` **5.12 kB** (gzip **1.45 kB**), uncommitted.
- Final clean-head counts and all legacy/claim/report/local-policy/full-gate results are recorded in the new exact-head logs before fresh review and queueing.

## Checks run at the head

The new exact-head suite/gate/diff files are generated after the omitted test and this REPORT are committed. Earlier red/green logs remain preserved for history but are not used as proof of the new commit. All unique logs remain under `web/studio/.superpowers/sdd/studio-desktop-workflows/checks/` and will be copied to the retained local evidence directory before cleanup.

The D76 scope advisory covers the ORDER-authorized notebook and implementation plan; ORDER-first checks pass. FILEMAP advisories concern unrelated existing folders. No generated roadmap/issue scope was edited, and no new warning was introduced.

Fresh exact-head review, green GitHub checks, protected merge, and fresh-main verification remain required. PR #3819 stays draft until a new review approves the clean head. It uses `References #3500`, and the issue remains open.

## Skills used

Receiving code review with evidence/source identity verification, clean-head reproduction, and verification before completion. A fresh independent exact-head delivery reviewer follows the finalized commit. No model compute, paid resource, credential, real user data, repository, terminal command, or production publish was used.

## Still open

The typed library remains unmounted. Full real-browser/Tauri mounting, real repository and guarded-terminal acceptance, runtime accessibility/mobile/offline parity, browser-data migration, production publication, and useful model conversation remain outside this repair. #3500 remains open for its full completion contract.

## Plain summary

The unavailable assertion and available-host integration test are now committed, so the 358-test result can be reproduced from the reviewed tree. Product behavior is unchanged; the new evidence binds the clean commit rather than the earlier dirty worktree.
