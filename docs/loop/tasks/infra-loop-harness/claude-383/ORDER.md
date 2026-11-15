# ORDER: infra-loop-harness, claude-383 (archive four branches that have no PR, then delete them)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8 (record migration: work that lives only on four branches is moved into the repository before the branches go). Base: `d5eb4a556` (`d5eb4a55656726b04ea79b06968f91905933c925`, main after #3976).

## Goal

Keep the work on four origin branches that have no PR on `main`, byte-identical, then delete the branches. The owner, Casey, 14 Nov 2026, in claude's chat, after a survey of open PRs and branches: "deepseek is currently still driving its session, so leave that alone, but the rest of the ones we can clean up".

| Branch | Base (merge base with main) | Tip | Why it is not merged |
|---|---|---|---|
| `claude/ladder-pin` | `fbf15e56f3ab1961b30610893392e7a97fd0ab18` | `721a4784f265efea6b490d5c3100779e4c4b1607` | claude-339's pre-merge history; claude-339 is on main |
| `claude/evidence/writer-admission-074d54b2` | `54be88b727ee5642fc4b9bfdc069874281d36a8a` | `8cb2ff9813515463665ab8c2c6269a0f976222c0` | an evidence branch for claude-150, which is on main |
| `claude/route-in-loop-next` | `675f5eb43b99e5ee634cd45b5fd517f69a638c52` | `d4bcfe2c7dad146441602ac40a41dcecd9f2d6e1` | a `loop next` route feature never opened as a PR; its number, claude-371, was later used on main by another unit, and the owner's 14 Nov rule gives a plan change a plan row, not a harness feature |
| `kimi/studio-final-acceptance` | `fa5004861d1bcad84f3e09da250cb290ddf70e3e` | `c17a8468220c9bf20998f5b8cef44828eec594bf` | kimi-71 and kimi-73's pre-merge history; both are on main |

## How this unit was claimed

By hand, as claude-377 was: this session cut `.worktrees/claude-383` and the branch `claude/branch-archive-1115` off `origin/main` at the base. No open order names this unit's files: the open PRs at the claim were #3980, #3979, #3978, #3977, #3941, #3937 and #3889 (#3977 has merged since). Deepseek's branches (`deepseek/*`, `try-reply-*-v3`) and kimi's open #3941 are left alone, as the owner asked.

## Cancel probe (run first)

- **Does each branch still exist at the tip above?** `git ls-remote origin refs/heads/<b>`.
- **Does a PR keep its commits?** `gh pr list --state all --head <b>` is empty for all four, so the commits live only on the branch.
- **Classed:** instrument (repository hygiene). Nothing cancels the unit unless a branch has moved.

## Where the patches go

`docs/loop/tasks/infra-loop-harness/claude-383/branch-archive/`. The usual home, `archive/history/branch-archive/`, is outside every open item's scope; this unit's own folder is always inside it. Moving the four patches and adding their INDEX rows is left to the next archive unit.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/claude-383/**`

## Done when

1. **One patch per branch,** `git diff --binary --full-index <base> <tip>`. Applied with `git apply --cached` to a scratch index read from the base, each gives exactly the tip's tree.
2. **The REPORT records** each branch, base, tip, tip tree, patch SHA-256 and the restore command.
3. **A fresh exact-head review passes,** re-running the restore check.
4. **`loop pr-check`** passes. The PR is queued with `loop queue` and verified on fresh main.
5. **After the merge,** each branch is deleted with `--force-with-lease=refs/heads/<b>:<tip>`, and one #2028 note lists them.

## Stop and decide (never wait) if

- a branch's tip moved: archive the new tip, or leave that branch;
- a patch holds a secret: drop that branch from the unit and report it.

## Rules check

No model, GPU or owner path. Nothing is deleted before its patch is on `main`.
