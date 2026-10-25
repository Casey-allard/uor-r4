# REPORT: infra-loop-harness, claude-111

Lab: claude. Base: `12facdf94`. Order: [ORDER.md](ORDER.md), committed first (`5eab00333`). The cancel probe held: `loopbench.rs` ran `git fetch` but no `checkout`, `switch` or `reset`; no open PR touched the bench code; and `bench-main` was a clean detached HEAD behind `origin/main`.

References #2904

## What did not work, first

- **A line after "Last checked" would not have been seen.** The bench page renders only the parsed parts of `STATE.md`. Of the "What works now" table, it shows the first five rows, and the table already has six (five labs and the roadmap). So a "left alone" line must be a row, and the first one. Its status starts `blocked:`, which the page colors red.
- **The first negative gate was refused.** `loop guard` refused writing a set-aside patch to the session scratchpad, which is outside the ORDER's scope. The set-aside became `git checkout 12facdf94 -- crates/uor-r4-harness/src/loopbench.rs`, restored with `git checkout HEAD -- …`, after the fix was committed.
- **Setting the whole fix aside pins the move, not the guards.** On the base, the feed never moves HEAD. So the four "left alone" tests failed only on the missing line, never on HEAD. A second set-aside kept the move and made each guard inert (`if false && …`). Each of the four then failed on HEAD itself; the named-branch test failed because `git symbolic-ref HEAD` found HEAD detached off `main`.
- **Round 1 of the exact-head review (at `d086e5c01`) approved, with four advisory findings.** Three were fixed in `d2a4c098d`:
  - **A failing post-checkout hook made a real move read as a failure.** git runs the hook after HEAD moves, and the hook's exit code becomes checkout's. The real repo's git-lfs hook exits 2 when `git-lfs` is not on PATH. The step now reads HEAD again and reports the move, with the hook's error on stderr.
  - **A failed step named the commit as `?`.** It now names HEAD when HEAD can be read.
  - **The move test did not prove the cycle's own fetch,** because the single run before it had already fetched. The test now moves `origin` again after the single run.
  - The fourth (ignored files) is under Still open.
- **The live feed does not have the step yet.** The feed process (pid 49525, started 24 Oct from `~/.cargo/bin/uor-harness`) runs the installed binary. It gets the step only once that binary is rebuilt from `main` and the feed is restarted. This unit does neither: it may not overwrite the installed binary, and the process belongs to another session.

## What changed

Each `--every` cycle of `loop board --bench` now starts by refreshing the checkout the feed runs in, right after the fetch and before the pages are written (`sync_checkout` in `loopbench.rs`):

- **Moves it** to the fetched `origin/main` with `git checkout -q --detach <sha>` when HEAD is detached, `git status --porcelain` is empty (untracked files count), no merge, rebase, cherry-pick, revert or bisect is in progress, and HEAD is an ancestor of `origin/main`. The feed prints `checkout moved from <sha12> to origin/main`.
- **Leaves it alone** otherwise. It prints `checkout left at <sha12>, not moved to origin/main: <reason>` and puts a `bench checkout` row first in `STATE.md`'s "What works now" table, with the same reason. The reasons are: `HEAD is branch <name>`, `a <op> is in progress`, `<n> uncommitted or untracked path(s)` and `<n> commit(s) not on origin/main`.
- **Never switches a named branch,** whatever the tree's state.
- **Keeps writing the pages** when the step itself fails; the failure becomes the row's reason. A move that a failing post-checkout hook follows still counts as a move.
- **A single run** (no `--every`) does not touch the checkout, as before.

