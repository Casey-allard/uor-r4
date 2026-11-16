# ORDER: end-to-end-integration, codex-10

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-9](../codex-9/ORDER.md). Reviewed head: `8db88b18a` (codex-9, awaiting its merge).

## Why this unit exists

codex-9 put the exact `CodeIndex` in production. This unit wires it into the served Hamming read:
the session keeps one `HammingIndex` per read, the read catches it up to the row, and the `Mean`
rule on single-code keys selects from it instead of scanning. A session switch runs the same stream
with the index and with the scan, so the exchange can be proven identical before any claim about
cost.

## Claim record

`loop claim end-to-end-integration --lab-topic read-index-wire` was refused because
`crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983. This unit changes the integer
stack and its tests, adds no dependency and does not change `Cargo.lock`.

## Method

- `HammingIndex` per read: a `CodeIndex` per copy head and per type head (the ask heads share the
  type heads' keys); built from `ServedHamming`, cleared on reset and restore, caught up per row.
- `CopyHeads::mean_at` and `TypeHeads::mean_by` take an optional index; the indexed path is
  `CopyRule::Mean` on copy key 1 and the type/ask heads. Latest, longest, pair heads and the forced
  replay keep the scan.
- `ServedHamming::stream`/`readout` take the read indexes; the session passes its own, or an empty
  slice when the switch is off.
- `IntegerStackSession::set_hamming_index(bool)` and the session field; the default is on.
- A session test exports a read-bearing synthetic trunk, steps the same 96 ids twice, and compares
  tokens and the Hamming trace with the index on and off, then repeats after `reset`.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/tests.rs` (call sites only)
- `crates/uor-r4-integer/tests/hamming_index_parity.rs` (new)
- `docs/loop/tasks/end-to-end-integration/codex-10/**`

Out of scope: the pair-key index, a 32k session run, context wiring, a schema, a dependency or a
GPU run.

## Done when

1. The indexed session and the scanning session serve identical tokens and Hamming traces over 96
   steps and again after a reset.
2. The default path has the index on; the switch is the only way to force the scan.
3. Reset and restore clear the index; a cleared index catches up correctly.
4. No existing test is weakened; the library suite passes.
5. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if the indexed session differs from the scan in any step, or if the wiring needs a change to
`choose`/`nearest`. Do not add the pair-key index here.

STOP and report when done.
