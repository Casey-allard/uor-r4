# ORDER: infra-loop-harness, kimi-14 (widen PR 8's scope to the pr-check test file)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `1c728ff87`.

## Goal

Widen `infra-loop-pr08-migration`'s `scope:` by one entry:
`crates/uor-r4-harness/tests/loop_prcheck.rs`.

PR 8's counted CHANGES records (claude and deepseek at `e05f9feeb`) require rendered-lock
both-way tests in `tests/loop_prcheck.rs` (repair order kimi-11). Those tests now exist on
the PR 8 branch, but `loop pr-check --local` fails its `scope` rule there: the item's scope
names only `crates/uor-r4-harness/tests/loop_migrate*.rs`, and `pr-check` reads the scope at
the merge base, so the widening must land on main first. This is the same pattern as #2991
and #3000 (PM lane: scope lines are the PM's to maintain).

## Cancel probe (run first)

At `1c728ff87`, is `crates/uor-r4-harness/tests/loop_prcheck.rs` absent from
`docs/roadmap/items/infra-loop-pr08-migration.md`'s `scope:`? If it is already listed, the
task is void. Class: instrument. Result: absent (grep finds 0 matches), so the task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-pr08-migration.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item's `scope:` lists `crates/uor-r4-harness/tests/loop_prcheck.rs`, and the diff is that one entry.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- Another open PR already changes this item's `scope:` line.
