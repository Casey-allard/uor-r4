# ORDER: training-step-cost, deepseek-2 (one-pass exact rival selection)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Base: `11dd0b623` (depends on PR #3823).

## Goal

Add a bounded top-eight selection kernel behind `--fast-rivals 1`. One warp selects a row's eight
best allowed positions in one launch, descending by score with ties to the lower token id,
replacing the eight whole-table `argmax` and mask launches.

The old path remains the default and is the exact reference for every non-tied choice. Current
Candle CUDA argmax does not guarantee an order for ties; the new path makes that case
deterministic as lower token id. The kernel gets its own PTX module, so `cuda_stack.cu`,
`spiralcore_fbs.cu` and every checkpoint fingerprint stay unchanged.

## Cancel probe

At the base, `best_positions` still makes `k` whole-vocabulary `argmax_keepdim` calls and masks
one picked position after each. If the new selector changes a non-tied choice, keep the flag off
and report a negative. On tied scores it must return lower token ids in ascending order.

## Done when

1. `train-in-space --fast-rivals 1` sets a thread-local training flag; default 0 preserves every
   existing file and code path. The flag is written to `config.json` only when on.
2. `best_positions` takes one block of 32 threads a row and returns exactly the eight ordered
   allowed positions, by score descending and token id ascending on ties.
3. Vocabulary, row count, allowed/scores shape and the eight-rival cap are checked before launch;
   no CPU or silent fallback runs under the fast flag.
4. A finite CUDA test checks the selector on non-tied and tied rows. A model-step test records
   whether loss and gradients match; any gradient difference is a negative for preserving the
   current training arithmetic and keeps the flag off. A CPU test compares the host reference
   with repeated selection and its tie rule.
5. The float-train CPU tests, stub-CUDA all-targets check, Clippy on changed lines,
   `git diff --check`, loop report and local PR checks pass. A finite synthetic model step is
   timed; no recipe training or scoring run is made.

## Stop and ask if

- The selector would require editing a pinned CUDA source or changing model math.
- The hardware selector cannot be exposed without a host sync or CPU fallback.
- Any checkpoint's plan fingerprint would move.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/build.rs`
- `crates/uor-r4-float-train/src/lib.rs`
- `crates/uor-r4-float-train/src/cuda_stack_kernels.rs`
- `crates/uor-r4-float-train/src/top8_positions.rs`
- `crates/uor-r4-float-train/src/top8_positions.cu`
- `docs/loop/tasks/training-step-cost/deepseek-2/**`
- `docs/roadmap/items/training-step-cost.md`
