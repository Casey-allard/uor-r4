# REPORT: one-model-end-to-end, claude-272 (the Hamming read at long windows)

Lab: claude. [ORDER](./ORDER.md). PR #3644, item #3407. Plan step: Casey's plan of 4 November, step P2 ("Run a finite GPU test of the Hamming read's memory and time at 512, 1,024 and 2,048").

**In one paragraph.** The Hamming read's `choose_sorted` now takes its L1 distance in chunks of query positions, under a 256 MiB budget for the `(batch, chunk, time, width)` difference. It also builds the candidate mask on the GPU instead of in a host loop over rows × time.
- **The test pod:** one RTX 5090, `vast-59063044`, on a $0.975/h offer; the job itself took 3.5 minutes (`job.log`, 05:39:07 to 05:42:34).
- **What it ran:** the timing test at 192, 512, 1,024 and 2,048 positions, at batch 4 and 16 and width 64; the crate's whole CUDA library suite (313 passed, 0 failed); and two negative gates, each failing exactly its own test.
- **What it found:**
  - The chunked read's peak memory stays at or under 1.33 GiB everywhere.
  - Wherever the unchunked read runs, the chunked read picks exactly the same positions.
  - Where it takes more than one chunk, it is 2.05× to 3.09× faster than the unchunked read. Where it takes one chunk, the two are the same speed (within 2.3%).
  - At 2,048 × 16 the unchunked read cannot run at all.
- **What "unchunked" means here:** the same new code with no budget, so one chunk and the new device mask. It is not the removed code, whose host mask loop is not timed here. That the new read picks what the removed one picked rests on the two equality tests and on the diff.
- **Not done:** no model was trained or scored at a long window, and a training step at 2,048 is untimed. Casey added that run to the plan as P2b, right after P2 unit 3.

## Tokens per second first

No decode tok/s was measured; this unit times one head's read, not a decode. One head's read covers about 997,000 positions per second at 192 × 4 (768 positions in 0.77 ms), 268,000 at 2,048 × 4 (8,192 in 30.6 ms) and 281,000 at 2,048 × 16 (32,768 in 116.8 ms). Per position, the read costs about 3.7× more at 2,048 than at 192: the distance is still quadratic in the window, and only its memory is now bounded.

## What did not work

1. **The ORDER's CPU test could not run on the CPU.** The project refuses CPU tensor maths (D24/D29); the laptop run of both new tests stopped with "CPU model compute is unavailable; use the admitted CUDA geometric path". Both equality tests moved to the CUDA path (`#[cfg(feature = "cuda")]`) and ran in the pod's library suite. The laptop keeps only the CUDA type-check (stub `nvcc`).
2. **At 2,048 × 16 the unchunked read failed with `CUDA_ERROR_INVALID_VALUE`, not out of memory.**
   - Its difference tensor has 16 × 2,048 × 2,048 × 64 = 2³² elements. candle 0.9.2 sizes every element-wise launch with the same `LaunchConfig::for_num_elems(n as u32)` cast (`third_party/candle-core-0.9.2/src/cuda_backend/mod.rs`, for example `:1102` for the binary map and `:385` for the unary map). The failing launch is most likely the broadcast subtraction's. At exactly 2³² the count wraps to 0, so the grid has 0 blocks and the launch is refused.
   - The memory pool held 16.02 GiB when it failed. The test prints "does not fit" for any error, so that label is the test's, not the cause.
   - **The hazard:** past 2³² the count wraps to a small number, and a launch would cover only part of the tensor without any error. In every case measured here, up to batch 16 at 2,048, each tensor the read builds has at most 2²⁶ elements: the difference chunk under the budget, and each `(rows, time)` tensor exactly 2²⁶. The budget does not bound them at every batch. A chunk takes at least one position, and the `(rows, time)` tensors grow as batch × time², so at 2,048 they reach 2³² at batch 1,024. Any path that builds a tensor past 2³² elements on the GPU would be wrong. Still open 1.
3. **At 192 (both batches) and at 512 × 4 the budget covers every row,** so the "chunked" read is the whole read in one chunk. Those three rows test the code path, not chunking. Chunking starts at 512 × 16 (128 positions a chunk) and is real in both 2,048 cases (128 and 32).
4. **The `nvidia-smi` samples cannot resolve one run.** They are about 0.14 s apart (1,508 in all; mean 137 ms, from 130 to 259 ms), and one run takes 0.77–117 ms. Their maximum, 17,034 MiB, comes 3 ms after the unchunked read's failed attempt at 2,048 × 16 ended, while its memory was still held, and it includes the context's own memory. The memory figures below are the pool's own high mark, `CU_MEMPOOL_ATTR_USED_MEM_HIGH`, read and reset around each form's six runs. candle allocates from that pool through `cuMemAllocAsync` (`third_party/cudarc-0.19.10/src/driver/safe/core.rs:866`, `:1605`).
5. **The job script's first draft had two faults,** both found by the pre-run review and fixed before it shipped:
   - its grep would have dropped the first timing line, which libtest prints after the test's name, on the same line;
   - a red test anywhere in the suite would have ended the job before any timing ran.
