# ORDER: infra-loop-harness, deepseek-26 (loop next: owner-direction source)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 1. Base: `a472d2781`.

## Goal

Casey's instruction was to fix the loop harness. The previous unit (#2969) built the `lane` and `premise-probes` sources; this one builds the remaining in-scope source: `owner-direction`, the first open row of `PLAN.md`'s `## Owner directions` table, offered before repairs as loop step 1 requires. It reuses one PLAN table reader for both sources. `gpu-collections` still needs the watcher file (`loopdrive.rs`) outside this item, so it remains a separate scope decision.

## Cancel probe (run first)

Class **instrument** (loop tooling, host only). At base `a472d2781`, `loopnext::run` prints `owner-direction: not built — owner directions live in PLAN, loop PR 8` and the not-built test pins that string; `PLAN.md` is absent on `main`. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopnext.rs`, `crates/uor-r4-harness/tests/loop_next*.rs`, `docs/harness/gates.json`, `docs/harness/README.md`, `docs/loop/README.md`, and this task folder.

## Done when

- `loop next` reads `origin/main:PLAN.md` and, when its `## Owner directions` table has a row whose status cell is `open`, offers that row's direction as the first decision (kind `owner-direction`) before repairs.
- The source reuses one `plan_open_row` reader with `premise-probes`; with no PLAN.md it names the missing file, and with no open row it says so.
- New tests pin the owner-direction offer and the missing-PLAN line; the existing not-built test is updated; the whole `loop_next` suite passes.
- The new `loop-next-owner-direction` gate runs the tests and its planted fault (removing the open-row filter) turns it red with "the open owner direction must be offered before repairs"; the `loop-next-sources` gate still passes.
- `AGENTS.md` stays 71 lines; `loop gates --full` GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS; receipts at the final head.

## Stop and ask if

- A GPU collection record must be defined now (out of this unit: changes the watcher, outside this item's scope).
- The PM wants a different owner-direction table shape than the documented one.