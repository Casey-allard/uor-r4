# ORDER: infra-loop-harness, claude-29 (scope for ending D59 and for the no-idle fix)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7c. Base: `4bb1feede`.

## Goal

Casey's direction in Claude's chat, 2026-10-22T13:50:03Z: "can you fix the loop harness errors because they are causing the labs to stop their loops and wait without timers to resume so they become stale... I don't want everything to get piled up behind a single review". His answer at 2026-10-22T13:56:44Z on the D59 fallback question: "End D59 now".

Two follow-up PRs do the work. Both need paths that no item's `scope:` lists today:

- **claude-30, end D59:** the switch file `docs/harness/d59.json`, its listing in `docs/harness/owner-paths.json`, the D59 entry in `docs/integration/DECISIONS.md`, and the D59 line in `AGENTS.md`. All four are owner paths, and claude-30's ORDER carries the owner approval.
- **claude-31, no idle stops:** `loop next` gains a WAIT outcome and a `--wait` poll, and `loop drive` re-polls instead of exiting on WAIT. That touches `crates/uor-r4-harness/src/loopdrive.rs` and its tests, and the `AGENTS.md` rule on pending owner questions.

The planted fault: **the files that end D59 and stop idle exits sit outside every item's scope, so no PR may change them.**

## Cancel probe (run first)

At `4bb1feede`, does any item's `scope:` list `docs/harness/d59.json`, `docs/harness/owner-paths.json` or `crates/uor-r4-harness/src/loopdrive.rs` together with `loopnext.rs`? If one does, this task is void. Class: instrument.

## Calls

- `infra-loop-harness`'s `scope:` gains exactly six entries:
  - `docs/harness/d59.json`
  - `docs/harness/owner-paths.json`
  - `docs/integration/DECISIONS.md`
  - `AGENTS.md`
  - `crates/uor-r4-harness/src/loopdrive.rs`
  - `crates/uor-r4-harness/tests/loop_drive*.rs`

  Nothing else in the item changes.
- Owner paths stay owner paths. Scope only lets a PR touch them, and `pr-check`'s `owner-paths` rule still demands the owner's quoted approval in that PR's ORDER.
- This is the PM's edit, made at the owner's direction. Kimi checks it.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (`docs/roadmap/items/*.md` is in the item's scope)
- `docs/roadmap/README.md` (shared), only if the render changes it
- `docs/loop/tasks/infra-loop-harness/claude-29/**`

## Done when

- `cargo xtask roadmap check` passes, and the render leaves the README unchanged.
- The umbrella's `scope:` equals the old list plus the six paths.
- `loop report --check`, `loop pr-check --local`, claim-wording and the copyright check all pass.

## Stop and ask if

- `roadmap check` refuses an entry.
- Any other item's text would have to change.
