# REPORT: native-space-state-contract, codex-15

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim native-space-state-contract --lab-topic q31-lanes-serving` was refused with exit 2
  because `Cargo.lock` in this item's scope overlaps claude-390's PR #4006. This slice adds no
  dependency, does not touch `Cargo.lock`, and continued under the owner's D85 direction.
- The ORDER's first bit-identical replay plan — a `K = 1` `/13` state whose bytes equal the `/12`
  state — **cannot occur on a served model**: `StackShape::validate` requires `width` to be a
  multiple of 32, so `K = width / 4` is a multiple of 8. The test and the ORDER now use the
  reachable case: a `/13` state whose `K` gates all equal the one trained `/12` gate.
- A one-ulp gate change is invisible to the integer transport: `q31_hamilton_rotate` scales by
  `2^-31`, so changing a gate coordinate by `1` moves a result by less than one integer. The
  planted fault uses `1 << 24`, above one Q31 ulp, and is still inside the closed domain.
- The first carry of a fresh session is zero and every gate rotates zero to zero, so a changed
  gate cannot show on the first step. The fault test steps once untraced, then traces the step
  where the gate must appear.
- **`/13` is still refused by the `uor-r4-lut` facade** (`crates/uor-r4-lut/src/format.rs` knows
  `/1`, `/2`, `/12` only). That is the next slice, so an engine built on the facade — including
  `uor_r4_lut::stack::StackModel` used by some older report binaries — cannot read a `/13`
  artifact yet. The D11 engine (`IntegerStackModel`, and therefore `uor-chat`) can.
- No model, capability or GPU result is measured. This slice proves wiring and exactness, not
  quality; the per-lane token objective is still the open question.
- Clippy's full-crate `--lib --tests` run still exits 101 on the same two pre-existing deny lints
  (`place_asks.rs:1421`, `tests/openai_loopback.rs:1658`) plus pre-existing warnings; none are in
  this diff and the touched files are clean.

## What changed

- `crates/uor-r4-integer/src/stack/q31_serving.rs`:
  - `Q31LaneServingSession`: restores the exact `32K + 16` bytes through
    `Q31LaneGates::from_bytes`, returns them losslessly, and exposes `lanes`, `gate`,
    `epoch`, `step_shift` and `transition(lane, value)`. No new arithmetic.
  - tests: per-lane replay against the shipped operator, the repeated-gate exact replay against
    `/12`, and the planted one-lane fault plus the refusals.
- `crates/uor-r4-integer/src/stack/session.rs`:
  - `IntegerStackModel.native_q31_lanes: Option<Q31LaneServingSession>`; `parse` attaches a `/13`
    container's state automatically, with no side-load.
  - `attach_native_q31_lane_state` refuses a model without a recurrence layer, a model that
    already holds the single-gate state, and `K != shape.lanes()`; it binds the serving digest to
    `uor-r4.native-q31-lanes-serving/1`, the model digest, `K` and the exact bytes.
  - `attach_native_q31_state` now also refuses when the per-lane state is attached.
  - `native_q31_lane_state` / `native_q31_lane_state_bytes` accessors.
  - `fbs_image`, `fbs_trunk_image`, the snap-trace refusal, `set_native_q31_trace` and
    `native_q31_step_preflight` treat the lane state like the single state.
  - `stack_native_q31_lanes` takes both states and selects `transition(lane, value)` for `/13`
    and the shared `transition(value)` for `/12`; the carry trace is unchanged.
- `crates/uor-r4-integer/src/stack/mod.rs` re-exports `Q31LaneServingSession`.

Size: about 250 lines of source, tests and docs.

## Measurements

- `cargo test -p uor-r4-integer --features serving-wasm q31_serving` — `12` passed (9 before plus
  3).
- `cargo test -p uor-r4-integer --features serving-wasm --lib` — `655` passed, `1` ignored.
- A `/13` K=8 model serves each recurrence lane through
  `Q31LaneGates::transition(lane, before)` for all four tokens and both layers (16 carries per
  step, each equal to the shipped operator at that lane's gate).
- A `/13` artifact whose 8 gates all equal the trained `/12` gate serves the same four tokens and
  the same 16 carry records per step as the `/12` artifact on the same trunk.
- Changing lane 3's gate changes only layer 0 lane 3's transport on the traced step; the other
  seven layer-0 lanes are unchanged.
- No token, capability or GPU number is claimed.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm q31_serving` — `12` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` — `655` passed, `1`
  ignored.
- `cargo fmt -p uor-r4-integer -- --check` — no diff in the touched files.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` —
  no warning in the added lines (the full crate fails on two pre-existing deny lints).
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the
  head.

## Skills used

- `native-space-cycle`: the state type, its bytes and its operator are the shipped ones; serving
  only selects the gate by lane.
- `solution-integrity`: checked the attach/edit order, the exclusivity of the two states, and the
  exact-replay claim before trusting the test.
- `scope-guard`: stayed in the integer stack; no facade, kernel, dependency or `/12` change.
- `test-freeze`: added tests only; no existing test changed.

## Still open

- The `uor-r4-lut` facade `/13` support.
- The per-lane token objective with a token-level co-primary measure, then a GPU qualification
  under a sheet and non-owner FIT if positive; otherwise per-layer-lane gates (B), then a
  different objective (C).
- A `/13` artifact's CLI end-to-end run (fresh `uor-chat` process) is not separately exercised
  here; it uses the same `IntegerStackModel::parse` path the tests cover.

## Plain summary

A served file can now carry one gate per lane, and the real recurrence uses it: the model attaches
the `/13` state at load, and lane `i` advances through gate `i` instead of the one shared gate.
`/12` is untouched, and an all-equal-gate `/13` reproduces the `/12` serving exactly. The facade
and the quality question are the next slices.
