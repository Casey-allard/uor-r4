# REPORT: training-step-cost, deepseek-3 (fused L1 rival search)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Order: [ORDER.md](ORDER.md). Base: `2b46c7080` (after PR #3825).

## What did not work, first

- **The fused path is not a bit-identical training replacement.** Its loss is bit-equal in the
  finite model step, but `embedding.weight`'s largest relative gradient move is 1.14%, the same
  delta as the selector-only tie result. It remains off by default pending a controlled training
  comparison.
- **The first parity fixture tied in L1.** Old and new chose the same set but ordered the lower
  scores differently. The fixture now uses a deterministic continuous LCG and 32-wide vectors;
  exact order agrees there.
- The selector-only path from deepseek-2 saved only 1.07% of a synthetic step. This fused search
  removes the score matrix and the selection passes together.

## What changed

- **`top8_positions.cu`:** `nearest_top8_positions` gives one block a query row. Eight warps
  score table candidates with coalesced width scans and keep local top-eight lists; thread 0
  merges them. The `(rows, vocabulary)` score matrix is never written.
- **`top8_positions.rs`:** a three-input Candle op checks query/table/target shapes, widths up to
  1,024 and the eight-rival cap, then launches 256 threads a row. CPU use is refused.
- **`in_space_long.rs`:** the nearest objective uses fused positions under `--fast-rivals 1`.
  The target score, rival-vector gather, hinge and gradients remain the existing path.
- Tests compare fused positions with the old score-plus-selection path and time both search
  shapes. The prior model-step timing test now exercises the fused path.

No default path, serving path, pinned CUDA source or checkpoint fingerprint changes.

## Measurements

One RTX 5090. Search medians alternate the old score matrix plus repeated argmax and the fused
kernel, after two warm-ups and over ten timed calls a shape:

| Rows | Vocabulary | Width | Old search | Fused search | Speed-up |
|---:|---:|---:|---:|---:|---:|
| 256 | 4,096 | 256 | 7.565 ms | 1.492 ms | 5.07× |
| 2,048 | 4,096 | 256 | 47.323 ms | 2.859 ms | 16.55× |
| 4,096 | 4,096 | 256 | 93.519 ms | 4.458 ms | 20.98× |

The synthetic nearest-objective step, 16 windows × 192 tokens in four micro-batches of four:
**139.524 ms old, 83.627 ms fused, 40.1% shorter (1.67×)**. Loss is identical:
`1.0039109885692596`. `embedding.weight`'s largest relative gradient move is **1.14%**; the test
does not localize the cause, with tied rival selection the leading hypothesis.

This is not a b64L or production-speed claim. The step uses `FbsArm::OneAtomActive`,
`served=false`, no Hamming read and no optimizer update. Served-mode timing and parity are
unverified in this unit.

## Checks run at the head

Result-producing code head: `d429eb7b9`. Later commits change records only.

- `cargo test --release --offline --locked -p uor-r4-float-train`: 297 library tests, 7 snapshot
  tests, 9 trainer tests, 2 dependency/kernel-scan tests and 12 finite controls passed; 0 failed.
- The pod's real nvcc 12.8 build compiled the CUDA-enabled library, binaries and tests before
  running the named device tests.
- RTX 5090 `the_fused_nearest_selector_matches_the_old_search_on_untied_rows`: passed, including
  a case where the target is the nearest table row and must be excluded explicitly.
- RTX 5090 `time_the_fused_nearest_search`: passed and produced the search table above.
- RTX 5090 `the_fast_rival_selector_keeps_the_loss_and_measures_tie_gradient_delta`: passed,
  now through the fused path, with the measurements above.
- Clippy on all targets: completed with the crate's established warnings; none is on a changed
  line.
- `git diff --check`: passed.

Spend about $0.12 across two short RTX 5090 runs; timing logs are archived at
`vast-archive/vast-59603424/uor-r4/jobs` and the final target-exclusion check at
`vast-archive/vast-59604710/uor-r4/jobs`.

## Skills used

No system-provided Codex skill applied.

## Still open

1. Run a matched real b64L A/B. The fused search cuts the synthetic step by 40.1%, enough to
   matter, but tied gradients differ. That run must compare top-1, openings and copies as well
   as step time before defaulting the flag.
2. A capability comparison must decide whether deterministic tie choices help or hurt. Until it
   does, default stays off.
3. If the real run holds, make the fused search the served-training path; the separate
   selector kernel remains useful for writer-cell objectives.

## Plain summary

The trainer used to write every query's scores against every token and then scan the vocabulary
eight times. The fused kernel now computes the L1 scores and keeps the eight nearest wrong tokens
without writing that large matrix. Search time falls 5.1×–21× and a finite model step falls 40.1%,
with identical loss. Tied rival choices still move the synthetic gradient by 1.14%, so the feature
stays behind an off-by-default flag and needs one real training comparison before Claude should
use it.