6. **`uor-pod down` failed its laptop-archive step again** ("uor-r4/pods: Is a directory"), as in FRICTION.md's 31 October and 1 November lines. The results were already in the HF store and read back equal, so `down --confirm-archived` deleted the pod.
7. **The stop-and-decide rule ("chunked 2,048 too slow to train") cannot be judged yet.** It compares the read with the trunk's forward at the same window, which nobody has timed at 2,048. P2b times a training step at 2,048 against 192.

## What changed

`crates/uor-r4-float-train/src/flock_read_hamming.rs`:
- **`choose_sorted`** keeps its signature and behaviour, and calls `choose_sorted_in` with the budget `DISTANCE_CHUNK_BYTES = 1 << 28`.
- **`distance_chunk`** picks the most query positions whose `(batch, chunk, time, width)` f32 difference fits the budget, clamped to 1..=time.
- **`mean_l1_rows`** takes `narrow(1, at, n)` chunks of the query, then broadcast-subtracts, takes the absolute value and the mean over width, and concatenates the chunks. Each element's mean over width is the same expression at any chunk size, so the result does not depend on the chunking.
- **`candidate_mask`** builds `s >= first && s < t && s < limit` from device `arange`, `broadcast_ge`, `broadcast_lt` and `broadcast_mul`. Limits past the window are clamped to it, since `s` never reaches `time`.
- **`is_candidate`,** the host rule, is now test-only: it is the mask test's reference.
- **Three tests:**
  - `the_read_in_chunks_chooses_as_the_whole_read` (CUDA). The chunked distance equals the whole distance bit for bit at chunks of 1, 7 and 20 of 20. `distance_chunk` returns each chunk size from its byte budget. Picks and the real flags are equal under the tie rule on and off, three limit sets (the window, a cut at 7, per-row limits) and `first` 0 and 1.
  - `the_device_mask_is_the_candidate_rule` (CUDA). The device mask equals `is_candidate` at every entry, for four limit sets (below, at and past the window, and `usize::MAX`) and `first` 0, 1 and 2.
  - `the_read_at_long_windows` (CUDA, ignored, run on the pod). The timing and pool-peak run below. It asserts equal picks wherever the unchunked read runs.

## Measurements

On the RTX 5090 (32,607 MiB, driver 595.71.05), release build at `267c1bc6d`, one head (width 64), `HAMMING_KEPT` picks, tie rule on, `first` 1. Times are the median of five timed runs after one warm-up, each ending in `device.synchronize()`. Memory is the pool's peak bytes in use over each form's six runs. The "unchunked (new mask)" columns run `choose_sorted_in` with a `usize::MAX` budget: one chunk, with the device mask. They stand in for the removed read's memory, not its time, since the removed host mask loop is not in them.

| positions × batch | rows a chunk | chunked: ms | chunked: GiB | unchunked (new mask): ms | unchunked (new mask): GiB | same picks |
|---|---|---|---|---|---|---|
| 192 × 4 | 192 (one chunk) | 0.770 | 0.07 | 0.753 | 0.07 | yes |
| 192 × 16 | 192 (one chunk) | 2.991 | 0.29 | 2.998 | 0.29 | yes |
| 512 × 4 | 512 (one chunk) | 5.297 | 0.51 | 5.288 | 0.51 | yes |
| 512 × 16 | 128 | 10.161 | 0.52 | 21.608 | 2.04 | yes |
| 1,024 × 4 | 256 | 10.242 | 0.52 | 21.031 | 2.03 | yes |
| 1,024 × 16 | 64 | 30.560 | 0.58 | 92.792 | 8.13 | yes |
| 2,048 × 4 | 128 | 30.584 | 0.57 | 94.493 | 8.13 | yes |
| 2,048 × 16 | 32 | 116.794 | 1.33 | does not run (What did not work, 2) | 16.02 at the failure | — |

- **The read's other quadratic term is now its memory floor.** The `(rows, time)` tensors are not chunked. At 2,048 × 16 each f32 or u32 one (the distance, its tie-key steps, the masked distance, the sort order) takes 0.25 GiB, and the u8 mask 64 MiB. They do not shrink with the chunk, and they do not scale with the width.
- **A forward has four heads,** so a read layer at 2,048 × 4 costs about 4 × 30.6 = 122 ms. That figure is projected from one head, not measured.
- **The other gates, from the same pod and pin:**
  - the CUDA library suite: 313 passed, 0 failed, 8 ignored;
  - `neg-chunk` (`at.saturating_sub(1)` as the chunk start): the chunk test FAILED at `flock_read_hamming.rs:2189`, the bit check, and the mask test passed;
  - `neg-mask` (`broadcast_le` in place of `broadcast_lt` against `t`): the mask test FAILED, with `left: 1` against `right: 0`, and the chunk test passed;
  - the restored source passed both;
  - the test binary `uor_r4_float_train-42192b48de362986` has sha256 `1a7e6472…f708` (in `test-bins.sha256`).

