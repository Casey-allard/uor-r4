# Working in one checkout

**Owner direction, 12 October 2026:** labs work directly in `~/uor-r4` rather than each
keeping a full worktree, so data is not duplicated and cleanup is trivial.

## Why

Measured on 12 October 2026, before cleanup: a worktree cost **1.3 GB even with no build
output**, because git materialises a second full checkout of the repository, and
`research/` alone is ~1.0 GB of mostly binary material. Five worktrees duplicated ~6.5 GB
of identical files. `scripts/storage/worktree-audit` prints the current numbers.

A worktree is not a copy of the git history — the object store in the main `.git` is
shared. What it duplicates is the *working tree*: every tracked file, checked out again.

## The rules

1. **One branch, one owner, at a time.** Before you start, claim the branch you will work
   on. `uor-harness team claim --branch <name> --lab <lab>` records it and
   refuses if another lab holds a live claim. Release it when you merge.
2. **Never switch a branch you do not hold.** `git checkout` changes what every process in
   the directory sees, including another lab's build.
3. **Keep the tree clean between sessions.** `git status` should be empty when you stop.
   Uncommitted work in a shared directory is not "your" work — it is everyone's problem
   the moment someone else commits.
4. **Commit or stash before you hand over.** If you are mid-task, commit to your own
   branch (a WIP commit is fine) or `git stash push -m "<lab>: <what>"` and write the
   stash name into your handover note.
5. **Serialise builds.** One `cargo` invocation at a time. Two concurrent builds in one
   checkout fight over `target/` and produce confusing failures. Source edits, documents,
   analysis and reviews do not need the build lock.
6. **`git gc --auto` is fine; `git clean -xdf` is not.** Ignored local data under `/local/`
   belongs to a lab. Never clean another lab's ignored data.

## Handing over

Write a four-line note where the next lab will find it — in the issue, and in
`docs/labs/<topic>-<date>/README.md` if the work is finished:

```
lab:      <who>
branch:   <branch, or 'none' if you were on main>
state:    clean | wip-committed | stashed:<name>
next:     <the one action the next lab should take>
```

## When a worktree is still correct

- Two labs genuinely need two branches at the same time and neither can wait.
- A long, risky experiment must not be disturbed by branch switches.
- A preserved state must survive a cleanup (the `preserved/` pattern).

In those cases use one, and treat it as temporary: remove it after the merge with
`git worktree remove <path>`, or run `scripts/storage/uor-hygiene --apply`, which removes
a worktree only when it is clean, has no ignored data, and its HEAD is already in
`origin/main` or is the head of a merged PR.

## Guardrails

- `uor-harness check` includes `status.no_orphan_branches`, which reports a
  branch whose PR merged but which still exists.
- `scripts/storage/worktree-audit --plan` prints, for every worktree, whether it is
  removable, whether it holds unmerged commits, and the exact `format-patch` command to
  preserve them first. It never removes anything itself.
- The pre-commit hook (opt-in, `uor-harness git install-hooks`) blocks a commit
  on a branch held by another lab. It installs into the shared `.git/hooks`, so install it
  deliberately, not by default.

## What this does not change

Isolation was never the goal in itself. The goal is that two labs cannot corrupt each
other's work. Disjoint write scopes, one branch owner at a time, and a clean tree between
sessions give that more cheaply than five copies of the same gigabyte.
