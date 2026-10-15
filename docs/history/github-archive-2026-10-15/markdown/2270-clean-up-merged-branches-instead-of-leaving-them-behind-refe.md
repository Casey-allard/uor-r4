# #2270 — Clean up merged branches instead of leaving them behind (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:50:41Z
- **Updated:** 2026-10-13T02:52:50Z
- **Closed:** 2026-10-13T02:52:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2270
- **Merged:** 2026-10-13T02:52:30Z
- **Merge commit:** 707ee84ade0c1b447d653a2979c4827a3d0b59ed
- **Base / Head:** main / UOR-Foundation:harness/merged-branch-cleanup
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

No model compute. Milestone reference: #2028.

## What and why

Owner direction (12 October 2026): merge branches into main through their PRs, and delete them
afterwards. GitHub auto-delete is **off** for this repository (`delete_branch_on_merge=false`) and
the setting cannot be changed without org admin, so the cleanup has to be explicit — and it was
being skipped. Six branches whose PRs had merged were still present, with stale worktrees behind
three of them.

## Result

Decision: **KEEP** (tooling).

Adds `cli.py git clean-merged`:

- **Detects both merge shapes.** A merge commit leaves the branch head as an ancestor of
  `origin/main`. A **squash merge** — what this repository's queue uses — never does, so those are
  identified by a `[gone]` upstream plus nothing unmerged. The first version of this command used
  only the ancestor test and therefore wrongly reported six genuinely-merged branches as unmerged;
  the fix is in this PR.
- **Never touches a branch checked out in a worktree**, so no other lab's live work is at risk.
- **Defaults to reporting**; `--apply` deletes with `git branch -d`, which refuses if git itself
  does not agree the branch is merged — the belt to the detection brace.

```
$ python3 scripts/harness/cli.py git clean-merged
local branches: 9 merged into origin/main and not checked out, 5 in a worktree, 6 unmerged

merged and safe to delete:
  bpb-split
  kimi/gate-ops-2026-10-12
  ...
```

Also enables `fetch.prune` globally so stale remote refs stop accumulating, and renames an
appended helper that shadowed the hook installer's `_run` — which broke 8 of its tests. The suite
caught it; every hook test passes again.

```
python3 -m unittest discover -s tests/harness -t .   ->  245 tests, OK
python3 scripts/harness/cli.py check                 ->  43 rules, 0 error
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] No model compute
- [x] Tooling only; no research record required, and no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Branch deleted after merge (the point of this PR)

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

