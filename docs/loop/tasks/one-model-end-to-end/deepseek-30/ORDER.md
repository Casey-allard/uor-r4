# ORDER: one-model-end-to-end, deepseek-30 (record the 2,048-step result)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `efee4049e`.

## Goal

Record the completed 2,048-step reply-SFT extension, classify its selected
checkpoint against the registered first-match outcome map, verify the sealed
CPU and GPU evidence, and close this dose. Do not start seed 2 or another
extension.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-29/REPORT.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-30/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The selected checkpoint, every checkpoint reading, DEV curve, artifact
   hashes, CPU hardware identity, seal size and readback are recorded.
2. The result is classified by the registered first-match outcome map.
3. The dose-closing decision is explicit: no seed 2 and no second extension.
4. The notebook has one result row, and no capability, M2 or useful-chat claim
   is made.
5. Report, policy and exact-head review gates pass.

Never rerun or extend this configuration in this unit. Never touch another
lab's pods, branches, leases or files.
