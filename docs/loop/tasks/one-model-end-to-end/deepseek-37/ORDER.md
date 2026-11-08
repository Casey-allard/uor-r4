# ORDER: one-model-end-to-end, deepseek-37 (repair opening-weight evidence)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `a3ac3f326`.

## Goal

Repair the exact-head review findings on PR #3857 without changing the
opening-weight result. The result REPORT must support its runtime identity and
pod accounting, and state the EOS median precisely.

## Finished steps

- deepseek-36 opened and recorded the fixed result in commits `9fe7d5f24` and
  `8ab214b87`.
- The first fresh review at `8ab214b87` approved the measurements, selector,
  outcome row and seal, and requested this documentation repair.

## Cancel probe

- The review requested no model rerun, code change, selector change or re-score.
- The reviewed readings, artifact hashes and 91-entry seal are already final.
- A launch receipt can bind the reviewed runtime identity and pod lifecycle
  without new compute.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-36/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-37/**`
- `docs/research/notebook/float-training.md`

## Done when

1. A task-local launch receipt records the reviewed runtime identity, its
   reproducible derivation and the owned pod's up/release/down lifecycle.
2. The REPORT cites that receipt and says the selected checkpoint's EOS median,
   not an unqualified EOS rank, is 2.
3. The result measurements, closed configuration and notebook row are unchanged.
4. Report, policy, gate and diff checks pass at the repaired head.
5. A new fresh exact-head review approves and the same PR merges.

## Stop and ask

Any change to the readings, model, selector, outcome map, sealed artifact or
pod lifecycle receipt stops the repair. Do not rerun training or broaden the
result.
