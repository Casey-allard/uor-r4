# ORDER: infra-loop-harness, claude-111 (the bench feed keeps its own checkout at origin/main)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7c (the bench feed, as in claude-25, -27 and -32). Base: `12facdf94`.

## Goal

On 25 Oct Casey saw "last commit 28 hours ago" on the bench and read it as the loop being down.

- The bench folder is `.worktrees/bench-main/local/bench`, gitignored, inside the `bench-main` worktree.
- The page's Git panel runs `git` in that folder, so it shows that worktree's HEAD.
- The worktree was a detached HEAD left at `642674f97`, 28 hours old.
- The feed (`loop board --bench DIR --every 120`) was in fact writing every 2 minutes. It runs `git fetch` and reads `origin/main` with `git show`, but it never moves the checkout it runs in.
- The Claude lab moved the worktree by hand and started a session-only refresher loop, which dies with its session.

The planted fault: **a clean, detached feed checkout stays at its old commit while `origin/main` moves.**

**Why the infra-loop-harness item.** Its scope holds `loopbench.rs` and `tests/loop_bench*.rs`. Codex's idle #3169 holds an order on it, and #3169's files do not overlap this unit's. So this ORDER is hand-written under the run file's umbrella exception.

## Cancel probe (run first)

Class: instrument. Run at `12facdf94`; all must hold, or the unit is cancelled:
- `loopbench.rs` never moves HEAD: it runs no `checkout`, `switch` or `reset`. **Holds.**
- No open PR touches `loopbench.rs` or `tests/loop_bench*.rs`, so no other lab (Kimi, Codex or another) owns a change there. **Holds:** the four open PRs (#3169, #3230, #3231, #3235) do not.
- The case exists: `bench-main` is a detached HEAD, clean, and an ancestor of `origin/main`. **Holds** (it sits at `12facdf94` now only because it was moved by hand).

Result: all hold, so the unit goes ahead.

## Calls

At the top of every `--every` cycle, after the fetch and before the pages are written, the feed checks the checkout it runs in (the `--repo` root):
- **Moves it** to the fetched `origin/main` with `git checkout --detach <sha>` only when all hold:
  - HEAD is detached;
  - `git status --porcelain` prints nothing (untracked files count as dirty);
  - no merge, rebase, cherry-pick, revert or bisect is in progress;
  - HEAD is an ancestor of `origin/main`, so the move only goes forward and no commit is left behind.
- **Leaves it alone** otherwise. It then writes one line into `STATE.md`, under the "Last checked" line, naming the reason and the commit the Git panel shows, and prints the same line.
- **Never switches a named branch:** when HEAD is a branch, the step refuses, whatever the tree's state.
- A single run (no `--every`) does not touch the checkout, as today.

## Files in scope

- `crates/uor-r4-harness/src/loopbench.rs`
- `crates/uor-r4-harness/tests/loop_bench*.rs`
- This task folder

## Done when

- A host test, in a temporary git repository with every `GIT_*` variable removed, runs one `--every` cycle in each case:
  - a clean detached checkout moves to the fetched `origin/main`;
  - a dirty detached tree is left alone and the line is written;
  - a named branch is left alone and the line is written;
  - a detached HEAD with commits `origin/main` lacks is left alone.
- With the fix set aside, the move test fails.
- The existing bench tests, clippy `-D warnings`, `loop gates --full`, `loop report --check` and `loop pr-check --local` all pass.
- A fresh exact-head review approves, and the PR is queued with `loop queue` and verified on fresh `main`.
- The REPORT says `References #2904`.

## Stop and ask if

Stop and decide, never stop and wait.

- The step would need to touch any checkout other than the one the feed runs in.
- The bench needs the checkout on a named branch.
