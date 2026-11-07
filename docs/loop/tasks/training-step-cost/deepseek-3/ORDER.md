# ORDER: training-step-cost, deepseek-3 (fused L1 rival search)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Base: `2b46c7080` (after PR #3825).

## Goal

For the `nearest` objective under the existing `--fast-rivals 1` flag, compute the whole-table
negative mean-L1 scores and their exact eight-best wrong tokens in one CUDA search. This removes
the `(rows, vocabulary)` score matrix and the subsequent eight selection rounds. The old path
remains the default and the downstream hinge, target score and gradients remain unchanged.

This unit changes no serving path, checkpoint fingerprint or default training recipe.

## Cancel probe

The selector-only kernel from deepseek-2 measured 3.69×–19.23× faster, but saved only 1.07% of a
synthetic training step and changed tied gradients. If fused scoring cannot maintain the
non-tied rival set or does not beat the old score-plus-selection path, record the negative and
leave the flag off.

## Done when

1. A CUDA kernel computes each query row's L1 score against every table row and selects its eight
   nearest non-target tokens without materializing the score matrix.
2. The kernel is used only by the `nearest` objective under `--fast-rivals 1`; every other
   objective and the flag-off path stay unchanged.
3. The downstream loss and gradients use the selected positions, target score and table vectors
   as before.
4. A finite CUDA test compares fused positions with the old score-plus-selection positions on
   deterministic untied inputs. A model-step test records loss, gradient delta and clean step
   time.
5. The float-train CPU suite, CUDA type-check, Clippy on changed lines, `git diff --check`, loop
   report and local PR checks pass. No full recipe training or scoring run occurs.

## Stop and ask if

- The fused path would alter the target score, hinge, gradient expression or serving format.
- Its selected set changes on an untied deterministic fixture.
- It requires touching a pinned stack source or a checkpoint fingerprint.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/top8_positions.rs`
- `crates/uor-r4-float-train/src/top8_positions.cu`
- `docs/loop/tasks/training-step-cost/deepseek-3/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/training-step-cost.md`
