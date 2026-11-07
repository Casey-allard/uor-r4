# REPORT: one-model-end-to-end, deepseek-24

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop build and static checks; no
training or model evaluation yet.

## What did not work, first

- **The earlier unit had no common packet interface.** `served-reply-panel`
  scored only `UORLUT01`; the float decoder was private, and
  `cpu-checkpoint-grade` used a different row layout. Comparing their published
  top-1 totals would compare unlike contexts.
- **A memory-bearing checkpoint is not admitted by this first parity reader.**
  The exact `(b64L)` target has no turn memory or reply-read cut. Other
  checkpoints are refused before forward work rather than silently comparing
  partial serving rules.
- **Local `cargo test --no-run` initially failed to link without CUDA runtime
  libraries.** Stub `nvcc` plus empty local `libcuda`/`libcurand` shims proved
  the Rust build and let the two pure comparison tests link and run. Actual model
  evaluation still requires the pod's RTX 5090.

## What changed

- Added `PacketCheckpoint` in `uor-r4-float-train`. It loads the exact
  checkpoint and reads, restores the run's geometric flags, requires the native
  nearest objective, and exposes teacher-forced token ranks plus greedy packet
  replies.
- Added `packet-parity` in the D37 float-training crate with feature
  `packet-parity`. It uses the
  same `m2-fresh256` renderer and reference-answer rule as
  `served-reply-panel`, then compares the float and `CpuFbsForward` decisions
  row by row.
- The integer side uses `CpuFbsForward`'s public serving interfaces and the
  reviewed panel's raw-nearest plus point-key tie/override order. It does not
  depend on the diagnostic API from the earlier draft that #3833 did not land.
- The report records chosen-token mismatches, exact rank mismatches, float and
  integer top-1 counts, EOS rank for every row, the number of EOS rank
  disagreements and the largest rank delta.
- The output binds the checkpoint's model/config files, the source run config
  that carries its forward flags, its Hamming sidecar when present, the
  artifact, tokenizer, panel inputs, labels, plan and manifest hashes, and
  refuses case or target misalignment.
- Review round 1 found that the first identity block omitted the run-level
  forward config and treated the optional Hamming sidecar as mandatory. Both
  are fixed: the source run config is always hashed, while Hamming is included
  only when the checkpoint contains it.

## Measurements

No parity number has been read yet. The implementation is ready to add to the
SFT pod chain after both dependency PRs merge.

## Checks run at the head

- `cargo check --offline --locked -p uor-r4-float-train --lib`: passed.
- `packet_reader_loads_the_exact_geometric_checkpoint_shape`: passed on CPU.
- Stub-CUDA `cargo check -p uor-r4-float-train --features packet-parity
  --bin packet-parity`: passed.
- `packet-parity` pure tests: 2 passed.
- Stub-CUDA `cargo clippy -p uor-r4-float-train --features packet-parity
  --bin packet-parity --no-deps -- -D warnings`: passed with the crate's four
  pre-existing lint families allowed by name.
- `served-reply-panel` CPU tests: 11 passed.
- `rustfmt --check` on the new binary: passed.
- `git diff --check`: passed.

Not run: CUDA model forward, packet parity on the base or SFT checkpoint, or a
pod job.

## Skills used

No system-provided Codex skill applied. The repository's loop, geometry, GPU
and D37 float-training rules were used directly.

## Still open

- Rebase after deepseek-20/21 land and run the instrument on `(b64L)` seed 1.
- Wire the parity report into the SFT checkpoint bundle.
- Record the actual same-prompt/same-reference float/integer differences.

## Plain summary

The project now has one packet-level comparison path for float and integer
decisions: the same prompt and EOS-bearing answer go through both readers, and
every changed token or rank is counted. The code compiles and its comparison
tests pass; the first real model reading remains for the GPU run.
