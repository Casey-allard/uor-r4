# ORDER: infra-loop-harness, claude-26 (the PM's item edits for the bench, while Kimi is out)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7c. Base: `a8ff5690e`.

## Goal

Kimi, the PM, is out of tokens. Three of Casey's bench decisions wait on item edits that are normally the PM's. Casey's direction, in Claude's chat on 22 Oct: "Claude does them (Recommended)", and for the copied sentence, "Claude, in claude-26 (Recommended)".

The planted fault: **the bench feed's new files, and the one sentence that matches the kit, have no item scope that allows them.**

## Cancel probe (run first)

At `a8ff5690e`, does any open item's scope list `crates/uor-r4-harness/src/loopbench.rs`, `tests/loop_bench*.rs` or `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md`, or carry an `AGENTS.md`-only inbox rule? No. The task stands. Class: instrument.

## Calls

- `infra-loop-harness`'s `scope:` gains `crates/uor-r4-harness/src/loopbench.rs`, `crates/uor-r4-harness/tests/loop_bench*.rs` and `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md`. Nothing else in the item changes.
- A new item, `infra-bench-inbox-rule`, is added with `scope: [AGENTS.md]`. It holds the per-lab inbox line Casey chose: "Addressed per lab (Recommended)".
- The rendered `docs/roadmap/README.md` is refreshed with `cargo xtask roadmap render`.
- Rephrasing the kit sentence itself waits for claude-27. `pr-check` reads scope at the base, so the path must be in scope on `main` first.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` and `docs/roadmap/items/infra-bench-inbox-rule.md` (`docs/roadmap/items/*.md` is in the item's scope)
- `docs/roadmap/README.md` (shared)
- `docs/loop/tasks/infra-loop-harness/claude-26/**`

## Done when

- `cargo xtask roadmap check` passes, and the render reproduces the README byte for byte.
- The umbrella scope lists the three paths, and the new item's scope is exactly `[AGENTS.md]`.
- `pr-check --local`, claim-wording, links and the copyright check all pass.

## Stop and ask if

- `roadmap check` refuses a scope that more than one open item shares.
- Any other item's text would have to change.
