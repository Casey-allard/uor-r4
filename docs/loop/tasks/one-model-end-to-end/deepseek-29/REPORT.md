# REPORT: one-model-end-to-end, deepseek-29

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Run pin:
`efee4049e7a9745b5ddecb3a1c2955c4bcddb861`, reviewed runtime
`b5aa959857f387bdc2329d0e68e243cb6473e88fa46f56553f95a47c884eaa80`.

**Outcome row 6: partial movement; the registered 2,048-step dose is complete
and closed.** Step 1,280 was selected with 142/512 continuation and 52/256 free
EOS stops, but zero EOS top-1, zero complete replies and zero complete pairs.

## What did not work, first

- PR #3845 merged after Claude's docs-only claude-315 change, so its pre-merge
  runtime identity no longer matched fresh `main`.
- A post-merge D67 review found that CPU grading did not seal the host CPU
  identity required by D84.
- The 1,024-step run reached row 6, and the 2,048-step extension reached row 6
  again: stopping moved, but accepted reply content did not.
- The final checkpoint regressed on free EOS stops to 2/256, so training longer
  was not monotonically better.

## What changed

- The job records `cpu.json` and `cpu.txt` before model work and every base or
  checkpoint grade carries a D84 label plus an exact SHA-bound hardware sidecar.
- Missing CPU evidence, missing sidecars, or a post-bind grade mutation fails
  before sealing.
- OOM retry compacts bulky partial-attempt checkpoints, exports, panels and
  scores into a sealed hash manifest while retaining logs and progress.
- Gate and mutation coverage require both CPU labels, both grade bindings,
  CPU-file seal inclusion and retry compaction.
- `docs/harness/claims.json` carries this branch's claim and is the sole
  out-of-item-scope path; D76 makes that scope finding advisory.

## Measurements

The run trained all 2,048 steps in 1,485.344 seconds, about 24m45s. The pod was
up 2,210 seconds, about 36m50s and $0.3456 at $0.563/hour. No OOM retry
occurred.

At steps 256/512/768/1,024/1,280/1,536/1,792/2,048, continuation guards were
140/139/140/139/142/138/134/143, EOS top-1 was always 0, median EOS rank was
always 2, free EOS stops were 18/26/10/20/52/5/28/2, and complete rows and
complete-distinct pairs were always 0. The selector chose step 1,280.

At the selected checkpoint, EOS top-8 and top-32 are 256/256. The audit
rendering gives 42 EOS and 214 cap stops; the selection and outcome readings use
`per-turn/v1`.

Selected artifact `9f349697...`, model `6671a209...`, Hamming read
`81fe3c41...`. DEV nearest/writer top-1 reached 6,094/379 at step 1,280 and
ended 6,375/358 at step 2,048. Training loss ended at 0.947761.

## Checks run at the head

- Fresh FIT and D67 reviews passed at the landed runtime before launch.
- Base artifact `9a370566...` and the 512-row fingerprint reproduced exactly;
  all four corpus-panel overlap counts were zero.
- `gate.sh` passed 235 checks; hardware tests passed; the mutation suite caught
  99/99 before launch.
- The full store readback passed all 136 `SHA256SUMS` entries.
- CPU identity is AMD Ryzen 7 7800X3D, x86_64, 16 logical CPUs; the base and all
  eight checkpoint grades have matching hardware sidecars.
- The seal is 97,394,905 bytes including its manifest, under 100 MiB;
  `READBACK=verified`, `EXIT=0`, manifest `7a20b0b3...`.
- The selected artifact, model and Hamming hashes are recorded above.
- The pod was released and deleted; project admission returned to 3/4 pods.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84, geometry
and delivery rules were used directly.

## Still open

- The model still produces no complete accepted reply. This dose is closed; no
  seed 2 or second extension follows.
- `m2-reply512` remains unavailable. No capability, M2 or useful-chat claim is
  made.

## Plain summary

This unit repaired the merged runtime and D84 CPU evidence, ran the one
registered 2,048-step extension, and closed it on another row-6 result.
