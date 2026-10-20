# Workspace, storage and hygiene

This page says where every kind of file lives, and how the laptop is kept tidy. The owner decided it on 20 October 2026 (see the [change log](../rules/changes.md)). The rule itself is item 9 of section 5 of the [charter](../../CHARTER.md).

## Where files live

Sizes as of 20 October 2026. To find a particular file, or a folder removed from the laptop, use [Storage indexes](../reference/storage.md).

| Place | Size | What is there |
|---|---|---|
| GitHub `archive/` | 357 MB | Every dated record before 18 October, indexed by [`archive/CATALOG.md`](../../archive/CATALOG.md). |
| GitHub live | 137 MB | Source, docs, records and small fixtures. |
| HF private store `caseyallard/uor-r4-store` | 22.1 GB | `codex/` 10.8 GB, `data/` 2.9 GB, `vast-archive/` 2.5 GB, models and bases 3.4 GB, `research-archive/` 1.1 GB, `toolchain/` 0.8 GB, `bin/` 0.5 GB, plus `results/`, `pod-setup/` and `local-archive/2026-10-20/` (the backup of the laptop workspace). |
| HF public | 4.5 GB | `uor-r4-geometric-214m` (3.4 GB) and `uor-r4-data` (1.1 GB). |
| iCloud `UOR-R4/results/` and `X10 Pro backup 2026-10-01/workspace/` | 198 GB | Cold, read-only: run and workspace tars from before 20 October. The index of both is in [`archive/storage/`](../../archive/storage/README.md). |
| The laptop | one clone | `~/uor-r4`, and nothing else. |

## Where each new file goes

- **In git:** source, docs, records and fixtures under 5 MB.
- **In the HF store:** datasets, tokenizers, checkpoints, corpora, run outputs, binaries and workspace backups. Record each one in git by its path and its sha256.
- **Never uploaded:** build output.
- **Uploaded selectively, at the end of a task:** scratch.

Results go to `results/<lab>/<topic-date>/` with a `SHA256SUMS`: use `/root/uor-store put <dir> results/<lab>/<topic-date>/` on a pod, or `hf upload caseyallard/uor-r4-store <dir> results/<lab>/<topic-date> --repo-type dataset` from the laptop, then read it back (`hf download … --repo-type dataset`) and check the sums. `uor-pod down --to-store` is different: it archives a whole pod to `vast-archive/<pod>/`.

## The laptop

```text
~/uor-r4/                          the one clone; owner checkout, clean main
~/uor-r4/.worktrees/<lab>-<topic>/ one worktree per open PR
~/uor-r4/local/<lab>/<topic>/      scratch, gitignored
~/uor-r4/target/<lab>/             build output (CARGO_TARGET_DIR)
```

Nothing project-related lives anywhere else: no `~/uor-r4-local`, no second clones, no `/tmp`.

## Per task

- **Start:** make the worktree `~/uor-r4/.worktrees/<lab>-<topic>`, claim the branch `<lab>/<topic>`, and set `CARGO_TARGET_DIR=~/uor-r4/target/<lab>`. Keep scratch in `~/uor-r4/local/<lab>/<topic>/`.
- **Finish, after the merge is verified on fresh `main`:** upload what is worth keeping to the HF store. Then remove the worktree, the branch (both sides), the claim and the scratch. A task that ends without a merge keeps its branch until its lab or the PM decides.

## After every merge

Run `uor-hygiene`. It is a dry run and shows what it would remove. Then run `uor-hygiene --apply`. The rules live in the Rust harness (`crates/uor-r4-harness/src/hygiene.rs`); the script builds the harness from the checkout it is in and runs it, and `uor-hygiene --help` lists every rule and its guard. It never removes a branch with an open PR or a folder holding a `.keep-local` marker, and it moves scratch and temp files to the Trash rather than deleting them.

## Weekly

The PM posts a storage receipt on #2028 with:

- free disk space;
- the size of the HF store;
- worktrees older than 3 days;
- branches outside the registry;
- scratch older than 7 days;
- anything project-like outside `~/uor-r4`.

## Branches

`main`, `gh-pages` (the live documentation site and the Studio) and `gh-pages-preview` are long-lived and never deleted. They are listed in the registry `docs/harness/branches.json`. Every other branch is `<lab>/<topic>`, one per open PR, and is deleted after its merge.

## Not in git

These paths are ignored on purpose.

| Path | Why |
|---|---|
| `.uor-models/` | The runtime model store. Models are large and rebuilt or fetched from the HF store. |
| `*.u16` | Token corpora. They live in the HF store under `data/` and in the public `uor-r4-data`. |
| `/research/` | Retired on the laptop. Its source is in `projects/`; full copies are in the HF store (`research-archive/2026-10-19/`) and in iCloud (`scripts/storage/research-data restore <name>`). |
| `local/` | Per-lab scratch. |
| `.worktrees/` | Other checkouts of this same repository. |
| `target/` | Build output. |
