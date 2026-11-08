# REPORT: bind replacement-row and final-instrument evidence, kimi-62

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `02804377ee71d0fdc7509e900d81619645dc2fa4`, tree `f07340c38bdbd9227b31f278d9926950fb33b493`; original merge base: `2ef27b81fc7adc7910ab9466d8305dff44f45dbf`. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially. Agent-39 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- The SessionList deletion test accepted any row titled `New conversation`; duplicate titles and a wrong active marker could satisfy it. The replacement's real ID was not present in the DOM.
- The notebook stopped at kimi-60 and did not state zero unhandled errors or cite the instrument repair. The kimi-61 report also described final results as merely preserved and located the focused log incorrectly.
- Roadmap text said every changed-session run stops, contradicting the explicit `canStop === false` refusal.

## What changed

- Existing native session rows now expose their real `data-session-id`. The deletion test requires `li[data-session-id="session-new-2"][aria-current="true"]`, its expected title, and no current marker on the old row, so title-only or wrong-row evidence cannot pass.
- Notebook links now run through kimi-62 and state 411/411 with zero unhandled errors. The final report names that result and the correct focused/exact-head log paths. Roadmap distinguishes stoppable runs from non-stoppable predictions.

## Measurements

- Red-first SessionList replacement assertion: **1 failure / 6 passes** because no session ID existed. After adding it: **7/7 focused tests pass** with the exact replacement active row.
- At the kimi-62 head: **411/411 tests across 47 files with zero unhandled errors**. Plain-JS **112/112**, typecheck/build, all four legacy checks, report/local-PR checks, and all 11 full gates pass. Browser session flow, exact transcript restoration, and delayed-request rejection remain green. The later **414/414** remount/disposal results belong to kimi-63, and the current **415/415** results belong to kimi-64; they are not claimed at this older head. Agent-40 subsequently returned CHANGES for the remount/canStop relabel and evidence gaps; kimi-63 owns those repairs, and kimi-64 corrects this evidence lineage.

## Checks run at the head

`red-replacement-row-before.log` and `replacement-row-after.log` are under `local/kimi/studio-agent-session/`; `teardown-focused.log` is the focused teardown evidence. The exact-head directory contains identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs after commit. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 remains draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Receiving review with base verification, red-first TDD and honest-test guidance, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This completes the replacement-row and final instrument evidence, not Phase A. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

The replacement's real session ID and active marker are now proven, the notebook/report record the final zero-error suite with correct log paths, and the roadmap states the non-stoppable exception. Fresh review and protected delivery remain pending.
