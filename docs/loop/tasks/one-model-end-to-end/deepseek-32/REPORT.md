# REPORT: one-model-end-to-end, deepseek-32

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Run pin:
`435d1b4f739fc54697ddebe01993707f793b85a9`, reviewed runtime
`c8f9937a197573af83f34d167c246f14167840f738361429fe4c1896e0c1147d`.

**Outcome row 6: partial movement; the content-ratio configuration is closed.**
The selected step-512 checkpoint preserves continuation and reaches 145/256
free EOS stops, but EOS top-1 and complete accepted replies remain zero. A
second row 6 closes this data intervention and authorizes neither seed 2 nor a
second ratio extension.

## What did not work, first

- Increasing M-World to 67.98% of total tokens and 43.35% of scored targets moved
  free stopping from 52/256 to 145/256, but EOS never reached top-1.
- No checkpoint completed an accepted reply or a complete-distinct pair.
- The selected step was not the final checkpoint: step 512 had the most free EOS
  stops, while step 2,048 reached 129.
- `m2-reply512` remains unavailable, so no M2 claim is possible.

## What changed

No production code or experiment parameter changed after the reviewed pin.
This unit records the completed run, classifies it against the registered
first-match outcome map and closes the configuration.

## What ran

One exact-pin RTX 5090 run from `(b64L)` seed 1 for 2,048 steps on OASST2 once
plus the excluded M-World store three times. All eight checkpoints were exported,
graded and scored with the unchanged `per-turn/v1` panel and deterministic
selector. No OOM retry occurred. Pod `vast-59672099` was released and deleted.

## Measurements

| Step | Guard/512 | EOS top-1 | EOS median | Free EOS | Complete rows | Complete pairs |
|---:|---:|---:|---:|---:|---:|---:|
| 256 | 139 | 0 | 2 | 30 | 0 | 0 |
| **512 selected** | **138** | **0** | **2** | **145** | **0** | **0** |
| 768 | 137 | 0 | 2 | 38 | 0 | 0 |
| 1,024 | 137 | 0 | 2 | 37 | 0 | 0 |
| 1,280 | 137 | 0 | 2 | 110 | 0 | 0 |
| 1,536 | 138 | 0 | 2 | 12 | 0 | 0 |
| 1,792 | 136 | 0 | 2 | 8 | 0 | 0 |
| 2,048 | 140 | 0 | 2 | 129 | 0 | 0 |

At step 512, teacher-forced EOS top-8 and top-32 are both 256/256. DEV
nearest/writer top-1 was 6,116/330 at selection and 7,062/439 at step 2,048.
Training took 1,369.986 seconds; pod uptime was about 37m24s and cost about
$0.35.

## Evidence

- Selected artifact:
  `406c9a5c05d63c8f2b6044ee6de6ea78d9fa91582dc03c4aa3e4fe877846e384`.
- Selected model:
  `11195de63f508a454c085cb538be33aa1d26c3c282e0248b85aa2e9b61e23133`.
- Selected Hamming read:
  `e4562c3aa18e3aeaa72ce646ed5d2403b5eb5394476e304d6f83ee8526817c6a`.
- Seal: 135 manifest entries, `BYTES=77,343,856`;
  `SHA256SUMS` SHA-256
  `a183ae67c3563d26c2cf3f0e437f5bbe64e2d735dd9b2a0608d8b32da663164f`;
  `READBACK=verified`, `EXIT=0`.
- CPU evaluation host: AMD Ryzen 7 7800X3D 8-Core Processor, x86_64, 16
  logical CPUs, with matching SHA-bound sidecars on the base and all eight
  checkpoint grades.
- Store root:
  `results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/runs/s1-content-ratio-20261108t055107z-adb12718d6c9ea88/`.

## Checks run at the head

- Full store readback verified every `SHA256SUMS` entry.
- The landed runner gate passed 297 checks and the mutation suite caught 125/125
  before launch; the same gate and mutation results reproduced on the landed
  `main` tree.
- Base artifact `9a370566…` and all registered input hashes reproduced.
- Corpus-panel overlap was zero in all four categories.
- The selector ranked all eight checkpoints deterministically and selected step
  512.
- The seal entered at 77,358,299 bytes including its manifest, below 100 MiB,
  after preserving the raw-input hashes in `control/post-use-inputs.json`.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84,
geometry and delivery rules were used directly.

## Still open

- The content gap remains: stopping moves, but EOS top-1 and accepted reply
  completion do not.
- No seed 2 or further ratio extension follows from this row-6 result.
- No mechanism, M2, seed-reproducibility or useful-chat claim is made.

## Plain summary

The ratio intervention made the model stop much more often but never made it
choose EOS first or complete an accepted reply. The content-ratio configuration
ends here.
