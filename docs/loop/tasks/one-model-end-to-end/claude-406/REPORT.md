# REPORT: one-model-end-to-end, claude-406 (how fast the discrete model's CUDA scoring runs on a D87 student batch)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Even the fast path does not make a corpus pass short** [claude]. Rows-K scores 64 coordinates in 127 ms, 2,389 times faster per coordinate than the baseline. But a call costs at least about 81 ms even at K = 1, so the 64-row cap holds it near 780 targets/s. One pass over the projected ~1.87B targets would take about 28 days.
- **Arm A as first written was too slow to finish** [claude]. Run c406-run1 repeated the baseline six times over 64 coordinates per kind. It ran 27 minutes without leaving arm A and was stopped. Arm A now runs once, with its first call timed apart as the warm-up and a progress line every 8 coordinates. So arm A's figure is a median over one pass, not over repeats.
- **The integer rate fields round down to 0 for arm A** [claude]. A whole arm-A pass takes 95–303 s, so whole-number rates per second print as 0. The comparison per coordinate is printed separately and is the figure that counts.
- **The Route model has 20 routes, not 64** [claude]. Routes are indices 0 to 19. Route K = 32 and K = 64 are therefore the same single call over 20 coordinates. At K = 8 and K = 16 the last call is short, and the test's printed row-coordinate rate overstates those two rows (corrected in the table).
- **The planted fault checks its own way** [claude]. It counts coordinates whose rows-K losses differ from a fresh baseline. That shows the moved target reaches the losses (16 of 16). It does not drive the parity loop's mismatch path, and its bar is `caught > 0`, not all 16.
- **The test does not record `nvidia-smi` itself** [claude]. The ORDER asked for it; the job ran it before and after.
- **Only one batch, one seed and checkpoint 0 were measured** [claude]. A trained model may cost more per call; M1 recorded per-batch cost rising as its arms trained.

## What changed

- `crates/uor-r4-discrete/src/student_throughput.rs`, which is new: the `#[ignore]` CUDA test `student_batch_scoring_speed_baseline_against_rows_k`.
  - It builds claude-404's 64-row student batch, V4096/B4/C192, Route read, seed 1, checkpoint 0.
  - It fixes two coordinate sets before any timing: the first 64 VectorBlocks of the batch's target tokens, and the 20 routes (indices 0 to 19).
  - It times arm A (baseline `propose`, one coordinate per launch, one pass) against arm B (rows-K `score_rows_k` at K = 1, 8, 16, 32 and 64, one warm-up and five timed repeats, median).
  - It requires every rows-K coordinate's status to be 0 and its 16 loss words to equal the baseline's.
  - It plants one moved target in rows-K's input only.
  - Scoring only: no choose, apply or epoch change, and the test asserts the epoch is unchanged.
- `cpu_oracle_parity.rs`: `pub(crate)` on claude-404's student-batch helpers, a visibility change only.
- `lib.rs`: one `mod` line, gated `cfg(all(test, feature = "cuda"))`.
- No kernel source, trainer default or serialization change.

## Measurements

Run on wick's RTX 4090 (driver 595.99.02, nvcc 12.8 V12.8.93, `UOR_NVCC_ARCH=compute_89`, built `rowsk.ptx` sha256 `3b61c193…b5f0`, `discrete.ptx` `2df7b47b…b7a9`), job c406-run2, 16 Nov 2026 [claude]. `nvidia-smi --query-compute-apps` listed no other process before or after.

