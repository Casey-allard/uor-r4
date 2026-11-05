# Prep sheet: the served quantizer chains' share of a training step (float-kernel-fusion, claude-283, 5 Nov 2026)

Lab: claude. Item: float-kernel-fusion (#3678). Order: [claude-283](../../loop/tasks/float-kernel-fusion/claude-283/ORDER.md), THE ORDER (#2028 6634218223) step 5(c).

TEST FITNESS: FIT (round 1, a fresh non-author reviewer, 5 Nov). Its three should-fixes are in: R1 takes out U's batch difference, I-5 gates the stages that hold no chain call, and the re-run keeps its noise margin.

**Why.** The item's *Cancel if* stops the fused quantizer kernels (done-whens 2 and 3) if the two chains take under 5% of a trained-as-served step at P2b's recipe. No existing measurement splits them (the ORDER's cancel probe):
- [claude-157](../../loop/tasks/fbs-cost-profile/claude-157/REPORT.md) predates training as served;
- [deepseek-5](../../loop/tasks/one-model-end-to-end/deepseek-5/REPORT.md) probed the host;
- `--profile K` puts the chains inside other forward stages and the one `backward` stage.

## Question

At P2b's recipe with `--train-as-served 1`, what share of a training step's wall time do the two served quantizer chains take, forward and backward together?
- **`activation_quantize16`** (`hamilton_sheet.rs`), through `served_activation_quantize16`: twice a layer, on the gated output of `composed_recurrence` (`geometric_stack_native.rs:2137`) and of the gated sublayer `mlp` (`:1785`). The calls at `:1788` and `:2140` run only while gate telemetry is captured, never in these runs.
- **`q16_residual` with its rounding on:** the trunk's embedding and each residual sum, after every sublayer and the read (`:1656` onward), and every FBS projection's output through `served_projection_q16` (`:1406`).

## Base artifact

- **No checkpoint.** Each run trains from a fresh initialization at `--seed 1`, as P2b's runs did.
- **The binary:** `train-in-space`, built `--release --locked --features cuda` at the pin, with its SHA-256 recorded. The pin is the unit's reviewed code commit, recorded at launch. The PR head may differ from it only in documents.
- **The switch is off unless armed** (this unit's tests). Armed runs are timing only: their losses and checkpoints are not results.

## Data split

| Input | Store path | SHA-256 |
|---|---|---|
| TRAIN, both arms (P2b's long TRAIN) | `results/claude/one-model-end-to-end-long-context-2026-11-04/inputs/train-long/` | `tokens.u16` `61019954ef82d7b33d2d5a101f436fece29452f8580f8f3268e35229ccd0cb3c`; `response_mask.u8` `29b34d2c9ed3a56bf638a3afd6863e5744b2c1d3f838a1034f19a77d1e02818d`; `manifest.json` `0a7f32d2aa54edf0169c3174473deccdb099f17487fe9140b08ef0b369cd8e65` |
| DEV (the final step's evaluation, outside the window) | `w2-inputs/2026-10-25/driver-data/ft/dev/` | `tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| tokenizer, and the data root's default TRAIN | `w2-inputs/2026-10-25/driver-data/` | `tokenizer.json` `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`; `mw-balp/train` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |

- **No panel is scored.** Neither p-long nor p-v is fetched.
- **What is measured is time,** so no held-out data is spent.

## Arms

**The recipe** is claude-266's (b), as P2b ran it: `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 --turn-memory-dropout 500 --train-as-served 1 --width 256 --hamming-read 4`, with the long TRAIN, `--seed 1`, `--profile 64`, `--steps 137` and `--eval-every 137`.
- **The window:** 64 pairs, training steps 8–135, counted from 0 as `profile_window` counts them (`in_space_long.rs:10525`). `profile.json` is written at step 135. The one evaluation, at step 136, the last, is outside it: the option refuses `--steps` or `--eval-every` at or under `8 + 2K = 136`.

**The arms:**
- **(L):** `--context 2048 --batch 4`;
- **(S):** `--context 192 --batch 16`.

**Three runs per arm, in this order:** T1, U, T2.
- **U** is unarmed.
- **T1 and T2** add `--time-without-quantizers 1`. In each, pairs `j = 1, 3, …, 63` (steps `8 + 2j` and `9 + 2j`) skip both chains, and pairs `j = 0, 2, …, 62` keep them.

## Readings

Every reading comes from `profile.jsonl`. Row `j` holds step `8 + 2j`, its stage GPU times, its `step_ms` (profiled), and `next_unprofiled_ms`, the clean-clock time of step `9 + 2j`. In a T run, a row is kept if `j` is even and skipped if `j` is odd.

Write `K(r)` and `S(r)` for run r's median `next_unprofiled_ms` over its even and odd rows.
- **R1, the share** (decides): per T run, `[(K(T) − S(T)) − (K(U) − S(U))] / K(T)`. The arm's share is the mean of T1's and T2's.
  - Even and odd pairs train on different batches in every run, at the same seed. U skips nothing, so `K(U) − S(U)` is what the batches alone put between the classes, and it is taken out.
  - The uncorrected share, `(K(T) − S(T)) / K(T)`, is reported beside it.
- **R2, the noise,** per arm: the largest of two spreads and a floor.
  - **T1 against T2:** `|share(T1) − share(T2)|`.
  - **U split in halves:** U's `(K − S) / K` over rows 0–31 against the same over rows 32–63.
  - **The floor, 0.25% of a step:** each spread is one draw, so a near-zero R2 would leave the bar no margin. The grader was checked on synthetic runs, where a 4.99% share reads inconclusive.
- **R3, where the time sits:** per class, the median GPU time of each stage, and their difference by group (forward, objective, `backward`, optimizer). The `served.round` and `served.restore` stages (the export rounding, not these chains) are reported apart.
- **R4, the counts:** from `quantizer-timing.json`, the calls and elements per forward of each chain in each class.
  - The tensor ops per call, read from the source at the pin, are given with their lines.
  - Kernel launches per step are calls × ops, as an estimate.
- **R5, run-to-run drift:** U's median unprofiled step against T1's and T2's kept medians.

## Bar

| Outcome | Reading | What it unlocks |
|---|---|---|
| **Cancel** | At both arms, share < 5% and `5% − share > 2 × noise` | Done-when 1 stands alone. The item is set done, and the fusion is reported as not worth its pin. |
| **Go** | At either arm, `share − 5% > 2 × noise` | Done-whens 2 and 3, the fused kernels, are the next units. Development may use the 4090; qualification stays on a 5090. |
| **Inconclusive** | Anything else | One re-run at `--profile 256` (steps and eval-every 521), read by the same table with its own R2. |

- **If the re-run is inconclusive too,** the arm reads Go. A wrong Go is caught by done-when 4, which times chain against fused, and the flag defaults on only when the fused kernels pay. A wrong Cancel is never revisited.
- **The ORDER's done-when 5** reads Go at "at or above 5%" and takes noise from kept pairs split into halves. This bar is stricter: it asks for the margin both ways, and takes the noise from T1 against T2 and from U. The REPORT records the change.

## Instrument checks

- **I-0:** the crate's CUDA library suite at the pin passes, `--test-threads=1`. These tests are named:
  - `a_kept_forward_quantizes_as_an_unarmed_one_on_cuda` (a kept forward's values and gradient match an unarmed one's, and a skipped forward's gradient is the identity's);
  - `a_skipped_forward_returns_both_chains_input_bit_for_bit`;
  - `the_quantizer_timing_switch_skips_the_window_s_odd_pairs_only`;
  - `the_quantizer_timing_record_refuses_a_step_out_of_its_pair`;
  - `served_surrogate_gradients_are_exact_on_cuda`.
- **I-1:** every run exits 0. Its `profile.json` says `profiled_steps: 64`. Each T run wrote `quantizer-timing.json`, and its check passed (the run errs otherwise).
I-2 and I-3 are the ORDER's I-1 and I-2.
- **I-2, nothing acts before pair 1:** T1's row-0 loss (step 8) equals U's, as text. If it fails, compare T1 with T2.
  - If T1 equals T2 but not U, the switch leaks: an instrument fault, repaired before any reading counts.
  - If T1 differs from T2 as well, training is not reproducible from run to run. Then I-2 and I-3 tell nothing, and that is reported. The evidence that the switch acts, and only on its pairs, then rests on I-0's CUDA test and the record's check.
- **I-3, the switch acts:** T1's row-1 loss (step 10, the first skipped step) differs from U's.
- **I-4, the same graph:** in each T run, the kept and skipped forwards make the same calls per forward to each chain, and more than zero.
- **I-5, nothing else moves:** in each T run, the stages that hold no chain call take the same time in both classes.
  - **The set:** `final_norm`, `readout.*`, `points.select`, `objective.*`, `optimizer.*`, `served.round`, `served.restore` and `end`. The chains are called only from the trunk's `q16` sites, from `linear`'s rank-3 projections and from the two gates. The read and memory modules (`flock_read_hamming.rs`, `flock_read_memory.rs`, `flock_read_register.rs`) call neither chain.
  - **The test:** per row, the sum of the set's GPU times; its median over skipped rows against kept rows differs by at most `max(R2, 0.5%)` of `K(T)`.
  - **If it fails:** skipping changed work outside the two chains (for one, compute that depends on values, after the activations leave the grid). That is an instrument fault, and R1 is not read.
- **The last evaluation is outside the switch.** It passes `step + 1 = 137` as its served step (`in_space_long.rs:11185-11188`), which `quantizer_timing_skip` places past the window. If it did not, the record's check would fail the run (exit 5), not misread it.

## Steps

1. **On the pod:** fetch the pin and check that it is that commit and clean. Build, and record the binary's SHA-256. Run I-0.
2. **The inputs:** fetch them and check every SHA256SUMS and the pins above.
3. **The runs:** T1-L, U-L, T2-L, T1-S, U-S and T2-S, each into its own new folder, with `nvidia-smi` logged.
4. **Seal** with `SHA256SUMS.files`, then DONE, then EXIT.
5. **On the laptop:** copy the folder off and put it in `results/claude/float-kernel-fusion-measurement-2026-11-05/`. Read it back and check it. Only then take the pod down. Grade R1–R5 and I-1 to I-5 from the copies (jq and awk, in `local/claude/`).

## Minimum test power

- **Samples:** each T run gives 32 kept and 32 skipped clean-clock steps, and each arm has two T runs.
- **The size of a 5% share:** P2b measured 0.645 s a step at (L) and 0.125 s at (S) ([claude-276](../../loop/tasks/one-model-end-to-end/claude-276/REPORT.md)). So 5% is about 32 ms and 6 ms.
- **Known drift:** P2b's two seeds' mean steps differ by 0.08% at (L) (0.6449 and 0.6444 s) and 0.16% at (S) (0.1253 and 0.1255 s). These are means of whole runs, not the spread of single steps. The margin that counts is R2, measured in this run.
- **If the noise is larger:** if R2 exceeds 2.5% of a step at an arm, that arm cannot cancel, and reads go only with a share past `5% + 2 × noise`, over 10%. Otherwise the table reads inconclusive, and the re-run at `--profile 256` gives 128 samples a class.

## Cost

- **The pod:** one RTX 5090.
- **The time:**
  - the build, about 3 minutes;
  - I-0, about 3 minutes;
  - the inputs, under 2 minutes;
  - three (L) runs of 137 steps at about 0.65 s, plus start-up and the final evaluation, about 8 minutes;
  - three (S) runs, about 3 minutes.
- **In all:** about 25 minutes, under $0.50 at today's offers.
- **The unit's bound:** $3.

## Stop rule

- **On a fault:** any failed instrument check or non-zero run exit ends the job with its own code (inputs 3, I-0 4, a run 5). Nothing after it runs. Every exit seals.
- **On the pod:** a watcher is armed on every job. Nothing is written into an existing folder. Results come off the pod before `down`.
- **On cost:** past $3, stop and report what was measured.
