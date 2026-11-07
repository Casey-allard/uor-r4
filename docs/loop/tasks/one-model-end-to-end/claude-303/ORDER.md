# ORDER: one-model-end-to-end, claude-303

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `3f41847a3` (`origin/main`, 7 Nov 2026).

## Goal

This serves Casey's one track (#2028 6675163438, "Yes, one track"): the trunk's no-memory top-1 must beat its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band, at every seed.

claude-302 (#3808, merged as `3f41847a30eb8a6020109620e525047b89c279ea`) could not train width 1,024 on (b64L)'s recipe. Its two diagnostic stops point at one instrument fault: **micro-batching does not lower peak memory.**
- At batch 16 the whole batch peaked at 66,411 MiB, micro4 and micro2 each at 66,379, and the read-off control at 21,131.
- At batch 64, micro 16 and 8 both ran out of a 97,887 MiB card.

This unit finds the cause, fixes it in the trainer, and measures the fix on the GPU, so that width 1,024 can be rerun under claude-302's sheet.

## The cause, read from the source (to be confirmed by the tests)

- candle-core 0.9.2's `backward()` calls `GradStore::or_insert` for every input of every op.
  - An input that tracks no gradient (a detached tensor or a constant) therefore gets a gradient.
  - Such an input is never in `sorted_nodes()`, so `backward()` never removes that gradient.
- The Hamming read's copy heads gather their payload, `(positions, 16, width)` per head, from `current`, which is a lookup in a detached table. So each copy head leaves a payload-sized gradient that no optimizer reads.
- `step_gradient` then copies every id of each micro-batch's GradStore into the accumulator (`add_grads`). Those dead gradients therefore pile up across micro-batches until the optimizer step, and the step's memory follows the whole batch however it is split.

## The fix

After each forward's `backward()`, keep only the gradients of the variables that this forward's loss reaches: those `loss.sorted_nodes()` lists with `is_variable()`. Every optimizer reads only variables' gradients, so the step is unchanged.

## Claim check

- **(a) The step:** claude-302's outcome map reruns a NOT_RUN, and I-e's rule turns a failure into a diagnosis. This unit is that diagnosis, and the repair it needs.
- **(b) What it changes in the model:** nothing in what is learned. It changes what one training step holds in memory, so width 1,024 can train.

## Cancel probe (run first)

Run on 7 Nov, before any work. Class: instrument.
1. **No hold:** `loop next` shows the kill switch off.
2. **No running pod of mine:** claude-302's two pods are down.
3. **`step_gradient` and `add_grads` are unchanged since the pin `ff5156bcc`:** the 178 lines DeepSeek added to `in_space_long.rs` touch neither function (`git diff ff5156bcc HEAD`).

Result: GO.

## Done when

1. A CUDA test, run on the pod in I-0, shows that a GradStore from `backward()` holds a gradient for a detached input, and that the fix keeps exactly the variables' gradients with bit-equal values. The test fails with the fix set aside. It is CUDA-only because the crate refuses model compute on the CPU (D24/D29).
2. The crate's library tests pass at the head, on the laptop, and the CUDA tests pass on the pod, including `micro_batches_weighted_by_their_scored_rows_give_the_whole_batchs_step`.
3. On one pod, under a one-page sheet with a non-author FIT and a D67 review, peak memory at width 1,024 is measured for batch 16 whole, micro4 and micro2, with the read on and off. It is sealed with a SHA256SUMS that was read back, and the pod is down.
4. A REPORT with the negatives first, one row in `docs/research/notebook/float-training.md`, and `References #3407`.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The host test shows no dead gradients** (the cause is wrong): the GPU step then measures memory by stage instead, under an amended sheet.
- **The cap:** the sheet's cost section sets it.
- **Casey says "hold".**
- **The laptop's free disk falls under 5 GiB.**

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
