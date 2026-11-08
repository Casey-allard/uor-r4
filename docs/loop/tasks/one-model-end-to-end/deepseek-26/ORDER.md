# ORDER: one-model-end-to-end, deepseek-26 (execute reply SFT seed 1)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `6685b859b`.

## Goal

Execute the already reviewed reply-masked SFT seed-1 unit with one
DeepSeek-owned RTX 5090, using the exact merged pin and reviewed runtime from
the reply-SFT sheet. Export and score the base and every available checkpoint,
seal and read back the evidence, take the pod down, and apply the deterministic
outcome map to the selected checkpoint.

No seed-2 run starts here. It remains conditional on outcome row 3 and a fresh
decision review. `m2-reply512` remains unavailable and no substitute may be
scored under that name.

## Cancel probe

- The exact output root must be absent in the store.
- The merged `main` pin must reproduce the reviewed runtime recorded by fresh
  FIT and D67 GO in the reply-SFT sheet after this ORDER is committed.
- `uor-pod status` must retain room inside four running pods and $8/hour without
  touching Claude's two busy pods.
- All 18 bound input/checkpoint identities must rehash before the first forward.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-26/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-25/REPORT.md`
- `docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md`
- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The exact merged pin and reviewed runtime launch the one-GPU job.
2. The base controls reproduce every registered reading before training.
3. All available checkpoints are exported and scored without replacement, and
   failures preserve their evidence.
4. The job seals no more than 100 MiB, reads the complete seal back, and the pod
   is released and taken down.
5. The REPORT applies the selected checkpoint and first-match outcome map,
   records the notebook row, and explicitly separates a configuration result
   from a mechanism result.

## Stop and ask if

Ask means decide, record and continue. Never touch Claude's pods, branch, lease
or files. Never weaken an input hash, panel, EOS gate, continuation floor,
selector, seal or geometry constraint. Never cancel a healthy run for a
self-set timer; renew and record it.
