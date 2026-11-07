# REPORT: add typed Changes and Terminal workflows, kimi-9

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Plan: [plan](../../../../superpowers/plans/2026-11-07-studio-desktop-workflows.md). Base: `a60eceb65b7d38a14dba5384c0085c881e8445a7`. Exact delivery-head commit/tree and final logs are recorded on the PR rather than embedding a self-referential hash here.

## What did not work

- `StudioApp` had only Agent, Workspace, Models, and Guide; there was no typed Changes or Terminal workflow despite the existing native adapter. Navigation tests failed for the missing destinations, and the new panel suite initially failed to resolve `adapters/desktopActions.js`. The intended red evidence is in `red-desktop-workflows.log`.
- The new fixture had three missing parentheses in asynchronous assertions, corrected before reaching the missing-module failure. Those are instrument errors, not product defects.
- One host-state test reused the same mounted component for a failure and an unavailable host, preserving old state. Separating those scenarios was necessary. Several mocks also declared only the command name while assertions inspect the argument object; their signatures were corrected. All product tests then passed without changing the implementation to accommodate fixtures.
- No real repository, terminal command, packaged GUI, live-page cutover, production publish, browser-data migration, useful conversation, model capability, M7/M8, production-readiness, or accessibility-certification result is claimed.

## What changed

- Added typed `DesktopActions` contracts and a re-export of the existing plain-JS `createDesktopActions`, plus one repository-path helper that accepts only the optional native `treeSummary().rootPath` rather than guessing a folder name is a full path.
- Added `ChangesPanel`: explicit Refresh calls `native_git_status` and `native_git_diff`, summary reports changed-file count/truncation/timeout, diff output is inert, and Copy uses the exact raw diff without warning suffixes. Host failures and unavailable state are visible.
- Added `TerminalPanel`: explicit app-session `native_enable_terminal`, and every run uses the existing adapter's `native_confirm_terminal_command` before any `native_run_terminal_command` with exact cwd/command/confirmation fields. Approval cancellation runs nothing; empty input and errors are explicit; output is inert.
- Added operation invalidation on workspace change/unmount, resets active-operation state, and uses the latest repository on the next action. Unavailable hosts disable actions without invocation.
- Added six-destination navigation and palette commands, host injection through `StudioApp`, adapter/component exports, and restrained typed-only output styling. Existing generation/source/privacy behavior is unchanged.

No dependency, legacy JavaScript/HTML, `desktop_actions.js`, Tauri host, engine, worker, tokenizer, `studio_api.js`, model artifact, publisher, deployment, `.github/**`, `Cargo.lock`, real repository/command/browser data, owner-controlled path, or generated roadmap/issue file was changed. Cargo output stayed under `target/kimi-lexlean`.

## Measurements

- Focused workflow/composition checks: **39/39 tests pass** (`7` desktop tools, `2` AppShell, `30` StudioApp).
- Complete typed suite: **350/350 tests pass across 43 files**.
- Plain-JS suite: **112/112 tests pass**.
- `npm run typecheck`: PASS with no TypeScript errors.
- `npm run build`: PASS; `dist/studio-foundation.js` **155.00 kB** (gzip **42.94 kB**) and `dist/studio-foundation.css` **5.12 kB** (gzip **1.45 kB**), uncommitted.
- `verify_studio_page.mjs --strict-router`: PASS; both strict pages clean.
- `verify_studio_page_load.mjs --strict-router`: PASS; planted traps detected, actual external request lists empty.
- `verify_desktop_client.mjs`: PASS; binary contract, 32 streamed tokens, 38 invoked commands, zero remote requests.
- `verify_studio_worker.mjs`: PASS; 32 expected tokens, bit-identical save/restore, guarded close/reset, six trap hits, zero remote requests.

## Checks run at the head

Red-first evidence is `red-desktop-workflows.log`; green focused evidence is `green-desktop-workflows.log`. Other local logs are `typecheck.log`, `full-vitest.log`, `build.log`, `plain-js.log`, and `legacy-*.log` under `web/studio/.superpowers/sdd/studio-desktop-workflows/checks/`.

Final exact-head suite/gates, claim/report/local-policy checks, diff/status, and head/tree are rerun after this report and posted on the PR before queueing. Fresh-context review, green GitHub checks, protected merge, and fresh-main verification remain delivery requirements. The PR uses `References #3500`; the issue remains open.

## Skills used

Brainstorming within the approved typed-shell design, implementation planning, red-first TDD, systematic fixture/state/type debugging, and verification before completion. A fresh independent exact-head delivery review follows the committed head. No model compute, paid resource, credential, real user data, repository, terminal command, or production publish was used.

## Still open

The typed library remains unmounted. Full real-browser/Tauri mounting, real repository and guarded-terminal acceptance, runtime accessibility/mobile/offline parity, browser-data migration, production publication, and useful model conversation remain outside this unit. #3500 remains open for its full completion contract.

## Plain summary

Typed Studio now has six destinations. Changes and Terminal use the existing native adapter for bounded Git inspection and guarded commands, with exact raw output and explicit failure/cancellation/stale-result behavior. All declared checks pass, but no real repository, command, or packaged application acceptance is claimed.
