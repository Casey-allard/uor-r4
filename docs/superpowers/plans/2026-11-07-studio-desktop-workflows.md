# Typed Changes and Terminal workflows implementation plan

## Context

The existing plain-JS `createDesktopActions` adapter already forwards native Git status/diff and guarded terminal operations with per-command approval. `StudioApp` has no Changes or Terminal destinations. This unit adds typed panels through that exact adapter rather than duplicating the native interface or command policy.

## Approach

Add a typed adapter contract/re-export, two panels in `DesktopTools.tsx`, and six-destination navigation in `StudioApp`/`AppShell`. Changes refreshes status and diff together, renders inert bounded results, and copies the exact raw diff. Terminal requires app-session enablement and still calls per-command native approval for every run. Unavailable hosts, empty inputs, failures, cancellation, stale async completion, and workspace changes are explicit.

## Steps

1. Commit the ORDER alone, then this plan. Install pinned packages with `npm ci`.
2. Add red-first `DesktopTools.test.tsx` for the actual adapter command sequence, raw copy, inert output, truncation/timeout, unavailable host, errors, approval cancellation, and stale completion. Update AppShell/StudioApp navigation tests for six destinations. Run and retain the intended missing-module/missing-destination failures.
3. Implement typed adapter contracts and panels. Use existing adapter methods only; no direct `invoke` in components, no real repository/command in tests, and no new dependency. Guard async actions by sequence/unmount and disable active operations.
4. Integrate Changes/Terminal into `StudioApp`, palette commands, section event handling, exports, and restrained typed-only output styling. Preserve existing destinations and generation/source/privacy behavior.
5. Rerun focused tests, then `npm test`, `npm run typecheck`, `npm run build`, `node --test assets/js/*.test.mjs`, and all four legacy checks. Keep Cargo output under `target/kimi-lexlean`.
6. Update typed README, product note, notebook, and REPORT with exact counts, red-first evidence, head/tree, command boundaries, and remaining limits. Run claim/report/local-policy/full gates and review the scoped diff.
7. Commit, push a draft PR with `References #3500`, obtain fresh exact-head review, repair if needed, then protected queue and fresh-main verification. Preserve evidence and clean only this worktree's generated outputs. Do not publish or start another unit in this session.

## Assumptions

- Browser builds show truthful unavailable states; native host behavior is tested only through the existing adapter with fixture `invoke` results, not a new mock host implementation.
- Terminal enablement does not remove per-command approval. The typed panel additionally requires its enabled state before Run is available.
- Repository paths come from the existing optional native `treeSummary().rootPath` or explicit input; no folder name is guessed to be a full path.
- This is typed-component evidence only. Full browser/Tauri mounting, real repository/command acceptance, migration, accessibility acceptance, and publication remain under #3500.

## Verification contract

The report must include missing-panel/navigation red-first evidence, exact command sequences and negative cases, full green counts, exact delivery head/tree, and explicit non-goals. Component fixtures are not real repository, terminal, desktop GUI, accessibility, or model-capability evidence.
