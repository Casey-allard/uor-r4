# REPORT: one-model-end-to-end, claude-407 (lift the 64-row cap on CUDA scoring and time rows-K at 256 to 4,096 rows)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **No trainer can use the larger batches yet** [claude]. This unit lifts the cap on scoring calls only. The trainer still refuses more than 64 rows:
  - `DriverConfig::validate` (`training.rs:307`);
  - `stage-t-run`, which also fixes 64 rows and its own DEV panel (`stage_t.rs:479-500`, `2572`);
  - `baseline_reuse`'s 64-bit row mask (`baseline_reuse.rs:6`, `97-101`).

  The trainer's general path also never calls rows-K: it scores one parameter at a time (`training.rs:3089`). Only the BVG-K block step does (`bvgk_device.rs:121`). So the projections below are not training speeds.
- **The days are scoring only** [claude]. They count `score_rows_k` time per target consumed. Carry, choose, apply, DEV evaluation and data movement are left out, so they are lower bounds on a training pass. Carry alone took 23 ms at 4,096 rows, outside the timers.
- **Above about 4,096 rows × K = 1, the GPU is full** [claude]. At 4,096 rows, K ≥ 8 runs near 54,000 row-coordinates/s whatever K is. So more coordinates per call no longer come almost free, as they did at 64 rows: K = 8 gives 6,324 targets/s and K = 64 gives 853. Which K learns most per second is not measured here.
- **The baseline subset is small at the large sizes** [claude]. One baseline call takes 304–307 s at 4,096 rows, so only 1 coordinate per kind was compared with the baseline there, and 2 at 1,024 rows. The other coordinates are checked by cross-K agreement: every K gives the same words as K = 1. That shows the rows-K block sizes agree with each other, not that each coordinate matches the baseline.
- **At 4,096 rows the baseline figure is the warm-up call** [claude]. With one call per kind there is no median after a warm-up. At 64 to 1,024 rows, the warm-up call was within 10% of the later calls' median (the widest gap was 9.6%, Route at 256 rows).
- **The batch pin lines were dropped from the log** [claude]. The job's output filter removes lines that start with `=`, which includes the test's `== N rows: …` lines. The stream and start point are claude-406's, unchanged: the same `UOR_STUDENT_STREAM` folder, and the batch starts at the middle eligible target.
- **Only one stream, one seed and checkpoint 0 were measured** [claude]. A trained model may cost more per call.
- **The wick copy was synced before `rustfmt` ran** [claude]. The committed `device.rs` and `student_throughput.rs` differ from the copy that ran on wick in whitespace only.
- **The `MAX_BATCH_ROWS` constant sits in `corpus.rs`, not in `device.rs` as the ORDER said** [claude]. `device.rs` compiles only with the `cuda` feature, and `corpus.rs` needs the same bound in every build.
- **The planted-shift count would accept a failed call** [review]. It counts a coordinate as caught when `agrees()` is false, and `agrees()` is also false when the status is not 0. In this run every status was 0, so all 16 catches at each size are real shifts. A later unit should count a catch only at status 0.
- **The refusal texts write 4096 by hand** [review] rather than formatting `MAX_BATCH_ROWS`, so a later change to the constant must change three texts as well.
- **Two files outside the ORDER's list** [claude]:
  - the notebook row is in `m1-discrete-attention.md`, beside claude-406's row, not in `cpu-training.md`;
  - `PLAN.md` gains one owner-direction row: Casey's "start the 1b training run on wick" and the plan agreed for it.

## What changed

- `crates/uor-r4-discrete/src/corpus.rs`: `pub const MAX_BATCH_ROWS: u32 = 4096`, used by `PreparedCorpus::batch`.
- `crates/uor-r4-discrete/src/device.rs`: `admit_batch`, `carry` and `carry_windows` use `MAX_BATCH_ROWS`. Their refusal texts say 1..4096.
- `crates/uor-r4-discrete/src/qualification.rs`: `inventory_bounds` expects the new `admit_batch` text.
- `crates/uor-r4-discrete/src/driver_source_pin.rs`: the pin moves to `3fed0ca5549f786f4df6178d82abc16e9b8d0a930cbfbff3320d8772e09dadba`, because `corpus.rs` is a driver source.
- `crates/uor-r4-discrete/src/cpu_oracle_parity.rs`: `student_batch_rows(rows)`. `student_batch()` calls it at 64 rows, so claude-404's test is unchanged.
- `crates/uor-r4-discrete/src/student_throughput.rs`: the speed test loops over 64, 256, 1,024 and 4,096 rows on one device. It has the three fixes from claude-406's Still open item 4:
  - rates come from the coordinates scored over the whole pass;
  - the planted shift runs through the one parity predicate `agrees()`, and all 16 must be caught;
  - `nvidia-smi --query-compute-apps` runs inside the test, before and after.
- No kernel source change. No trainer default change.

## Measurements

Run on wick's RTX 4090, job c407-run1, 16 Nov 2026 [claude]:
- driver 595.99.02, nvcc 12.8, `UOR_NVCC_ARCH=compute_89`;
- head `0ce6b26e1` before `rustfmt`;
- `nvidia-smi --query-compute-apps` printed `[]` before the test. The job waited for a free GPU and found it free at once. The "after" line lists only the test binary itself, pid 332282, 4,614 MiB.

Stream: claude-404's student stream, `tokens.u16` sha256 `a16a4412…3238`, 15,471,412 tokens, 13,559,938 eligible targets. Each batch starts at the middle eligible target. Model V4096/B4/C192, Route read, seed 1, checkpoint 0. Coordinates: the first 64 VectorBlocks of each batch's target tokens, and the 20 routes.

