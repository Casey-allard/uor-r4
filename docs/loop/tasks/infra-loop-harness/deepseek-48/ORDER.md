# ORDER: infra-loop-harness, deepseek-48 (fix the two carded prep.rs bugs, #3041 and #3042)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (the scope widened in #3055 covers `crates/uor-r4-harness/src/prep.rs`). Base: `15d5728c7`. `References #3041, #3042` (the PM closes them after merge).

## Goal

Fix the two verified `prep.rs` bugs, with an inline regression test each:

1. **#3041 - `test_fitness` reads the FIRST verdict.** A sheet that appends a later review (a `FIT` under an older `NOT_FIT`, or a later `NOT_FIT` over an older `FIT`) is read by its first line only, so the appended verdict never wins. The D22 gate must read the **last** anchored verdict line.
2. **#3042 - `render_fit` emits dirty text.** Its clearance paragraph ends `(D22). ` with a trailing space and leaks nine spaces of continuation indentation before `Every mechanism...`; the block is pasted into sheets and `git diff --check` rejects it. Emit clean plain text and move the paragraph into a `FIT_CLEARANCE_NOTE` const so a test can hold it to that standard.

## Cancel probe

Class instrument (harness code; no GPU). At the base: `test_fitness` returns the first verdict (`return verdict` inside the loop); `render_fit`'s literal contains `(D22). \n\n         Every mechanism...`. Both verified in source and by the cards' authors. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/prep.rs` (the two functions + inline tests)
- this task folder

## Done when

- `test_fitness` returns the last anchored verdict; tests cover an appended FIT over an old NOT_FIT and a later NOT_FIT over an old FIT.
- `render_fit` emits the clean `FIT_CLEARANCE_NOTE`; a test fails on a trailing space or leaked continuation indent.
- `cargo test -p uor-r4-harness`, `loop report --check`, `loop pr-check --local`, `loop gates --full` pass; a non-author exact-head review is requested (code PR).

## Stop and ask if

- A fix would need a file outside `prep.rs` (it should not; tests are inline in the same file).