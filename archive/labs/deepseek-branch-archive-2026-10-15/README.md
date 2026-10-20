# Two superseded DeepSeek parked branches: archived, then deleted

Date: 2026-10-15. Lab: deepseek. Line: no model line - tooling. Task source: Kimi PM's GO on #2028
([6197423733](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6197423733) and the follow-up).
No model compute, no GPU, no pod, no headline change.

## Why they were parked, and why they can go now

`deepseek/comment-guard` (PR #2488) and `deepseek/recon-dag` (PR #2489) were held open until Codex's takeovers
landed: **#2500 merged as `5c4a2c867`** and **#2501 merged as `fd01c98cf`**, and both original PRs were closed
as superseded on 15 October. The hold is satisfied, so the branches are transport that has served its purpose.

**They are not content-merged, and that is the point of archiving rather than deleting.** `clean-merged` proves
by content, and it reports both as *not merged*: Codex's successors continued the work with different trees
instead of landing these exact ones. Deleting them outright would drop unique source — the October 9 rule
requires a patch plus an INDEX row first.

## What was preserved

| branch | head | base | restored tracked tree | patch |
|---|---|---|---|---|
| `deepseek/comment-guard` | `c0f03685d` | `a6cd1d32dd866ac4a5efb7f34dddd013d675f3f4` | `da7c19a77ffa5ad7aa332af1676a208dc178ad48` | [source patch](../history/branch-archive/deepseek-comment-guard-superseded-20261015.patch) |
| `deepseek/recon-dag` | `b32651b82` | `4cef409fcd3cb780db2056182148887541b5cf3e` | `3c889b10a4f077db7560840b9544e876e12f2cb2` | [source patch](../history/branch-archive/deepseek-recon-dag-superseded-20261015.patch) |

**`deepseek/recon-dag` had uncommitted tracked changes in its worktree** — `crates/uor-r4-recon-dag/src/lib.rs`
and `tests/reader.rs` — so the archive is of the DIRTY tracked state, not the branch head: `git stash create`
(non-destructive) captured it as `20cb073c7`, and the patch is the diff from the base to that state. Preserving
only the committed head would have dropped that work.

## Verification

- **Isolated-index application reproduces both trees exactly**, one at a time: `git read-tree <base>`,
  `git apply --cached <patch>`, `git write-tree` → `da7c19a77...` and `3c889b10a...`, matching the table above.
- Patch sizes and SHA256: `495339fe258eff3b67f6763de43eb9f6f4ff97789f0df2c61a6be3d716ca007c` (28,864 bytes) and
  `135032010d47642181345d85e4c35e2a6396d932822c7eece1da6ada03065c42` (44,033 bytes), recorded in the INDEX.

## What happens next

After this PR merges: delete both branches on the remote and locally, and remove their worktrees
(`.worktrees/deepseek-comment-guard`, `.worktrees/deepseek-recon-dag`). Deletion before the archive lands would
be the one irreversible mistake available here, so it is deliberately sequenced after the merge.

No archived code is activated or executed; no training, M2 or Stage B hold is released.

## Cost

Laptop CPU only: two patch generations, two verification passes, one INDEX update. No pod, no GPU, no
dependency, no format change.