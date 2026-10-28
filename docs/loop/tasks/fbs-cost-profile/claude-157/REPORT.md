# REPORT: fbs-cost-profile, claude-157

Lab: claude. Order: [ORDER.md](ORDER.md). Sheet: [fbs-cost-profile-2026-10-28](../../../../harness/prep/fbs-cost-profile-2026-10-28.md) (TEST FITNESS: FIT, PRE-RUN: PASS). Run head: `78bbc883fab77f63ab417fb481a9bd726a5dfda8`.

One training step is now split by stage, at claude-152's (a) recipe and at width 512 with no read.
- **What dominates:** the backward (62% / 54% of a step), then the cell objective's whole-table search (13% / 34%), then, in (a), the two Hamming reads (16%).
- **The six-layer trunk** is 5% / 9% of a step.
- **The serving half** of the item (done-when 2) goes to its next unit.

## What did not work, first

- **The serving profile could not be built in this unit.** Its stage-timed FBS step must live in `crates/uor-r4-integer/src/stack/session.rs`, which the item's scope did not list. Adding it was blocked twice:
  - the loop guard and `loop suborder` read an item's scope from `origin/main`, so my edit to the item's `scope:` line in this branch did not count;
  - `loop claim infra-loop-harness` refused to claim the item-file edit separately, because its `docs/roadmap/items/*.md` overlaps this unit's open claim of the item file.

  So this unit lands the scope line, and the item's next unit builds the serving profile once it is on main.
- **The builder subagent wrote nothing.** `loop suborder` wrote its order into the worktree's `local/claude/claude-157/orders/`, but the guard looks in the main checkout's `local/`. The builder also reported that it started at about 130k tokens of context, near its 150k hand-back mark. So the instrument was written in the top session.
- **Clippy** flagged `(step - PROFILE_WARMUP) % 2 == 0` (`manual_is_multiple_of`). It is now `.is_multiple_of(2)`.
- **The FIT reviewer found that the sum bar nearly holds by construction.** The event-to-event stages tile the GPU's span, and both spans start and end with the GPU idle. The reviewer's fix is in the run, before it started: `clock_overhead`, the profiled step's own host time against the unprofiled one. It is the check that the clock does not slow what it times.

## What changed

- **`crates/uor-r4-float-train/src/geometric_stack_native.rs`:** `pub mod stage_clock`.
  - It is thread-local and off unless started. When off, `mark` only checks the thread-local: no name is built and nothing waits.
  - When on, each mark records a CUDA event with timing (`CU_EVENT_DEFAULT`) on candle's stream, and the host's `Instant` beside it.
  - `finish` reads the event gaps.
  - The forward pass marks: the embedding, each layer's recurrence and feed-forward, each read, the final norm and the readouts.
- **`crates/uor-r4-float-train/src/in_space_long.rs`:**
  - marks for the scored rows' selection, the objective's table search and hinge, the loss's copy to the host, the backward and the three optimizer updates;
  - `LongConfig.profile` (left out of `config.json` when 0), `profile_window`, `ProfileRow` and `summarize_profile`;
  - `profile.jsonl` and `profile.json`;
  - refusals of a window longer than the run, and of a device that is not CUDA.
