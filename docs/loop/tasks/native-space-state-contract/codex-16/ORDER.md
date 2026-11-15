# ORDER: native-space-state-contract, codex-16

Lab: codex. Item: [native-space-state-contract](../../../../roadmap/items/native-space-state-contract.md).
Parent: [codex-15](../codex-15/ORDER.md). Reviewed head: `62853d976`.

## Why this unit exists

The one-space token objective was negative twice with one shared gate (codex-6, codex-7) and
codex-12 concluded that the single shared 4-word gate was the structural limit, choosing per-lane
gates. This unit was ordered to measure that per-lane objective with a token-level co-primary
measure, so a positive alignment alone could not be read as a serving win.

## What the ordered measurement found, before any fix

The ordered run (same fixture as both negatives: `synthetic_fbs_trunk(20_261_115)`, tokens
`[3, 5, 8, 13, 21, 34, 55, 89]`) reported that the trained state did not reach serving: a 90°
quarter-turn on every lane's gate left **every** served query word, every served token and every
next-token rank identical. Tracing one step showed why: the lane's `transport` differed completely
between the two gates while the carry `after` was identical. `stack_native_q31_lanes` multiplied
the Q31 transport by the Q31 decay and shifted the product right by 31, then shifted the sum right
by 31 again. The carried term was divided by `2^31` twice and lost; only the gate-independent drive
survived. The native Q31 state was therefore never served, in any gate configuration, so both
earlier negatives measured a dead path and codex-12's "structural limit" conclusion is falsified.

## What this unit does

1. **Fix:** `stack_native_q31_lanes` keeps the transport-times-decay product at Q62, the exponent
   of the ordinary recurrence's `stack_hamilton` output, and applies the one shift that returns the
   sum to Q31. No other term or scale changes.
2. **Regression:** a permanent test proves a quarter turn on every lane changes at least one carry
   `after` and at least one served query word, and reports the token and rank movement.
3. **Re-measure:** the per-lane objective on the fixed path, with the same fixture and the same
   token-level co-primary (next-token rank under the served `query_q12`, and served tokens).
4. **Record:** the REPORT states the defect, the fix and the re-measured signs; it also records that
   the earlier negatives are invalid as evidence about gates.

## Claim record

`loop claim native-space-state-contract --lab-topic q31-lane-objective` was refused with exit 2
because `Cargo.lock` in this item's scope overlaps claude-384's PR #3983. This unit adds no
dependency and does not touch `Cargo.lock`; it continues under the owner's D85 direction.

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs` (`stack_native_q31_lanes` only)
- `crates/uor-r4-integer/tests/q31_lane_objective.rs` (new file)
- `docs/loop/tasks/native-space-state-contract/codex-16/**`

Out of scope: `Cargo.toml`, `Cargo.lock`, the facade, the model crate, CUDA/GPU, and any claim
about language, capability or model quality.

## Done when

1. The quarter-turn regression fails on the pre-fix code and passes on the fixed code.
2. Every existing Q31 test still passes: `q31_serving`, `q31_token_objective`,
   `q31_training_to_serving`, `q31_artifact_fresh_process`, and the whole library.
3. The per-lane run reports per-lane and total alignment, served tokens and next-token rank before
   → after, on the fixed path; the sign of every quantity is in the REPORT, negative first.
4. The trained `/13` state round-trips, the served gates equal the trained gates, and training one
   lane changes only that lane's gate.
5. Formatting, touched-file Clippy, `git diff --check`, `uor-harness check --only
   harness.native_space_contract`, the REPORT check and the exact-head review pass.

## Cancel

Stop if the fix needs a change outside the one arithmetic site, a dependency, a GPU, or if the
existing Q31 suites cannot be made to pass without weakening a test. Do not tune the fixture after
seeing the result.

STOP and report when done.
