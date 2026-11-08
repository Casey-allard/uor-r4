# ORDER: one-model-end-to-end, deepseek-38 (repair pod cost arithmetic)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `a3ac3f326`.

## Goal

Repair the second exact-head review's pod-accounting finding on PR #3857. Make
the launch receipt and both result reports distinguish the logged billing rate
and uptime from an explicitly approximate cost.

## Finished steps

- deepseek-36 recorded the fixed result and closed weight 8 in commits
  `9fe7d5f24` and `8ab214b87`.
- deepseek-37 added the runtime and pod lifecycle receipt in commits
  `ef6711f8a` and `957734b61`.
- The second fresh review at `957734b61` confirmed every model reading, hash
  and outcome row, and found the exact cost inconsistent with the logged
  `1,610` seconds and `$0.563/hour`.

## Cancel probe

- The result and every measurement remain sealed and unchanged.
- This is an arithmetic and wording correction in task documentation only.
- No new model, pod, score or external measurement is required.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-36/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-37/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-38/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The launch receipt records an approximate cost computed from the logged
   uptime and hourly rate, with its formula.
2. Both reports use the approximate value and do not present a rate-derived
   estimate as an exact charge.
3. The measured result, selector, outcome row and notebook are unchanged.
4. All report, policy, gate, diff and seal checks pass at the repaired head.
5. A new fresh exact-head review approves and the same PR merges.

## Stop and ask

Any change to the result, readings, selector, hashes or sealed artifact stops
the repair. Do not rerun training or create another result PR.
