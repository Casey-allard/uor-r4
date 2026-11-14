# ORDER: native-space-state-contract, codex-10

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-9](../codex-9/ORDER.md). Reviewed head: `e0e4d949e`.

## Why this repair exists

The reviewed code head could not enter the merge queue after Actions recovery. Canceling and
rerunning work created duplicate canceled and successful check runs at the same SHA. The aggregate
rollup returned to success, but the direct queue mutation still returned:

`Pull request 5 of 5 required status checks are expected.`

The code, checks and exact-head approval are unchanged. The defect is the check-run history attached
to the old SHA, not a missing test result.

## Goal

Record the merge-queue instrument failure and create a fresh documentation-only head with one clean
pull-request workflow set. Queue that head without close/reopen, cancellation or duplicate reruns.

## Files in scope

- `docs/loop/tasks/native-space-state-contract/codex-9/REPORT.md`
- `docs/loop/tasks/native-space-state-contract/codex-10/**`

## Size

About 10 to 40 documentation lines.

## Done when

1. The queue failure and its stale check-run mechanism are recorded.
2. The new commit changes documentation only; the native Q31 code tree is unchanged.
3. CI, docs and harness each run once at the new head and pass.
4. The new head has a fresh counted review, `loop pr-check --pr 3977` passes, and the protected
   merge queue accepts the head.

## Stop

- Do not alter Q31 code, tests, dependencies or serving APIs.
- Do not fix or generalize the harness queue command in this unit.
- Do not cancel, close/reopen or rerun healthy checks at the new head.
