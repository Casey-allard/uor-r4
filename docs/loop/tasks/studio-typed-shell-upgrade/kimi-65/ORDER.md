# ORDER: reconcile final evidence counts and replacement assertion, kimi-65

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `5a7c8dfbe686173809f0539d56772ad867562009`, tree `9504a14715230f905bf69a1652dcbe3c761b0fea`; merge base: `8688eb1a9b26e8b934c3badc7bd0cc05179ebd1f`. Agent-42 returned CHANGES and blocked ready/queue on PR #3843.

## Findings to repair

- kimi-63 REPORT says 415 at its actual 414-test head. Restore 414 there and keep 415 with kimi-64/current.
- The replacement selector checks title before explicitly asserting the row exists. Add a direct non-null assertion so source-mutation failure is unambiguous.
- The restoration diff command's empty output was not preserved. Record it at the final head.

## Cancel probe

Compare every repair report's total against its own exact-head log: kimi-57=395, 58=405, 59=408, 60=411, 61=411, 62=411, 63=414, 64/current=415. Any mismatch blocks review. Run the replacement test after the explicit selector assertion and preserve the restoration diff log.

## Files in scope

- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-63/REPORT.md` for its actual 414 total.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-64/REPORT.md` only to state the corrected lineage.
- `web/studio/src/components/SessionList.test.tsx` for explicit replacement presence.
- `docs/research/notebook/m4-serving.md` for the final report link/evidence only.
- This `kimi-65` repair folder.

No product behavior change, real-data migration, production cutover/publication, native/package work, model/worker/tokenizer/`studio_api.js` change, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- Every report's test total matches its own exact-head evidence; no later result is claimed at an older head.
- Replacement presence is asserted before title/marker checks. The restoration diff log is preserved and empty.
- Report/local-PR/full gates and the complete product matrix pass at the new exact head, followed by fresh exact-head approval.

## Stop and report

Stop before another historical/current evidence mismatch, weakening an assertion, changing product behavior, or publishing. Continue only after fresh approval.
