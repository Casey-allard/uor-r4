# ORDER: native-space-state-contract, codex-14

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-13](../codex-13/ORDER.md). Reviewed head: `8cef91f6b3200ab4415324725562c5768a9a95b1`.

## Why this unit exists

codex-13 added the `Q31LaneGates` state type (K gates plus one shared cursor). This slice, the
second of the codex-12 decision, teaches the `UORLUT01` container to carry it under a new
`uor-r4.lut-stack/13` schema, so the state can be serialized, inspected and (next slice) served
per lane. No serving wiring or recurrence change is in this slice.

## Claim record

`loop claim native-space-state-contract --lab-topic q31-lanes-artifact` was refused with exit 2
because `Cargo.lock` in this item's scope overlaps PR #3983. This slice adds no dependency and
does not touch `Cargo.lock`; it continues under the owner's D85 direction.

## Schema

- New schema `uor-r4.lut-stack/13`, distinct from `/12`: an engine that serves one gate must refuse
  the per-lane artifact rather than drop the other gates.
- The table `q31.gates` holds the exact `32K + 16` state bytes as little-endian `u32` words
  (`8K + 4` words), where `K = shape.width / 4`.
- The header record `q31_lanes = { state, sha256, lanes, epoch, step_shift, version }` binds the
  digest, `K` and the shared cursor.
- A `/13` artifact carries no other optional mechanism record; a `/12` artifact and a `/13`
  artifact each refuse the other's record.
- Loading validates the table kind/length, `K = shape.width / 4`, the digest, the cursor, and every
  gate's closed Q31 domain through `Q31LaneGates::from_bytes`. A plain export is byte-identical to
  before.

## Files in scope

- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/q31_serving.rs` (tests only)
- `docs/loop/tasks/native-space-state-contract/codex-14/**`

Out of scope: the serving wiring and recurrence (next slice), `Cargo.lock`, any CUDA or GPU run.

## Done when

1. The exporter writes the exact per-lane state under `/13`, and its bytes round-trip through the
   parser.
2. The parser validates and exposes the exact bytes; a tampered word, a wrong `K`, a `/13` without
   the record and a `/13` carrying another optional record are refused.
3. A plain `/12` export is unchanged.
4. `cargo test -p uor-r4-integer --features serving-wasm` focused tests pass, and formatting,
   touched-file Clippy, `git diff --check`, `uor-harness check --only
   harness.native_space_contract`, the REPORT check and the exact-head review pass.

## Cancel

Stop if the layout cannot be exact, if the schema would let an older engine drop gates silently, or
if the slice needs a serving or kernel change. Do not start the serving wiring here.

STOP and report when done.
