# REPORT: make AgentComposer teardown deterministic, kimi-61

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `8a04d146a87f33e1c7606ff317d0fc5fcfbd7106`, tree `19db9302dea04850b63770db67ba50009a1b9c33`; original merge base: `2ef27b81fc7adc7910ab9466d8305dff44f45dbf`. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially.

## What did not work

One full-suite run at the base failed with unhandled `ReferenceError: document is not defined` in `AgentComposer.tsx:140` during effect cleanup. The test file already imported `act` but called `render(null, host)` outside it after every test. Preact could defer cleanup until after jsdom teardown. Focused tests and a later full run passed, confirming a timing-sensitive instrument race rather than a stable product failure. A green rerun alone was not accepted as proof.

## What changed

Every AgentComposer test now unmounts through the existing `act` helper before host removal and mock restoration. No production cleanup was changed, and no global guard or error suppression was added. The REPORT chain names this as an instrument repair.

## Measurements

- Original evidence: 411/411 tests passed with **1 unhandled error**, so the run failed.
- Focused AgentComposer/StudioApp teardown: **60/60 tests**, no unhandled errors, at `local/kimi/studio-agent-session/teardown-focused.log`.
- Final complete suite at the exact head: **411/411 tests across 47 files with zero unhandled errors**. Plain-JS **112/112**, typecheck/build, all four legacy checks, report/local-PR checks, and all 11 full gates pass. Fresh review and fresh-main verification are **NOT_RUN**.

## Checks run at the head

`full-vitest-teardown-rerun.log` records the interim green rerun; the original unhandled error remains preserved in the base evidence. The exact-head evidence directory contains identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs after commit; the focused teardown log is at `local/kimi/studio-agent-session/teardown-focused.log`. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 remains draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Systematic debugging, red-first preservation, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This is a test-instrument repair only. Agent/session authority delivery still needs fresh exact-head approval and protected merge/fresh-main verification. Real browser-data migration, final lifecycle ownership for other subsystems, native/shared packaging, packaged macOS acceptance, and production publication remain later Phase A units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

AgentComposer test unmounts now flush through `act` before jsdom teardown. Production behavior is unchanged. The intermittent cleanup error is classified as an instrument race, with final checks and fresh review pending.