- **`crates/uor-r4-float-train/src/bin/train-in-space.rs`:** `--profile K`.
- **Tests:**
  - host: the window, the summary (medians, group sums, both bars missed in each direction, the clock's overhead) and the config key;
  - CUDA: `the_stage_clock_times_a_step_without_changing_its_loss`, which checks that the loss is bit-identical with the clock off and on, and that every stage reads a GPU time.
- **`docs/roadmap/items/fbs-cost-profile.md`:** `session.rs` added to `scope:`, for the next unit.

## Measurements

**The run:** one RTX 5090 (Vast `vast-58147266`, driver 580.178.04), seed 1, pinned TRAIN.
- **Arms:** 72 steps each; steps 8, 10, … 70 profiled, each against the unprofiled step after it.
- **Binary:** sha256 `efb94adcab029adccc1501c82b23dd601514fa5fa230cf1d7448726d71f770d7`, built and tested on the pod from the run head.
- **Pod:** about 10 minutes, about $0.10.

**Instrument checks:**

| Check | (a) | (w0) |
|---|---|---|
| I-1 CUDA tests on the pod's build | `149 passed; 0 failed; 5 ignored`, the clock's test among them | (same build) |
| I-2 `config.json` | profile 32, width 256, 4 heads, pinned TRAIN | profile 32, width 512, 0 heads, pinned TRAIN |
| I-3 profiled steps | 32 | 32 |
| I-4 training rate against the recipe's | `31,718` against `32,049` (−1.0%) | `38,809` against `38,578` (+0.6%) |
| Stages sum against the unprofiled step | `96.74 / 96.54` ms = `1.002` | `78.91 / 78.98` ms = `0.999` |
| Clock overhead | `+0.9%` | `+1.2%` |
| Unprofiled step range (32 steps) | `93.3–100.1` ms | `72.5–85.5` ms |

The pairing count in I-3 and the 1% rule are arithmetic, not checks (the FIT reviewer's note): a row is written only once it is paired.

**Stage medians, GPU ms (host ms in brackets), over 32 profiled steps:**

| Stage | (a) width 256, 4 heads | (w0) width 512, no read |
|---|---|---|
| embedding | 0.43 | 0.68 |
| each recurrence (×6) | 0.55–0.56 (0.55; the third layer's, index 2, right after the read: 1.69) | 0.84 (0.83–0.84) |
| each feed-forward (×6) | 0.17–0.18 | 0.20 |
| Hamming read in the stream, after the second layer (index 1) | **7.95** (6.82) | — |
| final norm | 0.04 | 0.04 |
| Hamming readout | **7.94** (6.81) | — |
| scored rows' selection | 0.01 | 0.01 |
| objective's table search | **13.00** (0.75) | **26.61** (1.11) |
| objective's hinge | 0.08 | 0.11 |
| loss copied to the host | 0.02 (13.25) | 0.01 (25.59) |
| backward | **60.44** (59.54) | **42.90** (41.95) |
| main optimizer | 1.65 (2.51) | 2.18 (3.06) |
| read optimizers | 0.59 | 0.00 |
| decay optimizer | 0.22 | 0.15 |
| **sum** | **96.74** | **78.91** |

**By group, GPU ms and share of the sum:**

| Group | (a) | (w0) |
|---|---|---|
| forward | 20.74 (21%): the trunk 4.80, the reads 15.89, the final norm 0.04 | 6.94 (9%): the trunk 6.90 |
| objective | 13.11 (14%) | 26.73 (34%) |
| backward | 60.44 (62%) | 42.90 (54%) |
| optimizer | 2.46 (3%) | 2.34 (3%) |

**The top three, with the launch-or-GPU call** (host time at least 0.9 of GPU time is a host-bound stage; the loss's copy is left out):
- **(a), `top3`:**
  1. **backward:** host 0.99, so host-bound;
  2. **table search:** host 0.06, so GPU-bound;
  3. **the stream's Hamming read:** host 0.86, so GPU-bound, near the line.

  `top3_forward`: the table search, then the stream's Hamming read, then the Hamming readout.
- **(w0), `top3`:**
  1. **backward:** host 0.98, so host-bound;
  2. **table search:** host 0.04, so GPU-bound;
  3. **main optimizer:** host 1.40, so host-bound.

  `top3_forward`: the table search, then two recurrences at 0.84 ms each (host 0.99, so host-bound).

The table-search stage holds its setup too: the table's read, the target upload and the mask (the FIT reviewer's note). The loss's copy has a host time of 13 ms (26 ms in (w0)). That is the host waiting for the GPU to finish the forward. The host ran ahead of the GPU in the Hamming reads, the final norm and the table search; elsewhere in the forward the two keep pace.

**The cheapest change for each top cost** (item done-when 3):
1. **The backward** (60.4 / 42.9 ms).
   - Its host time is 0.98–0.99 of its GPU time, so it is host-bound by the 0.9 rule. That fits two causes: the host launching kernels while the GPU keeps pace, or the host waiting at a sync inside the backward. This run does not tell them apart.
   - Which of its parts dominates is not measured here. The cheapest change can be named only after a kernel-level profile of the backward with the driver's profiler, which the sheet named as this result's next step.
   - A hint, as a hypothesis: (w0)'s backward is 17.5 ms less than (a)'s, at twice the width with no read. That suggests the reads' backward is at least that large. The two arms differ in two ways, so this is not a measurement.
2. **The objective's table search** (13.0 ms at width 256, 26.6 ms at 512).
   - It is GPU-bound, and it grows with the table, 4,096 × width.
   - **The change:** search the rivals among a subset instead of the whole table, for example the tokens in the writer's kept cells, the set the served writer already scores. Or search every few steps and keep the rivals.
   - **Effect on prediction:** a hypothesis. A rival set that misses the nearest wrong token weakens the hinge; it must be run against (a).
3. **(a)'s Hamming reads** (7.9 ms each, 15.9 ms in all).
   - They are GPU-bound, and they cost the same at two different places in the model. That suggests a shared part dominates both, for example the comparison of each position with every earlier one.
   - **The change:** compute that once per forward and share it, where the two reads use the same keys. If the shared part dominates, that at most halves their cost.
   - **Effect on prediction:** none if exact, checked as an identical loss. That the shared part dominates is a hypothesis.

   For (w0) the third cost is the main optimizer (2.2 ms, 3% of the step), host-bound. Fusing its per-parameter updates is the change. Prediction is unchanged if exact.

The recurrence meets the host-bound rule (host 0.99 of GPU time), as suspected, but at 0.55–0.84 ms a layer, all six together are 3–6% of a step. It is not where training's time goes.

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-cost-profile-2026-10-28/`
- `claude-157-78bbc883.tar` (sha256 `3a057e7ca8921474d54e4c815a5bdd80cef688026e3ae5217a75c7b835eced61`, in `SHA256SUMS-claude-157`), read back with the same sha256;
- `job.sh` and `watch.sh` beside it.

## Checks run at the head

- **`cargo test -p uor-r4-float-train --lib -- in_space_long`** (host): `21 passed; 0 failed`, the three new host tests among them.
- **CUDA tests on the pod's build at the run head:** `149 passed; 0 failed; 5 ignored`.
- **`cargo clippy -p uor-r4-float-train --no-deps --all-targets -- -D warnings`:** no finding in the lines this unit wrote. Four findings predate it (`in_space_long.rs` `decode_speed`, `spiralcore_fbs_sheet.rs:3506`, `flock_read_register.rs:360` and `:541`).
- **`cargo fmt --check`:** clean.
- **`loop report --check`:** PASS. **`loop gates --full`:** 11 of 11 green.
- **The fresh in-unit review, in two parts at `2c0f4d15a`:**
  - **The code:** APPROVE, six low findings. Four are fixed after the run; none touches a measured path:
    1. `stage_clock::reset()` at `run_long`'s start, so a step that failed mid-way leaves no clock on;
    2. checked arithmetic for the window's end, so an absurd `--profile` is refused, not wrapped (a new host test covers it and the other two refusals);
    3. the doc of the `end` stage;
    4. a note that the sheet's and Voronoi objectives count in `objective.readback`.

    Two are left as notes. `summarize_profile` takes stage names from the first row, which holds while every row has the same stages. And a checkpoint inside the window would precede an unprofiled step; this run's `--eval-every` put none there.
  - **The numbers against the sealed outputs:** CHANGES, two medium and four low findings, all fixed in the text:
    - the backward's share is 62%, not 63%;
    - "host-bound" now names both causes it fits;
    - the host-ahead sentence names the right stages;
    - the layer numbering is consistent;
    - the shared-read saving is stated as at most half, if it holds;
    - 25.6 ms is rounded to 26.
- **The code at the PR's head** differs from the run head `78bbc883` only by those four fixes and the new host test: `22 passed; 0 failed` on the host, clippy unchanged.

## Skills used

- **holding-the-goal.** The owner asked for the review of "what exactly is costing us speed and compute time". The run measures training's stages and nothing else. The serving half goes to the next unit rather than being approximated.

## Still open

- **The served token's profile** (item done-when 2), in the item's next unit. `session.rs` is in the item's scope once this merges. The plan is in the ORDER: a stage-timed twin of `stack_step_fbs`, and a whole-table nearest timed beside the writer.
- **The backward's split,** with the driver's profiler.
- **The three cheaper changes above,** each needing a run against (a) for its effect on prediction.

## Done when

1. **Met.** The three instrument checks pass, and their outputs are above.
2. **Met.** Two recipes are profiled on an RTX 5090. Each stage is summed against the measured step time (`1.002`, `0.999`), and the top three are named.
3. **Met for training.** There is one line for each top cost: measured where measured, labelled hypothesis otherwise.
4. **Met.** The sheet had a fresh FIT before the run, and there is a notebook row and this REPORT.
5. **Met on merge.** The outputs are sealed and read back.

## Plain summary

A training step takes about 97 ms at width 256 with the reads, and 79 ms at width 512 without them.
- **The model's own six layers** take only about 5 ms of that.
- **Most of the time** goes into the backward pass (60% or so). After that comes the cell objective's search of the whole 4,096-token table for the nearest wrong answer, then the two Hamming reads.
- **Doubling the width** costs time almost only through that table search.
- **What to make cheaper first** is therefore the objective's search and the backward, not the trunk.

The serving side is measured next.
