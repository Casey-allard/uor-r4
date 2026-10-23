# REPORT: infra-loop-harness, claude-65 (`loop close --verify` keeps a red gate's log)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). The item status is unchanged.

## What did not work, first

1. **The flake this was built for is still unnamed.** The #3090 verify went red twice on `loop-drive` (exit 101) on 2026-10-23, and green on the third run. Every other run at the same commit was green: run directly, in the nested verify path, through `loop gates --full` (all 11 GREEN), and 10 isolated runs. Its log went with the verify worktree. This unit only makes the next red keep its log; it does not find or fix the flaky test.
2. **My first attempt to capture the log failed.** A shell loop copied the log out of the verify worktree while the verify ran. zsh aborted it on an unmatched glob, and it did catch the log on the next try, but that run was green, so there was nothing to read. That workaround is what this unit replaces.
3. **One documentation line is out of scope.** `docs/harness/README.md:293` describes `done-when` and is still true, since the worktree is still removed whatever the result. It does not mention the kept logs. Adding that is a one-line follow-up outside this ORDER's scope.

## Cancel probe

At the base, a red `loop close --verify` leaves no gate log under the main checkout. The new test shows it: with the fix set aside, it panics at `tests/loop_close.rs:806` because `local/claude/verify-logs/7` does not exist. The task stands.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopclose.rs` | New `keep_red_logs`, called by `done_when` only when `loop gates --full` ran and exited non-zero, before `git worktree remove --force`. It takes the newest run folder under the verify worktree's `local/<lab>/gates/`, and copies each gate whose `.exit` is not `0` (`<gate>.log` and `<gate>.exit`) to `<main>/local/<lab>/verify-logs/<n>/<run>/`. It then prints `KEEP done-when red gate log(s) <gates> copied to <path>`. A copy failure prints `NOTE done-when red gate logs not kept: …`. The verdict, the removal and every other proof are unchanged. The module and function docs say so. |
| `crates/uor-r4-harness/tests/loop_close.rs` | `a_red_verify_keeps_the_red_gate_log_and_a_green_one_keeps_none`. A red gate (`false`) leaves exactly one run folder holding `g.log` and `g.exit` (`1`), the output names that folder, and the verify worktree is gone. A green gate (`true`) creates no `verify-logs` and prints no `KEEP`. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- `cargo test -q --locked -p uor-r4-harness --test loop_close`: 29 passed, 0 failed (28 before, plus the new test).
- **Negative gate.** `git diff origin/main -- crates/uor-r4-harness/src/loopclose.rs` was set aside with `git apply -R`. The new test then FAILED (panic at `loop_close.rs:806`, no `verify-logs/7`). After restoring the fix it passed, 1 passed.

## Checks run at the head

- `cargo clippy -q --locked -p uor-r4-harness --all-targets -- -D warnings`: clean.
- `cargo fmt -p uor-r4-harness --check`: clean.
- `loop report --check`, `loop pr-check --local`, `git diff --check`.

## Skills used

- **`scope-guard`:** two harness files plus this folder. The README line is named above, not edited.
- **`solution-integrity`:** the test fails with the fix set aside, and that was checked.

## Still open

- The `loop-drive` flake itself. The next red verify will name its test under `local/<lab>/verify-logs/<n>/`.
- `docs/harness/README.md:293` could mention the kept logs.

## Plain summary

When the post-merge check fails, it now saves the failing gate's log before it deletes its scratch copy, and says where. Until now a failure left nothing to read.
