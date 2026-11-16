# ORDER: end-to-end-integration, codex-11

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-10](../codex-10/ORDER.md). Reviewed head: `cf5a24b68`.

## Why this unit exists

The read index is wired and exact, but every measurement so far is a selection or a single query.
This unit measures a **whole session** both ways — from the index and from the scan — at three
contexts, and records the ratio as it grows. It also settles the read-free overflow idea: after
inspection, it is not built, and the reason is recorded.

## Claim record

`loop claim end-to-end-integration --lab-topic session-scan-timing` was refused because
`crates/uor-r4-integer/src/stack/**` overlaps another lab's open order. The unit adds one test file
and docs, no dependency and no `Cargo.lock` change.

## Method

- One read-bearing synthetic trunk at context 1,024, 2,048 and 4,096 (width 32, two layers, four
  Hamming heads, copy key 1, the Mean rule).
- The same token stream served twice per context: index on (the default) and `set_hamming_index
  (false)` (scan); tokens compared at every step.
- Times printed, not asserted; the assertions are token equality, a ratio above 1.2 at 1k, and a
  ratio that grows with context.

## Files in scope

- `crates/uor-r4-integer/tests/hamming_index_session_timing.rs` (new)
- `docs/loop/tasks/end-to-end-integration/codex-11/**`

Out of scope: any source change, the read-free overflow policy (not built; see the REPORT), a
schema, a dependency or a GPU run.

## Done when

1. Tokens are identical between the indexed and scanned sessions at all three contexts.
2. The ratio is printed at each context and grows; the assertions are shape, not absolute time.
3. The REPORT records the read-free overflow decision and why.
4. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if the sessions differ in any token, or if the test needs a source change. Do not build the
overflow policy here.

STOP and report when done.
