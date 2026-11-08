# REPORT: correct evidence lineage and finish deletion/mutation checks, kimi-64

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `f252285d67d4d4cb7f772dc944c52875f21de10e`, tree `250047b3e34042a5cc7fe94defd5ed697792371c`; original merge base: `efee4049e7a9745b5ddecb3a1c2955c4bcddb861`. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially. Agent-41 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- kimi-62 REPORT claimed the later 414/remount/disposal results at its own 411-test head. Those results belong to kimi-63, not the earlier head.
- Notebook omitted the direct kimi-62 report link. The wrong-row log demonstrated DOM attributes but not a production source mutation. The generation-specific delete-error branch lacked a direct test.

## What changed

- kimi-62 is restored to its actual 411-test evidence and kimi-63 to its actual 414-test evidence; 415 belongs to kimi-64/current. The notebook now links every repair report and records the current 415 total.
- A temporary production mutation marked `session-new-1` active and made the replacement assertion fail as required. Exact source restoration was confirmed by `git diff --exit-code`, and the same assertion passes afterward. No mutation remains in the delivery.
- A modal-open-then-generating test proves the stop-first delete error and no deletion call. Verification/read failures retain their separate error text.

## Measurements

- Source mutation: replacement assertion **FAILED** as required in `replacement-source-mutation.log`; restored source assertion **PASS** in `replacement-source-restored.log`.
- Generation-specific delete refusal: **PASS** in `generation-delete-refusal.log`, with the stop-first message and zero deletion calls.
- Final complete suite: **415/415 tests across 47 files with zero unhandled errors**. Plain-JS **112/112**, typecheck/build, all four legacy checks, report/local-PR checks, and all 11 full gates pass. Browser exact response, exact transcript restoration, and delayed-request rejection remain green.

## Checks run at the head

Evidence is under `local/kimi/studio-agent-session/`: `replacement-source-mutation.log`, `replacement-source-restored.log`, and `generation-delete-refusal.log`. The exact-head directory contains identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs after commit. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 remains draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Receiving review with base verification, source-level mutation testing, red-first TDD and honest-test guidance, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This completes evidence lineage and deletion/mutation coverage, not Phase A. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

The kimi-62 report now matches its actual 411-test head; current results belong to the correct repair reports. A real source mutation failed and was restored, and the generation-specific delete branch is proven. Fresh review and protected delivery remain pending.
