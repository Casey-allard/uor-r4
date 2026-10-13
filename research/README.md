# research/

Research material collected during the project: papers, notes, analysis scripts and data.
**All of it is tracked in git.** This is a research project and the repository is the record,
so nothing here is removed to save space — see the owner direction below.

## A local disk note, not a repository change

Three directories are large and largely dormant:

| directory | size | files |
|---|---|---|
| `research/prime-analysis` | 535 MB | 7,713 |
| `research/ai-research` | 358 MB | 3,337 |
| `research/archives` | 99 MB | 129 |

They stay **tracked in git** so every clone and every future reader has them. Because they are
dormant, a working copy may keep them out of the checkout *locally* without changing the
repository, using git's own `skip-worktree` bit:

```sh
scripts/storage/research-data drop-local     # mark them skip-worktree and remove the working copies
scripts/storage/research-data restore-local  # bring them back into this checkout
```

That is a per-checkout convenience. No commit is made, nothing leaves the repository, and
`git status` stays clean because the files are still tracked. A fresh clone gets everything
normally.

They are also archived to the MD5-verified iCloud store as a second copy, listed in
[docs/harness/research-data.json](../docs/harness/research-data.json):

```sh
scripts/storage/research-data status
scripts/storage/research-data restore <name> --run
```

**Owner direction, 12 October 2026:** do not delete research material to reclaim space. If
something is dormant or its purpose is unclear, keep it — archive a copy if useful, and leave
the repository whole. Evidence is the point of this project.
