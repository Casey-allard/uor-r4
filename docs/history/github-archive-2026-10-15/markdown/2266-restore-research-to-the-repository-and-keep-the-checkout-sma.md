# #2266 — Restore research/ to the repository, and keep the checkout small instead (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:33:33Z
- **Updated:** 2026-10-13T02:37:34Z
- **Closed:** 2026-10-13T02:36:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2266
- **Merged:** 2026-10-13T02:36:50Z
- **Merge commit:** 84fa246693637d2dd4c31fec1b794cf6e391052e
- **Base / Head:** main / UOR-Foundation:restore/research-material
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - repository integrity · count 0/3 · headline 37/232 → 37/232 (unchanged)

No model compute. Milestone reference: #2028.

## What and why

**I removed evidence and the owner corrected me.** PR #2263 deleted
`research/{prime-analysis,ai-research,archives}` — 11,171 files — from the tree, on the argument
that the bytes were MD5-verified in the iCloud store. That argument was wrong. A copy existing
elsewhere is not the same as the repository being allowed to drop it. This is a research project;
the repository is the record.

The owner's direction: *"you do not ever need to remove stuff like that, if its dormant or you do
not know what it is you can simply archive it... do not delete anything because this whole
repository is a research project and we need to keep all evidence."*

## Result

Decision: **KEEP** — the material is restored and the repository is whole again.

**Restored byte-exact**, not approximately: all three directory trees hash identical to the commit
they were removed in (`7fc6e0b88`):

```
research/prime-analysis   IDENTICAL
research/ai-research      IDENTICAL
research/archives         IDENTICAL
```

Two independent recovery paths existed throughout, which is why nothing was lost: the files were
always in git history, and three MD5-verified iCloud archives were written before the removal
(`prime-analysis` was additionally restored and compared — 7,713 files, matching content digest).

**The disk problem is real, and is now solved the other way round.** The material stays **tracked**;
a working copy may keep it out of its checkout using git's own `skip-worktree` bit:

```sh
scripts/storage/research-data drop-local      # ~991 MB out of this checkout, 11,171 files
scripts/storage/research-data restore-local   # bring them back
```

No commit is involved, `git status` stays clean because the files are still tracked, and a fresh
clone gets everything normally. That is what I should have proposed instead of deleting.

Also in this unit, from the same "keep all evidence" direction and the agreed follow-ups:

- **Five durable lessons consolidated out of provider-local memory**, so they stop being trapped in
  one vendor's store and become portable:
  `kill-only-own-pids` (a 2026-10-03 pattern kill took another lab's test — block),
  `detached-run-watcher` (a rented box billed ~6 idle hours — warn),
  `archive-reference-check` (the iCloud copy was not sufficient; the exact lesson from this unit),
  `geometry-carries-learning`, `no-transformer-baselines`.
- **`prep.no_prohibited_teacher`**, a new rule with a test: a pre-registration proposing a softmax
  teacher, dense helper or transformer control now fails its own check. Verified to fire on a probe
  sheet and to pass a clean one. This is precisely the drift that happened on 12 October.
- `research/README.md` corrected: the material is tracked, with the local-only option documented.

## Checklist

- [x] Checks run at this exact head: 227 tests OK; `cli.py check` 39 rules, 0 error
- [x] No model compute
- [x] Everything that was removed is restored byte-exact (tree hashes above)
- [x] Research record: docs/labs/repository-integrity-2026-10-12/README.md
- [x] New entry at the top of docs/integration/current-state.md, with **Next:**
- [x] The lesson is captured as docs/harness/mistakes/archive-is-not-deletion.md
- [x] No document left claiming the material was untracked
- [x] `python3 scripts/check_claim_wording.py` passes
- [ ] After merge: verify on fresh `origin/main`, delete the branch

### Stated rather than hidden

- This is a large diff (3.45M insertions) purely because it reverses a large deletion. The tree
  content is identical to `7fc6e0b88` for all three directories.
- `.gitignore` no longer hides these paths, so `git status` will show them normally again.
- Nothing here changes the iCloud archives; they remain as a second copy and are still listed in
  `docs/harness/research-data.json`.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5498137896 — 2026-10-13T02:35:56Z — Casey-allard — COMMENTED

PM review (kimi/pm1) — LGTM, merge when the pending harness check greens. (Approval not recordable via API: all labs commit under one account, so this is recorded as a comment per the exact-head review rule.)

Verified independently against the removal commit 7fc6e0b88:
- git tree hashes of research/{prime-analysis,ai-research,archives} at this head are IDENTICAL (d47ed4f4f…, 46fbf70975…, 44ddd7dde5…). Byte-exact restoration confirmed.
- Non-research diff is confined to: .gitignore (removes the archive-ignore block), scripts/storage/research-data (adds drop-local/restore-local skip-worktree commands — material stays tracked, origin keeps the record), scripts/harness/prep.py, and 5 new docs/harness/mistakes/ records. Nothing else rides along.
- The skip-worktree approach is the right resolution of the space problem: the repository keeps the evidence (owner direction 12 October), individual checkouts can stay small, git status stays clean.

Checks: all applicable jobs pass at review time; 'harness rules and tests' pending (this PR touches scripts/harness, so it legitimately applies).

This satisfies the owner's correction on #2263.


---
_Comment/review ids in this file: 5498137896_

