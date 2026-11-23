# REPORT: infra-loop-harness, claude-444 (the documented-commands table holds `claim-wording`)

Lab: claude. ORDER: [ORDER.md](ORDER.md). Base `a9d8a35aa`.

## What did not work, first

- **The task as given was wrong about the cause.** I was sent to correct line 55 of
  antigravity's report on the ground that `uor-harness` has no `claim-wording` command. It has
  one. The line is true, so I did not change it; the table the rule reads was stale.
- **The first queue attempt was refused.** PR #4197 was opened by hand with the code change
  alone, and `uor-harness loop queue --pr 4197` refused it: no ORDER, so no REPORT and no review
  record could be checked. This ORDER and REPORT were added after the code commit, so the commit
  order is not the loop's order-first order.
- **One unrelated test failed once.** In the full run `an_inner_gate_is_respected`
  (`tests/loop_prcheck.rs`) failed on git's `unable to create temporary file: Invalid argument`
  while the suite ran in parallel. Run alone, that file passed 50 of 50. Not caused by this diff
  and not fixed here.
- **Same-lab review only.**

## What changed

- `crates/uor-r4-harness/src/doccheck.rs`: `claim-wording` added to `ACCEPTED` and to
  `RUST_ONLY_COMMANDS`, with a comment saying why. The first makes the rule accept the command
  `main.rs` already dispatches; the second lets a bare prose `-` after it through, as for the
  other commands that only the Rust entry point has. Flags are still checked against `FLAGS`.
- Same file, one test: `claim_wording_is_a_command_this_binary_runs`.
- Not changed: `main.rs`, the rule's logic, the real-repo test, and both documents of antigravity.

## Measurements

| Run | Result |
|---|---|
| Named test on `origin/main` `a9d8a35aa` | fails: 2 documented commands do not exist (both `uor-harness claim-wording`) |
| Named test with the change | passes |
| `documented_commands_tests` with the change | 2 of 2 pass |
| New test and named test with the two table lines removed (G2) | both fail |
| `uor-harness claim-wording` on this tree | prints the gate's pass line |

## What the numbers say

The rule failed because its table was behind the binary, and one table entry puts it right. The
new test fails without the entry, so it pins it.

## Checks run at the head

- G1: `cargo test -p uor-r4-harness the_real_repo_passes_the_ported_rules`: 1 passed (code head
  `2e4d9ed57`; the commits after it add only this folder).
- G2: with the two `"claim-wording",` lines removed: `claim_wording_is_a_command_this_binary_runs`
  and `the_real_repo_passes_the_ported_rules` both FAILED. Restored from a patch against
  `origin/main`; the restored diff is 18 insertions.
- G3: `cargo test -p uor-r4-harness`: all pass but the one unrelated failure above, which passes
  alone. `rustfmt --edition 2021 --check crates/uor-r4-harness/src/doccheck.rs`: clean.
  `uor-harness check`: no error, warnings only, none from this diff.
- CI at `2e4d9ed57`: "harness rules and tests" passed, with the eight other checks that ran.
- Fresh review (delegate:reviewer, given only the spec and the diff) at `2e4d9ed57`: APPROVE.
  One LOW, not changed: the `--bad-flag` assertion of the new test does not check the reason, so
  it would also hold for an unknown command; the first assertion is what pins the entry.

## Skills used

None loaded. The negative check followed the engineering conventions' set-aside rule.

## Still open

- **`hygiene` and `check-rules`** are dispatched by `main.rs` and absent from `ACCEPTED`. No
  document names them today, so the rule does not fail on them.
- **`claim-wording` is absent from `PORTED_COMMANDS`** in `ruleinventory.rs`, so the binary takes
  an unknown flag after `claim-wording` and ignores it. No test compares the lists.
- **Stale comments in `doccheck.rs`:** the counts "(14)" and "(11)", and the name
  `the_table_agrees_with_pythons_live_parser`, a test no longer in the tree.
- **The harness workflow does not run on pushes to `main`**, which is how a red rule reached it.

## Plain summary

A check that every `uor-harness` command named in the documents really exists was failing on all
new pull requests. The command it complained about does exist; the check's own list of commands
was missing it. The list now has it, and a new test fails if it is taken out again.
