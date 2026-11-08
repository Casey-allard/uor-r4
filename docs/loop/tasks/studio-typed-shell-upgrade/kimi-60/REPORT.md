# REPORT: close remaining agent/session fail-open and fidelity gaps, kimi-60

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `0f6d5c852b01bb60fc57e409502df42f4b4a849f`, tree `820b5aa8f311352eb99e4de7bd15f9471e9b21c3`; original merge base: `56ef8a4feed0fec94b5954ebbfbfd6dca403af68`. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially. Agent-38 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- `duplicate()` invoked the mutating legacy API before checking whether its pre-read succeeded. Resume and rename also mutated before their verification reads. Failed reads could therefore cause hidden side effects despite a null/false controller result.
- Generation completion refreshed only the composer, leaving session message counts/previews stale. A completion listener now synchronizes the real catalog and is removed on dispose.
- The new listener initially introduced a re-entrancy defect: synchronous stop completion refreshed the new session before interruption and changed `stopped` to `active`. Completion refresh is now suppressed only during that interruption window.
- Session-change interruption ignored `canStop === false`. Such predictions now remain running and unlabelled until actual completion, after which synchronization is allowed.
- SessionList asserted only the delete call, not replacement behavior. Repeated interruption was already inert in the red run, so it is now reported as characterization rather than a reproduced defect. kimi-58's command-group assessment is explicitly marked superseded.
- Network capture had no post-evaluation settle window, several StudioApp doubles had inconsistent active/catalog state, no-active delete-result wording was inconsistent, and the prior 384-test entry lacked a historical label.

## What changed

- Duplicate/resume/rename validate the target and catalog before any legacy mutation. Failed pre-reads cannot call those mutating APIs.
- Generation completion refreshes real message counts/previews. Synchronous interruption preserves the stopped snapshot; later ordinary completion synchronization still works. Non-stoppable predictions are never stopped/relabelled.
- SessionList deletion proves dialog closure, no error alert, the replacement active ID, and the replacement row through its real `data-session-id` and active marker. Shared StudioApp doubles have consistent active IDs/catalogs. No-active deletion returns `true` only after target removal; failures remain null.
- Request capture settles for one second after typed evaluation. A request scheduled 500 ms after evaluation is rejected; normal typed requests stay local and the restored transcript remains byte-identical.
- Reports distinguish actual defects from characterization, correct command-group history, mark the 384-test preview as historical, and describe no-active deletion consistently. Current records use 411 tests and actual advisories only.

## Measurements before the final committed repair head

- Red-first focused result: **3 failures / 54 passes** for failed pre-read mutation, stale completion metadata, and `canStop === false`. After repairs: **411/411 tests across 47 files**, typecheck PASS. Typed main JS **88.82 kB** (gzip **28.54 kB**); plain-JS **112/112**. All four legacy checks pass.
- Built browser: two distinct sessions, actual UOR arithmetic `42`, no exact response in the other transcript, byte-identical restored transcript, `generationComplete=true`, and `restoredMatchesOriginal=true`. Post-workflow requests **12**, all local; default **23**, all local; no raw TypeScript and zero runtime errors.
- Delayed mutation: old verifier accepts `https://delayed.example.invalid/probe.png` with an empty external list; repaired verifier rejects that exact URL. Earlier immediate/generic-response mutations also reject correctly. Reserved `.invalid` names only; no real endpoint was used.

## Checks run at the head

Red/focused/full logs are under `local/kimi/studio-agent-session/`; delayed evidence is under `verifier-delayed/`. Exact-head identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs are preserved in a head-specific directory after commit. Fresh-main logs and fresh review are **NOT_RUN**. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes.

Agent-38 returned CHANGES at the base. A later full-suite run produced one intermittent `document is not defined` effect-cleanup error; kimi-61 repairs the test's un-acted unmount as an instrument race without changing production cleanup. The follow-up fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 stays draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Receiving review with base verification, systematic debugging, red-first TDD and honest-test guidance, solution integrity, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This completes the bounded agent/session authority and its evidence. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later Phase A units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

Failed pre-reads can no longer create hidden mutations. Completion refreshes the catalog, non-stoppable predictions are not interrupted, SessionList proves replacement, and delayed requests are caught. Reports now distinguish defects from characterization and use current counts/advisories. Fresh review and protected delivery remain pending.
