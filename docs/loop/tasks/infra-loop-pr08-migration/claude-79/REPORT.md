# REPORT: infra-loop-pr08-migration, claude-79

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first head edited `archive/history/branch-archive/INDEX.md`. CI's archive-immutability gate refused it: files under `archive/` may be added but never modified. The record is now a new file beside the index, and the index is unchanged.
- These branches cannot be merged. Each sits on the history from before `main` was re-rooted on 13 Oct 2026 and shares no commit with it; a merge would bring the whole old tree back. Each was compared with `main` file by file instead.
- The branch-archive index named none of them, and the router-prep branch's one unique file was nowhere on `main`. Without this record, deleting them would have lost the ledger and left no trace of where the rest went.

## What changed

- `archive/history/branch-archive/pre-reroot-branches-closed-2026-10-24.md` (new): the 24 October record with one row per branch, giving its tip, the PR that delivered it and where its content lives on `main`:
  - `inference-fix`: PR #2128.
  - `pcopy-accessor`: PR #2131; main holds a superset.
  - `repro-fix`: PRs #2122 and #2123.
  - `tmp-census`: PR #2253.
  - `preserve/deepseek-router-prep-2026-10-12`: the router gate shipped in PR #2243.
- `archive/history/branch-archive/peer-audit-ledger-2026-10-12.md`: antigravity's 17-line peer-audit ledger for PRs #2233–#2242. It was the router-prep branch's only unique file, and its blob is `9378a7a86dad9c983009d9a815177dc87e8e77d2`, as on the branch.

Size: docs 4 files (line counts in pr-check's size line at the top of the PR body).

## Measurements

Nothing was measured. The evidence comes from read-only comparisons of each branch with `main`: blob hashes, line containment, and the merged PRs' heads.

## Checks run at the head

- The ledger's `git hash-object` is `9378a7a86dad9c983009d9a815177dc87e8e77d2`.
- `loop report --check`, `loop pr-check --local`, `git diff --check` and the links check: results are in the PR's review record at the head.

## Skills used

None.

## Still open

- Delete the five local branches once this merges (#3165).

## Plain summary

Five old branches on this laptop came from before the project's history was restarted on 13 October, so they could not be merged into the current history. Each was checked against the current code. Four were already fully included through earlier pull requests. The fifth held one small file, a 12 October review ledger, and that file is now kept in the project's archive. The archive index now records where each branch's work went, so the branches can be deleted without losing anything.
