# ORDER: infra-loop-harness, claude-52 (the archive catalog is append-only)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `origin/main` once claude-51 (#3010, the scope widening) has merged.

## Goal

Casey's rule for the archive catalog, #2982 at 2026-10-22T22:08Z (6352383423): **"Append-only (Recommended)"**. Under it:
- `catalog.jsonl` may only gain lines, and its existing rows stay byte-identical;
- `CATALOG.md` and `catalog/*.md` may change only to `docs-catalog`'s exact output.

Part B of `xtask archive-verify --base REV` reports a modified base-revision archive file as **not** a modification only in two cases:
1. **`archive/catalog.jsonl`:** its bytes at REV are a prefix of its bytes now, the prefix ends on a line boundary (REV's bytes are empty or end in `\n`), and the whole file parses with `docs_catalog::parse_records`.
2. **`archive/CATALOG.md` or `archive/catalog/<page>.md`:** its bytes now equal the entry for that path in `docs_catalog::render(parse_records(catalog.jsonl now), exists)`. `exists` is the same tracked-file universe that `docs-catalog --check` uses.

Every other modified or deleted base-revision archive file fails as before. That includes a catalog row edited, reordered or removed, a hand-edited page, and a deleted page. Each passing case is printed, like the existing `archive stub replaced:` line:
- `archive catalog appended: archive/catalog.jsonl (+N lines)`;
- `archive catalog regenerated: <path>`.

PR 8 (#2982) appends 2 rows and regenerates `CATALOG.md` and `catalog/harness-governance.md`. It should pass once it merges `main`.

## Cancel probe (run first)

At the base, does `archive-verify --base origin/main` on PR 8's tree still report the 3 catalog files as modified? If it already passes them, the task is void. Class: instrument.

## Files in scope

- `xtask/src/archive_verify.rs` (from claude-51)
- This task folder

## Done when

1. **Unit tests in `archive_verify.rs`.** These pass:
   - an appended row;
   - a page equal to `render`.

   These fail:
   - an edited, removed or reordered row;
   - an append after a base without a final `\n`;
   - a row that does not parse;
   - a hand-edited page;
   - a deleted page;
   - a modified non-catalog archive file.
2. **Planted fault:** with the prefix condition removed, a test fails.
3. **PR 8's tree,** merged with this branch locally: `archive-verify --base origin/main` passes, and lists the 3 catalog files as appended or regenerated.
4. **Checks pass:** `cargo test -p xtask`, clippy `-D warnings` (no new lints), `report --check` and `pr-check --local`.

## Stop and ask if

- Matching `render` needs a universe that differs from `docs-catalog --check`'s.
- PR 8's catalog change is not a pure append after all.
