# ORDER: one-model-end-to-end, deepseek-275 (timed place-read parity)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `93e089ff256654e75a19641b7e2fbd2bc8e24524`.

## Goal

Exercise the integer session's timed FBS step with an armed, opening and
stepping place register, and compare it with the normal step. This test-only
unit changes no serving behaviour and does not touch the stage-3 pin or pod.

## Done when

1. One test drives the same `PLACE_TALK` rows through normal and timed sessions.
2. Every `NativeStep`, including query and kept cells, is equal at every row.
3. `last_place()` is equal at every row and pins arm, open, step and stop.
4. The test fails if the timed path omits the place offer.

## Cancel if

- normal and timed steps disagree; report the first differing row as a serving
  defect rather than weakening the comparison;
- a production file other than the test file must change.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-275/**`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `docs/research/notebook/float-training.md`

Size budget: 120 changed lines.

## Stop and report

Stop when the test and negative gate pass. Do not start a GPU run, change the
trainer, or modify Claude's running stage-3 work.
