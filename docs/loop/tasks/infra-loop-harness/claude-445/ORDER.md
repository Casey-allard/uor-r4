# ORDER: infra-loop-harness, claude-445

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md).
Base: `f495c4cc0` (`origin/main`, the merge of PR #4197). A repair of claude-444, found by its own
verify on fresh `main`.
Setup row: 5 (the nearest row, as in claude-444; no row names this rule).

## Why

`harnessrules::tests::the_real_repo_passes_the_ported_rules` fails on `main` at `f495c4cc0`. The
rule `harness.documented_commands` reports one command: line 68 of
`docs/loop/tasks/infra-loop-harness/claude-444/REPORT.md` names, in backticks, the harness
command with an unknown flag, as an example of a flag the binary ignores. The rule is right to
refuse it: a document must not name an invocation that the table refuses.

The cause is mine. claude-444's REPORT was added after its code commit, and I did not run the
named test again at the head that held the REPORT. The CI job "harness rules and tests" had not
finished at that head when the PR was queued, and the queue does not wait for it.

## What it does

1. Line 68 of claude-444's REPORT says the same thing without a backticked invocation.
2. Nothing else changes: no code, no rule, no test.

## Gates

- **G1:** `cargo test -p uor-r4-harness the_real_repo_passes_the_ported_rules` passes at the
  head that holds this ORDER and its REPORT (it fails on the base).
- **G2:** the CI job "harness rules and tests" passes at the head before the PR is queued.
