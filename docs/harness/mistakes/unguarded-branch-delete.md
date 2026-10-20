---
id: unguarded-branch-delete
title: A branch was deleted on the remote without proving the pull request had merged
date: 2026-10-13
severity: block
class: delivery
trigger: [delete branch, remove branch, git push origin --delete, gh pr merge, auto merge, merge queue, clean up, after merge, orphan branch]
paths: [.github/**, crates/uor-r4-harness/src/githelp.rs, docs/harness/claims.json]
milestones: [m1, m2, m3, m4, m5, m6, m7, m8]
mechanism: ""
guard: none: no machine check can intercept a direct `git push --delete`; the guard is the CLI refusing (`uor-harness git delete-merged` proves the merge first), which only helps if it is the command you run
evidence: [archive/labs/harness-rust-port-2026-10-13/README.md]
status: active
---

## What happened

Merging a pull request, an agent ran `gh pr merge --auto`, polled the PR state in a loop, and then
ran the branch cleanup **unconditionally**:

```sh
git push origin --delete <branch>     # not guarded by the merge result
git branch -D <branch>
```

Partway through the poll the PR **auto-closed as `DIRTY`** — a concurrent merge had made it conflict
— so it never merged. The loop exited without noticing, and the delete ran anyway. The remote branch
was the only copy of that work outside one local commit.

The work was recovered: the commit was still reachable locally, the branch was recreated from it,
rebased onto `main`, and merged as #2302. Nothing was lost. **But nothing in the sequence prevented
the loss either** — the recovery depended on the local clone still holding the commit.

## Symptom

A branch disappears from the remote while its pull request is **closed, not merged** — so the work
it carried exists nowhere but a local clone. The tell is a merge script that reports success: it
printed nothing about a problem and the cleanup ran to completion.

Two smaller symptoms in the same family, both seen while building the guard:

- A guard prints "would delete" while its own message says the branch is refused.
- A branch under another lab's live claim is recommended for deletion.

## Lesson

**A poll loop that treats an unexpected state as "done" reports success it did not observe, and a
delete with no precondition acts on an assumption rather than on evidence.** `gh pr merge --auto` on
a merge queue can transition a PR to `CLOSED` when a concurrent merge makes it conflict; a loop
checking only for `MERGED` and `DIRTY` exits as if it had finished.

The rule — verify the merge commit on a fresh `origin/main`, *then* delete — was followed in spirit
(the merge was queued) and not in fact (the verification never ran before the delete). An unmerged
remote branch is usually the last copy of that work, and its loss is not reversible from the remote.

**A merged PR is also not proof a branch is finished.** A lab can merge and keep going, or hold the
branch while a follow-up lands. Ownership outlives the merge, so a live claim must block deletion
regardless of the PR state.

## How to apply

Delete with the guarded command, never with a raw `git push --delete`:

```sh
uor-harness git delete-merged --branch <name>          # report and refuse
uor-harness git delete-merged --branch <name> --apply  # delete only if proven
```

It refuses unless a merged PR's merge commit is an ancestor of `origin/main`, and it refuses equally
when GitHub is unreachable, when `origin/<branch>` holds commits the merge did not carry, and when
**any lab holds a live claim** — checked first, because a claim needs no network to read.

When scripting a merge, **verify the state you actually need and abort on anything else**:

```sh
gh pr merge "$PR" --auto
for _ in $(seq 1 30); do
  state=$(gh pr view "$PR" --json state -q .state)
  [ "$state" = MERGED ] && break
  [ "$state" = OPEN ] || { echo "PR is $state, not merged - stopping"; exit 1; }
  sleep 15
done
[ "$state" = MERGED ] || exit 1          # never proceed on a loop that merely ended
```

And when a guard refuses, set **the flag the caller acts on** as well as the message a human reads —
a refusal that only changes its wording is not a refusal.
