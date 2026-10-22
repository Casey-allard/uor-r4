# REPORT: infra-loop-harness, claude-32 (bench checks: PASS/FAIL words, one row per gate at main's head)

Lab: claude. Base: `683e182ba`. Order: [ORDER.md](ORDER.md), committed first (`b206af880`). The cancel probe found that `gate_line` passed the ledger's verdict through, only upper-cased, so `GREEN`, `RED` and `CAUGHT` reached the bench unchanged.

## What did not work, first

- **The bench read "Checks 0 of 23 passed" with main green.** Three causes:
  - The page counts only `PASS`/`FAIL`, and the feed wrote `GREEN`/`RED`.
  - Every row was a lab's run on a PR branch, so it read "older commit".
  - The page's git chips read the owner checkout, two days old with 23 uncommitted files, because the bench folder lived inside it.
- **The git chips and the gates-at-head runs are fixed outside this PR, operationally.** The bench now runs from a detached worktree kept at `origin/main` (`.worktrees/bench-main`). A refresher there runs `loop gates --full` whenever main moves. The bench's `INBOX.md` is a symlink to the main checkout's, so the `AGENTS.md` inbox rule still holds.
- **The old bench tests never caught this.** Their fixtures used `pass`/`fail`, never the loop's own `GREEN`/`RED`.

## What changed

| Commit | What |
|---|---|
| `b206af880` | This ORDER, alone. |
| `578183475` | `loopbench.rs`: `bench_result` maps `GREEN`/`PASS` to `PASS` and `RED`/`FAIL`/`REFUSED` to `FAIL`, and drops `CAUGHT`. `gates()` keeps one row per gate: the newest run at `origin/main`'s head, else the newest run. New test `bench_checks_say_pass_or_fail_one_row_per_gate_preferring_main_head`. |
| this REPORT's commit | This REPORT. |

## Measurements

None. This is a feed fix, with no model run.

## Checks run at the head

- `cargo test -p uor-r4-harness --test loop_bench`: 13 passed.
- **Negative gate:** with the `loopbench.rs` change set aside, the new test fails at `loop_bench.rs:417` (one row per gate). Restored, 13 pass.
- `cargo clippy -p uor-r4-harness --all-targets --all-features -- -D warnings`: exit 0. `cargo fmt --check`: clean.
- `loop gates --full`: 8 GREEN. `--break all`: 8 CAUGHT.
- `loop report --check`, `loop pr-check --local`, claim-wording and the copyright check: see the PR body.

## Skills used

scope-guard: `loopbench.rs`, `tests/loop_bench.rs` and this task folder.

## Still open

- **The page hides the feed's per-lab "How we know" column.** So a stale presence line (a lab's last heartbeat intent) reads as current. A follow-up could put the staleness in the Status text itself.
- **The branch chip reads `HEAD`.** The bench worktree is detached at `origin/main`, because only one checkout can hold `main`.

## Plain summary

The bench showed main's green checks as zero passed, because it and the loop use different words, and because it listed every lab's old branch runs. It now shows one result per check, in the bench's own words, preferring the run on main's latest commit.
