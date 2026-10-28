# Prep sheet: one training step, profiled by stage (fbs-cost-profile, claude-157, 28 Oct 2026)

Lab: claude. Item: [fbs-cost-profile](../../roadmap/items/fbs-cost-profile.md). Order: [claude-157](../../loop/tasks/fbs-cost-profile/claude-157/ORDER.md), with Casey's words verbatim. Code: this unit's head, built and tested on the pod from that exact commit.

TEST FITNESS: FIT, round 1. PRE-RUN: PASS. Both by one fresh non-author reviewer, at `4251b4209`. Its five notes, none blocking, are applied below and in the code at the run's head:
- **The sum bar nearly holds by construction.** The stages tile the GPU span from the first mark to the last, and both spans start and end with the GPU idle. So `profile.json` now also gives the median profiled step's host time against the unprofiled one (`clock_overhead`). That is the check that the clock does not slow what it times.
- **"Backward" holds every stage's gradient.** `profile.json` adds `top3_forward`, ranked among the forward and objective stages only, and the unlock rules read that list.
- **The launch-or-GPU call** needs host time at least 0.9 of GPU time, and leaves out `objective.readback`, whose host time is the wait for the whole forward.
- **Two checks are arithmetic, not checks.** The pairing count in I-3 and the 1% rule cannot fail, so the REPORT gives them as arithmetic.
- **The table-search stage also holds its setup:** the table's read, the target upload and the mask. The REPORT says so.

## The question

**Where does one training step's time go?** The owner, 28 Oct: "I don't care what we serve at long as it is cheap".

Every training cost on record is an end-to-end rate: (a) at `32,049` context tokens/s and width 512 with no read at `38,578` (float-training notebook, claude-152 and claude-153). No run splits a step by stage. The suspects are:
- the six-layer recurrence, which steps through 192 positions in order (suspected to be bound by kernel launches);
- the cell objective's search of the whole table for the nearest wrong token, about `1,048,576` differences per scored row by arithmetic;
- the reads.

This run measures which it is. It trains nothing that is kept and scores nothing.

## Steps: the arms

Both arms run `train-in-space --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --lr 0.001 --lr-end 0 --seed 1 --steps 72 --eval-every 72 --profile 32` on the pinned TRAIN, one arm at a time, on one RTX 5090:

| Arm | Extra flags | Recipe |
|---|---|---|
| (a) | `--hamming-read 4` | claude-152's (a): width 256, 4 Hamming heads |
| (w0) | `--width 512 --hamming-read 0` | claude-153's (w0): width 512, no read |

- **The window.** Steps 0–7 are warm-up. Then steps 8, 10, … 70 are profiled, and each is followed by an unprofiled step.
- **A profiled step** records a CUDA event (timing on) and the host's clock at each stage boundary, with no synchronization between them. The boundaries are:
  - the embedding;
  - each layer's recurrence and feed-forward;
  - each read;
  - the final norm and the readouts;
  - the selection of scored rows;
  - the objective's table search and hinge;
  - the loss's copy to the host;
  - the backward;
  - the three optimizer updates.
- **An unprofiled step** is timed as every training step is: from its start through `device.synchronize()`.
- **Outputs:** `profile.jsonl` gets one row per profiled step; `profile.json` gets each stage's median, group sums, the sum against the median unprofiled step, and the top three.

## Bar (pre-registered)

The item's done-when 1, per arm:
- **Sum:** `stage_sum_ms / median_unprofiled_ms` within `[0.90, 1.10]` (`within_10pct` in `profile.json`).
- **The clock's own cost:** `clock_overhead`, the median profiled step against the median unprofiled one, within ±10%. Outside it, the stage times are labelled as distorted by the clock.
- **Top three:** two lists, each of the three stages with the largest median GPU time:
  - `top3`, over all stages;
  - `top3_forward`, over the forward and objective stages only.
- **Launch or GPU:** for each stage in either list, its median host time beside its GPU time.
  - Host time at least 0.9 of GPU time: read as waiting on launches.
  - Below that: read as waiting on the GPU.
  - `objective.readback` is left out, because its host time is the wait for the whole forward.

A sum or an overhead outside its band is a miss. It is reported with its ratio, and the stage times are still given, labelled with the miss. It is not a stop.

## Instrument checks

- **I-1.** The crate's CUDA tests pass on the pod's build. That includes `the_stage_clock_times_a_step_without_changing_its_loss`: one batch's loss is identical, bit for bit, with the clock off and on, and every stage reads a GPU time. The test count is recorded.
- **I-2.** Each run's `config.json` holds `"profile": 32`, its arm's width and head count, and the pinned TRAIN (`"train_split": null`).
- **I-3.** `profile.json` holds 32 profiled steps, each with an unprofiled step after it.
- **I-4.** The median unprofiled step is compared with the training rate the arm's recipe ran at: (a) `32,049` and (w0) `38,578` context tokens/s, which is `95.9` and `79.6` ms per step of `16 × 192` tokens. A gap over 20% is reported (another pod, another day), not a stop.

## Data split

- **TRAIN:** the pinned `mw-balp` (`PINNED_TRAIN_*`), checked by sha256 at load.
- **DEV:** the pinned DEV, read only by the two checkpoints, steps 0 and 72.

Nothing is scored against a bar, and no model is kept beyond the run's folder.

## Base artifact

None. Each arm starts from its seed-1 initialization, as (a) and (w0) did.

## Minimum test power

Each arm gives 32 profiled steps and 32 unprofiled ones.
- Every number is a median over 32 steps, so one slow step moves none of them.
- The spread of the 32 unprofiled times is printed with the median.
- A stage under 1% of the step is reported but not ranked against another under 1%.

## Cost

- **Hardware:** one RTX 5090 on Vast, at about $0.40–0.70/h.
- **Time:** about 40 minutes: build and CUDA tests about 20, two arms of 72 steps and two checkpoints each about 10.
- **Total:** under $1.
- **Lease:** 2 hours, inside the caps (4 pods, $8/h).
- **Results:** sealed to `results/claude/fbs-cost-profile-2026-10-28/` with `SHA256SUMS`, and read back before `down`.

## Stop rule

- **I-1 fails.** Stop: the clock changed the computation, or the build is not the head's.
- **I-2 or I-3 fails.** Stop: the run is not the registered one.
- **The pod fails or the lease would pass the caps.** Stop, seal what exists, and report it.

Nothing else stops the run.

## What each result unlocks

- **The table search is in `top3_forward`:** the next training unit makes it cheaper. Candidate changes:
  - a coarse search over the 256 writer cells;
  - fewer rivals;
  - a search made every few steps.
- **The recurrence is in `top3_forward` and launch-bound:** fusing its kernels is the cheaper change.
- **The backward dominates:** the next step is a profile of the backward's own stages, with the driver's profiler.
