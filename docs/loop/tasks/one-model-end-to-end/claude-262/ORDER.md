# ORDER: one-model-end-to-end, claude-262 (training as served learns again)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `a9d0461e2`.

## Goal

Casey, 2 Nov 2026: "your goal is to get us to a working chat model using the training and serving methods we have described - you need to figure out how to stitch all of that together" (quoted in [claude-260's ORDER](../claude-260/ORDER.md)). Every model on that path trains with `--train-as-served 1`, and since #3565 that mode learns about ten times less per step than plain training:

- deepseek-5's four 256-step runs at `36ca53c81` (#3407, comment `6595148437`) had DEV nearest 23/20 for served against 225/229 for plain, and served-evaluator top-1 11/6 against 36/61. Over 256 steps the served loss moved 1.00423 → 1.00400, and plain moved 1.00419 → 1.00316.
- claude-261's 2,048-step served runs reach DEV nearest 921–2,314. claude-259's form, before #3565, reached 11,357 and up at the same step.

claude-261's session handed this unit to me. It finds the cause, fixes it and reruns deepseek-5's FIT sheet at the fixed head. [deepseek-10's decision](../deepseek-5/DECISION.md) names the same next unit: "a focused cost repair of the served activation quantizer followed by this exact matched comparison".

**Plan step (the claim check):** this unit unblocks the training step of every model on the item's path. No served model can be trained to a useful point while the served forward drops the trunk's activations.

## The fault, found in source before this ORDER

`activation_quantize16` (`hamilton_sheet.rs`) is the float stand-in for the integer session's `stack_quantize16` (`stack/kernels.rs`). The integer step holds both quantized activations at `PRODUCT_EXP = 2·RESIDUAL_EXP = −32` before requantizing them: the MLP's gated product (`stack_step_fbs`, `session.rs:3453`) and the recurrence's output (`stack_recurrence_core`, `session.rs:3947`). It keeps 15 bits of the row's maximum: `s = max(bit_length(max_int) − 15, 0)`, quantum `2^(−32+s)`. A row with maximum 1 is served at a quantum of 2^−14.

The float stand-in runs the same formula on the real-valued activation, as if it were already the integer mantissa. Its scale starts at 1 and can only halve. For any row with maximum below 32,768 the scale stays 1, so every activation is rounded to a whole number. Activations of size about 1 or smaller become 0 or ±1. So in a served training forward:

- the MLP and recurrence outputs are mostly 0;
- the gradient to `mlp.down.weight` and `rec.out.weight`, the input-times-gradient product, is 0 wherever its input rounded to 0, so those weights barely move;
- the gradient still flows straight through to the gate and up weights, whose effect the forward cannot show.

The served evaluator (`--served-forward 1`) has the same fault, so it also undercounts a plain checkpoint (36/61 against 225/229 nearest).

The existing tests missed it because their rows were 40,000, −12,001, 70,001 and so on. At 2^15 and above the old scale and the integer one agree, since the exponent only shifts `s` by 32.

## Cancel probe (run first; results)

1. **Already fixed?** `git show origin/main:crates/uor-r4-float-train/src/hamilton_sheet.rs` has no exponent in `activation_quantize16`. Open PR #3579 (claude-261) changes only its constants' dtype (`36ca53c81`), with values unchanged. Not cancelled.
2. **Another lab's open order on these files?** The item is claimed by claude-261's worktree, so `uor-harness loop claim one-model-end-to-end --lab-topic served-learning-repair` refused: "item one-model-end-to-end is already claimed by worktree …/claude-copy-key-k2". This ORDER is written by hand, as claude-261's and others were. Open orders that share files:
   - **#3579 (claude-261), `hamilton_sheet.rs`:** its dtype repair is inside the function this unit rewrites. The rewrite builds every constant in the activation's dtype, so it keeps that repair. Whichever PR merges second merges `main` forward and keeps this unit's function body. I never touch #3579, its branch, its worktree or its pods (`vast-58896165`, `vast-58912154`).
   - **#3579 and #3583 (deepseek-9), `in_space_long.rs`:** this unit adds only a test there, apart from their lines.

   Not cancelled.
3. **Inputs:** the sheet's TRAIN, DEV, masks and tokenizer are in the HF store at `/workspace/w2-inputs/2026-10-25/driver-data` on the pod, by deepseek-5's pins. Not cancelled.

Classed mechanism-instrument: the served graph's float stand-in is wrong, and the mechanism (training through the served grid) is untested until it is right.

## Files in scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs` (only if the profile puts the served step's cost in the per-step export rounding)
- `crates/uor-r4-integer/src/fbs_snapshot.rs` and `crates/uor-r4-integer/src/stack/tests.rs` (added while building: the float crate cannot see `stack`, which compiles only for serving and tests. So the activation exponent is a snapshot constant, `FBS_ACTIVATION_EXP`, which a stack test holds equal to `PRODUCT_EXP`, as `FBS_FIXED_EXP` is held to `RESIDUAL_EXP`.)
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/claude-262/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **The quantizer is the integer one.** `activation_quantize16` scales by `2^−ACTIVATION_EXP` before the integer formula. `ACTIVATION_EXP` is `fbs_snapshot::FBS_ACTIVATION_EXP`, which a stack test holds equal to `PRODUCT_EXP`; the same test runs `stack_quantize16` on a realistic row. Its scale comes from at most six exact comparison passes, not 49. A laptop host test pins the scalar reference to `stack_quantize16`'s formula on real activations: row maxima from 2^−20 to 2^31, every power-of-two boundary, and the saturated end. **With the fix set aside, that test fails.**
2. **On CUDA (I-0 on the pod):**
   - (a) the tensor quantizer equals the scalar reference value for value, with the straight-through gradient exact;
   - (b) a model-level check: one served training forward and backward gives every layer's `mlp.down.weight` and `rec.out.weight` a gradient whose norm is within 4× of the plain forward's, and never 0.

   **With the fix set aside, both fail.** Run on the pod, with the build line read.
3. **The cost is measured, not assumed.** A profiled served run (`--profile`) at the fixed head names each stage's share of the served step. Any stage that alone takes the served step past 2× plain is repaired in this unit if it is in scope, or named with its cause if it is not.
4. **deepseek-5's sheet reruns at the fixed head** (the four registered jobs, and the corrected evaluation step of deepseek-10). Its own bar reads the result:
   - instrument door: both served checkpoints byte-identical after `fbs-snapshot --round-as-exported`;
   - operational door: served at most 2× plain time;
   - quality smoke.

   Decode tokens/s are reported first.
5. REPORT negatives first, one notebook row, and the item's "What is missing" line updated.

## Stop and ask if

Each of these is decided in the unit, never waited on:
- **The fixed served arm is still materially below plain.** Classify instrument against mechanism (§4.2) before any claim. Then turn the remaining surrogates (D11 snaps, Q16 residual, Q12 query) off one at a time in a finite CUDA gradient check, and name the next cause.
- **The rerun needs more than one pod or $3.** Re-scope in a new ORDER.
- **A fix would change the exported artifact's format or the integer session.** Stop: the float stand-in must match the integer graph, not the reverse.
