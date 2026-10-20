---
id: archive-is-not-deletion
title: A large tracked directory was removed from the repository because a copy existed in iCloud
date: 2026-10-12
severity: block
class: evidence
trigger: [free disk space, out of space, remove, delete, untrack, git rm, archive, iCloud, dormant, unused, cleanup]
paths: [research/**, docs/**, docs/harness/research-data.json, .gitignore]
milestones: [m1, m2, m3, m4]
mechanism: ""
guard: none: the decision is "may the repository drop this", which is the owner's and is not machine-checkable
evidence: [archive/labs/repository-integrity-2026-10-12/README.md, docs/labs/working-in-one-checkout.md, docs/mechanisms/README.md, AGENTS.md]
status: active
---

## What happened

Facing 31 GiB free, an agent archived `research/{prime-analysis,ai-research,archives}` — 11,171
files, ~1 GB — to the MD5-verified iCloud store, then removed them from the git tree and added
them to `.gitignore` (PR #2263). The reasoning was explicit and wrong: the bytes were verified in
a second location, so the repository "did not need" them.

The owner corrected it: *"you do not ever need to remove stuff like that, if its dormant or you do
not know what it is you can simply archive it... do not delete anything because this whole
repository is a research project and we need to keep all evidence."* The material was restored
byte-exact in PR #2266.

## Symptom

A commit whose diff is enormous and mostly deletions, in a repository that is a research record
rather than a product. The files were not source code, which made the removal feel safe — and
3,368 of them were nonetheless Python analysis scripts, and 4,207 were markdown.

## Lesson

**A copy elsewhere is not permission for the repository to drop evidence.** "Archived" and
"removed" are different operations, and only the first is this project's answer to a disk problem.
For a research repository the tracked tree *is* the record; it is also the only copy a future
reader, auditor or lab actually receives.

The correct answer to "this is large and dormant" is to keep it tracked and stop materialising it
locally, which git supports directly: `skip-worktree` keeps files tracked, out of the working
directory, and out of `git status`, at no cost to any other clone.

## How to apply

1. Never `git rm` research material to reclaim disk. Not when a copy exists, and not when the file
   looks irrelevant.
2. To shrink one checkout: `scripts/storage/research-data drop-local` (skip-worktree, ~991 MB),
   reversed by `restore-local`. The repository is unchanged; a fresh clone gets everything.
3. If a file is genuinely unknown, leave it in place — an unexplained file is a question, not a
   deletion candidate.
4. Archive a second copy if it is useful (`cloud-store put`), and record it in
   [docs/harness/research-data.json](../../harness/research-data.json) — but archive *in addition*,
   never instead.
