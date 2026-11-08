# REPORT: preserve non-stoppable runs on remount and finish evidence, kimi-63

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `95ac4cdf8d6ced7f7fc1cc22f84bc18df7b38d43`, tree `c5de27ac144b5fdbb575d0a641dd1a80c2c779c9`; original merge base: `2ef27b81fc7adc7910ab9466d8305dff44f45dbf`. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially. Agent-40 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- Adapter remount skipped the actual stop call for `canStop === false` but dispatched a synthetic stopped outcome anyway. This cleared a live prediction, allowed a second send, and contradicted the non-stoppable invariant.
- The completion listener had no post-dispose assertion. The wrong-row marker mutation lacked an explicit log, and notebook links omitted direct kimi-60/kimi-61 evidence.
- Delete failures always said “Stop the current reply first,” including verification/read failures. The wrong-row fixture also revealed shared summary mutation across tests.

## What changed

- Synthetic remount stop now requires `active.canStop !== false`. Non-stoppable predictions remain `generating` with their active session and no stop call; actual completion still finishes them. Stoppable remount behavior is unchanged.
- Completion after disposal does not change catalog metadata until an explicit later refresh. The wrong-row mutation rejects the wrong active marker even with the expected title. Shared summary objects are cloned per test.
- Delete errors distinguish generation from verification/read failures. Notebook links the full repair chain, including kimi-60/kimi-61 and this repair, with current zero-error totals.

## Measurements

- Red-first remount: `stopped` instead of `generating`, reproducing Agent-40's finding. Red-first SessionList also included the outdated delete-message expectation and shared-title leakage. After fixes: **26/26 focused tests** and a separate wrong-row mutation pass.
- At the kimi-63 head: **414/414 tests across 47 files with zero unhandled errors**. Plain-JS **112/112**, typecheck/build, all four legacy checks, report/local-PR checks, and all 11 full gates pass. Browser exact response, transcript equality, active-row identity, and delayed-request rejection remain green. The current **415/415** result and generation-specific delete/source-mutation evidence belong to kimi-64, not this older head.

## Checks run at the head

`red-remount-before.log`, `remount-after.log`, and `replacement-wrong-row-mutation.log` are under `local/kimi/studio-agent-session/`. Exact-head identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs are preserved in a head-specific directory after commit. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 remains draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Receiving review with base verification, systematic debugging, red-first TDD and honest-test guidance, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This completes the bounded agent/session authority and evidence, not Phase A. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

Non-stoppable predictions now survive controller remount with actual completion, disposal stops catalog refresh, wrong-row evidence rejects false markers, and reports link the full repair chain. Fresh review and protected delivery remain pending.
