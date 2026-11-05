# REPORT: float-kernel-fusion, claude-283 (the measurement before fusing)

Lab: claude. [ORDER](./ORDER.md). PR #3685, item #3678. Plan step: THE ORDER of 4 November (#2028 6634218223), step 5, the kernel unit, part (c). Base `1f6570d59`. The code is `56fc29438`. The run used the pin `e7007ff1d`, which is that code plus the sheet, on one RTX 5090 (Vast `vast-59182275`).

**In one paragraph.** The item's cancel rule does not fire, so the fused kernels (done-whens 2 and 3) are the next units.
- **The measurement:** at P2b's recipe with `--train-as-served 1`, skipping the two served quantizer chains shortens a training step by **7.5% at (S)**, 192 × 16 (noise 0.46%), and by **2.6% at (L)**, 2,048 × 4 (noise at its 0.25% floor).
- **The decision:** (S) clears the bar's Go margin (`share − 5% > 2 × noise`), so the sheet's decision is **GO**. (L) alone would have cancelled.
- **Where the time is:** almost all in the forward, about 1.2 ms in each of the 12 sublayers at (L) and 0.7 ms at (S), plus 1.9 ms and 1.0 ms in the backward.
- **The checks:** every instrument check passed (I-0 to I-5).
- **What it changes in the model:** nothing. The switch is off unless `--time-without-quantizers 1` arms it, and an armed run is timing only.

## Tokens per second first

Training throughput, read from the unarmed runs U, and the upper bound of what removing the chains entirely would give:

| Arm | Positions a step | U: median clean step | Positions a second | Without the chains (T, skipped pairs) | Positions a second |
|---|---|---|---|---|---|
| (L) 2,048 × 4 | 8,192 | 646.36 ms | 12,674 | 630.17–630.50 ms | 12,993–13,000 (+2.6%) |
| (S) 192 × 16 | 3,072 | 124.34 ms | 24,707 | 114.88–115.08 ms | 26,695–26,741 (+8.1%) |

- **This is an upper bound.** A fused kernel still reads and writes each tensor once per call; skipping does neither. The fused kernels' gain is below these figures, and done-when 4 measures it.
- **Decode was not timed here.** The item notes that a cached decode step costs an estimated 8–9 ms, most of it fixed per forward, and that the forward runs these same chains. At one position a forward, the chains' roughly 1,240 tensor ops a forward (see Measurements) would be launch-bound. That is a reason to expect a larger share in decode, but it is an estimate. Done-when 4 times the cached decode step.

## What did not work

1. **(L) alone would cancel.** At 2,048 × 4 the chains take 2.6% of a step, under the 5% bar with margin. The Go rests on (S), the short window. A forward makes the same 62 chain calls at both shapes, and the (S) step is about 5 times shorter, so a fixed cost a call would weigh more there; that is an inference, not a measurement. Training at long windows gains little from the fusion.
2. **The stage clock cannot see this saving, and it came close to the ORDER's stop rule.** On profiled steps, the skipped class is not faster. Its clock overhead (`step_ms / next_unprofiled_ms − 1`) is 2.48% at (L) and 3.89% at (S) in T1 (2.47% and 3.91% in T2), against 0.10% and 0.13% for T1's kept steps.
   - At (L), T1's profiled skipped steps have a median `step_ms` of 645.81 ms, while their stages sum to a median 629.40 ms of GPU time and 629.41 ms of host time. About 16 ms lies outside every stage, and this run does not say what it is.
   - The ORDER's stop rule asks for unprofiled steps only if the overhead differs between classes by more than the share. The difference was 2.38% against a share of 2.58% at (L), and 3.76% against 7.51% at (S), so the rule did not fire.
   - The decision uses unprofiled steps only, as the sheet pre-registered.
   - What it suggests, as an inference: under the stage clock, a skipped step waits on something outside the stages' marks, such as the clock's own syncs or host work between stages. claude-157 found its backward stage host-bound (host 0.98–0.99 of GPU time), which fits, but neither run tells launch overhead from a wait.
