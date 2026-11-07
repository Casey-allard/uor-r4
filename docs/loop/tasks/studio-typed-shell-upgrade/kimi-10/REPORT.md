# REPORT: repair desktop operation identity and workspace clearing, kimi-10

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `a60eceb65b7d38a14dba5384c0085c881e8445a7`. Prior reviewed head: `02c4ef67055a075409b4fb1eb8b449e08dd03ef2`, tree `927e26a8b41700a57575d226619933aac37d5eaf`. The new exact delivery-head commit/tree and final logs are recorded on PR #3819 rather than embedding a self-referential hash here.

## What did not work

- Fresh review returned CHANGES. Enable and Run shared one operation counter: starting either during the other made the older finalizer stale and left a permanent busy label. The review's CHANGES record is posted as comment `6708346319`.
- `uor-files-changed` ignored `rootPath: ''`, so clearing/importing/demo workspace state allowed stale output and the next action kept the old repository. Tests covered only non-empty replacements.
- Empty terminal path/command was only disabled, with no guidance. Timeout and unmount branches, unavailable invocation, and available-host StudioApp composition lacked direct evidence. Unavailable-Changes wording was also ambiguous.
- Red-first repair checks reproduced seven failures: both overlap directions, both empty-rootPath cases, empty-input guidance, and unavailable wording in component/composed views. Timeout, unmount, and available-host integration controls already passed.
- No real repository, terminal command, packaged GUI, live-page cutover, production publish, browser-data migration, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- Enable and Run now have independent operation refs. Enable is disabled during Run, and Run is disabled during Enable, so no shared-counter stale-finalizer path remains. Completion, errors, workspace changes, and unmount clear the appropriate state.
- Every workspace-path event updates path and revision, including `rootPath: ''`. Stale results are ignored and the next action uses the latest path; unavailable Refresh/Run states remain disabled.
- Empty working directory shows `Choose a project folder and enter an allowed command.`; an empty command shows `Enter an allowed command.` No operation runs.
- Unavailable Changes now says `Native Git is unavailable in this browser build; use the desktop build.` Unavailable invocation is explicitly asserted.
- Added overlap, clearing, empty-input, timeout, unmount, unavailable-invocation, and available-host StudioApp regressions while retaining every original adapter/approval/inert-output test.

No dependency, legacy JavaScript/HTML, `desktop_actions.js`, Tauri host, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real repository/command/browser data, owner-controlled path, or generated roadmap/issue file was changed. Only typed component/tests and authorized records changed.

## Measurements

- Focused workflow/composition checks: **47/47 tests pass** (`14` desktop tools, `2` AppShell, `31` StudioApp).
- Complete typed suite: **358/358 tests pass across 43 files**.
- Plain-JS suite: **112/112 tests pass** at the prior head and rerun at this repair head before delivery.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **155.21 kB** (gzip **42.98 kB**) and `dist/studio-foundation.css` **5.12 kB** (gzip **1.45 kB**), uncommitted.
- Repaired-head plain-JS, strict page, runtime page-load, desktop-client, worker, claim/report/local-policy and full-gate results are recorded in the new exact-head logs before fresh review and queueing.

## Checks run at the head

`red-desktop-repairs.log` retains all seven intended failures. `green-desktop-repairs.log`, `full-vitest-repairs.log`, `typecheck-repairs.log`, and `build-repairs.log` record the repaired implementation. New exact-head suite/gate/diff files supersede the 02c4ef670 evidence; all earlier unique logs remain preserved locally under `web/studio/.superpowers/sdd/studio-desktop-workflows/checks/`.

The D76 scope advisory covers the ORDER-authorized notebook and implementation plan; ORDER-first checks pass. FILEMAP advisories concern unrelated existing folders. No generated roadmap/issue scope was edited, and no new warning was introduced.

Fresh exact-head review, green GitHub checks, protected merge, and fresh-main verification remain required. PR #3819 stays draft until a new review approves this head. It uses `References #3500`, and the issue remains open.

## Skills used

Receiving code review with source verification, red-first TDD, async operation/state analysis, and verification before completion. A fresh independent exact-head delivery reviewer follows the repaired commit. No model compute, paid resource, credential, real user data, repository, terminal command, or production publish was used.

## Still open

The typed library remains unmounted. Full real-browser/Tauri mounting, real repository and guarded-terminal acceptance, runtime accessibility/mobile/offline parity, browser-data migration, production publication, and useful model conversation remain outside this repair. #3500 remains open for its full completion contract.

## Plain summary

Enable and Run can no longer invalidate each other's busy state, clearing the workspace root invalidates stale output, and empty-input/unavailable states are explicit. The expanded component suite is green; no real desktop or command acceptance is claimed.
