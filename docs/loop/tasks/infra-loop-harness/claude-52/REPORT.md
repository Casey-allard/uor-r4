# REPORT: infra-loop-harness, claude-52 (the archive catalog is append-only)

Lab: claude. Base: `2987514aa`, merged forward to `origin/main` after #3010 (the scope widening). Order: [ORDER.md](ORDER.md), committed first (`5f703ee6a`).

The cancel probe stands: on PR 8's tree merged with `origin/main`, `archive-verify --base origin/main` reported 3 modified files (`archive/CATALOG.md`, `archive/catalog.jsonl` and `archive/catalog/harness-governance.md`), so the task stands.

## What did not work, first

- **PR 8's tree still fails this gate as committed, and correctly so.** With this branch merged in, `archive-verify` still reports `archive/catalog/harness-governance.md` as modified. `docs-catalog --check` on the same tree also reports that page as stale (1 stale file), so the gate and the generator agree. The page is not `docs-catalog`'s output for the current data.

  After running `docs-catalog` in the probe, the gate passes:
  ```
  archive catalog appended: archive/catalog.jsonl (+2 lines)
  archive catalog regenerated: archive/CATALOG.md
  archive catalog regenerated: archive/catalog/harness-governance.md
  archive immutability vs origin/main: 10802 files, 0 modified, 0 deleted
  ```
  So PR 8 needs to merge `main` and re-run `docs-catalog`. Separately, the review on #2982 (6352662309) found that PR 8's two new rows point at the wrong archive path. That is the rows' content, which this gate does not judge.
- **One existing clippy lint in this file was fixed in passing:** `if_same_then_else` on the symlink and mode branch. The two conditions are merged and behaviour is unchanged. The other lints on `main` are untouched: `book_stage.rs:342`, `docs_relink.rs:249` and `figures.rs:1149`.

## What changed

| Commit | What |
|---|---|
| `5f703ee6a` | This ORDER, alone. |
| `01961a473` | `xtask/src/archive_verify.rs`: the rule and its tests. It also made `docs_catalog::git_universe` `pub(crate)`. |
| the scope fix commit | That visibility change is reverted: `docs_catalog.rs` is outside the item's scope. The gate builds the same universe inline, `Universe::new(tracked_files(root))`, which is exactly what `docs-catalog --check` uses. |
| `606dbbdd1` | Merge of `origin/main`, for #3010's scope entry. |
| this REPORT's commit | This REPORT. |

**The rule,** from Casey's decision on #2982 (6352383423, "Append-only (Recommended)"). A modified base-revision archive file counts as not modified only when one of these holds:
- **`archive/catalog.jsonl`:** its base bytes are a prefix of its bytes now, the base is empty or ends in `\n`, the file grew, and the whole file parses with `docs_catalog::parse_records`.
- **`archive/CATALOG.md` or `archive/catalog/<x>.md`** (with no `/` in `<x>`): its bytes equal that path's entry in `docs_catalog::render`, run over the current data.

It fails closed:
- if the base blob cannot be read, or the current catalog cannot be read or parsed, nothing is exempt;
- deletions and mode or symlink changes still fail;
- every other archive file still fails.

The logic is in pure functions (`is_catalog_page`, `catalog_appended`, `catalog_page_regenerated`, `catalog_change`). Each passing case is printed, beside `archive stub replaced:`.

## Measurements

None. This is a gate change.

## Checks run at the head

- `cargo test -p xtask`: 210 passed (200 existing, 10 new).
  - Unit tests pass for an appended row and a regenerated page.
  - These are not exempt: an edited, removed or reordered row; an append after a base with no final `\n`; an unparsable appended line; no growth; a hand-edited `CATALOG.md`; a page `render` does not produce; lookalike paths (`archive/catalogue.jsonl`, `archive/x/catalog.jsonl`).
  - Integration tests run `part_b` on temporary repos under the gitignored `local/archive-verify-tests/`. They strip every `GIT_*` variable and assert each repo's toplevel.
- **Planted fault:** with the prefix condition removed, `rewritten_rows_are_not_appends` fails (25 of 26). After restoring, `cmp` matched and all tests pass.
- `cargo clippy -p xtask --all-targets -D warnings`: no lint in `archive_verify.rs`. The 3 lints already on `main` in other files remain.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one file in scope. A visibility change to `docs_catalog.rs` was reverted, because that file is outside the item's scope.

## Still open

- **PR 8:** merge `main`, correct the two appended rows' `path` and `prs`, then re-run `docs-catalog`.

## Plain summary

The archive check now lets the archive's index grow by new entries, and lets its generated pages be rebuilt, exactly as Casey chose. Any other change to archived files still fails.
