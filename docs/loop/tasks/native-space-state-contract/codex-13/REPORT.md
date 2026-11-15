# REPORT: native-space-state-contract, codex-13

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim native-space-state-contract --lab-topic q31-lane-gates` was refused with exit 2
  because `Cargo.lock` in this item's scope overlaps PR #3983. This slice adds no dependency and
  does not touch `Cargo.lock`; it continued under the owner's D85 direction.
- This is **slice 1 of three** from the codex-12 decision: the native-vector state type only. The
  `/13` artifact schema/parser/exporter and the serving wiring are the next slices; no artifact or
  serving behaviour changed here.
- A first compile failed with a mutable/immutable borrow in the lane-lookup error closures; the
  lane count is captured before the lookup.
- The type lives in the crate's inline test module, so the change is a library and its tests, not
  a separate test file.

## What changed

- `crates/uor-r4-native-vector/src/lib.rs` gains `Q31LaneGates`: `K` admitted Q31 quaternions plus
  one shared `(epoch, step_shift)` cursor.
  - Exact bytes `32 K + 16` (version 1 and three zero padding bytes at the end); `K = 1` is 48
    bytes, the synthetic `K = 8` is 272.
  - `new`, `lanes`, `gate`, `gates`, `set_gate`, `set_cursor`, `epoch`, `step_shift`,
    `to_bytes`, `from_bytes`, and `transition(lane, value)`.
  - The transition is the existing `q31_hamilton_rotate` at the lane's gate; no new arithmetic.
  - `from_bytes` refuses a length that is not `32 K + 16`, an unknown version, nonzero padding, a
    shift above 31, and any out-of-domain gate.
- Two tests: the exact round-trip and per-lane dispatch (each lane's transition equals the shared
  operator at that gate, an out-of-range lane is refused), and the malformed-length/version/
  padding/shift/domain refusals plus an empty-lane refusal.

Size: about 210 lines of library, tests and the ORDER/REPORT.

## Measurements

- `cargo test -p uor-r4-native-vector` — `25` passed (23 before, plus the two lane-gate tests).
- The round-trip is byte-exact for `K = 3` (112 bytes) with epoch `3` and shift `8`.
- `transition(0, v)` and `transition(1, v)` equal `q31_hamilton_rotate` at gates 0 and 1
  respectively; lane 3 is refused.
- A flipped gate word, a bad version, nonzero padding, a 32 shift, a 47-byte state and an
  out-of-domain gate are all refused.
- No model, capability or GPU result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-native-vector` — `25` passed.
- `cargo fmt --check -p uor-r4-native-vector` — passed.
- `cargo clippy --locked --no-deps -p uor-r4-native-vector --lib --tests` — clean in the touched
  file.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: one representation, the same typed operator selected by lane, exact bytes
  and lossless serialization; the cursor stays one shared value.
- `scope-guard`: stayed in the native-vector crate and did not touch `Cargo.lock` or the open
  order's paths.
- `solution-integrity`: refused every malformed state at load, including the domain, version and
  padding, and tested each refusal.
- `test-freeze`: added tests inside the library; no existing test changed.

## Still open

- **Slice 2:** the `uor-r4.lut-stack/13` schema, parser, exporter and `q31_lanes` record with K
  gates, a bit-identical replay fixture and a planted fault (one lane's gate changed).
- **Slice 3:** serving wiring (`Q31LaneServingSession`, `IntegerStackModel` attachment, the
  recurrence's lane-selected transition) and the per-lane token objective.
- If the per-lane dot is positive, the GPU qualification with a token-level co-primary measure and
  a sheet/FIT; otherwise per-layer-lane gates (B), then a different objective (C).

## Plain summary

The one-space state now has a per-lane form: K quaternions, one shared cursor, exact bytes, and a
dispatch that uses the same Hamilton operator at each lane's gate. Malformed states are refused.
The artifact and serving still use the single-gate `/12` path until the next slices land.
