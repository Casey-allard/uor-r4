# REPORT: infra-loop-harness, kimi-34 (record the position premise probe)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).

References #2904

## What did not work, first

- PLAN kept offering an already-answered finite-control probe. Repeating it would add no information.
- The 10/10 result is not evidence that position improves order or language. Actual CUDA/PTX/device validation and the registered paired order experiment remain pending; #3111 stays draft and all holds remain unchanged.
- No other premise is closed by this unit.

## Cancel probe

At `742bf59bc`, the position premise was still `open`. The selected command had passed 10/10 at `b03d08e85`, recorded on #2843 (`6380719988`). The PM status record stands.

## What changed

- PLAN marks only the position finite-control premise `closed`, dates the answer, and retains the pending registered order test and cancellation condition.
- D61 records the evidence, exact non-claims, rejected alternatives and reopening cost.

## Measurements

`cargo test -p uor-r4-discrete --lib position_mechanism`: **10 passed, 0 failed, 240 filtered out**. Host-only finite controls; no GPU training, evaluation or model work.

## Checks run at the head

`loop next`, `loop report --check`, `loop pr-check --local`, `loop gates --full`, and `git diff --check`.

## Skills used

`uor-research-architect` for exact claim scope and the loop delivery path; `superpowers:verification-before-completion` for fresh evidence before status claims.

## Still open

The registered H_pos versus matched endpoint + shuffled-null experiment, actual CUDA/PTX/device controls, and every hold are unchanged. No item or milestone is marked done.

## Plain summary

The finite position tests have answered their probe, so the plan will offer new work instead of repeating them. This records only that finite result; it does not claim that the position mechanism helps a model.