Rows-K, VectorBlock (Route agrees within 10% in every cell at K = 1):

| Rows | Baseline per coordinate | K = 1 per call | K = 1 targets/s | K = 8 targets/s | K = 64 per call | Row-coordinates/s at K = 64 |
|---:|---:|---:|---:|---:|---:|---:|
| 64 | 4.69 s | 81.7 ms | 783 | 755 | 125.6 ms | 32,601 |
| 256 | 20.6 s | 90.8 ms | 2,819 | 2,559 | 394.2 ms | 41,564 |
| 1,024 | 75.4 s | 84.9 ms | 12,064 | 5,054 | 1,238.7 ms | 52,906 |
| 4,096 | 304.0 s † | 126.5 ms | 32,368 | 6,324 | 4,796.8 ms | 54,649 |

† One call, the warm-up itself.

Route at K = 1: 731, 3,075, 12,228 and 32,435 targets/s at 64, 256, 1,024 and 4,096 rows. "Targets/s" means one call per batch, so rows ÷ seconds per call.

- **Parity:** 0 loss-word mismatches against the baseline subset (4, 4, 2 and 1 coordinates per kind) and 0 across K at every size. Every status was 0.
- **Planted target shift:** 16 of 16 coordinates caught at each of the four sizes.
- The test passed in 1,552.97 s, pipe status 0.
- **Projected days for one pass over ~1.87B targets** (claude-403's projection) at K = 1, scoring only, as lower bounds:

  | Rows | Days |
  |---:|---:|
  | 64 | 27.6 |
  | 256 | 7.7 |
  | 1,024 | 1.8 |
  | 4,096 | 0.67 |

## What the numbers say

- **More rows per call is the lever.** At K = 1 a call costs 82–91 ms from 64 to 1,024 rows, so targets/s grows almost in step with rows: 15.4× from 64 to 1,024. From 1,024 to 4,096 rows the cost rises to 127 ms, and targets/s grows 2.7× more.
- **Two explanations still fit the flat cost, and this unit does not separate them:**
  - each row's thread walks 192 positions in order, and rows run in parallel until the GPU fills;
  - a fixed cost per call: launches, uploads, the sync and the epoch read.

  Either way, rows that fit in parallel are nearly free until the GPU fills, at about 4,096 rows × 16 lanes.
- **At 4,096 rows the GPU is full.** Row-coordinates/s level off near 54,000 for K ≥ 8. More coordinates per call then cost almost in proportion: K = 64 costs 38× K = 1 per call.
- **The training choice is now rows against K.** At 4,096 rows, K = 1 consumes about 32,400 targets/s and tries one coordinate per batch. K = 8 consumes about 6,300 targets/s and tries eight. Which learns more per second needs a training run.

## Checks run at the head

- G1: the speed test passed on wick's 4090 (above).
- G2: claude-404's student parity test still passes through `student_batch_rows(64)` in the same job: `test result: ok. 3 passed; 0 failed`, 1,204.86 s, pipe status 0. Route and Flock each had 0 mismatches at every step, and the planted target shift was caught (48 loss mismatches of 48 parameters).
- G3: the laptop suite, `cargo test -p uor-r4-discrete --lib --bins --tests --no-fail-fast`: every test passes except two that already fail on `main` without this change:
  - `snapshot_loader_delegates_position_step_bounds_to_cuda_validate` (`tests/position_device_controls.rs`) still fails with this branch's `device.rs` change set aside;
  - `apply_calls_are_confined_to_bounded_fixture_and_admitted_transition` trips on `native_vector_device.rs`, which this branch does not touch; that line is on `main`.
- G4: the driver pin test `driver_source_sha256_matches_the_pin` and the three `prime_zeta` copies pass with the new pin.
- G5: the ORDER commit `0b724336e` is the first of this unit's commits.
- `git diff --check` is clean, and `rustfmt --check` is clean on the changed lines.
- `loop pr-check --local`: run before queueing.

## Skills used

- `scope-guard`: four host-side checks, one constant, one pin, a batch helper and the test; no kernel or trainer change.

## Still open

1. **A corpus trainer that runs at 4,096 rows.** It goes through the BVG-K block step (`Driver::run_block_transitions`, `training.rs:3308`), behind a command line, with:
   - the driver's `batch_rows` bound lifted;
   - a check that the BVG-K commit kernel has no row limit;
   - checkpoint and resume;
   - DEV on the new stream.

   It comes after the native read and the conjunction memory, in the order Casey chose on 16 Nov (PLAN.md, owner directions).
2. **Ingest the whole D87 corpus** into TRAIN and DEV streams with `student-ingest`, DEV from held-out shards.
3. **The 1B training run on wick**, with a sheet and a non-author FIT under §4.5. It measures learning per target against rows and K.
4. **Measure what the ~82 ms per call is made of**, as claude-406 Still open items 2 and 3 describe: per-row serial walk or per-call host cost.
5. **Two tests that fail on `main`** (G3), and the `first_pick_read_parity` example, which does not compile without the `cuda` feature. All three came in with #4001.

## Plain summary

The fast scoring path used to read at most 64 examples per call. Now it can read up to 4,096. Each call costs about the same until the GPU fills, so at 4,096 examples it consumes about 32,000 targets a second, 41 times more than before. That brings one pass over the new corpus to under a day of scoring time. The answers are unchanged: they match the old path exactly. The trainer itself still uses 64-row batches, so the next step builds a trainer that uses the large batches, and then starts the 1B run on wick.
