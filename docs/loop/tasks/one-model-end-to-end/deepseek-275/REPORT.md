# REPORT: one-model-end-to-end, deepseek-275 (timed place-read parity)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). ORDER: [ORDER.md](ORDER.md). Base:
`93e089ff256654e75a19641b7e2fbd2bc8e24524`.

**Status:** the merged place read's timed integer step now has the missing
normal-versus-timed control and is decision-identical at every row. This is a
test-only unit: no production path, artifact, training result or milestone changed.

## What did not work, first

- claude-351 names this gap: the timed step was not run with a place read. Its
  duplicate register-and-offer call could diverge without a failing check.
- There was no production defect. The response was a test, not a refactor.

## What changed

- `crates/uor-r4-integer/src/stack/tests.rs`: one test compares the complete
  `NativeStep` and `last_place()` on every `PLACE_TALK` row.

No production file differs from `93e089ff2`.

## Measurements

- `cargo test -p uor-r4-integer --features serving-wasm --lib the_timed_step_serves_the_place_read_like_the_normal_step`:
  **1 passed, 0 failed**.
- Negative gate: removing the timed path's offer made the test **fail** at the
  first open row. The production file was restored byte-for-byte.
- The test compares all 22 rows of the fixture's token stream.

## Checks run at the head

- The targeted parity test after restoring the production file.
- The deliberate negative gate, and `git diff --check`.

## Skills used

`working-discipline`, `solution-integrity` and `scope-guard` (latest-main
branch state, shipped caller, test-only edge).

## Still open

- No GPU, trained artifact or stage-3 p/n pair was run.
- Response quality remains Claude's live run.

## Plain summary

The normal and timed integer steps now produce the same token, query, kept
cells and place register on every fixture row. Removing the timed place offer
makes the test fail. No serving code changed.