3. **The launch counts are estimates from the source, not counts of kernels.** No `nsys` ran. The calls and elements a forward makes are counted by the switch. The ops a call makes are read from `hamilton_sheet.rs`, and a candle tensor op is at least one kernel.
4. **One seed, one pod, one card.** Seed 1, an RTX 5090 (driver 580.178.04), one run of each arm and class. T1 and T2 agree within 0.16% (S) and 0.05% (L), and U's halves within 0.46% and 0.03%. Nothing was run on the 4090.
5. **The bar is not the ORDER's.** The ORDER's done-when 5 reads Go at "at or above 5%" and takes the noise from kept pairs split into halves. The FIT-reviewed sheet asks for the margin both ways, and takes the noise from T1 against T2, U's halves, and a 0.25% floor. By the ORDER's wording the decision is the same: (S) is at or above 5%.
6. **The checkpoints were stored with the seal.** The six runs wrote 1.06 GB of checkpoints, 0.71 GB of them from the four armed runs, which are timing only and not results. They are in the store because the seal covers every file the job wrote.
7. **Eight CUDA tests were not run with this unit's code.** I-0 ran the crate's CUDA library suite with the sheet's named tests, and 8 tests stayed ignored:
   - `flock_read_hamming::tests::the_read_at_long_windows`, a GPU timing run that no recent unit has run;
   - the six `spiralcore_fbs_arms::gpu_tests::*` and `in_space_long::tests::three_steps_trained_as_served_save_a_step_the_export_leaves_unchanged`, which claude-282 ran on a 5090 at `d6fc3a2c5` (its G1b and G1c), before this unit's code. The three-step test goes through the served path the switch sits in.
   That the switch changes nothing when it is off rests on the code review and on the CUDA timing test's check that a disarmed forward gives the unarmed values.

## What changed

