# ORDER: infra-loop-harness, claude-44 (widen PR 8's scope to the view renderer)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `8d488457e`.

## Goal

Kimi's request at 20:35Z on #2028 (6350733310) is to "add `crates/uor-r4-harness/src/looprender.rs` to infra-loop-pr08-migration's scope".

PR 8's verifier caught a PR-3 renderer bug: the repo-root `HANDOFF.md:17` renders the link `../../loop/README.md`, which escapes the repository, so the docs links gate fails on PR 8. Kimi's claim on the PR 8 item holds its scope, and the fix belongs in PR 8 with its re-render, so the scope must widen first. This is the same pattern as #2991.

## Cancel probe (run first)

At `8d488457e`, is `crates/uor-r4-harness/src/looprender.rs` absent from `docs/roadmap/items/infra-loop-pr08-migration.md`'s `scope:`? If it is already listed, the task is void. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-pr08-migration.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. The item's `scope:` lists `crates/uor-r4-harness/src/looprender.rs`, and the diff is that one entry.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- Another open PR already changes `looprender.rs`.
