# ORDER: training-step-cost, deepseek-1 (one loss read a step)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Base: `f66f023d1`.

## Goal

Remove the blocking device-to-host loss copy from every micro-batch of a training step. The
current `step_gradient` calls `loss.to_scalar::<f32>()` inside the micro-batch loop. A (b64L)
step uses four micro-batches, so this waits for the device four times before its backward is
complete.

Keep each micro-batch's detached scalar on the device, finish all forwards and backwards, then
copy and sum the scalars once. Preserve the gradient order, each forward's share, the host
`f32 -> f64` arithmetic order, and the finite-loss refusal. This unit changes no objective,
rival, model, checkpoint or serving calculation.

## Cancel probe

At the base, `in_space_long.rs` still has one `to_scalar::<f32>()` per micro-batch in
`step_gradient`, while the b64L recipe uses `--micro-batch 16` on batch 64. If either statement
is false at the rebased head, stop and re-scope. If the deferred read changes the loss or a
gradient in the existing CUDA equivalence test, revert it and report a negative.

## Done when

1. `step_gradient` performs one loss-device read after its micro-batch loop.
2. Each detached scalar's share and the final reduction use the same values and order as before.
3. CPU tests pin the weighted f64 sum and the non-finite refusal; the existing CUDA test still
   pins split versus whole-batch loss and gradients.
4. The float-train library tests, Clippy for the crate, the stub-CUDA check, `git diff --check`,
   loop report and local PR checks pass. The base carries rustfmt drift in `in_space_long.rs`;
   this unit preserves that baseline and runs no whole-file formatter, so it makes no formatting
   claim. No model, pod or GPU run occurs in this unit.
5. A REPORT states the code change and makes no speed claim without a device measurement. The
   notebook row belongs to that later measurement unit, because no research run occurs here.

## Stop and ask if

Ask means decide, record and continue.

- Making the read deferred would require changing the objective, gradient order or values.
- The existing CUDA equivalence test cannot be run at the later device-measurement unit.
- Work would touch Claude's active branch, another lab's worktree or a pod.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `docs/loop/tasks/training-step-cost/deepseek-1/**`
- `docs/research/notebook/float-training.md`
