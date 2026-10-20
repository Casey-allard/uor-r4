# #2305 — The guard refused to clean up its own branch: ancestry cannot prove a squash merge (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T06:02:35Z
- **Updated:** 2026-10-13T06:14:54Z
- **Closed:** 2026-10-13T06:14:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2305
- **Merged:** 2026-10-13T06:14:38Z
- **Merge commit:** 6de4b8d425173e1fe6c1d0aa72711e734d8ce080
- **Base / Head:** main / UOR-Foundation:fix/squash-content-proof
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Result

Decision: **KEEP** (tooling).

The guarded branch delete — added in #2303 an hour ago — **refused to clean up its own branch**:

```
REFUSED to delete fix/guarded-branch-delete: origin/fix/guarded-branch-delete has 2 commit(s)
not in main - it holds work the merge did not carry
```

It was wrong, and for the reason this repository already documents elsewhere: **a squash merge leaves
the branch's commits with no ancestor in `main`.** For a branch whose content is byte-identical to
`main`:

| test | result |
|---|---|
| `rev-list --count main..branch` | **2** — commits with no ancestor |
| `git diff --stat main...branch` | **9 lines** — merge-base diff |
| `git diff --stat main..branch` | **empty** — direct comparison ← the one that tells the truth |

Judging on either of the first two would refuse to clean up **every squash-merged branch**, which is
most of them here and exactly the case the guard exists to serve. The content proof is now a direct
diff — the same one `clean-merged` already used.

### A test that failed on its own documentation

The first version asserted `assertNotIn("rev-list", body)`. The function contains a **comment**
explaining why `rev-list` is the wrong test, so the assertion failed on the explanation. It now
asserts on **calls** (`git("rev-list"`), not on text. A test that fails on its own prose is testing
the wrong thing.

### Also: a partial delete is no longer ambiguous

`--apply` reported `deleted: false` with the reason buried in a `local_error` field, and the CLI
printed "would delete" — so a delete that half-succeeded could read as a no-op, or worse, as success
while the remote branch survived. Local and remote are now reported **separately**.

```
python3 -m unittest discover -s tests/harness -t .   -> 307 passed
python3 scripts/harness/cli.py check                 -> 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Mistake recorded: docs/harness/mistakes/unguarded-branch-delete.md
- [x] Branch deleted after merge, with the guarded command

### Stated rather than hidden

- **This is the second bug in the guard within an hour of writing it**, and both were found by
  pointing it at real branches rather than by reasoning about it. The first refused a live claim's
  branch; this one refused every squash-merged branch. Neither would have shown up in a unit test I
  wrote from my own assumptions about how merges work.
- The `rev-list` comparison was not wrong in general — a branch genuinely holding unmerged commits is
  exactly what the guard should refuse. It is the *squash* case that ancestry cannot see, and the
  direct diff covers both.
- I am still on `fix/guarded-branch-delete` locally, so the guard cannot delete it (it refuses to
  remove a branch checked out in a worktree). The remote branch is gone; the local one goes when I
  switch off it.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

