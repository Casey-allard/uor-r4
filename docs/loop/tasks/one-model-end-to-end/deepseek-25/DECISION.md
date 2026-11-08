# Decision: reply-SFT stop point and best-checkpoint selection

Lab: deepseek, deepseek-25. Item:
[`one-model-end-to-end`](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Status: **approved by the DeepSeek top session after the adversarial
D67 review**. The owner directed this review's findings to be resolved before
delivery.

## Problem

The older deepseek-23 ORDER still said to stop whenever the continuation guard
fell below 93, while the registered sheet applies that guard at the best
checkpoint through 1,024. The sheet did not define "best", so outcome rows 1 and
3-7 were not reproducible and could change after seeing the results.

## Decision

1. The only in-training decision stop is row 2 at step 512. The continuation
   guard is recorded at every checkpoint and is applied after all available
   checkpoints through 1,024.
2. If any checkpoint has `guard_top1 >= 93`, only those guard-safe checkpoints
   are candidates. If none are guard-safe, all checkpoints remain candidates
   and the first selection key is guard top-1 descending.
3. Candidates are ordered by teacher-forced EOS top-1 descending, `per-turn/v1`
   free EOS stops descending, `per-turn/v1` complete accepted rows descending,
   EOS median rank ascending, guard top-1 descending, then step ascending. The
   first candidate is the selected best checkpoint. The `one-user-turn/v1`
   generated panel remains an audit control.
4. The job writes the ranked candidates and selected checkpoint to
   `best-checkpoint.json`. The outcome map is applied to that exact selection.

This does not change a threshold, the one-seed exploratory scope, the M2 bar,
the training recipe or the mechanism. It makes the registered decision order
mechanical and reproducible.

## Reversal

Low. Restore the reconciled ORDER and sheet, remove the selection record and
rerun FIT and D67 before any new launch.
