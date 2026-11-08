# REPORT: one-model-end-to-end, deepseek-31

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Run pin:
`435d1b4f739fc54697ddebe01993707f793b85a9`, reviewed runtime
`c8f9937a197573af83f34d167c246f14167840f738361429fe4c1896e0c1147d`.

**Outcome row 6: partial movement; this content-ratio configuration closes.**
The selected step-512 checkpoint preserves continuation and reaches 145/256
free EOS stops, but EOS top-1 and complete accepted replies remain zero. A
second row 6 on this distinct data intervention authorizes neither seed 2 nor
another ratio extension.

## What did not work, first

- **More M-World weight moved stopping, not complete content.** The selected
  checkpoint has 145 free EOS stops, up from 52 in the closed dose, but zero
  EOS top-1 and zero complete accepted rows.
- **The movement did not persist monotonically.** Free EOS stops were
  30/145/38/37/110/12/8/129 across steps 256 through 2,048. Step 512 remained
  the deterministic winner; the final checkpoint recovered to 129 but did not
  complete a reply.
- **The exact `m2-reply512` bytes remain unavailable.** Its deterministic
  lookalike fails both registered hashes, so it is not used and no M2 claim is
  possible.

## What changed

- Built a 65,071-document mixed store: OASST2 reply-masked TRAIN once and the
  same excluded M-World v2 store three times. It has 11,786,293 tokens,
  5,710,018 scored targets and 223,469 scored EOS at 39,136 per million.
- The old mix was 41.45% M-World by total tokens but 20.32% by scored reply
  targets. The new mix raises those shares to 67.98% and 43.35%.
- Added the closed `content-ratio/2048/s1-content-ratio` recipe, block-bound
  data hashes and output root, and gate/mutation coverage for wrong recipes,
  counts, labels, sheet/report pairs and override attempts.
- Successful runs now re-hash the raw training inputs, record their hashes and
  sizes in `control/post-use-inputs.json`, and remove only the raw token/mask
  payloads before sealing. The source store and manifest preserve recovery.

## What ran

- One DeepSeek-owned RTX 5090 pod, `vast-59672099`, at $0.563/hour.
- Exact merged pin `435d1b4f7`, seed 1, 2,048 steps from the same `(b64L)`
  base with unchanged geometry, mask, optimizer, checkpoint scoring, selector,
  continuation guard and panels.
- No OOM retry occurred. All eight checkpoints were exported, graded and scored
  before training continued.
- The pod was released and deleted after the seal was verified.

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

At the selected checkpoint, teacher-forced EOS top-8 and top-32 are both
256/256. The audit rendering (`one-user-turn/v1`) gives 67 EOS stops; selection
and outcome readings use the registered `per-turn/v1` values above.

DEV nearest/writer top-1 rose from 3,185/192 at step 0 to 6,116/330 at the
selected step and ended at 7,062/439 at step 2,048. Training loss ended at
0.9493008314 after 1,369.986 seconds (18,369 context tokens/s). Pod uptime was
about 37m24s, about $0.35 at the observed rate.

Corpus-panel overlap remained zero for exact panel strings, exact accepted
assistant replies and both six-gram classes over 65,071 documents and 223,469
assistant replies. The base artifact reproduced as `9a370566…`.

## Evidence

- Selected artifact:
  `406c9a5c05d63c8f2b6044ee6de6ea78d9fa91582dc03c4aa3e4fe877846e384`.
- Selected model:
  `11195de63f508a454c085cb538be33aa1d26c3c282e0248b85aa2e9b61e23133`.
- Selected Hamming read:
  `e4562c3aa18e3aeaa72ce646ed5d2403b5eb5394476e304d6f83ee8526817c6a`.
- Run config:
  `2eda0a4c212fd56d186b05fabbca5bf9cc93cc6a5085bbab26009e22e6089389`;
  initialization record:
  `dd84e45ef33087e4547eaab5edd60bbcf8c0d8f8c784289c96e793dd088de01b`.
- CPU evaluation host: AMD Ryzen 7 7800X3D 8-Core Processor, x86_64, 16
  logical CPUs. Base and eight checkpoint grades carry matching SHA-bound
  hardware sidecars.
- Seal: 135 manifest entries, `BYTES=77,343,856`, 77,358,299 bytes including
  `SHA256SUMS`; manifest SHA-256
  `a183ae67c3563d26c2cf3f0e437f5bbe64e2d735dd9b2a0608d8b32da663164f`;
  `READBACK=verified`, `EXIT=0`, `STOP_REASON=natural-or-failure`.
- Full readback verified every entry in `SHA256SUMS`.
- Store root:
  `results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/runs/s1-content-ratio-20261108t055107z-adb12718d6c9ea88/`.

## Checks run at the head

- Landed runtime gate: 297 checks passed.
- Landed mutation suite: 125 of 125 mutations caught.
- Hardware-evidence tests passed, including successful-input compaction and
  mismatched-hash refusal.
- ShellCheck over the job, launcher and hardware helpers passed; `bash -n`,
  `git diff --check`, `loop report --check` and `loop pr-check --local` passed.
- Fresh FIT and adversarial D67 reviews approved runtime head `53f8f4f2b` and
  recomputed the normalized runtime identity above.
- The protected merge queue landed PR #3850 as `435d1b4f7`; the landed tree
  reproduced the same runtime identity, gate and mutation results.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84,
geometry and delivery rules were used directly.

## Still open

- The answer-content gap remains: moving the M-World ratio raises stopping but
  still does not put EOS first or complete an accepted reply.
- The content-ratio configuration is closed. No seed 2, second ratio extension
  or useful-chat claim is authorized.
- Claude's independently registered ratio/mix work remains the separate
  pretraining route. `m2-reply512` remains unavailable.

## Plain summary

Tripling M-World's weight made the model stop more often, but it still never
chose EOS first and never completed an accepted answer. The ratio lever is
closed at this recipe and dose.
