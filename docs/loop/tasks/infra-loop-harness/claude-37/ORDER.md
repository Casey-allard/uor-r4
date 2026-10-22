# ORDER: infra-loop-harness, claude-37 (widen PR 8's scope to the lesson-id validator)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `0aac25aca`.

## Goal

Kimi's request at 17:26Z on #2028 (6346971551), repeated at 18:32Z (6348296166), is "add `crates/uor-r4-harness/src/mistakes.rs` to infra-loop-pr08-migration's `scope:`".

PR 8 (#2982) keys every lesson `L-NNN`, but `mistakes.rs`'s kebab-case id rule (`:148-150`, `:217-218`) refuses uppercase ids. Kimi's claim kimi-9 holds the PR 8 scope, so the change cannot come from her own PR without colliding with it, and she is now out of tokens. PR 8 waits on exactly this line.

## Cancel probe (run first)

At `0aac25aca`, is `crates/uor-r4-harness/src/mistakes.rs` absent from `docs/roadmap/items/infra-loop-pr08-migration.md`'s `scope:`? If it is already there, the task is void. Class: instrument.

## Calls

- **One edit:** append `crates/uor-r4-harness/src/mistakes.rs` to that item's `scope:` list. Nothing else in the item changes.
- **No code:** the validator change itself is Kimi's, in PR 8.

## Files in scope

- `docs/roadmap/items/infra-loop-pr08-migration.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item's `scope:` lists `crates/uor-r4-harness/src/mistakes.rs`, and the diff touches only that line.
2. `report --check` and `pr-check --local` pass.
3. Any roadmap render check passes.

## Stop and ask if

- The roadmap README or `issues.json` renders scopes, so the edit needs a re-render outside this scope.
