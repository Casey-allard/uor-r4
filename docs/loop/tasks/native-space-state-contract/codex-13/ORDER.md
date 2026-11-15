# ORDER: native-space-state-contract, codex-13

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Reviewed head: `e0bb6af7a`.

## Why this unit exists

The codex-12 decision chose **per-lane gates, shared across layers** as the next one-space
configuration and named the `/13` state. This is slice 1 of three: the native-vector state type
itself, before the artifact schema and serving wiring. It is the foundation the training and
serving slices build on.

## Claim record

`loop claim native-space-state-contract --lab-topic q31-lane-gates` was refused with exit 2
because `Cargo.lock` in this item's scope overlaps PR #3983. This slice adds no dependency, does
not touch `Cargo.lock`, and continues under the owner's D85 direction.

## State schema

- `Q31LaneGates` is `K` quaternions (four little-endian Q31 words each) plus the shared cursor:
  `epoch: u64`, `step_shift: u32`, version `1`, three zero bytes. The exact bytes are
  `32 K + 16`, so `K = 1` is 48 bytes and the synthetic `K = 8` is 272 bytes.
- The version and padding are checked on load; every gate is checked against the closed Q31
  domain through `Q31VectorState`; the step shift is `0..=31`; a length that is not
  `32 K + 16` is refused.
- There is one cursor, not one per lane: the type owns the shared `(epoch, step_shift)`.

## Shared operators and implementations

- The transition is the existing `q31_hamilton_rotate(gate, value)`; `transition(lane, value)`
  selects the lane's gate and is the only new dispatch. No arithmetic is added.
- The AOT CUDA kernel and the artifact schema are unchanged in this slice.

## Replay

A finite fixture builds `K` distinct gates and a cursor, round-trips the exact bytes, and requires
each lane's transition to equal `q31_hamilton_rotate` at that lane's gate. A flipped gate word, a
wrong length, a wrong version, nonzero padding and an out-of-domain gate are refused.

## Files in scope

- `crates/uor-r4-native-vector/src/lib.rs`
- `docs/loop/tasks/native-space-state-contract/codex-13/**`

Out of scope: `Cargo.lock`, `Cargo.toml`, the artifact schema/serving (the next slices), the CUDA
kernel, any GPU run.

## Size

About 150 to 250 lines of type, tests and records.

## Done when

1. `Q31LaneGates` serializes and restores the exact `32 K + 16` bytes, with the shared cursor.
2. `transition(lane, value)` equals the shared operator at that lane's gate; an out-of-range lane
   is refused.
3. Malformed length, version, padding, shift and gate domain are refused.
4. `cargo test -p uor-r4-native-vector` passes, and formatting, touched-file Clippy,
   `git diff --check`, `uor-harness check --only harness.native_space_contract`, the REPORT check
   and the exact-head review pass.

## Cancel

Stop if the layout cannot stay exact and closed-domain, if per-lane dispatch would add a second
arithmetic definition, or if the slice needs an artifact/serving change. Do not start the
artifact or serving slice here.

STOP and report when done.
