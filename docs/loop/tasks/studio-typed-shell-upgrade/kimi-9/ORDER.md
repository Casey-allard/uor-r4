# ORDER: add typed Changes and Terminal workflows

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500), continuing the phased design after merged composer/rich-reply PRs #3817/#3818. Base: `a60eceb65b7d38a14dba5384c0085c881e8445a7`.

## Goal

Add typed Changes and Terminal destinations to `StudioApp` using the existing native desktop action adapter. Provide bounded Git inspection and guarded terminal workflows with truthful unavailable states, inert output, and no new host interface or live-page cutover.

## Cancel probe

The plain-JS adapter already exposes `createDesktopActions` with `available`, `gitStatus`, `gitDiff`, `enableTerminal`, and `runTerminalCommand`, including per-command native approval. The typed `StudioApp` has no Changes/Terminal destinations. If these cannot be composed through that exact adapter without duplicating host behavior or executing unapproved commands, stop rather than creating a parallel API.

## Files in scope

- New `web/studio/src/adapters/desktopActions.ts` for typed adapter contracts/re-export only.
- New `web/studio/src/components/DesktopTools.tsx` and `DesktopTools.test.tsx` for Changes/Terminal panels.
- `web/studio/src/components/{StudioApp.tsx,StudioApp.test.tsx,AppShell.tsx,AppShell.test.tsx,types.ts}` for destinations/integration.
- `web/studio/src/main.ts`, `web/studio/src/studio.css`, `web/studio/README.md`.
- This task folder and `docs/superpowers/plans/2026-11-07-studio-desktop-workflows.md`.
- `docs/products/web-client.md` and `docs/research/notebook/m4-serving.md` for scoped evidence.

No other paths are authorized. In particular, do not change legacy JavaScript/HTML, `desktop_actions.js`, the Tauri host, engine, worker, tokenizer, `studio_api.js`, model artifacts, publisher, deployment, `.github/**`, `Cargo.lock`, real repositories/commands/browser data, other labs' work, or owner-controlled/generated files. Add no dependency.

## Done when

- Red-first tests prove the missing panels, then exercise the actual plain-JS adapter: explicit Refresh calls `native_git_status` and `native_git_diff`; summary/truncation/timeout states and raw Copy preserve exact diff text; host failures are visible; no HTML in status/diff becomes active.
- Enable Terminal calls `native_enable_terminal`; every Run calls `native_confirm_terminal_command` before any `native_run_terminal_command`, with exact cwd/command/confirmation fields. Cancelled approval runs nothing; errors and empty paths/commands are explicit. Output stays inert.
- Unavailable hosts disable actions without invocation. Async refreshes/runs disable their action, ignore stale completion after workspace change/unmount, and do not fabricate success.
- AppShell and StudioApp expose Agent, Workspace, Changes, Terminal, Models, Guide, with palette navigation and truthful unavailable-host states. Existing destinations/source/privacy behavior remains green.
- Focused and complete Studio tests, typecheck/build, plain-JS tests, legacy static/runtime checks, claim/report/local-policy checks, and full gates pass. PR uses `References #3500`; #3500 stays open.
- Product notes and notebook distinguish typed-component evidence from live desktop parity. No production publish, real repository/command execution, browser-data migration, cutover, useful-chat, M7/M8, production-readiness, or accessibility-certification claim.
- Fresh exact-head review, protected merge, and fresh-main verification are required before cleanup.

## Stop and report conditions

Stop before changing files outside this scope, adding a second host interface, weakening approval/failure behavior, or weakening a test/verifier. Stop after this component unit and protected delivery; do not mount the library, run real commands, invoke the publisher, or start another UI unit in this session.
