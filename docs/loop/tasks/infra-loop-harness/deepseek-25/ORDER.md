# ORDER: infra-loop-harness, deepseek-25 (loop next: lane and premise-probe sources)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 1. Base: `ce7782a74`.

## Goal

Casey's instruction was to fix the loop harness. `loop next` currently prints three sources as not built (`gpu-collections`, `premise-probes`, `lane`). This unit builds the two whose inputs are defined -- the lab's lane (owner-only registry lane plus PM item `lane:`/`rank:` keys) and the first open premise probe in `PLAN.md`'s `## Premises` table -- with tests and a planted-fault gate. `gpu-collections` stays a named not-built source: a GPU run needs a collection record the watcher (loop PR 10) does not write, and the watcher lives outside this item's scope; the source says exactly that instead of guessing.

## Cancel probe (run first)

Class **instrument** (loop tooling, host only). At base `ce7782a74`, `loopnext::run` has a `// 5-7. not built yet.` block printing the three sources with fixed reasons, and `tests/loop_next.rs::the_not_built_sources_are_printed_with_their_loop_prs` pins those strings. No item carries `lane:`/`rank:`, `docs/harness/labs.json` has no `lane`, and `PLAN.md` is not on `main`. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopnext.rs`, `crates/uor-r4-harness/src/loopclaim.rs`, `crates/uor-r4-harness/tests/loop_next*.rs`, `docs/harness/gates.json`, `docs/harness/README.md`, `docs/loop/README.md`, and this task folder.

## Done when

- `ItemFields` parses optional `lane:` and `rank:` keys; `loop next` resolves the lab lane from `origin/main:docs/harness/labs.json` and offers the lowest-`rank` claimable ready item in that lane before `any`.
- `loop next` reads `origin/main:PLAN.md`, and when its `## Premises` table has a row whose status cell is `open` with a backticked probe command, offers that command as a `premise-probe` session.
- With either input absent the source names what is missing (PLAN.md not on main; no lane assigned; no item in the lane) rather than guessing, and `gpu-collections` names the missing collection record.
- New tests in `tests/loop_next_sources.rs` pin the lane ordering, the premise-probe selection, and the missing-input lines; the existing not-built test is updated to the new reasons; the whole harness suite passes.
- The new `loop-next-sources` gate runs the tests and its planted fault (removing the lane filter) turns it red with "the top of the lab lane must be offered before any".
- `AGENTS.md` stays 71 lines; `loop gates --full` GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS; receipts at the final head.

## Stop and ask if

- A GPU collection record must be defined (out of this unit: it changes the watcher, which is outside this item's scope).
- The PM/owner wants a different PLAN premise-table shape than the one documented here.