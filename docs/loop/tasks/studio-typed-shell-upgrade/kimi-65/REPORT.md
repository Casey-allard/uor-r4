# REPORT: reconcile final evidence counts and replacement assertion, kimi-65

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `5a7c8dfbe686173809f0539d56772ad867562009`, tree `9504a14715230f905bf69a1652dcbe3c761b0fea`; original merge base: `8688eb1a9b26e8b934c3badc7bd0cc05179ebd1f`. Current identity is in `identity.log` and `merge-base.log`, not embedded self-referentially. Agent-42 returned CHANGES at the base and blocked ready/queue on PR #3843.

## What did not work

- kimi-63 REPORT incorrectly used the later 415 total at its actual 414-test head. The correct lineage is kimi-62=411 and kimi-63=414; 415 belongs to kimi-64/current.
- The replacement selector checked its title before explicitly asserting the row exists. The source-mutation failure was valid but less direct than it should be.
- The restoration diff's empty output was not retained as evidence.

## What changed

- kimi-63 now records 414 at its own head and attributes 415 to kimi-64/current. kimi-64 explicitly records the corrected split, and the notebook links this final reconciliation.
- The replacement assertion now requires a non-null real active row before its title/marker checks.
- The empty restoration diff is preserved at the final head.

## Measurements

- Report-total audit: kimi-57=395, 58=405, 59=408, 60=411, 61=411, 62=411, 63=414, 64/current=415. Each total now matches its exact-head log.
- Replacement test: explicit non-null real active row, expected title, and no active marker on the old row. Final complete suite: **415/415 across 47 files with zero unhandled errors**; plain-JS **112/112**, typecheck/build, all four legacy checks, report/local-PR checks, and all 11 full gates pass. Browser exact response, exact transcript restoration, and delayed-request rejection remain green.

## Checks run at the head

`replacement-restore-diff.log` is empty at the final head, confirming no source mutation remains. `replacement-source-mutation.log` and `replacement-source-restored.log` preserve the real source-mutation red/green pair. Exact-head identity, full-suite, four-legacy, harness/report/local-PR/full-gate, and built-browser logs are preserved after commit. Actual advisories are the notebook scope and existing FILEMAP rows; read-path passes. Fresh exact-head review, PR update, GitHub checks, ready/queue/merge, and fresh-main verification are **NOT_RUN**. PR #3843 remains draft with `References #3500`; issue #3500 remains open. No production publisher was invoked.

kimi-66 completes the evidence links and reproducible mutation records; this report remains the reconciliation unit.

## Skills used

Receiving review with exact-head verification, source-level mutation evidence, and verification before completion. No dependency, real-data migration, native/package work, or production cutover was added.

## Still open

This completes evidence lineage and assertion precision, not Phase A. Real browser-data migration, final lifecycle ownership for other subsystems, final visual acceptance, native/shared packaging, packaged macOS acceptance, and production publication remain later units. No useful-chat, M7/M8, accessibility-certification, or Phase B claim is made.

## Plain summary

The report totals now match their own heads, replacement presence is explicitly asserted, and the restoration diff is retained. Fresh review and protected delivery remain pending.
