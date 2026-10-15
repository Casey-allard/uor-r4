# #2287 — Local checkout cleanup: seven stale refs removed, two worktrees deliberately left (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:25:56Z
- **Updated:** 2026-10-13T04:28:01Z
- **Closed:** 2026-10-13T04:27:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2287
- **Merged:** 2026-10-13T04:27:37Z
- **Merge commit:** 5c0cf294578db4b26958bc212135479a1d955ff2
- **Base / Head:** main / UOR-Foundation:chore/checkout-cleanup
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Maintenance only; no model code touched.

## What Claude handed over, and what I found

Claude passed the local-checkout cleanup to the maintenance role: three other labs' worktrees, ~nine
stale refs, disk at 43 GB free. I checked every item instead of acting on the summary, and **one of
the three "already merged" worktrees is not safe to remove**.

### Done — seven stale refs

Fetch leftovers under `refs/remotes/pr/` and `refs/remotes/review/`. None exist on the remote, none
are branches. Removed all seven.

`git remote prune origin` does **not** remove these — prune only drops refs under
`refs/remotes/origin/`, and these are direct children of `refs/remotes/`. That is why they survived
years of fetches.

### Done — my own branch, which I had left behind

Deleting those made me check my own work, and **I had failed the rule I just shipped**:
`harness/install-and-branch-rule` was still on the remote after PR #2286 merged — in the very PR that
added the remove-your-branch rule. Verified merged, deleted on both sides.

The repository now holds **`main` and `gh-pages` only**, with no stray refs.

### Not done — and this is the important one

`codex-geometric-path-repair` has **no commits ahead of `main` and no PR**, so it looks like an empty
shell. It is not:

```
12 files changed, 221 insertions(+), 170 deletions(-)
new file: crates/uor-r4-training/src/whole_path_law.rs (1629 bytes)
```

**Uncommitted work that exists nowhere else.** `git rev-list --count`, the merged-PR check and the
GitHub UI all report "nothing here", because it was never committed. Removing the worktree deletes
it.

`kimi-lab` (8.5 GB) is active WIP — 550+ lines across 28 files, no PR. Left alone.

`agent-deepseek-router` (1.3 GB) is the one Claude called already merged, and it is *probably* right:
its two commits landed through #2245 and #2247, and the `read_router` test literals are in `main`
today. But the branch is **40 commits behind** `main`, so files like `temporal_cli.rs` differ because
`main` moved on rather than because of anything the branch holds. I can prove those two commits are
superseded; I cannot cheaply prove the branch holds *nothing* else — and 1.3 GB of another lab's
history is not deleted on an inference. It is my own lab's, so I will remove it on confirmation, proof
first.

### `uor-hygiene --apply` was deliberately not run

One of its cleanup candidates is exactly the clean-looking worktree that holds Codex's uncommitted
work. **Running it blind could destroy WIP.** 39 GB free is above the 30 GB floor but inside the
30–70 GB band the hygiene rule wants maintained, so this needs a decision rather than a script.

## Result

Decision: **KEEP** (maintenance).

```
python3 -m unittest discover -s tests/harness -t .   -> 281 passed
python3 scripts/harness/cli.py check                 -> 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Maintenance only; no capability or headline changed
- [x] Record: docs/labs/checkout-cleanup-2026-10-13/README.md, plus a current-state entry
- [x] Branch deleted after merge

### Stated rather than hidden

- **No worktree was removed.** Three labs' worktrees all remain, for three different reasons. This PR
  removes refs and records evidence; it does not reclaim the 10 GB, because the safest reclaim depends
  on other labs answering first.
- I cannot prove a 40-commit-behind branch holds nothing unique without checking 225 changed files by
  hand. "Probably redundant" is not "proven redundant", and the difference is someone's evidence.
- Disk is 39 GB and falling. If it reaches 30 GB before Codex acts, the hygiene script becomes urgent
  and the safe order changes — that is worth knowing now rather than then.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

