# ORDER: m4-export-uorlut01-cli, codex-6

Lab: codex. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Parent: [codex-4](../codex-4/ORDER.md). Reviewed head: `a8cf8e36dd7a0f56df391d80ffbec5a2f907cbff`.

## Why this unit exists

The one-space path has a state, an operator, an artifact and a serving path, but no **token
objective**: every training step so far used synthetic value/target records. This unit defines the
first finite objective that comes from tokens. It is host-only and makes no language claim; it
prepares the pre-registration for a GPU run.

## Claim record

`loop claim m4-export-uorlut01-cli --lab-topic q31-token-objective` was refused with exit 2 because
PR #3983 claims overlapping `crates/uor-r4-integer/src/discrete_d11.rs` scope. This unit adds one
test and records, and does not touch that file.

## Objective

- Tokens map to the synthetic trunk's writer rows: token `t`'s row is `embed[t*width .. +width]`,
  quantized to Q12 as the exporter does and mapped into Q31 (`q12 << 16`, the writer's ±8 range
  into ±1), eight 4-word blocks.
- At each transition `i`, the recurrence's lane carries `before` are the values; the next token
  `seq[i+1]`'s block for the same lane is the target.
- The shared oracle `Q31HamiltonModel::step_reference` trains the gate on those records.
- The independent measure is the integer dot `rotate(gate, before) · target` summed over the
  records, plus the served token ids on the fixed sequence before and after training.
- No new operator, no softmax, no dense product, no float in the serving path.

## Fixture and replay

Fixed synthetic trunk seed `20_261_115` (vocabulary 256, width 32, two recurrence layers, context
8) and the token sequence `[3, 5, 8, 13, 21, 34, 55, 89]`. The test:

1. exports the initial gate `[1 << 31, 0, 0, 0]` as a `/12` artifact and traces the sequence;
2. builds the token targets from the trunk's writer rows and trains the gate for 64 steps;
3. exports the trained state, requires the artifact to carry exactly it, and traces again; and
4. reports the alignment dot before and after training and the served token ids. A change in the
   served tokens is reported, not required: the first run may be a negative, and a negative closes
   this configuration only.

## Files in scope

- `crates/uor-r4-integer/tests/q31_token_objective.rs`
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/m4-export-uorlut01-cli/codex-6/**`

Out of scope: `crates/uor-r4-integer/src/discrete_d11.rs`, any production or format change, any
GPU run or language claim.

## Size

About 200 to 300 lines of test and records.

## Done when

1. The test builds token targets from the writer rows and trains the gate on the traced carries.
2. The artifact carries exactly the trained 48 bytes; the served token ids before and after are
   reported.
3. The alignment dot is reported; a positive or a negative result is recorded honestly.
4. The focused Q31 tests still pass, and formatting, touched-file Clippy, `git diff --check`,
   `uor-harness check --only harness.native_space_contract`, the REPORT check and the exact-head
   review pass.

## Cancel

Stop if the serving trace cannot supply the lane carries, if the token target requires a new
operator or a float conversion in serving, or if the objective cannot be measured with raw integer
words. Do not start a GPU training run.

STOP and report when done.
