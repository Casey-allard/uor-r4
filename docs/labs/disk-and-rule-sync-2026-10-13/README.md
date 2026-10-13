# Untrack `research/`, and make R10 pay for its own disk

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** owner direction 13 October ("untrack it"); the space and sync report

## What was actually wrong

The owner reported that worktrees were filling the disk. **They were — but worktrees were not the
cause.** Measured:

| | |
|---|---|
| `research/prime-analysis` + `ai-research` + `archives` | **1,002 MB** |
| copies on disk | **5** (owner checkout + 4 worktrees) |
| total duplicate | **5,010 MB** |
| `.git` pack (shared, correct) | 929 MB |
| free disk against a 30 GB floor | **26 GiB** |

`research/` was archived to iCloud on **12 October** and `docs/harness/research-data.json` records the
intent in its own words: *"untracked from git so every checkout stops carrying ~1 GB it rarely
reads."* **The untracking never happened.** Every worktree checkout inherited the whole archive, so
the cost of R10's worktree-per-lab rule was ~1.5 GB per lab rather than the ~500 MB it should be.

**So the mistake was mine twice over:** I mandated worktrees without measuring one, and I did not check
whether the half-finished migration from the day before had landed. A rule written without measuring
what it costs is the same defect as a rule whose code disagrees with it.

## What changed

- **`research/` untracked** — 11,456 paths removed from the index; **no file content changed**.
- **`/research/` ignored**, with the reason and the restore command in `.gitignore`.
- **`research/` removed from every worktree.** The owner checkout **keeps its copy**, so local reads
  cost nothing and no byte is deleted: the material is in `icloud:UOR-R4/results/owner-docs/`,
  MD5-verified, plus a 7,713-file content comparison for the largest entry.

### Measured after

```
free disk        26 GiB -> 31 GiB
worktrees        1.4-1.6 GB each -> 330-590 MB each
owner checkout   research/ 1,002 MB, kept
```

## The rule, brought in line with the code

Two statements disagreed with what the harness actually does, which is how four labs end up on four
behaviours:

| | said | code does | now |
|---|---|---|---|
| `AGENTS.md` | "older than **30 minutes** reads STALE" | **5** minutes (R3, D28, `PRESENCE_TTL_MINUTES`) | **five minutes** |
| R10 | per-worktree `CARGO_TARGET_DIR`, no disk guidance | ~500 MB/worktree once `research/` is gone | **a disk budget and a cadence** |

**R10 now carries the cadence the owner asked for**, because a rule without one is what filled the
disk:

1. On merge — verify on fresh `origin/main`, delete the branch both sides, **remove your worktree
   immediately** (not "when hygiene runs").
2. Before removing it — local-only material goes to the archive first (`cloud-store put`, MD5
   round-trip), then the local copy goes.
3. After every merge — `uor-hygiene --apply` **from the owner checkout**.
4. `df -h /` after your next delivery; below 30 GB is a stop condition.
5. Per-worktree build caches are the second sink (~150–260 MB each) — remove the cache with the
   worktree.

R10 also now states the thing that made the worktree unnecessary for coordination in the first place:
**heartbeat and claims write to the live tier, so no lab needs a writable checkout to coordinate.**

## Limitations, stated

- **This does not shrink `.git`.** The 929 MB pack still holds every `research/` blob, because
  `git rm --cached` keeps history. A clone is still ~2.9 GB. Shrinking it needs a history rewrite,
  which is a different decision and not proposed here.
- **The bytes are no longer in the repository at all.** Restoring them needs
  `scripts/storage/research-data restore <name> --run` against iCloud. If the store is unreachable,
  they are unreachable.
- **`git rm --cached` does not delete files**, so the space only came back once the worktree copies
  were removed by hand. A lab that checks out an older commit gets the files back.
- **The duplicated copies in other labs' worktrees were removed by me, not by their owners.** They
  were untracked-and-archived, so nothing unique was lost, but I am recording that I touched them.
