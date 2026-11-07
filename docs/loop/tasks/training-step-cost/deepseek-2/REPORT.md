# REPORT: training-step-cost, deepseek-2 (one-pass exact rival selection)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Order: [ORDER.md](ORDER.md). Base: `11dd0b623` (after PR #3823).

## What did not work, first

- **The fast selector is not a drop-in training replacement.** It keeps the synthetic step's
  loss bit for bit, but `embedding.weight`'s gradient moved by 0.75% in the first device run and
  1.14% in the measured run. The cause is not localized; changed rival choices on tied scores are
  the leading hypothesis. The flag stays off; a controlled training comparison is required
  before it can become a default.
- **The first design would not have preserved gradients.** Replacing the nearest objective with
  the existing row-loss kernel acted on a deliberately detached score table, so the loss would
  have carried no gradient to the query or table. The unit now replaces only the exact position
  selection; the established differentiable rival and target path is unchanged.
- **The first draft changed the claim to a fast nearest loss and referenced the wrong kernels.**
  The delivered form is a selector, not a new loss, and it uses its own CUDA module.
- **Candle CUDA argmax is not deterministic on tied scores.** The first device parity run made
  an all-equal row and returned `[0,8,4,2,10,6,1,9]`; the new selector returns
  `[0,1,2,3,4,5,6,7]`. Non-tied rows agree. The flag is therefore an intentional deterministic
  tie mode, not a bit-identical drop-in on ties, and remains off by default.
- **The one-step model test is not gradient-identical.** Its loss is bit-equal with and without
  the flag, but `embedding.weight` moves by 0.000052805 at scale 0.007034846 (0.75% relative) on
  the synthetic nearest-objective step. The test does not localize the cause; tied rival choices
  are the leading hypothesis. It is a negative for exact behavioral preservation, not floating
  noise.
- **The selector expands the item's scope.** Its own module needs `build.rs`, `lib.rs`,
  `cuda_stack_kernels.rs` and two new source files. The item file records that scope in this
  branch; the policy checker reports it as an advisory because it reads the pre-change scope at
  the merge base.

## What changed

- **`top8_positions.cu`:** one warp per row. Each lane keeps the top eight allowed positions of
  its strided scan; lane 0 merges the 32 lane lists. Order is score descending and token id
  ascending on ties. Non-tied order agrees with repeated `argmax`; tied order is now explicit.
- **`top8_positions.rs`:** a Candle custom op that admits contiguous F32 scores and a U8 allowed
  mask, checks shape and the eight-rival cap, and launches one 32-thread block a row. It refuses
  the CPU and performs no fallback.
- **`in_space_long.rs`:** `--fast-rivals` is a thread-local flag read by `best_positions`. Off,
  the old repeated argmax code runs unchanged. On, the new selector returns the same positions
  where scores differ and deterministic lower-id positions on ties.
- **`train-in-space.rs`:** `--fast-rivals 1` applies only to `--out` training with a nearest-style
  objective. It is absent from `config.json` when off.
- **The build:** `top8_positions_f32.ptx` is its own ahead-of-time module. The pinned
  `cuda_stack.cu` and `spiralcore_fbs.cu` are byte-unchanged, so checkpoint fingerprints do not
  move.
- **Tests:** a CPU test compares the host model of the warp merge with repeated argmax, with ties,
  masks and fewer than eight allowed entries. A CUDA test checks exact agreement on non-tied rows
  and the lower-id rule on a fully tied row.

Size: 9 files, 450 insertions, 31 deletions in the code commit.

## Measurements

One RTX 5090, alternating old and fast calls, 3 warm-ups and 20 timed calls a shape. Medians:

| Rows | Vocabulary | Repeated argmax | One-pass selector | Speed-up |
|---|---:|---:|---:|---:|
| 256 | 4,096 | 0.286 ms | 0.078 ms | 3.69× |
| 2,048 | 4,096 | 2.260 ms | 0.142 ms | 15.89× |
| 4,096 | 4,096 | 4.994 ms | 0.260 ms | 19.23× |

The selector is fast enough to matter, but its share of one training step is smaller. A finite
nearest-objective step over 16 windows of 192 tokens, four micro-batches of four, measured
147.255 ms old against 145.676 ms fast: **1.07%**. Its loss was
`1.0039109885692596` in both paths, bit for bit. The largest relative gradient move was **1.14%**
(`embedding.weight`). The test does not localize the cause; tied rival selection is the leading
hypothesis, not a measured cause.

## Checks run at the head

Result-producing code head: `a6a74a7f1`. Later commits change comments, tests and records only.

- `cargo test --release --offline --locked -p uor-r4-float-train`: 297 library tests, 7 snapshot
  tests, 9 trainer tests, 2 dependency/kernel-scan tests and 12 finite controls passed; 0 failed.
- `fast_rivals_restore_their_prior_state_after_an_error_or_panic`: passed.
- Stub-nvcc CUDA all-targets check: passed with `CUDARC_CUDA_VERSION=12080` and
  `UOR_NVCC_ARCH=compute_120`.
- RTX 5090 `the_one_pass_rival_selector_keeps_non_ties_and_makes_ties_deterministic`: passed,
  1 test in 0.33 s.
- RTX 5090 `time_the_one_pass_rival_selector`: passed and produced the three medians above.
- RTX 5090 `the_fast_rival_selector_keeps_the_loss_and_measures_tie_gradient_delta`: passed after
  recording the gradient difference rather than asserting exact equality.
- Clippy on all targets: completed with only the crate's established warnings; none is on a
  changed line.
- `git diff --check`: passed.

All listed tests ran. Not run: a real b64L recipe, model scoring, or a capability comparison
between the tie modes.

## Skills used

No system-provided Codex skill applied. The repository's charter, geometry, D37 and loop rules
were used directly.

## Still open

1. Run a matched real b64L A/B. The current synthetic reading is a 1.07% step saving with a
   non-zero gradient delta, so it does not justify changing the active recipe.
2. A capability comparison must decide whether deterministic tie choices help or hurt. Until it
   does, default stays off.
3. The higher-value follow-up is to keep this selector and also fuse whole-table L1 scoring into
   the same pass, removing the score matrix as well as the eight selection rounds.

## Plain summary

The trainer's nearest-token search used eight whole-vocabulary CUDA passes. The new selector finds
the same eight wrong tokens in one warp-per-row pass and is 3.7× to 19× faster for the tested
shapes. It does not preserve the old gradient exactly when scores tie, however: Candle's old
argmax does not promise a tie order. A finite model step is 1.07% faster with identical loss but a
1.14% gradient delta. The feature therefore stays behind an off-by-default flag and needs a real
training comparison before Claude should use it.