The ORDER's base was claude-404's branch. After claude-404 merged (#4040, `59703aa99`), these commits were moved onto `origin/main` as they were. `git diff` of `crates/uor-r4-discrete` against the tested branch is empty, so the code here is the code that ran.

The stream was claude-404's student stream: `tokens.u16` sha256 `a16a4412…3238` (15,471,412 tokens), `response_mask.u8` sha256 `25b9090d…a7c1`, 13,559,938 eligible targets. The batch starts at eligible target 6,779,969: 64 rows, 12,288 window tokens.

| Kind | Path | K | Per call | Per coordinate | vs baseline | Row-coordinates/s | Targets/s at one call per 64-row batch |
|---|---|---:|---:|---:|---:|---:|---:|
| VectorBlock | baseline | 1 | 4.729 s | 4,728.7 ms | 1× | 13 | 13 |
| VectorBlock | rows-K | 1 | 82.2 ms | 82.2 ms | 57× | 778 | 778 |
| VectorBlock | rows-K | 8 | 85.2 ms | 10.65 ms | 444× | 6,009 | 751 |
| VectorBlock | rows-K | 16 | 85.7 ms | 5.36 ms | 882× | 11,945 | 746 |
| VectorBlock | rows-K | 32 | 91.6 ms | 2.86 ms | 1,652× | 22,362 | 698 |
| VectorBlock | rows-K | 64 | 126.7 ms | 1.98 ms | 2,389× | 32,337 | 505 |
| Route | baseline | 1 | 4.727 s | 4,727.0 ms | 1× | 13 | 13 |
| Route | rows-K | 1 | 80.9 ms | 80.9 ms | 58× | 790 | 790 |
| Route | rows-K | 8 | 82.2 ms | 12.33 ms | 383× | 5,190 † | 778 |
| Route | rows-K | 16 | 82.3 ms | 8.23 ms | 574× | 7,776 † | 777 |

† Corrected from the test's printed 6,230 and 12,445. The 20 routes split into calls of 8, 8 and 4 at K = 8, and 16 and 4 at K = 16. The test's `rates()` assumed every call scores K coordinates. The corrected figure is 20 × 64 row-coordinates over the whole time. "Per call" is the mean over those calls. The VectorBlock rows are exact, because 64 divides evenly.
| Route | rows-K | 20 (K32 and K64) | 88.6 ms | 4.43 ms | 1,066× | 14,442 | 722 |

- **Parity:** 0 loss-word mismatches across every coordinate, block size and kind. Every rows-K status was 0.
- **Planted target shift:** 16 of 16 coordinates differ.
- The test passed in 527.98 s.
- **Projected days for one pass over ~1.87B targets** (claude-403's projection), at one call per 64-row batch:
  - baseline: about 1,660 days;
  - rows-K K = 1: about 28 days;
  - rows-K K = 64: about 43 days.

  These count targets consumed only. They time `score_rows_k` alone: carry (run once, outside the timer), choose, apply and data movement are left out. So they are lower bounds on a training pass. A larger K scores more coordinates per target, which is more training per target. That benefit is not measured here.

## What the numbers say

- **The baseline wastes the GPU.** It runs 16 threads per coordinate, and each thread walks every row serially. Spreading rows over threads gives 57× at K = 1.
- **The GPU fills only near 64 × 64 × 16 threads.** A call costs 81–92 ms from K = 1 to K = 32, and only at K = 64, 65,536 threads, does it rise to 127 ms. So up to K = 32, extra coordinates are almost free.
- **What the ~81 ms floor is made of is not measured.** Two explanations fit a cost that stays flat from K = 1 to K = 32:
  - work per row, since each thread walks its row's 192 positions in order;
  - a fixed cost per call: three launches, target and index uploads, a device-to-host sync and the epoch read.

  If it is per-row work with room left on the GPU, more rows per call would raise targets/s. Still open item 2 tests that. As a hypothesis, not a measurement: 4,096 rows at K = 1 is 65,536 threads, the size that cost 127 ms here. That would be about 32,000 targets/s, about 0.7 days per pass. This ignores the 64 times larger carry and window memory.

## Checks run at the head

- G1: the test passed on wick's 4090 (above), with 0 mismatches and the planted fault caught.
- G2: the existing `cpu_oracle_parity` tests passed in the same job: `cuda_parity_catches_one_planted_fault_per_step ... ok`. The bounded student parity test is claude-404's own result.
- G3: the ORDER commit is the first of this unit's commits.
- `git diff --check` is clean, and `rustfmt --check` is clean on the new file.
- `loop pr-check --local`: run before queueing.

## Skills used

- `scope-guard`: one new test module, a visibility change and one `mod` line; no kernel or trainer change.

## Still open

1. **Lift the 64-row cap.** It sits in `Device::admit_batch` (`device.rs:1448`) and `PreparedCorpus::batch` (`corpus.rs:255`). Then measure the same table at 256, 1,024 and 4,096 rows, with the same baseline parity on a subset. This is the next unit.
2. **Measure the per-row floor.** Time carry and baseline apart from the counterfactual launch inside `score_rows_k`, and look at the per-position read for reuse. `propose_reused` exists for this.
3. **Remove per-call host syncs.** Each call uploads targets and indices and reads back status; the stream could stay on the device.
4. **Test fixes for the next run of this test:**
   - compute rates from the coordinates actually scored;
   - send the planted shift through the parity predicate and require all 16;
   - record `nvidia-smi` inside the test.
5. **The timed training run on wick**, with a sheet and a non-author FIT under §4.5, measuring learning per target at the chosen rows and K.

## Plain summary

The current trainer scores one candidate change at a time with 16 GPU threads, which takes 4.7 seconds on a real batch from the new corpus. The faster path already in the code gives exactly the same numbers and is 57 to 2,389 times faster. Even so, it still reads only 64 examples per call, which works out to about four weeks per pass over the corpus. The next step is to let each call read thousands of examples, which the GPU has room for.
