# ORDER: writer-admission, claude-290

Lab: claude. Item: [writer-admission](../../../../roadmap/items/writer-admission.md). Base: `4ea097661`.

## Claim check

- **Plan step:** P6, its second unit (the bound is 2 units). Writer admission, #3394, is on Casey's call of 2 Nov: "Yes, add both".
- **What it changes in the model:** the shared vector table, which is the input embedding, the writer's table and nearest's table at once. Under a new training option, the table's two port blocks are held at each token's cell roots. The writer reads its 8 cells from those same blocks. So nearest's own objective trains admission, and a writer that holds lets serving drop whole-table nearest. That is 484 of 744 µs a step at short context (claude-288's parts).

## Goal

Done-when 2 of the item, and only that. The run is done-when 3, in the next unit.

> 2. **A geometric change to admission,** for example a margin the 16 roots' spacing allows, cells kept by the target's own ports, or a kept count chosen by the margin. It ships with host tests, and with CUDA tests where tensors are involved.

**What claude-289 found** ([REPORT](../claude-289/REPORT.md)): at seed 2, both query ports fell to the roots' centre. Their mean |x| went from `2.2e-5` to `5.4e-7`, while the rest of the query grew to about 4. There all 16 roots tie and admission is a lottery. So the change must be one the tie point cannot satisfy.

**The change: anchored ports** (`--writer-anchor`, off by default).
- Token `t` lives in cell `c = t mod 256`. The writer's address score reads port A (coordinates `0..8`) against root `c / 16`, and port B (coordinates `W-8..W`) against root `c mod 16`.
- With the option on, the table's two port blocks for token `t` are set to exactly those two SpiralCore roots. This happens once when the model is built, and again after every optimizer step: a projection, so weight decay and gradients cannot move them. The other `W - 16` coordinates stay learned.
- **What anchoring changes:**
  - With anchored ports, the writer's address score for cell `c` is the ports' share of the mean L1 distance from the query to every token in `c`. Up to the factor `8 / W`, it is the part of nearest's own score that the cell fixes.
  - So the recipe's own token objective (`cell` or `nearest`) is what trains admission. No separate address hinge is left to lose, unlike claude-150's `voronoi`. And the table's ports cannot fall to the centre together with the query's.
- **What it claims, as the design review restated it:** the centre is not stationary for a row whose trunk predicts the target's roots.
  - For a port coordinate where the `cell` hinge is active, the query's gradient is `(sign(q − t) − sign(q − r)) / W`, for the target's value `t` and the best rival's value `r`.
  - That gradient is nonzero only when `q` lies strictly between them. At `q ≈ 0`, a target at ±2 against a rival at ∓2 gets the full pull. A target at ±2 against a rival at 0 gets it about half the time. A rival that shares the target's root gives none.
- **What it does not claim:**
  - **An uninformed trunk.** Averaged over rows whose ports do not depend on the target, the signs cancel, and the centre is still a fixed point in expectation. That is the caution in claude-289's Still open, item 1.
  - **Seed 2's timing.** Seed 2 had already fallen there before step 2,048, while nearest read 76. The run in done-when 3 decides whether anchoring holds, and must read the ports before step 2,048.
- **Decay on the query side is left as it is.** `decayed()` (`geometric_stack_native.rs:784`) decays every rank-2 `.weight` by `1 − lr · 0.1` a step, with or without a gradient. That includes the trunk's maps that produce the query's port coordinates.
  - **Checked against claude-150's schedule, decay alone does not explain the fall.** Its lr ran from 0.0005 at step 0 down to 0 at step 32,768, linearly.
    - From step 4,096 to 32,768 the sum of the lr is 0.0005 × 12,544 ≈ 6.27.
    - So decay scaled each decayed weight by about e^(−0.1 × 6.27) ≈ 0.53, about 1.9×.
    - The ports fell about 40× over the same steps (`2.2e-5` to `5.4e-7`), so most of the fall came from the gradient: the L1 pull toward the median of the targets' port values.
  - The anchor protects only the table side. Done-when 3's sheet reads the query ports' |x| before step 2,048, with a stop rule. Excluding the port-producing weights from decay is held in reserve, not as the expected fix.
- **Where the code lives:** `in_space_long.rs`, not `native_vector_writer.rs`. Editing the writer's file moves `source_sha256` in its `binding()` (line 47), which existing writer checkpoints carry. The anchor builds the roots from the same `ProjectionPlan::new(8, 8)` the writer reads, at scale 1. A CUDA test pins that the two agree: an anchored token's vector, read as a query, scores 0 on its own cell.
- **Gradients and steps:**
  - The table's gradient is masked to 0 on the port columns before the main step. Otherwise the clip norm (taken over all gradients) and the Adam moments would be spent on columns that are overwritten anyway.
  - The projection runs after `opt_main`'s step. `opt_decay` steps only `rec.decay` parameters, never the table.
  - Train-as-served restores the float values before the step (`in_space_long.rs:11644`), so the projection sees float values. Q12 holds 0 and ±2 exactly, so the rounded and exported tables keep the roots.
