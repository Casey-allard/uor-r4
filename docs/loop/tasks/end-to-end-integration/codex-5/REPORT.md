# REPORT: end-to-end-integration, codex-5

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic q31-real-states` was refused because the item is
  still claimed by this lab's codex-3 worktree (PR #4015, queued). The claim is this lab's own; the
  unit adds one test file and docs, no dependency and no `Cargo.lock` change.
- **The c387 states are not local.** The HF store has no materialised snapshot here, and Claude is
  running its read training on wick. This unit therefore uses the committed `/7` artifact from the
  2026-11-04 CPU content-serving pilot: **real trained weights and real recurrence carries**, but
  not the c387 states and not the current p recipe.
- **The token ids are artificial.** No tokenizer is bound to that `/7` artifact, so the 32 ids are a
  fixed deterministic sequence in its 4,096-entry vocabulary. Real model, artificial inputs.
- **The target is the model's own next `query_q12` block**, not a gold next token (no reference
  tokens exist for these ids), so there is no quality measure here — only the update's L1 and
  whether a served token moves.
- **The first run had two measurement faults.** The target lift was `<< 16`, which places the largest
  queries (`2^15`) exactly on the Q31 boundary and compresses the scale, and the "lanes improved"
  count reported 0 of 64 while the total fell. Both are fixed: the lift is `<< 14` (Q12 -> Q26,
  four times the headroom), and the count compares each lane's before and after. The first table is discarded.
- **The L1 drops are tiny** (~0.15% shared, ~0.20% per-lane). Per-lane fitting its own objective
  slightly better is not a capability result.
- **One served token moved under the per-lane state** (position 17, `466` -> `82`); with no
  reference that is movement, not improvement. The shared state moved none.

## What changed

- `crates/uor-r4-integer/tests/q31_lane_objective_real.rs` (new, test-only): parses the committed
  `/7` artifact, attaches an identity `/13` state, traces 6 x 64 carries and the per-step queries on
  the fixed ids, trains the shared gate and 64 per-lane gates with the shipped `step_reference`, and
  serves the same ids under both trained states.

Size: about 250 lines of test and the ORDER/REPORT.

## Measurements

- Real artifact: width 256, six recurrence layers, K = 64; first served tokens
  `[290, 1787, 269, 3105, 1, 2772, 25, 802, …]`; max `|query_q12|` `32768`.
- Records: 31 transitions x 6 layers x 64 lanes.
- Shared gate: L1 `72,944,053,922,656 -> 72,834,350,400,872` (-0.15%).
- Per-lane gates: L1 `72,944,053,922,656 -> 72,798,947,235,786` (-0.20%); **64 of 64** lanes lower
  their own L1.
- Served tokens: identity and shared unchanged; per-lane differs at one of 32 positions.
- Runtime 2.7 s; no float, no exponent, no GPU.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --test q31_lane_objective_real` —
  `1` passed.
- `rustfmt --check` on the file — clean.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --test
  q31_lane_objective_real` — no warning in the file.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the first table was rejected for a real reason (target saturation and a
  wrong count), and the result is stated as movement on artificial ids, not a capability gain.
- `scope-guard`: one test file and docs; no source, schema, dependency or artifact change.
- `test-freeze`: a new file; no existing test changed.

## Still open

- The c387 states / D41 pilot run, when they are accessible and Claude's run is not in the way.
- Real token ids for the `/7` artifact (a bound tokenizer), and a reference next token for a
  quality measure.
- The read-averaging question: the note's diagnosis says the read loses the fact it found; this unit
  does not test that, and should not be read as doing so.
- The `l = 2, 3` ladder and the planted-symmetry probe (#4018) are the other live codex threads.

## Plain summary

On a real trained `/7` artifact, the shipped Q31 update lowers its own L1 in both configurations,
per-lane slightly more than shared, and all 64 lanes improve. The per-lane state moves one served
token; the shared state moves none. The ids are artificial and there is no reference, so this is
movement on real weights, not a capability result.
