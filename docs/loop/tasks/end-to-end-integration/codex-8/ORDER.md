# ORDER: end-to-end-integration, codex-8

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-6](../codex-6/ORDER.md). Reviewed head: `fdf24f701`.

## Why this unit exists

codex-6 proved the exact `code -> positions` index at a 4,096-token stream and reported the 32k
figure as an extrapolation (~128x at the vocabulary cap, ~137x at the measured average). The 97%
of decode cost that motivates I1b is at **32k**, so the evidence should be measured there, not
extrapolated: the index's distance count and the scan's wall time at the target context.

## Claim record

`loop claim end-to-end-integration --lab-topic read-index-32k` was refused because the item is still
claimed by this lab's codex-7 worktree (PR #4023, frozen for review). The claim is this lab's own;
the unit adds one test inside the existing module and docs, no dependency, no `Cargo.lock` change.

## Method

- A 32,768-token stream over a 256-code vocabulary; the eligible positions at row `t = 32767`.
- Compare the index's `nearest(query, 16)` to the shipped `nearest(query, t, 16, key)` exactly.
- Count the index's distance computations and print them against `t - 1`.
- Time one scan and one indexed query in the test build and print both; **no timing assertion** (a
  debug test is not a benchmark). The assertion is the exact equality and a count ratio above 64x.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs` (a test inside `code_index_tests`)
- `docs/loop/tasks/end-to-end-integration/codex-8/**`

Out of scope: wiring the index into the session, any production line, a timing harness, a schema, a
dependency or a GPU run.

## Done when

1. The indexed selection equals the scan exactly at 32k.
2. The distance count and its ratio are printed and the ratio is asserted above 64x.
3. The wall times are printed and labelled indicative, not asserted.
4. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if exactness fails at 32k or if the test needs a source change. Do not wire the index here.

STOP and report when done.
