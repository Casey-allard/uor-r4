# REPORT: native-space-state-contract, codex-10

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first CI run at reviewed head `e0e4d949e` sat queued until it was requeued. Canceling the
  stale suite and reopening the PR created duplicate canceled and successful check runs at that
  SHA.
- Rerunning every canceled job made every visible context and the aggregate rollup green, but the
  direct merge-queue mutation still returned `Pull request 5 of 5 required status checks are
  expected.` The problem is check-run history at the old SHA, not a missing test.
- `loop queue` and `gh pr merge` then enabled generic auto-merge rather than creating a queue entry,
  because the PR remained blocked despite the green rollup.

## What changed

This unit changes documentation only. It records the queue instrument failure and creates a fresh
head so GitHub starts with one clean pull-request workflow set. The Q31 code, tests, dependencies,
artifact bytes and serving API are unchanged from reviewed head `e0e4d949e`.

Size: one negative note and one repair record, about 50 documentation lines.

## Measurements

- Old exact head: all ten visible non-skipped checks passed.
- Old exact head aggregate check rollup: `SUCCESS`.
- Direct queue mutation at the old exact head: rejected with
  `Pull request 5 of 5 required status checks are expected.`
- No new model measurement or throughput result is made.

## Checks run at the head

- `loop report --check` - passed locally.
- `loop pr-check --local` - passed with existing advisory warnings.
- `git diff --check` - passed.
- The new GitHub head must run exactly one CI, docs and harness workflow set and pass before queue.
- Exact-head review records are posted on PR #3977; this report does not substitute for them.

## Skills used

- `working-discipline`: separated a GitHub check-history defect from the model result.
- `solution-integrity`: verified why aggregate success and queue eligibility disagreed before
  creating a fresh head.
- `native-space-cycle`: confirmed the repair changes no state or operator path.

## Still open

- Queue the fresh head after its single clean workflow set passes.
- Merge, verify the squash on fresh `main`, and remove the reviewed branch/worktree.
- `IntegerStackSession` still needs to consume the loaded Q31 state in its actual token recurrence.

## Plain summary

The model code and every check passed, but GitHub’s check history at one commit became too tangled
to enter the merge queue after queue recovery. This repair makes a documentation-only head so the
same reviewed code gets one clean set of checks and can finish delivery normally.
