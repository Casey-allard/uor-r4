# ORDER: infra-loop-harness, claude-85 (record the 24–25 Oct iCloud archive additions)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `8792328b5`.

## Goal

On 24 Oct the owner directed the close-out to free local storage through the iCloud cold archive, because the HF store is too small:

> "if it seems Huggingface is too small for the store, we can use the icloud store where we have been putting this stuff previously (I think in tar's also but you should check) that way we can have them upload to icloud and remove them locally and only fetch them if we need them later"

Six tars went up, each md5-checked against its iCloud copy and confirmed uploaded before the local data was removed:
- the leftovers of four labs' local folders and worktrees (kimi, deepseek, antigravity, codex);
- Claude's local data and this session's evidence;
- the laptop's Vast pod archives.

`archive/storage/icloud-index-2026-10-20.tsv` cannot be edited (archive files are add-only). So this unit adds:
- `archive/storage/icloud-index-2026-10-25.tsv`: the six rows, in the same columns;
- `archive/storage/icloud-additions-2026-10-25.md`: the owner's words and what each tar holds;
- a pointer row in `docs/reference/storage.md`.

## Cancel probe (run first)

At `8792328b5`, no committed index row names any of the six tars. The unit stands. Class: instrument (records).

## Files in scope

- `archive/storage/icloud-index-2026-10-25.tsv` and `archive/storage/icloud-additions-2026-10-25.md` (new)
- `docs/reference/storage.md`
- `docs/loop/tasks/infra-loop-harness/claude-85/**`

## Done when

- The six rows are committed, each matching its iCloud index file.
- The note quotes the owner, and `docs/reference/storage.md` points to both files.
- `loop report --check` and `loop pr-check --local` pass, and the archive-verify CI step passes.
- A fresh-context review at the exact head counts.
- The PR says `References #3165` and `References #2904`.

## Stop and ask if

A row's md5 does not match its iCloud copy.
