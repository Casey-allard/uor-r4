# ORDER: infra-loop-harness, claude-86 (delete merged PR branches automatically)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 5. Base: `5dc687f79`.

## Goal

#3175. The repository keeps merged PR branches (`delete_branch_on_merge: false`), and nobody here is an admin who could change that. Merged branches pile up until someone sweeps them by hand. This unit adds `.github/workflows/branch-cleanup.yml`, run with the workflow token's `contents: write`:

1. **On `pull_request: closed` with `merged == true`, same-repo heads only:** delete the head branch when its tip still equals the PR's final head. Never touch `main`, `gh-pages` or `gh-pages-preview`. A branch that moved after the merge is kept and reported.
2. **On `schedule` (daily) and `workflow_dispatch`:** delete every remaining same-repo head branch of a merged PR, under the same tip guard. Then list, without deleting, every other branch that is not one of the three and is not the head of an open PR, so unmerged work is never lost.

## Cancel probe (run first)

At `5dc687f79`, no workflow deletes branches. `gh api repos/UOR-Foundation/uor-r4` reports `delete_branch_on_merge: false` and `admin: false`. The unit stands. Class: instrument (repository hygiene).

## Files in scope

- `.github/workflows/branch-cleanup.yml` (new)
- `docs/loop/tasks/infra-loop-harness/claude-86/**`

## Done when

- The workflow parses.
- A fresh-context review at the exact head counts.
- This PR's own branch is deleted by the workflow when the PR merges.
- A manual `workflow_dispatch` run on main reports the sweep, and `#3175` closes.
- The PR says `Closes #3175`.

## Stop and ask if

The workflow token cannot delete refs. If so, report it on #3175 with the error.

## Owner approval

Owner path changed: `.github/**`.

Casey, in Claude's chat, 25 Oct 2026, by 2026-10-25T02:10:18Z: "I cannot change the rule I am not admin, this is why I asked you to do it and keep track of it because otherwise it will fall through the cracks and we will have hundreds of empty branches to fix again later".