| Commit | What |
|---|---|
| `5eab00333` | This ORDER, alone. |
| `2bfd3ad25` | `loopbench.rs`: `Checkout`, `IN_PROGRESS`, `sync_checkout`; `refresh` takes `sync` and returns the step's result; the STATE row and the printed line. `tests/loop_bench.rs`: helpers `git_out`, `advance_origin` (pushes from a second clone, so the local `origin/main` stays stale until a fetch), `cycle` (runs one real `--every 30` cycle, waits for its "wrote" line, then stops the child) and `first_state_row`; five new tests. |
| `d086e5c01` | This REPORT, first version (the round-1 review's head). |
| `d2a4c098d` | Round-1 advisories: HEAD is read again after a failed checkout; a failed step names HEAD; `advance_origin` can run again; the move test proves the cycle's fetch; new test `every_cycle_reports_a_move_that_a_failing_hook_follows`. |
| this REPORT's commit | This REPORT, updated. |

Size by class: code +138 −29 (`loopbench.rs`), tests +247 −1, docs this folder. The ORDER gave no size estimate, so there is no ratio.

## Measurements

None. This is a feed fix, with no model run.

## Checks run at the head

- `cargo test -p uor-r4-harness --test loop_bench`: 19 passed (13 old, 6 new):
  - `every_cycle_moves_a_clean_detached_checkout_to_the_fetched_origin_main`: a single run fetches but leaves HEAD; `origin` moves again, so the local `origin/main` is stale; one cycle moves HEAD to the newest commit; a second cycle prints nothing about the checkout.
  - `every_cycle_reports_a_move_that_a_failing_hook_follows`: a post-checkout hook that exits 2; HEAD moves, and the cycle reports a move with no STATE row.
  - `every_cycle_leaves_a_dirty_tree_and_says_so`: a modified tracked file, then an untracked file; HEAD and the edit stay; the row and line are written.
  - `every_cycle_never_switches_a_named_branch`: on `main`, HEAD and `refs/heads/main` stay.
  - `every_cycle_leaves_a_detached_head_with_commits_main_lacks`: a local commit stays checked out.
  - `every_cycle_leaves_a_checkout_mid_bisect`.
- Every test repository is inside its own temp directory, and every child process has every `GIT_*` variable removed (the file's `clean` helper).
- **Negative gate 1** (at `2bfd3ad25`): with `loopbench.rs` at the base, all five tests of that commit fail (the move test on HEAD not moved). Restored, 18 pass.
- **Negative gate 2** (at `2bfd3ad25`): with the four guards made inert, the four "left alone" tests fail on HEAD itself (the branch test where `git symbolic-ref HEAD` finds HEAD detached). The move test passes. Restored, 18 pass.
- **Negative gate 3** (at `d2a4c098d`): with the HEAD re-read made inert (`if true {`), only the hook test fails (`loop_bench.rs:850`, no "checkout moved" line). Restored.
- **Negative gate 4** (at `d2a4c098d`): with the cycle's `git fetch` removed, the move test fails (`:736`, HEAD not at the newest commit). Restored, 19 pass.
- `cargo clippy -p uor-r4-harness --all-targets --all-features -- -D warnings`: exit 0. `cargo fmt -p uor-r4-harness --check`: clean.
- `loop gates --full` (head binary): 11 GREEN, exit 0, at `2bfd3ad25` and again at `d2a4c098d`. The REPORT commit after it changes only this file.
- `cargo test -p uor-r4-harness --no-fail-fast` at `d2a4c098d`: every test binary passes, 0 failed (the lib: 676 passed, 1 ignored).
- `uor-harness check` at `d2a4c098d`: 211 rules, 0 errors. The 179 warnings are other labs' stale branch claims and heartbeats, none from this change.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

None.

## Still open

- **Rebuild the installed `uor-harness` from `main` and restart the feed.** Until then, the running feed (pid 49525) lacks the step. The session-only refresher in the claude-110 session keeps `bench-main` current meanwhile, so it should be stopped only after the restart. Not tracked as an item; the starting session relays it to Casey.
- **Ignored files are not guarded** (round-1 advisory 4). `git status --porcelain` does not list ignored files, and a plain checkout replaces an ignored file when the target commit starts tracking that path. Today nothing on `main` tracks a path under `local/`, so the bench folder is safe. Not tracked as an item.
- **The step moves only the checkout the feed runs in.** A bench served from any other folder still shows that folder's own checkout.

## Plain summary

The bench is a local web page that shows the project at a glance. Its "Last commit" panel asks git about the folder the page is served from. That folder sits inside a separate copy of the repository (a git worktree), which nothing moved forward, so the panel showed a commit 28 hours old and looked like the loop had stopped. The program that writes the page's files every two minutes now also moves that copy to the newest `main` each time. It does so only when the move is safe: the copy is not on a named branch, holds no unsaved or unshared work, and is in the middle of nothing. When the move is not safe, it leaves the copy alone and puts a red line at the top of the page's status list saying why.
