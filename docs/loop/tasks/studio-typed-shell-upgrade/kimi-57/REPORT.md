# REPORT: establish one agent/session authority, kimi-57

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `084a7f92e76926f917e1d9ba0d82f01847fad3aa`, tree `776c9bcca56a2c5ad9bc5f02f5c5624b5a21806e`. The current merge base after forward merges is captured in `merge-base.log`; exact final HEAD/tree are captured in `identity.log` and posted on the PR after push rather than embedded self-referentially. Before push they are local evidence.

## What did not work

- `SessionList` and `StudioApp` trusted direct legacy mutations and dispatched activation without verifying active identity. The store had no active-session ID/catalog/revision, and generation pinned only source/privacy. This left composed state dependent on multiple owners.
- Red-first checks failed on the missing controller, undefined `activeId`, generation for a mismatched session, direct component API access, and no active-row marker. These were instrument-valid product gaps, not test-count work.
- Two authority fixture assertions initially used a different fixture than the controller and ignored duplicate insertion order. Four composed doubles returned IDs without catalog entries or success without activation, which the verified controller correctly rejects. Those doubles were corrected to real API behavior rather than weakening assertions.
- An existing structural page-generation double omitted `activeSessionId`; direct invocation threw. The generation adapter now treats the absent structural method as unknown/null while the mounted bootstrap still requires the real method.
- The browser verifier initially had a nested-template syntax error before Chrome launched. Replacing it with concatenation fixed the instrument without changing product behavior.

## What changed

- `LifecycleStore` now owns active-session ID, a cloned session catalog, identity revision, and generation/session snapshots. Ordinary synchronization cannot relabel an active run; the interruption path records the old run as stopped and synchronizes the verified current identity.
- `createAgentSessionController` wraps the existing `SessionApi` and generation adapter. Create/resume verifies post-command active state; rename/duplicate/remove/export use the same reads. First send creates a session before model invocation. Unexpected active-ID change calls legacy stop once, records stopped, and never applies the old run to the new session.
- `SessionList` and `StudioApp` use the controller for session operations and generation. Composer identity comes from the store. Native list items expose `aria-current` without redundant active labels or interactive row roles.
- The built browser creates two distinct sessions, returns UOR's actual exact arithmetic `42` in the original, verifies its absence in the other transcript, and restores it on return with matching active markers. No fixture-generated assistant activity is inserted.
- Documentation now distinguishes this mounted agent/session contract from remaining browser-data migration, full lifecycle ownership, native/shared packaging, desktop acceptance, and publication.

## Historical measurements at the kimi-57 review head

- Red-first authority: missing module plus **2 lifecycle identity failures**; red-first composition: **8 failures**, including direct `sessions` access and no active marker.
- After implementation and fixture correction: **50/50** authority/lifecycle/generation tests and **40/40** composed component tests pass.
- Complete typed suite: **395/395 tests across 47 files**. Plain-JS suite: **112/112**. TypeScript and Vite application build pass; 55 modules are included. Typed main JS **87.34 kB** (gzip **28.26 kB**); index JS **151.91 kB** (gzip **44.00 kB**). Generated outputs remain uncommitted. All four legacy checks pass, including bit-identical worker save/restore and zero desktop/worker remote requests.
- Built browser: `created=true`, both resume ID/marker pairs true, `answerReady=true`, `otherHasAnswer=false`, `restoredAnswer=true`. The actual transcript is `17 + 25 = 42` with the R4 geometric place-value trace. Default/typed/mount-error pages retain zero external/raw-TypeScript requests and zero page runtime errors.
- Built-in-browser visual inspection showed existing sessions rendering with active status through the new authority; no persistent-profile mutation was made. This does not establish final visual parity, real-data migration, or accessibility certification.
- No real browser data, credential, external endpoint, second provider, or Phase B compute was used. Exact arithmetic is protocol evidence, not useful conversation.

## Checks run at the head

Red/pre-final logs are under `local/kimi/studio-agent-session/`. Exact-head identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs are preserved in a head-specific directory after commit. Fresh-main logs are **NOT_RUN**. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Agent-36 subsequently returned CHANGES for no-op operation verification, long rename, read-failure, synchronous interruption, browser-evidence, and smaller record/double gaps; kimi-58 owns those repairs. Agent-37 returned CHANGES for command grouping, repeated interruption, fresh-main wording, and evidence/double gaps; kimi-59 owns those repairs.

Fresh exact-head review, PR push/update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN** at this writing. The PR must keep `References #3500`; the issue remains open. No production publisher was or will be invoked in this unit.

## Skills used

Approved-design brainstorming, isolated worktrees, implementation planning/execution, red-first TDD and honest-test guidance, systematic debugging, solution integrity, modern web accessibility guidance, and verification before completion. The accessibility guide supports native lists and current-state semantics without redundant labels/roles. No new dependency was added.

## Still open

This delivers one mounted agent/session authority, not one-time browser-data migration, final lifecycle ownership for every subsystem, final Claude Code-style visual acceptance, native/shared packaging, desktop acceptance, or production publication. Legacy storage/worker APIs remain in place. Later Phase A units must complete those workflows and prove reopened state on real browser and packaged macOS paths. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

The typed Studio now verifies which session is active before it reports or runs anything. Session operations and generation use one controller and lifecycle store, and a changed session stops the old run instead of adopting it. A real browser proves that two sessions keep different transcripts and restore the original UOR arithmetic answer on return. Migration, native/desktop acceptance, and publication remain open.
