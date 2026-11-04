# ORDER: one-model-end-to-end, claude-272 (the Hamming read at long windows)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `1325bd064`. Draft PR #3644.

## Goal

claude-271 wrote conversations of up to 2,048 tokens. Training and scoring cannot read them yet. The Hamming read's `choose_sorted` (`flock_read_hamming.rs`) builds a `(batch, time, time, head_width)` f32 difference for each head, and then its absolute value as a second tensor of the same size. It also builds the candidate mask in a host loop over `rows × time` entries.

At width 64, one such tensor is 4.3 GB at 2,048 tokens with the training batch of 4, and about 17 GB with the scoring batch of 16 (shape estimates).

This unit computes the same distances in row chunks under a byte budget, and the same mask on the device. It then times the read on one GPU at 192, 512, 1,024 and 2,048 tokens.

- **Plan step:** Casey's 4 November plan, P2, in its own words: "Run a finite GPU test of the Hamming read's memory and time at 512, 1,024 and 2,048".
- **What it changes in the model:** the read that training and scoring use at long windows. Its picks stay byte-identical at every size.
- **No prereg** (§4.5): this is a finite kernel test, not training or evaluation.

## Cancel probe (run first)

1. Does `choose_sorted` still build the full difference and the host mask?
2. Does any open order hold `flock_read_hamming.rs`?
3. Does candle 0.9.2's CUDA `arg_sort_last_dim` take 2,048 columns? Its launch uses `block_dim = min(ncols_pad, 1024)` with striding threads (`sort.rs:87–93`).

**Result (4 Nov, ~05:45Z, at `1325bd064`). Class: instrument, no cancel.**
1. **Yes.** The full difference is at `flock_read_hamming.rs:1377`, and the host mask loop at `:1395`.
2. **No.** No open PR changes `flock_read_hamming.rs`.
3. **Yes, by reading.**
   - `candle-core-0.9.2/src/sort.rs:87–93` launches one block per row, with `ncols_pad.min(1024)` threads and `ncols_pad * 4` bytes of shared memory: 8 KB at 2,048.
   - `candle-kernels-0.9.2/src/sort.cu` strides each thread over the padded columns.

   The GPU test confirms it at 2,048.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `docs/loop/tasks/one-model-end-to-end/claude-272/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. **Chunked and on the device.**
   - `choose_sorted` takes query rows in chunks sized to a fixed byte budget, and builds `s >= first && s < t && s < limit` from device comparisons.
   - A CPU test pins `(order, real)` byte-identical to the unchunked form for:
     - chunk sizes 1, a non-divisor of `time`, and `time`;
     - the tie rule on and off;
     - row limits and `first` of 0 and 1.
   - **Negative gate:** an off-by-one chunk start fails it.
2. **The GPU test.** One ignored CUDA test runs on one pod (the smallest RTX 5090, one lease, under an hour). It times the read at 192, 512, 1,024 and 2,048 tokens, at batch 4 and 16 and width 64:
   - chunked at every size, and the old form where it fits;
   - with peak memory sampled by `nvidia-smi` during the run.

   On the same pod, the crate's CUDA library suite passes.
3. **A REPORT, negatives first,** and one notebook row.

## Stop and decide

- **Chunked 2,048 too slow to train** (each forward much slower than the trunk's): record the result. The next unit packs sign bits in an nvcc-built kernel (no NVRTC; never a dot product, per §3.3).
- **Any pick differs from the unchunked form:** this is a bug. Fix it; never loosen the test.
