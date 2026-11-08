# ORDER: make AgentComposer teardown deterministic, kimi-61

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Base: `8a04d146a87f33e1c7606ff317d0fc5fcfbd7106`, tree `19db9302dea04850b63770db67ba50009a1b9c33`; merge base: `2ef27b81fc7adc7910ab9466d8305dff44f45dbf`. Agent-38 returned CHANGES at the prior product head; the full suite also produced one intermittent `document is not defined` effect-cleanup error.

## Measured gap

`AgentComposer.test.tsx` imports `act` but unmounts outside it after each test. Preact can defer effect cleanup until after its jsdom environment is gone. `full-vitest.log` at the base captured one unhandled `document is not defined` from `AgentComposer.tsx:140`; focused teardown and a later full run passed, so this is a timing-sensitive instrument defect, not a stable product failure.

## Goal

Flush the unmount/effect cleanup through the existing `act` helper before removing the host and restoring mocks. Do not change production behavior or hide the error with a global guard.

## Cancel probe

Preserve the original intermittent error log, change only the test cleanup, and run focused plus complete suites. If an unhandled error remains, investigate ownership rather than ignoring it or weakening the test.

## Files in scope

- `web/studio/src/components/AgentComposer.test.tsx` only for `act`-wrapped teardown.
- `docs/loop/tasks/studio-typed-shell-upgrade/kimi-60/REPORT.md` only to name this instrument repair.
- This `kimi-61` repair folder.

No production source, real-data migration, cutover/publication, native/package work, dependency, external endpoint/credential, Phase B compute, useful-chat, M7/M8, or accessibility-certification claim.

## Done when

- Every AgentComposer test unmounts through `act` before host removal/mock restoration.
- Focused and complete suites have zero failed tests and zero unhandled errors; legacy checks, report/local-PR checks, and full gates pass at the new exact head.
- The intermittent failure is classified as an instrument cleanup race and fresh exact-head review approves before protected delivery.

## Stop and report

Stop before changing production cleanup, suppressing an unhandled error, or treating a green rerun alone as proof. Continue only after fresh approval.
