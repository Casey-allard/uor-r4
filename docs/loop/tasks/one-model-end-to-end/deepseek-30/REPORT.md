# REPORT: one-model-end-to-end, deepseek-30

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Run pin:
`efee4049e7a9745b5ddecb3a1c2955c4bcddb861`, reviewed runtime
`b5aa959857f387bdc2329d0e68e243cb6473e88fa46f56553f95a47c884eaa80`.

**Outcome row 6: partial movement; this dose closes.** The selected step-1,280
checkpoint preserves continuation and raises free EOS stops to 52/256, but EOS
top-1 and complete accepted replies remain zero. It does not reach an outcome
row above row 6. No seed 2 and no second extension are authorized.

## What did not work, first

- **Free termination moved without top-1 or content.** The selected checkpoint
  has EOS top-1 0/256. It never puts EOS first, despite EOS top-8 256/256 and
  median rank 2.
- **No complete accepted reply appeared.** All eight checkpoints have zero
  complete rows and zero complete-distinct pairs.
- **The best stopping checkpoint is not the final one.** Step 1,280 has 52 free
  EOS stops; step 2,048 has 2, and the 1,024-step run ended at 1.
- **The full extension did not add capability.** Longer training preserved the
  same ranking ceiling and the same content gap rather than completing replies.

## What changed

No production code changed. This unit records the completed run, closes the
registered dose and adds the result row.

## What ran

- One RTX 5090 pod, `vast-59662439`, at $0.563/hour.
- The exact merged pin and reviewed runtime, seed 1, 2,048 steps from the same
  `(b64L)` base, with the unchanged data, mask, optimizer, selector and panels.
- No OOM retry occurred.
- Every 256-step checkpoint was exported, graded and scored before training
  continued.

## Measurements

| Step | Guard/512 | EOS top-1 | EOS median | Free EOS | Complete rows | Complete pairs |
|---:|---:|---:|---:|---:|---:|---:|
| 256 | 140 | 0 | 2 | 18 | 0 | 0 |
| 512 | 139 | 0 | 2 | 26 | 0 | 0 |
| 768 | 140 | 0 | 2 | 10 | 0 | 0 |
| 1,024 | 139 | 0 | 2 | 20 | 0 | 0 |
| **1,280 selected** | **142** | **0** | **2** | **52** | **0** | **0** |
| 1,536 | 138 | 0 | 2 | 5 | 0 | 0 |
| 1,792 | 134 | 0 | 2 | 28 | 0 | 0 |
| 2,048 | 143 | 0 | 2 | 2 | 0 | 0 |

At the selected checkpoint, teacher-forced EOS top-8 and top-32 are both
256/256. The audit rendering (`one-user-turn/v1`) gives 42 EOS stops and 214 cap
stops; the selection and outcome readings use `per-turn/v1` as registered.

DEV nearest/writer top-1 rose from 3,185/192 at step 0 to 6,094/379 at step
1,280, then ended at 6,375/358 at step 2,048. Training loss ended at 0.947761
after 1,485.344 seconds of training (16,943 context tokens/s). Pod uptime was
2,210 seconds, about 36m50s and $0.3456 at the observed rate.

## Evidence

- Selected artifact: `9f349697c3b7cd054f70fe66cb86ffb407ebbf0dcdf63ec16b386174760558e1`.
- Selected model: `6671a20949cead1e321d9367511e5061537c19de3ffeb31e77a665db714718ac`.
- Selected Hamming read: `81fe3c414f10bb4b8a422554d7a81658b7294a9e7d1788c9f19d0edf1ee8cb24`.
- CPU evaluation host: AMD Ryzen 7 7800X3D 8-Core Processor, x86_64, 16
  logical CPUs. Base plus eight checkpoint grades carry matching SHA-bound
  hardware sidecars.
- Seal: 136 manifest entries, 97,394,905 bytes including `SHA256SUMS`;
  `BYTES=97,380,373`; manifest SHA-256
  `7a20b0b32051e76b5b55baf1d07eeac526c1c58a6921ba7a96fa6c5073f74dd1`;
  `READBACK=verified`, `EXIT=0`, `STOP_REASON=natural-or-failure`.
- Store root:
  `results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-2048-20261108t040705z-6b521de8f448498f/`.
- The pod was deleted after release. Project admission returned to 3/4 pods.

## Checks run at the head

- Full store readback from the completed root passed every entry in
  `SHA256SUMS`.
- Base artifact `9a370566...` and the 512-row fingerprint reproduced exactly.
- Corpus/panel overlap remained zero in all four categories.
- The selector ranked all eight checkpoints deterministically and selected step
  1,280.
- The run captured CPU identity, grade labels, per-grade hardware sidecars and
  the complete seal without an OOM or instrument failure.
- `gate.sh` passed 235 checks; hardware-evidence tests passed; the mutation suite
  caught 99/99 before launch.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84, geometry
and delivery rules were used directly.

## Still open

- The content gap: EOS ranking and stopping move, but no complete accepted reply
  appears.
- Claude's independently registered ratio/mix work remains the pretraining
  stopping route; this reply-SFT dose is closed.
- `m2-reply512` remains unavailable. No M2, seed-reproducibility, mechanism or
  useful-chat claim is made.

## Plain summary

The 2,048-step extension made the model stop more often at its best checkpoint,
but never moved EOS to first choice and never completed a reply. The single
registered dose is complete and closed.
