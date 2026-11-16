# ORDER: end-to-end-integration, codex-7

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-6](../codex-6/ORDER.md). Reviewed head: `fdf24f701`.

## Why this unit exists

ADR-0007 Amendment 1 narrows I1 to "the one `Read` interface and `ReadOut` that D86 step 6 needs",
with the native Q31 first-pick read as its reference and the Hamming copy/ask heads as the second
implementation. This unit lands the common **result** for those two reads and proves both implement
it without changing their arithmetic.

The two reads take different inputs — the Q31 read steps a history and a row; a Hamming copy head
selects over earlier tokens — so a single `select` trait over both would be a false abstraction.
What a stage consumes is their result. `ReadOut` is that result:

- `support`: the chosen positions;
- `weights_q31`: the selection's weights (empty for a row-writing read);
- `payload`: the row a row-writing read contributes (selection reads have none).

## Claim record

`loop claim end-to-end-integration --lab-topic read-out-interface` was refused because
`crates/uor-r4-integer/src/stack/**` overlaps claude-397's PR #4022. This unit adds a module and two
adapters with tests, no dependency and no `Cargo.lock` change.

## Method

- `ReadOut` with constructors `first_pick` and `selection`, an invariant `check`, and a `u16`
  position bound.
- `ServedQ31Read::read_out`: the pick and the row, stepped exactly as `serve` does (history mutated,
  `x` not written) and refused on an out word of `+2^31`.
- `CopyHeads::read_out`: the chosen positions and their Q31 margin weights, exactly `choose` plus
  `copy_weight_q31`.
- `ServedQ31Read::serve` consumes `read_out`: it writes the result's payload and traces its pick.
  This is a production-code change with **no output change**; the shipped `apply`/trace parity test
  is the check.
- Tests: the Q31 result equals the served step's written row over two rows and a third; the Hamming
  result equals `choose` and the margin weights; the `ReadOut` invariants.

## Files in scope

- `crates/uor-r4-integer/src/stack/read_out.rs` (new)
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/q31_read.rs` (adapter and test)
- `crates/uor-r4-integer/src/stack/hamming.rs` (adapter and test)
- `docs/loop/tasks/end-to-end-integration/codex-7/**`

Out of scope: session wiring (the session still calls `serve`), a `select` trait over both reads,
type/ask/flock/route/relation adapters, any output change, a schema, a dependency or a GPU run.

## Done when

1. `ReadOut` is public from the stack module and its invariant holds for both constructors.
2. The Q31 adapter reproduces the served step; the Hamming adapter reproduces the choice and
   weights; both tests pass.
3. No existing test changes; the served output is bit-identical and `serve` now consumes `read_out`;
   the library suite still passes.
4. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if the two reads cannot share a result without loss, if the adapters need a change to `choose`,
or if making `serve` consume `read_out` moves any output. Do not invent a common input.

STOP and report when done.
