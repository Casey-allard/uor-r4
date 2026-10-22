# ORDER: infra-loop-harness, claude-32 (bench checks: PASS/FAIL words, one row per gate at main's head)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7c. Base: `683e182ba`.

## Goal

Casey saw "Checks 0 of 23 passed" on the bench. The page counts a gate as passed only when its result is `PASS`, and as failed only when it is `FAIL`. The feed (`loopbench.rs`, `gate_line`) passes the loop ledger's words through unchanged: `GREEN`, `RED`, `CAUGHT`. It also lists every lab's run of every gate, and every one of those runs is on a PR branch, so all of them read "older commit".

The planted fault: **main's 9 green gates at its own head show as 0 passed.**

## Cancel probe (run first)

At `683e182ba`, does `gate_line` translate `GREEN` or `RED`? If it does, the task is void. Class: instrument.

## Calls

- **Results:** `GREEN` and `PASS` become `PASS`. `RED`, `FAIL` and `REFUSED` become `FAIL`. A `CAUGHT` line is a planted-fault run, not a gate's status, so it is dropped.
- **One row per gate:** use the newest run at `origin/main`'s head, otherwise the newest run. The row keeps its `lab/gate` name and its commit.

## Files in scope

- `crates/uor-r4-harness/src/loopbench.rs`
- `crates/uor-r4-harness/tests/loop_bench*.rs`
- This task folder

## Done when

- A test shows a head `GREEN` run becomes one `PASS` row, replacing an older lab run of the same gate; `RED` becomes `FAIL`; and `CAUGHT` is dropped. With the fix set aside, the test fails.
- The existing bench tests, clippy `-D warnings`, `loop gates --full`, `report --check` and `pr-check --local` all pass.

## Stop and ask if

- The bench needs other words besides `PASS` and `FAIL`.
