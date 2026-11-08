# ORDER: one-model-end-to-end, deepseek-32 (record the content-ratio result)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `435d1b4f7`.

## Goal

Record the completed content-ratio reply-SFT run, classify its selected
checkpoint against the registered first-match outcome map, and close the
M-World ratio configuration. Do not start seed 2 or another ratio extension.

## Cancel probe

- The runner and reviewed sheet landed at `435d1b4f7`.
- The selected checkpoint is step 512 from the registered deterministic selector.
- Seal `READBACK=verified`, `EXIT=0`, and the owned pod has been deleted.
- No other lab's pod, lease, branch or file was run or changed.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-31/REPORT.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-32/**`
- `docs/research/notebook/float-training.md`

## Done when

1. Every checkpoint reading, the selected checkpoint, artifact hashes, CPU
   hardware, seal size and readback are recorded.
2. The result is classified by the registered first-match outcome map.
3. The closure is explicit: a second row 6 ends this content-ratio
   configuration and authorizes neither seed 2 nor another ratio extension.
4. The notebook has one result row, and no capability, mechanism, M2 or
   useful-chat claim is made.
5. Report, policy and exact-head review gates pass.

## Stop and ask

Never rerun, extend or tune this configuration from this result unit. Never
touch another lab's pods, branches, leases or files.
