# REPORT: native-space-state-contract, codex-14

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim native-space-state-contract --lab-topic q31-lanes-artifact` was refused with exit 2
  because `Cargo.lock` in this item's scope overlaps PR #3983. This slice adds no dependency, does
  not touch `Cargo.lock`, and continued under the owner's D85 direction.
- This is **slice 2 of three**: the artifact schema/parser/exporter. The serving wiring is not
  done, so `IntegerStackModel` does not attach or serve a `/13` state yet; a `/13` artifact parses
  and inspects, and a fresh `uor-chat` would still use the `/12` path only.
- The container accessor carries `#[allow(dead_code)]` because only the next (serving) slice calls
  it; the container test exercises it now.
- The `/13` record's exclusivity from the single-gate `q31` record is enforced in `parse`, and the
  test covers the no-record case; a `/13` artifact carrying a hamming/memory co-record is not
  fixture-tested because the synthetic trunk has none, so that branch is covered by code and the
  reviewer's read rather than a fixture.
- Full-crate `cargo clippy --lib --tests` exits 101 on two pre-existing deny-level lints
  (`place_asks.rs:1421`, `tests/openai_loopback.rs:1658`) plus many pre-existing warnings, none in
  this diff; the touched files are clean under Clippy and formatting, which is the unit's
  touched-file bar.

## What changed

- `crates/uor-r4-integer/src/stack/format.rs`:
  - constants `STACK_FBS_Q31_LANES_SCHEMA = "uor-r4.lut-stack/13"` and
    `Q31_LANES_TABLE = "q31.gates"`;
  - `StackQ31Lanes` (`state`, `sha256`, `lanes`, `epoch`, `step_shift`, `version`) and an optional
    `Header.q31_lanes` / `Container.q31_lanes` / metadata field;
  - the schema expectation gives `/13` precedence, and `parse` validates the table name, kind and
    `8K + 4` length, `K = width / 4`, the exact `32K + 16` bytes through
    `Q31LaneGates::from_bytes`, the digest, the shared cursor and the version, and refuses a `/13`
    carrying another optional record;
  - `Container::q31_lane_state_bytes` returns the exact bytes after the parse validation;
  - `export_fbs_trunk_uorlut01_with_lane_gates` validates the state, writes it as one `u32` table,
    sets `/13` and the record, and leaves plain exports byte-identical (the key and table are
    written only when a state is passed).
- `crates/uor-r4-integer/src/stack/mod.rs` re-exports the new names and the exporter.
- One test in `q31_serving.rs` covers the K=8 round-trip (schema, record, exact bytes), per-lane
  replay against `q31_hamilton_rotate`, a flipped word refused by the digest, a wrong K refused by
  the exporter, a `/13` header without the record refused, and a plain export with no lane record.

Size: about 280 lines of format/export/mod/test and the ORDER/REPORT.

## Measurements

- `cargo test -p uor-r4-integer --features serving-wasm q31_serving` — `9` passed (8 before plus
  the lane-gate artifact test).
- `cargo test -p uor-r4-integer --features serving-wasm --lib` — `652` passed, `1` ignored.
- The `/13` artifact round-trips the exact `32K + 16 = 272` bytes for `K = 8`, with epoch `5` and
  shift `8`; each lane's reconstructed transition equals the shared operator at that gate.
- No model, capability or GPU result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm q31_serving` — `9` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` — `652` passed, `1`
  ignored.
- `cargo fmt --check -p uor-r4-integer` — the touched files are clean (other pre-existing files
  are not part of this diff).
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` —
  exits 101 on two pre-existing deny-level lints and many pre-existing warnings outside this diff;
  no warning in the added lines, and the touched files are clean.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `native-space-cycle`: the same state and operator; the artifact packs the exact bytes with no
  conversion.
- `solution-integrity`: validated the table, digest, cursor and domain at load; checked the
  plain-export unchanged path.
- `scope-guard`: stayed in the integer stack and did not touch `Cargo.lock` or the open order's
  paths.
- `test-freeze`: added a test inside the source file; no existing test changed.

## Still open

- **Slice 3:** `Q31LaneServingSession`, `IntegerStackModel` attachment, and the recurrence's
  lane-selected transition, with a bit-identical replay and a planted fault (one lane's gate
  changed).
- Then the per-lane token objective (a token-level co-primary measure), and a GPU qualification
  with a sheet/FIT if positive; otherwise per-layer-lane gates (B), then a different objective (C).

## Plain summary

A served file can now carry the per-lane state: a new `/13` schema holds K gates and one shared
cursor in exact bytes, the parser validates it and returns the bytes, and the exporter writes it
without changing a plain file. Serving does not use it yet; that wiring is the next slice.
