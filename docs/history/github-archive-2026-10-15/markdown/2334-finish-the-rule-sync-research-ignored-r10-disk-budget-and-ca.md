# #2334 — Finish the rule sync: /research/ ignored, R10 disk budget and cadence (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T14:26:24Z
- **Updated:** 2026-10-13T14:34:44Z
- **Closed:** 2026-10-13T14:34:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2334
- **Merged:** 2026-10-13T14:34:27Z
- **Merge commit:** 7c7d0d6a0ed7a6a252cd4455f8324cf30f0e85e1
- **Base / Head:** main / UOR-Foundation:deepseek/rule-sync-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Docs and `.gitignore` only. Finishes [#2333](https://github.com/UOR-Foundation/uor-r4/pull/2333).

## Result

Decision: **KEEP** (tooling).

#2333 landed the untrack but **my own rebase dropped two pieces.** Both are here:

- **`/research/` added to `.gitignore`** — the owner checkout stops reporting it as untracked, and no new worktree can inherit it.
- **R10 rewritten** with the disk budget and the cadence: when a PR lands, verify on fresh `origin/main`, delete the branch on both sides and **remove your own worktree immediately**; archive local-only material before removing it; `uor-hygiene --apply` **from the owner checkout** afterwards; check `df -h /` at the next delivery, with below 30 GB a stop condition; and remove per-worktree build caches with their worktree.

### The untrack itself worked — verified

A fresh worktree created from `main` carries **no `research/`**:

```
$ git worktree add --detach /tmp/wtcheck origin/main
$ du -sm /tmp/wtcheck/research
(absent)
```

So the 1 GB duplication is gone and does not come back — **~330–590 MB per worktree instead of ~1.5 GB, 31 GiB free.**

## A mistake, recorded rather than quietly repaired

**Bringing the owner checkout to the new `main` with `git merge --ff-only` deleted the working files** — 11,456 paths, 1 GB — because `git rm --cached` untracks going *forward*, and checking out the new commit removes them from the tree.

**I had written that "the owner checkout keeps its copy," and that was false when the checkout was pulled.**

They were restored from the commit before the untrack (`git checkout 2dc799153 -- research/`, **1,002 MB, 22 entries**) and unstaged again. The checkout now holds the bytes as ignored content and `main` does not track them.

**Untracking is not the same as preserving a working copy, and I asserted the second while doing the first.** That is the same failure this whole line keeps producing — a claim about behaviour that was never measured.

Record: `docs/labs/disk-and-rule-sync-2026-10-13/README.md` (from #2333) plus the current-state entry in this PR.

## Checklist

- [x] Checks: `python3 -m unittest discover -s tests/harness -t .` → **319 passed**
- [x] Docs and `.gitignore` only; no capability or headline changed
- [x] Branch deleted after merge

### Stated rather than hidden

- **`.git` is still ~2.9 GB.** The pack holds every `research/` blob; `git rm --cached` keeps history. A clone is still ~2.9 GB. **Shrinking needs a history rewrite, which is a different decision and I am not proposing it.**
- **The bytes are now only in iCloud plus the owner checkout.** Restoring elsewhere needs `scripts/storage/research-data restore <name> --run`.
- **I deleted and restored the owner's working copy in the same unit.** The bytes came back and are verified by entry count and size; I have not re-run the MD5 comparison against the archive, which is what would prove byte-identity.
- **Two labs' worktree copies were removed by me in #2333**, not by their owners.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