- **`hamilton_sheet.rs`:** the timing switch.
  - **What it does:** while a run is armed with its window of K pairs, `q16_residual` and `activation_quantize16` return their input on the training forwards of the window's odd pairs. Pair j is steps `8 + 2j` and `9 + 2j`. Everywhere else they run as before.
  - **Where it decides:** in `with_served_surrogate_training`, from the forward's step. A nested forward inherits the outer forward's class.
  - **The record:** it counts each chain's calls and elements, kept and skipped apart. Its `check` refuses a step that is out of its pair or counted twice.
  - **Tests:** five tests, four on the CPU and one on CUDA (named in the sheet's I-0).
- **`train-in-space.rs`:** `--time-without-quantizers 1` arms the switch around `run_long` and writes `DIR/quantizer-timing.json`.
  - **It is refused unless:** the run trains, it is served, `--profile` is at least 2, and both `--steps` and `--eval-every` are above `8 + 2K`.
  - **Tests:** one CPU test of the refusals.
- **The prep sheet:** [`float-kernel-fusion-measurement-2026-11-05.md`](../../../harness/prep/float-kernel-fusion-measurement-2026-11-05.md).
  - **FIT:** at round 1 (#3678 6646042219), with its three should-fixes applied.
  - **Pre-run review:** GO (#3678 6646093750), with its one grader fix applied.
- **The item:** its "measured share" now cites this result. The cancel rule did not fire, so done-whens 2 and 3 are the next units.
- **The notebook:** one row.

## Measurements

The decision readings, with `K` and `S` the median clean-clock step (`next_unprofiled_ms`) of the kept and skipped pairs. The share is `[(K(T) − S(T)) − (K(U) − S(U))] / K(T)`.

| Arm | T1 K / S (ms) | T2 K / S (ms) | U even / odd (ms) | Share T1, T2 | Share | Uncorrected | R2 (T1 vs T2; U halves; floor) | Reads |
|---|---|---|---|---|---|---|---|---|
| (L) | 646.33 / 630.17 | 646.36 / 630.50 | 645.83 / 646.51 | 2.61%, 2.56% | **2.58%** | 2.50%, 2.45% | 0.25% (0.05%; 0.03%; 0.25%) | cancel |
| (S) | 124.36 / 115.08 | 124.37 / 114.88 | 124.34 / 124.31 | 7.43%, 7.59% | **7.51%** | 7.46%, 7.62% | 0.46% (0.16%; 0.46%; 0.25%) | **go** |

**The decision:** GO. (S) reads go, since 7.51% − 5% = 2.51% is more than 2 × 0.46%.

**Where the time sits (R3).** T1's median GPU time a profiled step, kept minus skipped, by group:

| Arm | Forward | Backward | Objective | Optimizer | Served round and restore |
|---|---|---|---|---|---|
| (L) | +15.06 ms | +1.88 ms | −0.19 ms | −0.04 ms | −0.02 ms |
| (S) | +8.60 ms | +1.02 ms | −0.04 ms | +0.02 ms | −0.01 ms |

- **The forward's share is spread evenly.** Each of the six layers' `mlp` and `recurrence` stages carries about 1.2 ms at (L) and 0.7 ms at (S).
- **The backward's share is small.** It is the straight-through masks' multiplies.
- **U's groups differ by at most 0.25 ms** between its classes.

**The counts (R4).** These come from `quantizer-timing.json`, a forward, and are the same in both classes (I-4).

| Arm | `activation_quantize16` | `q16_residual` (residuals and FBS projections) |
|---|---|---|
| (L) | 12 calls, 31,457,280 elements | 50 calls, 133,169,152 elements |
| (S) | 12 calls, 11,796,480 elements | 50 calls, 49,938,432 elements |

**Tensor ops a call,** read from the source at the pin (each is at least one kernel launch):
- **`q16_residual`** (`hamilton_sheet.rs:652-673`): 13. The grid rounding takes affine, affine, floor, clamp and affine. The straight-through mask takes ge, to_dtype, lt, to_dtype, mul, mul, sub and add.
- **`activation_quantize16`** (`:732-769`): 49.
  - the row's magnitude: abs, max_keepdim, affine and clamp;
  - the starting scale: ones_like and affine;
  - six passes of the shift search, each ge, affine, where_cond, mul and mul, so 30;
  - the quantize: broadcast_mul, affine, floor, clamp and broadcast_div;
  - the straight-through mask: 8.

So a training forward makes about 12 × 49 + 50 × 13 = 1,238 tensor ops in the two chains, an estimate. Fused, it would make 62, one a call.

**The other readings:**
- **Drift (R5):** T's kept medians are within 0.005% of U's median at (L), and within 0.03% at (S).
- **Chain-free stages (I-5):** the median difference, kept minus skipped, as a share of `K(T)`:
  - (L): −0.017% (T1) and −0.033% (T2);
  - (S): +0.247% (T1) and −0.019% (T2);
  - the floor is 0.5%.

**Instrument checks:**
- **I-0:** the crate's CUDA library suite at the pin: 348 passed, 0 failed, 8 ignored (listed in What did not work, item 7). The sheet's five named tests passed, and so did the sixth timing test.
- **I-1:** each of the six runs exited 0, with 64 profiled rows. The T runs each wrote a record holding 64 kept and 64 skipped steps.
- **I-2:** T1's, T2's and U's step-8 losses are the same text. At (L) it is `1.0041638612747192`; at (S) it is `1.0043549537658691`.
- **I-3:** T1's step-10 loss differs from U's: `1.0041394233703613` against `1.004138708114624` at (L). T1 and T2 agree at step 10 too, so training is reproducible from run to run.
- **I-4:** as above, the kept and skipped forwards make the same calls.
- **I-5:** as above, inside the floor.

**The run:**
- **Pod:** `vast-59182275`, an RTX 5090 at $0.563/h. The job ran 00:31:35Z–00:44:56Z. From the pod's start (00:26:38Z) to the last read-back file (00:49:24Z) is about $0.21; the delete came after that.
- **Binary:** SHA-256 `a575de0b7857b71bb21406d3482c18580ecffe9773d5482502776a1aa802cdd8`.
- **Seal:** `SHA256SUMS.files` `ab64d33c0ccf02e3a3e5a6968d63865f729062d920d27bd7ce8acdd5405de6ac`, 170 files.
- **Store:** `results/claude/float-kernel-fusion-measurement-2026-11-05/c283/`, 174 files, read back 170 of 170 OK before the pod was deleted.
- **Grader:** `local/claude/fusion-measure/grade-283.sh` and its two jq files.
  - Before the run, it was checked on six synthetic cases: GO, CANCEL, a leak, a near-5% inconclusive, an I-5 fault, and a noisy negative share.
  - The noisy case reads INCONCLUSIVE with the 2.5% noise cap, and read a wrong CANCEL without it.

## Checks run at the head

Run at `e81f34572`, the branch with `main` merged. The commits after it change this REPORT and the notebook row's cost only.
- **The code against the pin:** `git diff e7007ff1d e81f34572 -- crates/uor-r4-float-train` is empty. The `main` merge touched `crates/uor-r4-cpu-train` and docs only.
- **`uor-harness loop report --check --order <ORDER>`:** PASS.
- **`uor-harness loop pr-check --local`:** PASS, with three advisory warnings:
  - rule-sources: the sheet's `## Stop rule` heading;
  - read-path: the ORDER is 6,938 B;
  - filemap: four rows missing from `docs/start/FILEMAP.md`, none of them this unit's.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **`cargo test -p uor-r4-float-train --target aarch64-apple-darwin --offline --no-fail-fast`:** 285 passed, 0 failed.
- **The stub-nvcc check** (`cargo check --features cuda --all-targets`): clean.
- **Fresh exact-head reviews:**
  - code: APPROVE, with two nits (Still open);
  - claims, round 1: CHANGES, four should-fixes and six nits, all fixed in this REPORT: the host-bound wording, the checkpoint size, the ignored tests, and this section;
  - claims, round 2 on the delta: CHANGES, one should-fix (the 16 ms bullet's values were not the grader's medians) and two nits (the ignored-test count, the cost), fixed and checked against the run files by the author.

## Skills used

None beyond the loop's own steps.

## Still open

- **Done-whens 2 and 3, the fused kernels,** are the next units. Kernel development may use Casey's RTX 4090 (Linux) once it is up. The gates that count run on a 5090 (Casey, 4 Nov, #2028 6638209391). The 4090 needs its own PTX target.
- **Done-when 4 must time the fused kernels** against the chains, in training and in cached decode. This unit's 7.5% is the ceiling, not the gain.
- **About 16 ms of a profiled skipped step at (L) lies outside every stage** (What did not work, item 2). This run does not say what it is; `nsys` would. It may bear on DeepSeek's served-rounding-cost (#3594), which shares `hamilton_sheet.rs`.
- **The next pod job should run `three_steps_trained_as_served_save_a_step_the_export_leaves_unchanged`** with `--ignored` (What did not work, item 7).
- **The code review's two nits:**
  - `QuantizerTiming::check` could also require equal calls a forward in both classes (I-4), which the grader checks today;
  - no CPU test pins the unarmed values; the CUDA timing test does.

## Plain summary

Before writing custom GPU kernels for the two rounding steps that make training match serving, we measured how much time they take. We turned them off on alternate pairs of steps inside the same run and compared the step times.

At the short 192-token window they take 7.5% of each training step. At the 2,048-token window they take 2.6%. The project's rule was to skip the work if both were under 5%. The short window is above it, so the fused kernels go ahead.

The real gain will be somewhat smaller, because a fused kernel still has to do the rounding once. The next units build the kernels and measure what they actually save.
