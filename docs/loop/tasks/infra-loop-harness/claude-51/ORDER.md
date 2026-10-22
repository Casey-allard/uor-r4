# ORDER: infra-loop-harness, claude-51 (the umbrella's scope gains the archive gate)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `2987514aa`.

## Goal

PR 8 (#2982) is the first change to edit the archive catalog. That is `archive/catalog.jsonl` and the pages that `docs-catalog` generates from it (`archive/CATALOG.md`, `archive/catalog/*.md`). `xtask archive-verify` fails it, because every edit to a file under `archive/` that existed at the base counts as a modification.

Casey decided the rule on #2982 at 2026-10-22T22:08Z (6352383423): **"Append-only (Recommended)"**. Under it:
- `catalog.jsonl` may only gain lines, and its existing rows stay byte-identical;
- the generated pages may change only to `docs-catalog`'s exact output.

No fitting item's scope covers `xtask/src/archive_verify.rs`. So this unit adds it to `infra-loop-harness`'s `scope:`, as claude-48 did for the links tool, and claude-52 makes the change.

## Cancel probe (run first)

At `2987514aa`, is `xtask/src/archive_verify.rs` absent from `infra-loop-harness`'s `scope:`? If it is already listed, the task is void. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (inside its own `docs/roadmap/items/*.md`)
- `docs/roadmap/README.md` (shared render)
- This task folder

## Done when

1. The item's `scope:` lists `xtask/src/archive_verify.rs`, and the item diff is that one entry.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- Another open PR changes `xtask/src/archive_verify.rs`.
