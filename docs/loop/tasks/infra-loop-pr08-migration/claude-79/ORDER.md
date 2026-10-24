# ORDER: infra-loop-pr08-migration, claude-79 (record the five pre-re-root branches; keep the router-prep peer-audit ledger)

Lab: claude. Item: [infra-loop-pr08-migration](../../../../roadmap/items/infra-loop-pr08-migration.md) (#2913). Setup row: 8. Base: `204cb0c9f`.

## Goal

Five local branches sit on the history from before `main` was re-rooted on 13 Oct 2026: `inference-fix`, `pcopy-accessor`, `repro-fix`, `tmp-census` and `preserve/deepseek-router-prep-2026-10-12`. They share no commit with `main`. The owner asked for each to be understood and finished into `main`, with nothing forgotten (#3165).

Read-only investigations found four of them already on `main`, delivered as PRs before the re-root. The fifth carries one file nowhere on `main`: a 17-line peer-audit ledger. This unit records each branch's disposition in the branch-archive index and adds that ledger byte-identically, so all five branches can be deleted with nothing lost.

## Cancel probe (run first)

At `204cb0c9f`, does the branch index name these branches, and is the ledger on `main`? No:
- `grep -c -E "inference-fix|pcopy-accessor|repro-fix|tmp-census|deepseek-router-prep" archive/history/branch-archive/INDEX.md` prints 0;
- blob `9378a7a86` is absent from `git ls-tree -r` of `main`.

The unit stands. Class: instrument (records).

## Files in scope

- `archive/history/branch-archive/pre-reroot-branches-closed-2026-10-24.md` (new; `archive/` is append-only, so the existing `INDEX.md` is not edited)
- `archive/history/branch-archive/peer-audit-ledger-2026-10-12.md` (new, byte-identical to blob `9378a7a86`)
- `docs/loop/tasks/infra-loop-pr08-migration/claude-79/**`

## Done when

- A new 24 Oct record beside the index has one row per branch: its tip, the PR(s) that delivered it, and where its content lives on `main`.
- The ledger file's blob is `9378a7a86dad9c983009d9a815177dc87e8e77d2`.
- `loop report --check`, `loop pr-check --local`, `git diff --check` and the links check (no new broken link) pass.
- The exact-head review is Claude's own fresh-context review (D66). References #3165.

## Stop and ask if

Any branch turns out to hold content that is not on `main`, other than the ledger.
