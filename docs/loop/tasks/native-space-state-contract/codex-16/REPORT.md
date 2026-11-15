# REPORT: native-space-state-contract, codex-16

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim native-space-state-contract --lab-topic q31-lane-objective` was refused with exit 2
  because `Cargo.lock` in this item's scope overlaps claude-384's PR #3983. This unit adds no
  dependency, does not touch `Cargo.lock`, and continued under the owner's D85 direction.
- **The ordered measurement found a serving defect, not a gate limit.** On the unfixed code the
  per-lane run reported the trained state did not reach serving at all: a 90° quarter-turn on every
  lane's gate left **0 of 256** served query words, the served tokens and the next-token ranks
  identical, and only **7 of 128** lane carries changed. Tracing one step showed a different
  `transport` with an identical carry `after`.
- **The defect:** `stack_native_q31_lanes` computed `transport (Q31) * decay (Q31)` and shifted the
  product right by 31, then shifted the sum right by 31 again. The carried term was divided by
  `2^31` twice and lost; only the gate-independent drive survived. The native Q31 state was never
  served, in any gate configuration, since the path was introduced.
- **Two earlier conclusions are therefore withdrawn.** The codex-6 and codex-7 negatives (and the
  "alignment fell, tokens unchanged" result) measured a dead path; codex-12's finding that the
  single shared 4-word gate is the structural limit is falsified — the limit was this double shift.
- **After the fix the per-lane configuration is still negative**, now on a live path: alignment
  fell, 3 of 8 lanes improved, served tokens unchanged, and the token rank got slightly worse. So
  per-lane gates (A) do not rescue the objective, but this is the first measurement that says
  anything about gates at all.
- The existing Q31 tests did not catch the defect: they compared a transport to its oracle and
  tokens across engines, never that a *different* state produces a different served result.
- The plain (non-Q31) path is **not** a baseline for the Q31 path: attaching a Q31 state replaces
  the trunk's learned rotations, so the identity gate is a different operator from the plain
  recurrence. The quarter-turn comparison isolates the gate correctly.
- The new fixture's token-level measure has no signal to improve on this synthetic trunk: the mean
  next-token rank is near chance (123.57 of 256) before training, so a token-level *rank* cannot
  become good here. It can only show movement or its absence.

## What changed

- `crates/uor-r4-integer/src/stack/session.rs`, `stack_native_q31_lanes`: the transport-times-decay
  product stays at Q62 (the exponent the ordinary recurrence's `stack_hamilton` output is at) and
  the one shift returns the sum to Q31. Three lines plus the comment; no other term, scale or path.
- `crates/uor-r4-integer/tests/q31_lane_objective.rs` (new): the per-lane objective with the
  token-level co-primary, a one-lane isolation test, and the quarter-turn regression that fails on
  the pre-fix code (`the trained state never reached the writer query`) and passes after.

Size: about 330 lines of test, 4 lines of source, and the ORDER/REPORT.

## Measurements

Same fixture as both earlier negatives: `synthetic_fbs_trunk(20_261_115)`, tokens
`[3, 5, 8, 13, 21, 34, 55, 89]`, 64 training steps per lane.

| Quantity | Pre-fix | Post-fix |
|---|---|---|
| Per-lane alignment total | `2,476,676,547,100,606,464 → 2,142,746,950,279,430,144` | `3,673,307,283,483,328,512 → 3,236,691,098,920,681,472` |
| Lanes with a higher alignment | 3 of 8 | 3 of 8 |
| Lane carries differing (start vs trained state) | 7 of 128 | 96 of 128 |
| Writer queries identical | yes | no |
| Served tokens | `[125, 236, 10, 186, 8, 161, 92, 205]` unchanged | `[125, 236, 138, 207, 8, 161, 92, 205]` unchanged |
| Next-token rank | `[109, 168, 246, 168, 186, 17, 6]` unchanged, mean 128.57 | `[109, 180, 231, 205, 123, 12, 5] → [109, 178, 231, 206, 123, 18, 6]`, mean 123.57 → 124.43 |
| Ranks improved | 0 of 7 | 1 of 7 |
| Quarter turn: differing query words | 0 of 256 | 224 of 256 |
| Quarter turn: served tokens | identical | `[125, 236, 138, 207, 8, 161, 92, 205]` → `[125, 236, 3, 207, 8, 49, 92, 205]` |

The trained `/13` state round-trips through the exporter and parse; the served gates equal the
trained gates; training lane 0 changes only lane 0's gate. No language, capability or GPU claim.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  q31_lane_objective` — `3` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm q31_serving` — `12` passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` — `663` passed, `1`
  ignored.
- `q31_token_objective`, `q31_training_to_serving`, `q31_artifact_fresh_process` — pass;
  `q31_gpu_state_to_serving` ignored as before (it needs a GPU state file).
- The quarter-turn regression was run against the pre-fix code and failed with "the trained state
  never reached the writer query", then passed on the fixed code.
- `rustfmt --check` on the new test file — clean (the crate at large is not format-clean; no
  pre-existing file was reformatted).
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` — no
  warning in the added lines (the full crate still fails on the same pre-existing deny lints).
- `git diff --check` — passed.

## Skills used

- `native-space-cycle`: the same state and scales; the fix restores the exponent the ordinary
  recurrence uses rather than introducing a new one.
- `solution-integrity` / `trace-back`: the surprising "state changes nothing" number was traced to
  the arithmetic before any gate conclusion was drawn; one fixture step was traced by eye, and the
  fix was proven to fail the new regression before it was accepted.
- `scope-guard`: one arithmetic site and a new test file; no pre-existing file was reformatted and
  no dependency changed.
- `test-freeze`: a new test file; no existing test was edited.

## Still open

- **Re-open the earlier negatives as evidence.** codex-6/codex-7 must be re-run on the fixed path
  before any conclusion about the single shared gate, and codex-12's A/B/C order should be revisited
  with the live measurements.
- The per-lane configuration (A) is still negative after the fix: 5 of 8 lanes' alignment fell and
  the token rank worsened slightly. The next configuration (B, per-layer-lane gates) is now worth
  measuring, on the fixed path.
- A token-level measure with a real signal needs a trained trunk (the V11 state or the D86 donor),
  not the random synthetic writer; that is the GPU/FIT path, which must not compete with the
  another session's donor pilot on wick.
- The `uor-r4-lut` facade still refuses `/13`.

## Plain summary

The ordered per-lane measurement found that the native Q31 state never reached the recurrence: a
double right shift divided the trained term out of every carry, so no gate configuration could
change served output. The one-site fix restores the term, and the new regression proves a quarter
turn now changes the carried state, the writer query and the tokens. With the path live, per-lane
gates are still negative (alignment falls, 5 of 8 lanes worse, tokens unchanged, rank slightly
worse) — the first measurement that actually tests gates.
