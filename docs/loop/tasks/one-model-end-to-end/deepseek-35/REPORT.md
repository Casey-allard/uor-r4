# REPORT: one-model-end-to-end, deepseek-35

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Preparation only; no pod or model run.

## What did not work, first

- The closed 1,024 and 2,048 reply-SFT doses never moved a first answer target
  into the top 32.
- The content-ratio checkpoint has the same opening failure despite stronger
  free EOS stopping.
- The aggregate teacher-forced top-1 metric hid this because it counts tokens
  after a correct opening.

## What changed

Added opt-in `--opening-weight`, which repeats only scored reply-opening rows
inside the existing geometric nearest loss. Default 1 leaves the recipe and
serialized config unchanged. The fixed experiment uses weight 8 on the original
OASST2-heavy mix for 1,024 steps.

## What ran

Local implementation tests and static checks only. No pod or model forward yet.

## Measurements

Pending. The matched weight-1 selected checkpoint had first-target top-1
0/256, top-8 0/256, ranks 59/894/4,041 and 1,580/2,576 later top-1 targets.

## Evidence

Pending the reviewed run.

## Checks run at the head

- `cargo check --offline --locked -p uor-r4-float-train --lib --bin
  train-in-space`: passed.
- Focused opening-weight tests: 3 passed, including nearest-only refusal and
  weighted row normalization.
- Full float-train library tests: 309 passed, 1 failed in
  `spiralcore_fbs_sheet::tests::a_saved_trunk_loads_back_exactly`
  (`owned artifact directory required`); the same failure reproduces on clean
  `main` and is unrelated to this change.
- Reply-SFT gate: 321 checks passed.
- Mutation suite: 136 of 136 mutations caught.
- `bash -n`, ShellCheck and `git diff --check`: passed.
- `loop report --check`: passed.
- Fresh-context exact-head TEST FITNESS: FIT, recorded on PR #3856.
- Fresh adversarial D67 review approved reviewed-runtime
  `fd303c0a72d178a467ebed99b4fefc05347e193896fc74638b2023cac0b120a1`.
  The three pinned store inputs were downloaded and matched their recorded
  hashes, the runner re-hashes them, the runner checks the serialized weight of 8
  before initialization approval, and the pod build runs the CUDA weighted
  micro-batch test before training. No GPU was free at review, so this grants no
  lease and does not touch the busy Claude pod.

Not run: pod creation, training, export, panel scoring, store write or seal
readback. A CUDA-only whole-versus-micro-batch weighted-gradient test is
compiled and run by the pod build before training.

## Skills used

No system-provided Codex skill applied. The repository's D22, D37, geometry and
delivery rules were used directly.

## Still open

- Merge through the protected queue, verify fresh `main`, then run the fixed
  opening-weight recipe when a DeepSeek GPU lease is available.
- No weight sweep and no seed 2 unless the positive content row is reached.

## Plain summary

This unit tests whether training each reply opening more often lets the
geometric model choose the first answer token from the prompt.
