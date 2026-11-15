# ORDER: m4-export-uorlut01-cli, codex-7

Lab: codex. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Parent: [codex-6](../codex-6/ORDER.md). Reviewed head: `e5a32f458`.

## Why this unit exists

codex-6 defined the first token objective and it was a negative on that configuration: the raw
writer target (`q12 << 16`) lowered the independent alignment dot and left the served tokens
unchanged. This unit tests the next configuration named there: a **normalized unit-scale writer
target**, on the same records, the same update and the same independent integer dot. A positive
unlocks the pre-registration for a GPU run; a negative closes this second configuration only.

## Claim record

`loop claim m4-export-uorlut01-cli --lab-topic q31-token-normalized` was refused with exit 2
because PR #3983 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. This unit
extends one test and the records; it does not touch that file.

## Objective

- Values and record construction are unchanged from codex-6: the real recurrence's `before`
  carries at each transition, against the next token's writer block for the matching lane.
- The target is normalized per block to unit Q31 scale: `q12[j] * 2^31 / max_abs(q12)`, so the
  largest coordinate is ±2^31 and the direction is preserved. The block's magnitude no longer
  enters the update.
- The shared oracle `Q31HamiltonModel::step_reference` trains the same 64 steps on the same 112
  records.
- The independent measure is the same integer dot `rotate(gate, before) · target` and the served
  token ids before and after.

## Fixture and replay

Seed `20_261_115`, vocabulary 256, width 32, two recurrence layers, context 8; sequence
`[3, 5, 8, 13, 21, 34, 55, 89]`. The new test exports the initial gate, traces, builds the
normalized targets, trains, exports the trained state, and reports the dot and tokens. It asserts
only the mechanics (state changed, artifact carries the bytes, alignment moved); the sign of the
delta is the result, recorded honestly.

## Files in scope

- `crates/uor-r4-integer/tests/q31_token_objective.rs` (adds one test; the raw-target test is
  unchanged)
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/codex-7/**`

Out of scope: `crates/uor-r4-integer/src/discrete_d11.rs`, any production or format change, any
GPU run or language claim.

## Size

About 80 to 150 lines of test and records.

## Done when

1. The normalized-target test trains the gate, and the artifact carries exactly the trained bytes.
2. The alignment dot and served tokens are reported before and after.
3. The notebook records the result as positive or negative for this configuration, with the next
   configuration named if negative.
4. The raw-target test still passes, and formatting, touched-file Clippy, `git diff --check`,
   `uor-harness check --only harness.native_space_contract`, `test-freeze` (the test-modifying
   commit names the change), the REPORT check and the exact-head review pass.

## Cancel

Stop if normalization leaves the Q31 domain, if the test cannot compare against the same
independent dot, or if the result would need a new operator. Do not start a GPU run.

STOP and report when done.
