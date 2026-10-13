# Repository integrity: research material was removed, and put back

**Question.** The machine ran low on disk. What may be removed from a research repository to
reclaim space, and what must stay?

**Line:** no model line — repository integrity and workspace. · count 0/3 unchanged · headline
37/232 → 37/232 (unchanged). No model compute was run.

## What happened

On 12 October, with 31 GiB free, an agent archived `research/{prime-analysis,ai-research,archives}`
— 11,171 files, about 1 GB — to the MD5-verified iCloud store, verified the archives, then removed
the files from the git tree and added the paths to `.gitignore` ([PR #2263](https://github.com/UOR-Foundation/uor-r4/pull/2263)).

The reasoning was stated and wrong: because a verified copy existed in iCloud, the repository was
argued not to need the material. The owner rejected it — *"do not delete anything because this whole
repository is a research project and we need to keep all evidence"* — and the material was restored
byte-exact ([PR #2266](https://github.com/UOR-Foundation/uor-r4/pull/2266)).

Nothing was lost, for two independent reasons: the files were always present in git history, and
three MD5-verified iCloud archives had been written before the removal. `prime-analysis` had
additionally been restored and compared (7,713 files, matching content digest) before the local copy
was removed. That is what made the mistake recoverable rather than destructive — and it is also why
it should not have been made: the verification effort was spent proving something that never needed
to be risked.

## What was wrong with the reasoning

Three separate claims were collapsed into one:

1. *A verified copy exists elsewhere* — true, and it was verified properly.
2. *The material is dormant* — true; nothing in the active crates referenced it.
3. *Therefore the repository may drop it* — false, and not implied by 1 or 2.

For a research project the tracked tree is the record. It is what a future reader receives, what an
auditor checks, and what a new lab clones. A second copy in object storage is a backup, not a
substitute. It also mattered in plain terms: 3,368 of the deleted files were Python analysis
scripts and 4,207 were markdown, not the PDFs and images the reasoning had assumed.

## The mechanism that actually solves the disk problem

The material is **tracked**, and one working copy can keep it out of its checkout using git's own
`skip-worktree` bit:

```sh
scripts/storage/research-data drop-local      # 11,171 files marked skip-worktree, ~991 MB released
scripts/storage/research-data restore-local   # materialised again from the repository
```

The files remain tracked in `origin`, `git status` stays clean, no commit is made, and a fresh clone
or worktree gets everything normally. This is a per-checkout convenience, not a repository change —
which is exactly the distinction the original change failed to make.

## What this unit also delivered

Five durable lessons were consolidated out of provider-local memory, where they were readable by
only one vendor's agent:

| lesson | what it preserves |
|---|---|
| `kill-only-own-pids` | a pattern kill over `cargo`/`rustc` names took another lab's test (2026-10-03) |
| `detached-run-watcher` | a rented machine billed about six idle hours with no watcher armed |
| `archive-reference-check` | the "iCloud copy is not enough" incident, 2026-10-03 |
| `geometry-carries-learning` | the overnight drift that distilled a geometric read from softmax |
| `no-transformer-baselines` | compute spent on controls the owner had withdrawn |

Plus `archive-is-not-deletion`, this record's own lesson, and a new rule with a test:
`prep.no_prohibited_teacher` fails a pre-registration that proposes a softmax teacher, a dense helper
or a transformer control — the drift that actually happened on 12 October.

## Evidence

```
227 tests pass (tests/harness)
python3 scripts/harness/cli.py check        -> 39 rules, 0 error
restored tree hashes identical to 7fc6e0b88 for all three directories
research-data drop-local                    -> 11,171 files skip-worktree, 991 MB released
```

## Limitations

- The iCloud archives remain a second copy; they are unchanged by this record and still listed in
  `docs/harness/research-data.json`. They are a backup, not a substitute for the tracked tree.
- `skip-worktree` is per checkout. A new clone materialises the ~1 GB until someone runs
  `drop-local`; that is the intended trade — completeness by default, small by choice.
- The deletions in #2263 were merged to `main` for about an hour. Anything that cloned in that
  window got a tree without this material; re-cloning or `git checkout 7fc6e0b88 -- research/...`
  restores it.

**Next:** the two remaining agreed follow-ups (the regression arena design, and wiring session
start to the harness brief) proceed on the corrected basis: archive in addition, never instead.
