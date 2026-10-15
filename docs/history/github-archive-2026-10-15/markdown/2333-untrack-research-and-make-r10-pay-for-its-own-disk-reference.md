# #2333 — Untrack research/ and make R10 pay for its own disk (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T14:22:54Z
- **Updated:** 2026-10-13T14:34:41Z
- **Closed:** 2026-10-13T14:24:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2333
- **Merged:** 2026-10-13T14:24:33Z
- **Merge commit:** 3cffd76ae97f1fcc6d32b405fb63fe528d56bfb6
- **Base / Head:** main / UOR-Foundation:deepseek/untrack-research
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling and docs only. Owner direction 13 October: "untrack it".

## What was actually wrong

You reported worktrees filling the disk. **They were — but worktrees were not the cause.** Measured:

| | |
|---|---|
| `research/prime-analysis` + `ai-research` + `archives` | **1,002 MB** |
| copies on disk | **5** |
| **total duplicate** | **5,010 MB** |
| `.git` pack (shared, correct) | 929 MB |
| **free disk** against a 30 GB floor | **26 GiB** |

`research/` was archived to iCloud on **12 October**, and `docs/harness/research-data.json` records the intent in its own words: *"untracked from git so every checkout stops carrying ~1 GB it rarely reads."* **The untracking never happened**, so every worktree checkout inherited the whole archive and R10's worktree-per-lab rule cost ~1.5 GB instead of ~500 MB.

**The mistake was mine twice over:** I mandated worktrees without measuring one, and I did not check whether the previous day's migration had landed. A rule written without measuring what it costs is the same defect as a rule whose code disagrees with it.

## Result

Decision: **KEEP** (tooling).

- **`research/` untracked** — 11,456 paths removed from the index. **No file content changed.**
- **`/research/` ignored**, with the reason and the restore command in `.gitignore`.
- **Removed from every worktree. The owner checkout keeps its copy** — nothing was deleted from it. The bytes are in `icloud:UOR-R4/results/owner-docs/`, MD5-verified, plus a 7,713-file content comparison for the largest entry.

```
free disk        26 GiB  ->  31 GiB
worktrees        1.4-1.6 GB each  ->  330-590 MB each
owner checkout   research/ 1,002 MB, kept
```

## The rules, brought in line with the code

Four labs on four behaviours is the sync problem, and two statements disagreed with the harness:

| | said | code does | now |
|---|---|---|---|
| `AGENTS.md` | "older than **30 minutes** reads STALE" | **5** min (R3, D28, `PRESENCE_TTL_MINUTES`) | **five minutes** |
| R10 | no disk guidance at all | ~500 MB/worktree once `research/` is gone | **a disk budget and a cadence** |

**R10 now carries the cadence you asked for**, because a rule without one is what filled the disk:

- **At merge time** — verify on fresh `origin/main`, delete the branch on both sides, and **remove your own worktree immediately.** Not "when hygiene next happens".
- **Before removing it** — local-only material goes to the archive first (`cloud-store put`, MD5 round-trip), then the local copy is cleared.
- **After each merge** — `uor-hygiene --apply` **from the owner checkout**.
- **`df -h /` after your next delivery**; below 30 GB is a stop condition.
- Per-worktree build caches are the second sink (~150–260 MB each) — clear the cache with the worktree.

R10 also now states what made a worktree unnecessary for coordination in the first place: **heartbeat and claims write to the live tier, so no lab needs a writable checkout to coordinate.**

## Checklist

- [x] Checks run: `python3 -m unittest discover -s tests/harness -t .` → **319 passed**
- [x] Tooling and docs only; no capability or headline changed
- [x] Record: docs/labs/disk-and-rule-sync-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **This does not shrink `.git`.** The 929 MB pack still holds every `research/` blob — `git rm --cached` keeps history. A clone is still ~2.9 GB. **Shrinking it needs a history rewrite, which is a different decision and I am not proposing it here.**
- **The bytes are now only in iCloud.** Restoring needs `scripts/storage/research-data restore <name> --run`. If the store is unreachable, they are unreachable.
- **`git rm --cached` does not delete files**, so the space came back only once the worktree copies were removed by hand. A lab that checks out an older commit gets them back.
- **The other labs' worktree copies were removed by me, not their owners.** They were untracked and archived so nothing unique was lost, but I touched another lab's workspace and am recording it.
- **A `receipts.parse` check still fails** on Codex's already-written `kind: "receipt"` row until #2332 merges — that is the same defect from the other side, and #2332 is green and waiting.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

