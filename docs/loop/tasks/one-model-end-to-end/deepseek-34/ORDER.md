# ORDER: one-model-end-to-end, deepseek-34 (opening diagnostic)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `0619d850a`.

## Goal

Add a first-answer-token diagnostic to the existing teacher-forced panel and run it
on the sealed content-ratio selected checkpoint. Keep the checkpoint selector and
outcome thresholds unchanged.

## Cancel probe

- The content-ratio run and same-scorer parity evidence are merged and sealed.
- The selected step-512 teacher report contains every per-target rank needed for
  this CPU-only read; no model training or GPU is required.
- The new metric is diagnostic only and cannot alter the already-closed ratio
  decision.

## Files in scope

- `docs/labs/one-model-end-to-end/instrument/**`
- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-34/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The teacher-forced report includes first-target top-1/top-8/top-32, rank
   minimum/median/maximum and target count.
2. The reply-SFT job records those fields in checkpoint readings without using
   them in the selector.
3. Existing teacher-forced and selector tests plus the job gate pass.
4. The selected step-512 checkpoint is rescored on CPU and the opening result is
   recorded with its input and artifact hashes.
5. One REPORT and notebook row state whether the remaining failure begins at
   the opening; no capability, M2 or useful-chat claim is made.

## Stop and ask

Any mismatch in the source artifact, tokenizer, panel hashes or target counts
stops the read. Do not change the selector, rerun the closed ratio
configuration or start a new training dose in this unit.