- **A known effect on the input side:** the table is also the input embedding. Its learned coordinates start at standard deviation 0.02 (`INITIAL_STD`), against roots at 0 and ±2. At the start the trunk's input is mostly the cell code, so 16 tokens share one code until the learned coordinates grow. Done-when 3 measures what this costs nearest.
- **Held for later:** a kept count chosen by the margin, as a safety net when the spread is under the roots' spacing. It needs the writer's file, and `validate()` caps it at 16 of 256 cells, so it cannot reach nearest's result alone.
- **Refusals:**
  - at width 8 the two ports are one block (`query_ports` aliases), so the option needs width ≥ 16;
  - the option refuses a model without the shared `embedding.weight`.
- **Recorded:** the option goes in `config.json` and is left out when off, so every existing run writes the same file as before.

## Cancel probe (run first)

1. No change to the writer's admission on `origin/main` since claude-289's evidence base: `git log 074d54b2..origin/main -- crates/uor-r4-float-train/src/native_vector_writer.rs` is empty, and no `writer_anchor`, `anchor_ports` or `port_anchor` appears in the float crate.
2. The item is still `ready`, with done-when 2 open, and no other open order holds its files (the claim checked overlap).
3. Serving still reads whole-table nearest, so the writer's cost case stands (claude-288 #3717, its parts).

All three are instrument checks of the project's state, not mechanism results.

**Result, 5 Nov ~13:25Z, at `origin/main` `4ea097661`: proceed.**
1. The log is empty: 0 commits touch `native_vector_writer.rs` since `074d54b2`. 0 lines in the float crate's source name `writer_anchor`, `anchor_ports` or `port_anchor`.
2. The item reads `status: ready`, and the claim's overlap check passed (draft PR #3727).
3. `fbs-export.rs` lines 23–24 still serve a `cell` or `nearest` run with the whole table's nearest readout.

## Files in scope

- `crates/uor-r4-float-train/src/native_vector_writer.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `docs/harness/prep/writer-admission*`
- `docs/loop/tasks/writer-admission/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/writer-admission.md`

## Done when

1. **The anchor, in `in_space_long.rs`.**
   - Each token's two port roots as a table-shaped tensor and a column mask, built from the writer's own `ProjectionPlan::new(8, 8)` roots.
   - The projection that writes them into `embedding.weight`, and the mask applied to the table's gradient before the main step.
   - All elementwise and CUDA-only. Refuses width < 16 and a model without `embedding.weight`.
   - `native_vector_writer.rs` is not edited.
2. **The option.** `LongConfig::writer_anchor` and `train-in-space --writer-anchor 0|1`:
   - applied at model build, and as the gradient mask and post-step projection at every `opt_main` step in both training loops (`in_space_long.rs` around 10797 and 11654);
   - recorded in `config.json` only when on.
3. **Host tests:**
   - the cell-to-root layout, for every cell, against the writer's own address order (cell `16 a + b`);
   - the width-8 refusal;
   - with the option off, the field is not written; with it on, it is written as `true` (`LongConfig` only serializes, and nothing reads the field back, since a saved table already carries the anchors).
4. **CUDA tests** (on a pod, finite kernel tests; no pre-registration needed, CHARTER §4.5):
   - after one training step with the option on, the port columns equal the roots exactly, and some learned column moved;
   - the table's gradient is exactly 0 on the port columns after the mask;
   - a query equal to token `t`'s anchored vector puts `t`'s cell first in the writer's `candidates`, with address score 0;
   - the port gradient at the centre for an informed row: with the query's learned coordinates equal to the target's and its ports at 0, the `cell` hinge's gradient on the query's ports is nonzero and points toward the target's roots;
   - negative gates: with the projection's call set aside, the port-equality test fails; with the mask set aside, the zero-gradient test fails.
5. **Gates at the head:**
   - the float crate's CPU suite with `--no-fail-fast`;
   - the stub-nvcc `--features cuda --all-targets` check;
   - the CUDA tests' log on the pod, sealed in the store;
   - `uor-harness check`, `loop gates --full`, `loop report --check`, `loop pr-check --local`.
6. **The record:**
   - a REPORT, negatives first;
   - the item's "What is missing" updated;
   - the item stays `ready` (done-whens 3 and 4 remain).

## Stop and decide if

- **The projection needs a file outside the scope** (`geometric_stack_native.rs`, the exporter): keep the projection in the training loop, which is in scope, and note what the exporter reads. Never widen the scope by hand.
- **Train-as-served quantizes the table so the roots are not exact:** project after the quantizer, or refuse the option with train-as-served, whichever keeps the served values equal to the roots; record which.
- **The fresh design review finds a way the tie point still satisfies the objective for an informed trunk:** fold its fix into the design before building, or record why not.
- **The CUDA tests need more than one short pod** (about $0.30): note it and keep going.
