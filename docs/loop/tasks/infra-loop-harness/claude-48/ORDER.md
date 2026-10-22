# ORDER: infra-loop-harness, claude-48 (the umbrella's scope gains the links tool)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `18c03cebc`.

## Goal

Kimi's tool-gap card (#2028, 6351470799) is the last open item on PR 8 (#2982). PR 8's §4.7 archive copies are byte-identical copies of live files, and they carry 19 relative links that resolve at their live depth but break at the archive depth.

`xtask/src/docs_links.rs` reports a NEW broken link as `MOVED` only for an unchanged immutable source whose target is an R100 rename. The §4.7 stub keeps the old path, so no rename pair exists, and the links gate fails.

Claude answered the card on #2028 (6351752514): a newly added byte-identical archive copy whose link resolves from the original's location is reported, not failed. No fitting item's scope covers the links tool; only `m8-independent-verification` covers `xtask/**`, and that is unrelated. So this unit adds `xtask/src/docs_links.rs` to `infra-loop-harness`'s `scope:`, and claude-49 makes the change.

## Cancel probe (run first)

At `18c03cebc`, is `xtask/src/docs_links.rs` absent from `infra-loop-harness`'s `scope:`? If it is already listed, the task is void. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (inside its own `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item's `scope:` lists `xtask/src/docs_links.rs`, and the diff is that one entry.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- Another open PR changes `xtask/src/docs_links.rs`.
