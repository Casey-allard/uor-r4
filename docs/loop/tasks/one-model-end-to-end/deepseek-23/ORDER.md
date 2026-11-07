# ORDER: one-model-end-to-end, deepseek-23 (executable reply-SFT run)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `66d20106d`.

## Goal

Turn deepseek-22's reply-masked SFT sheet into an executable, reviewed seed-1 chain:
acquire one DeepSeek-owned RTX 5090, restore the exact `(b64L)` seed-1 checkpoint,
train 1,024 steps on the already sealed mixed store, export and score every
256-step checkpoint, enforce the continuation and 512-step ranking stops, seal
the evidence, read it back and take the pod down.

No seed-2 run starts here. It remains conditional on outcome-map row 3 and a
fresh decision review.

## Cancel probe

- The mixed store, OASST2 held-out store, base archive, panel, tokenizer and DEV
  files must all be present at the paths and hashes in the sheet.
- `--init-from` and per-turn teacher-forced EOS ranks must be on `main` before
  the run pin is set.
- `uor-pod status` must retain room inside four pods and $8/h without touching
  Claude's two busy pods.
- `m2-reply512` remains unavailable; the run does not substitute another panel.

## Files in scope

- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-23/**`
- `docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md`
- `docs/research/notebook/float-training.md`

## Done when

1. The pod job validates the source pin and every full input hash before training.
2. It reproduces the base panel/guard controls, trains one schedule for 1,024
   steps, and freezes training while each 256-step checkpoint is exported and
   scored.
3. It stops at step 512 when EOS remains at top-1 zero with median rank at least
   7, or earlier when the 512-row continuation guard falls below 93.
4. It seals checkpoints, exports, grades, panel outputs and logs under a new
   store root, verifies the readback and takes the pod down.
5. A fresh FIT and D67 pre-run review examine the exact merged code, run chain,
   input packet and seal paths.
6. The seed-1 run completes or is recorded as `NOT_RUN`, and the REPORT applies
   the sheet's first-match outcome map.

## Stop and ask if

Ask means decide, record and continue. Never touch Claude's pods or branch.
Never weaken a hash, panel, EOS gate or geometry constraint. Never cancel a
healthy run for a self-set timer; renew and record it.
