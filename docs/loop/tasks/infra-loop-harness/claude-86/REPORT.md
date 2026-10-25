# REPORT: infra-loop-harness, claude-86

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #3175. References #2904.

## What did not work, first

- **The repository setting cannot be turned on.** `gh api repos/UOR-Foundation/uor-r4` reports `delete_branch_on_merge: false` and `admin: false` for the labs' token, and the owner is not an admin either. So the fix is a workflow, which needs only the workflow token's `contents: write`. The repository grants workflow tokens write scopes: `required-transport` already uses `checks: write`.
- **The "require branches to be up to date" ruleset change I handed to the owner on 24 Oct is not needed.** #3174 merged through the queue while one commit behind main. Its head did not contain #3172's `e05f9a5ca`, and it merged in 2 minutes. The queue tests the merged result itself. The change is dropped.

- **The first draft could have deleted unmerged work. The fresh review (CHANGES) found how.**
  - **Unencoded names (high).** Names went into the API path unencoded, so a branch named `main#x` would have sent its delete to `main`. `main` has no deletion rule, and no one here can add one.
  - **No tip check at delete time (medium).** Nothing re-checked the tip when the delete ran.
  - **Failed lists ignored (low).** A failed `gh pr list` could leave the open-PR list empty.
  - **Open PR bases (low).** A merged branch that is an open PR's base could be deleted.

  All four are fixed:
  - Only plain names (`[A-Za-z0-9._/-]`) are eligible.
  - The delete is an atomic compare-and-delete, `git push --force-with-lease=refs/heads/<b>:<sha> :refs/heads/<b>`.
  - `set -euo pipefail` aborts on any failed call.
  - Open PRs' base branches are protected.

## What changed

- **New `.github/workflows/branch-cleanup.yml`:**
  - `merged-head` runs on `pull_request: closed` when the PR merged from this repository. It deletes the head branch only if the tip still equals the PR's final head, never `main`, `gh-pages` or `gh-pages-preview`. A branch that moved after the merge is kept, with a warning.
  - `sweep` runs daily and on `workflow_dispatch`. It deletes every same-repo branch that sits exactly at a merged PR's head and is not an open PR's head. It lists every other non-protected branch in the job summary and never deletes those.
  - Name lookups use fixed-string matching. The REST delete is replaced by the compare-and-delete push, authenticated with the workflow token through `http.extraheader`, never in the URL.

## Measurements

- None yet. The PR's own merge exercises `merged-head`, and a manual run exercises `sweep`. Both are recorded on #3175.

## Checks run at the head

- **Round-2 review: APPROVE, with two small fixes taken.**
  - A failed delete that is not a lease mismatch (`(stale info)`) now fails the job with git's message, so a missing permission cannot hide as "kept".
  - The auth string is masked.
- **Not taken.** The sweep reads the newest 1,000 merged PRs. Older merged branches are only listed, which fails safe.
- **Local test of the compare-and-delete** on a scratch bare repository: with a stale expected sha the push is refused and the branch stays (`37a776e`); with the right sha it is deleted.
- `ruby -ryaml` parses the workflow (jobs `merged-head` and `sweep`; triggers `pull_request`, `schedule` and `workflow_dispatch`).
- `loop report --check` and `loop pr-check --local`: run before queue.

## Skills used

- None.

## Still open

- The live checks above, after merge.

## Plain summary

When a pull request merges, GitHub now deletes its branch automatically, unless someone pushed to it after the merge. A daily sweep catches anything missed and lists unmerged branches for review instead of deleting them.
