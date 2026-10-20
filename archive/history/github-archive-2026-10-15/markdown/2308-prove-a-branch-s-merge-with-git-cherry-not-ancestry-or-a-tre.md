# #2308 — Prove a branch's merge with git cherry, not ancestry or a tree diff (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:22:40Z
- **Updated:** 2026-10-13T06:24:42Z
- **Closed:** 2026-10-13T06:24:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2308
- **Merged:** 2026-10-13T06:24:20Z
- **Merge commit:** 1596dda23949031f4cbf4f93cab40b11e6871ecb
- **Base / Head:** main / UOR-Foundation:fix/cherry-content-proof
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Result

Decision: **KEEP** (tooling).

The guarded branch delete refused to clean up the branch that added it — **the third time in two
hours, in a third direction.** The pattern is the point:

| attempt | what it was | how it failed |
|---|---|---|
| `rev-list --count main..branch` | ancestry | a **squash merge leaves no ancestor**, so it reported 2 commits for a branch byte-identical to `main` and refused **every** squash-merged branch — most of them here, and exactly the case the guard exists to serve |
| `git diff --stat main..branch` | whole tree | a branch merely **behind** main shows as *removing* main's newer files, so it refused a branch whose PR had **just merged**, reporting "5 files changed" that belonged to other labs and that branch never touched |
| `git cherry origin/main origin/<branch>` | the actual question | `-` means the commit's changes are already upstream; `+` means they are not |

Verified on the real branches:

```
fix/switch-preserving-claims       -> allowed  (PR merged, content in main)
kimi/rung0-sheet-amend3-20261013   -> REFUSED  (live claim)
nonexistent/unmerged               -> REFUSED  (no merged PR)
main                               -> REFUSED
```

The test now pins the **call** (`git cherry`) rather than describing the fix, and names all three
wrong attempts — because each was plausible, each passed the case I had in mind, and only pointing the
guard at a *real* branch found them.

```
python3 -m unittest discover -s tests/harness -t .   -> 310 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check                 -> 49 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Mistake recorded: docs/harness/mistakes/unguarded-branch-delete.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **Three wrong attempts at one three-line question.** The lesson is not "pick the right git
  command" — it is that a guard reasoned about from first principles passed every case I imagined and
  failed every case reality provided. It should have been pointed at real branches on the first
  attempt, not the third.
- `git cherry` compares patch-ids, so a *rebased* branch (same change, different commit) reads as
  already-upstream, which is correct here but worth knowing if the guard is ever reused for something
  that cares about commit identity.
- A leftover duplicate comment from my earlier edit remains in `delete_merged_branch`; it is cosmetic
  and I have not cleaned it in this PR to keep the diff about the fix.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

