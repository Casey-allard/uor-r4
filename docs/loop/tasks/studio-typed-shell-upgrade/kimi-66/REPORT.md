# REPORT: complete evidence chain and mutation reproducibility, kimi-66

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `523ecbaac8f88780100e4f9b04b2bd5c1009cca7`, tree `ce1c6cb82dd3a1da33fc5ec7dbda86aaaeb146b5`; original merge base: `8688eb1a9b26e8b934c3badc7bd0cc05179ebd1f`. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially. Agent-43 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- Notebook linked the kimi-57 ORDER but not its REPORT and stopped at kimi-64, leaving the base and final reconciliation unreachable from one row.
- The source-mutation red log predated the final explicit non-null assertion. The immediate-external verifier variant had its probe removed after the rejection log, making the saved script non-reproducible.
- The DOM-only wrong-row demonstration needed the real source mutation as its primary evidence.

## What changed

- Notebook now links kimi-57 ORDER and REPORT, every repair report through kimi-66, and the full range. It retains 415/415 with zero unhandled errors.
- Repeating the real source mutation against the final explicit non-null assertion produced `expected null not to be null` as required. Exact source restoration produced an empty diff and a passing assertion. The exact one-line wrong-row patch is preserved at `local/kimi/studio-agent-session/replacement-wrong-row.patch`, and the kimi-65 report is now linked before this report.
- The preserved immediate-external variant again contains `https://interaction.example.invalid/probe.png` and rejects that URL at runtime. The delayed variant continues to reject its 500 ms request. Reserved `.invalid` names only; no real endpoint was contacted.

## Measurements

- Final source mutation: `replacement-final-source-mutation.log` FAILS with `expected null not to be null`. Restored source: `replacement-final-source-restored.log` PASS. `replacement-restore-diff.log` is empty.
- Immediate network mutation: `verifier-mutation/current-network-result.log` rejects the preserved probe. Delayed mutation: `verifier-delayed/current-result.log` rejects the 500 ms probe.
- Final complete suite: **415/415 across 47 files with zero unhandled errors**. Plain-JS **112/112**, typecheck/build, all four legacy checks, report/local-PR checks, and all 11 full gates pass. Browser exact response, exact transcript restoration, generation-specific delete refusal, and replacement-row identity remain green.

## Checks run at the head

Evidence is under `local/kimi/studio-agent-session/`: `replacement-final-source-mutation.log`, `replacement-final-source-restored.log`, `replacement-restore-diff.log`, `generation-delete-refusal.log`, `verifier-mutation/`, and `verifier-delayed/`. The exact-head directory contains identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs after commit. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 remains draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

## Skills used

Receiving review with exact-head verification, source-level mutation testing, red-first TDD and honest-test guidance, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This completes the evidence chain and mutation reproducibility, not Phase A. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

The notebook now links the complete evidence chain, the final explicit assertion fails under a real source mutation and passes after exact restoration, and the immediate/delayed external mutations are reproducible and rejected. Fresh review and protected delivery remain pending.
