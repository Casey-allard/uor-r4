---
id: in-space-trunk-to-text
title: Train the FBS trunk in its own space until its replies read their prompts
milestone: research
area: training
status: done
open_to: lab
needs_gpu: true
depends_on: [float-training-module, spiralcore-fbs-offline-float-sheet]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/geometric_stack_native.rs, crates/uor-r4-float-train/src/secant_descent.rs, crates/uor-r4-float-train/src/lib.rs, crates/uor-r4-float-train/Cargo.toml, crates/uor-r4-float-train/src/flock_read*.rs, crates/uor-r4-float-train/src/hamilton_sheet.rs, crates/uor-r4-float-train/src/spiralcore_fbs*.rs, crates/uor-r4-float-train/src/native_vector_writer.rs, Cargo.lock, docs/harness/prep/in-space-trunk-to-text*, docs/labs/in-space-trunk-to-text/**, docs/loop/tasks/in-space-trunk-to-text/**, docs/research/notebook/float-training.md, docs/roadmap/items/in-space-trunk-to-text.md]
---

## Why it matters

Casey's direction of 27 Oct 2026, in claude's chat: drive one learnable geometric mechanism end to end to generated text; train in the same space used for transport and prediction, and decode only at the end; always look at context. Until a trained model writes replies that answer their prompts, no milestone number means anything.

The FBS sheet trunk (`spiralcore_fbs_sheet::build_fbs_model`) is the project's only learned in-space model: six quaternion recurrence layers with the Hamilton gate and SpiralCore/FBS projections, and one shared vector table for input and output. The sheets trained it for 1,024 steps of 4 windows, about 6% of TRAIN seen once.

## What is missing

**Closed 28 Oct 2026 (claude-155, [REPORT](../../loop/tasks/in-space-trunk-to-text/claude-155/REPORT.md)).** The done-when below is met at both seeds by the plain bag of the user's turn (`--prompt-register 1 --register-select all`, on claude-152's cell-objective recipe), on its final checkpoints:
- openings `217` and `275` of 1,226 (≥ 197), with `T = 8,993` and `9,253` (≥ 7,002);
- `R16 = 0.166` and `0.155` (≥ 0.05);
- the greedy replies recorded at every checkpoint.

The replies still mostly template, and openings do not beat claude-152's `302.0 / 228.0`. The line pivots by CHARTER §4.4, with the owner's vote ("ok yes, my vote is to pivot to your diagnosed recommended new roadmap", 28 Oct). The next steps are a per-stage cost profile, a mechanism map, and a route-plus-exact-memory run. The state below is the record from 27 Oct.

**State on 27 Oct 2026 (claude, branch `claude/train-in-space`).** The runner `train-in-space` trains the trunk for tens of thousands of steps, with DEV text at every checkpoint, and reads its saved checkpoints back.

- **Next-token top-1 passes the count tables.** Whole-table nearest decoding reaches `8,607 / 14,287` DEV targets, against `6,300` for the bigram count and `7,002` for the M1 count table. That is the nearest objective at `lr 1e-3`, `16,384` steps, `long3/n-nearest-1e-3`.
- **The replies barely read their prompts.** On the 1,226 DEV rows that open a reply, the best checkpoint gets `220` (`lr 1e-3`) and its `lr 5e-4` twin `183`. Always naming the commonest opening token gets `161`, and three binomial standard deviations are `36`. The replies to the memory prompts open with "Got it" and continue with corpus templates. The opening point reads about 8–12 tokens back (`R16 = shift(16) / shift(0) = 0.011`, against `0.70` at step 0).
- **Lanes end at the shortest memory.** In the best checkpoint, 37–75% of lane positions select rate entry 0 after training: 37% in layer 0, and 59–75% in layers 1–5. Every layer starts at 33%. The clamp on the rate score has zero gradient outside `[0, 1]`.
- **The writer's address hinge cannot be met.** Its 16 roots are at least 0.5 apart in mean L1 and its margin is 1, so the hinge is never zero; at higher rates the model flattens its address ports and admission picks arbitrary cells.

## Done when

On the pinned DEV panel, decoded as the whole table's nearest token:

1. Reply-opening top-1 is at least `197` of 1,226. That is always naming DEV's commonest opening token (`161`) plus three binomial standard deviations (`36`). The same checkpoint's whole-panel top-1 must be at or above the M1 count table's `7,002`.
2. On that same checkpoint, the relative reach from `--reach`, `R16 = shift(16) / shift(0)`, is at least `0.05`. The 27 Oct best checkpoint reads `0.011`. An untrained trunk reads more, but it cannot meet (1).
3. Greedy replies to the six fixed DEV prompts are recorded beside the true replies, at every checkpoint, in the lab record.

Lead with tokens per second, decode and context separately.

## How to start

Read `crates/uor-r4-float-train/src/in_space_long.rs` and the 27 Oct row of `docs/research/notebook/float-training.md`. The checkpoints and text are in `caseyallard/uor-r4-store:results/claude/train-in-space-2026-10-27/`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU.
- **Pre-registration first** (§4.5): a GPU training or evaluation run that a milestone claim or this item's done-when rests on needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs. Casey's 27 Oct direction, recorded on #2028 (comment 6433848381), allows light gates for exploration runs; a run this done-when rests on still needs the sheet.
- **Runs on a pre-run review** (D67, §5.1): a GPU or training run goes ahead when the lab's own fresh pre-run review passes, inside the GPU caps.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in `docs/research/notebook/float-training.md`. A negative result closes a configuration, never a mechanism.

## Sources

- [`crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs:86`](../../../crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs)
- [`crates/uor-r4-float-train/src/native_vector_writer.rs:387`](../../../crates/uor-r4-float-train/src/native_vector_writer.rs)
- [`docs/research/notebook/float-training.md`](../../../docs/research/notebook/float-training.md)
