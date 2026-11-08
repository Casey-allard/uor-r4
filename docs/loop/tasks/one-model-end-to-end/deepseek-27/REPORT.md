# REPORT: one-model-end-to-end, deepseek-27

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no model, pod or spend.

## What did not work, first

- The step-512 best checkpoint did not make EOS top-1.
- No free-generated reply was a complete accepted answer, and no pair was both
  complete and distinct.
- The step-1,024 checkpoint regressed relative to step 512 on free EOS stops and
  continuation.

## What changed

Recorded deepseek-26's verified run and outcome in
[REPORT.md](../deepseek-26/REPORT.md), including the selected checkpoint,
sealed hashes, resource use and next dependency.

The run is outcome row 6, **Partial movement**. It is not a capability or M2
claim. Seed 2 remains unauthorized.

## Measurements

No new measurement ran in this unit. The recorded result is:

- selected checkpoint: step 512;
- continuation: 142/512;
- teacher-forced EOS: top-1 0/256, top-8 256/256, median rank 2;
- `per-turn/v1` free generation: 23/256 EOS stops, 0 complete accepted rows and
  0 complete distinct pairs;
- seal: 65,925,169 bytes, 87 files, verified readback;
- pod cost: approximately $0.228.

## Checks run at the head

- Every downloaded seal entry passed `SHA256SUMS`.
- `READBACK=verified` and `EXIT=0`.
- The selected artifact and checkpoint hashes were re-read from the seal.
- The pod was released and deleted.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute and
geometry rules were used directly.

## Still open

- One 2,048-step dose extension at a new root under a new execution unit.
- Seed 2 remains conditional.
- `m2-reply512` remains unavailable.

## Plain summary

The recorded seed-1 run moved EOS ranking and generated EOS stops, but not
complete accepted replies; the registered next step is one 2,048-step extension.