## What this shows, and what it does not

- **It shows** that the read now fits on one GPU at 2,048 positions, at the training batch (0.57 GiB) and the scoring batch (1.33 GiB). It picks exactly what the unchunked read picks wherever that can run, and it is 2.05× to 3.09× faster wherever it takes more than one chunk. The unchunked read cannot run at 2,048 × 16 on candle 0.9.2 on any card.
- **It does not time the removed code.** The unchunked columns use the new device mask, so the gain over the removed host loop is not measured here.
- **It does not show** that a model trains, or reads usefully, at 2,048:
  - no forward, training step or score was run at a long window;
  - the trainer still cuts 192-token windows (`SHEET_CONTEXT`);
  - a saved model refuses inputs past its trained `context` (`geometric_stack_native.rs:1165`);
  - the read's cost per position is still about 3.7× higher at 2,048 than at 192.

## Next

1. **P2 unit 3:**
   - make the window a training and scoring setting, with 192 as the default;
   - write the long TRAIN corpus and the long held-out panel from claude-271's generator.

   The read is no longer the limit there.
2. **P2b (Casey, 4 November: "Yes, add it"):** train at the 2,048 window on the long corpus, with a sheet, its FIT and a pre-run review. Score recall across turns against a no-memory control, and time a training step at 2,048 against 192. That step decides the stop rule in What did not work, 7.

## Checks run at the head

- **On the laptop**, at `267c1bc6d` (the merge `459f1a15e` adds only DeepSeek's CPU-training files): `cargo test -p uor-r4-float-train --no-fail-fast` gives the library 225 passed, `fbs-snapshot` 7, `spiralcore-fbs-sheet` 4, `train-in-space` 5, the dependency check 2 and the kernel controls 12, 0 failed. The two new tests and the timing test are CUDA-only, so the library count is unchanged.
- **The stub-`nvcc` type-check:** `cargo check --offline -p uor-r4-float-train --features cuda --all-targets` and the same `cargo clippy` both pass, with no warning in this file.
- **On the pod,** at `267c1bc6d`: the suite, the timing test, both negative gates and the restored run, as in Measurements. The job seals 21 files (`SHA256SUMS.files`, all OK on the laptop). Stored at `caseyallard/uor-r4-store:results/claude/hamming-read-long-windows-2026-11-04`; two files read back byte-identical.
- **`rustfmt`:** this crate is not rustfmt-clean at its base (41 diffs in this file before the change). The new lines follow the file's own style.
- **`loop gates --full`**, `loop report --check` and `loop pr-check --local`: run at the head before review.
- **Review:** a fresh-context review of the diff and this REPORT, recorded on PR #3644 by `loop review`. The job script had its own fresh-context pre-run review: PASS-WITH-FIXES, all four fixes applied before the run.

## Skills used

None applied.

## Still open

1. **The launch-size hazard past 2³² elements** (What did not work, 2). candle 0.9.2 casts element counts to `u32` for its launches. No path in this unit builds such a tensor any more. Nobody has checked other paths at long windows. The P2 unit 3 window change should assert, or chunk, any tensor it sizes by `batch × time × time`.
2. **A training step at 2,048 is untimed** (What did not work, 7). P2b times it.
3. **The packed-sign-bit kernel** (the ORDER's stop rule) is not needed for memory. It stays the remedy if P2b finds the read too slow next to the trunk.
4. **claude-270's served-mode parity check** is still owed, by the next GPU unit that scores a served checkpoint.
5. **The `uor-pod down` archive fault** (What did not work, 6) is tooling outside this lab's lane.

## Plain summary

The part of the model that looks back over earlier words used to build one huge table at a time. At 2,048 word pieces that table needed more memory than a top gaming GPU has. Its size also passed a limit in the GPU library, so it could not run at all.

This unit makes the model fill that table in small pieces. The answers are exactly the same, and it now needs at most about 1.3 GB at 2,048 word pieces. At the larger sizes it is also two to three times faster, because it never writes and rereads the huge table.

It does not make long conversations work yet. The model is still trained on pieces of 192 words, and a saved model will not read past the length it was trained on. The next unit lets the window be set. Then, at Casey's request, a training run at 2,048 tests whether the model actually learns to recall across turns.

References #3407.
