# REPORT: infra-loop-harness, claude-445 (claude-444's REPORT no longer names a refused invocation)

Lab: claude. ORDER: [ORDER.md](ORDER.md). Base `f495c4cc0`.

## What did not work, first

- **claude-444 left `main` red on the rule it set out to mend.** Its REPORT named, in backticks,
  the harness command with an unknown flag, and the rule refused it. For the time between the
  merge of PR #4197 and the merge of this unit, the named test failed on `main` for that line.
- **The cause was a check not run.** I ran the named test at the code head of claude-444 and not
  again at the head that held its REPORT. The CI job that runs it was still pending when the PR
  was queued.
- **Same-lab review only.**

## What changed

- `docs/loop/tasks/infra-loop-harness/claude-444/REPORT.md`, line 68: the sentence now says the
  binary takes an unknown flag after the command and ignores it, with no backticked invocation.
  The meaning is unchanged.
- No code, rule or test is changed.

## Measurements

| Run | Result |
|---|---|
| Named test on `main` `f495c4cc0` | fails: 1 documented command does not exist (claude-444 REPORT line 68, unknown flag) |
| Named test at the head, with this ORDER and REPORT in the tree | passes |

## What the numbers say

The rule passes on the whole tree again, this folder included.

## Checks run at the head

- G1: `cargo test -p uor-r4-harness the_real_repo_passes_the_ported_rules`: 1 passed, run with
  this REPORT in the tree.
- G2: the CI job "harness rules and tests" at the head, read before the PR was queued; its
  result is on the PR.

## Skills used

None loaded.

## Still open

- **The loop queue does not wait for the job "harness rules and tests".** A PR can be queued
  while that job is pending, which is how claude-444 merged red. Not changed here.
- The items left open by claude-444 stand as listed there.

## Plain summary

The report of the last fix quoted a wrong command as an example, and the check that looks for
wrong commands in documents caught it. The example is now described in words. This time the
check was run on the final text before merging.
