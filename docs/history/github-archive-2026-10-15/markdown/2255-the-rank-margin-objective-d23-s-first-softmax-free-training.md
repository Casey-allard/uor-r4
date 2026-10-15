# #2255 — The rank-margin objective: D23's first softmax-free training loss, with its CUDA kernel (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T15:39:48Z
- **Updated:** 2026-10-12T21:34:35Z
- **Closed:** 2026-10-12T15:42:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2255
- **Merged:** 2026-10-12T15:42:13Z
- **Merge commit:** bbbdd362a3731f68da3bf06112ba12aae783870f
- **Base / Head:** main / UOR-Foundation:kimi/margin-objective-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 → 2/3 (preparation PR; names rung 0) · headline: unchanged (M1 v5 memory 20/40, reply 37/232)

**What this is.** The replacement learning law's first piece (pre-registered rung 0, #2029 comment 6124773996 as amended in 6126881953): `Objective::MarginHinge`, a rank hinge `mean_k relu(1 − z_target + z_k)` over each row's strongest rival logits — **no exponential of any kind in the forward or the gradient** (the owner's no-softplus/no-softmax-shaped rule is honored by construction). Implemented for the CPU with its CUDA f32 forward/backward kernels, wired into `train` and `dialogue-train` behind `objective=hinge|hinge8`. Source work only: no model compute.

**Files**
- `crates/uor-r4-training/src/geometric_stack.rs` — the `Objective` enum + `StackConfig::objective` (a `Ce` config writes no field: every existing config stays byte-identical), the `MarginRank` op (CPU fwd/bwd; exact weighted-mean semantics of `CrossEntropy`), loss dispatch, nine focused tests.
- `crates/uor-r4-training/src/cuda_stack_kernels.rs` — `margin_rank_rows` / `margin_rank_grad`: one warp per row, per-lane top-k selection merged on lane 0, ties to the lower index (the CPU stable-sort's order).
- `crates/uor-r4-training/src/geometric_stack/cuda_ops.rs` — the launchers (f32; Metal refused for now).
- `crates/uor-r4-training/examples/geometric-stack.rs` — the `objective=ce|hinge|hinge8` flag; a pointer with a margin objective is refused (its own objective is the next rung).
- `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs` — the CUDA-vs-CPU margin parity test (loss + gradients at 1 and 8 rivals).
- `docs/labs/margin-objective-2026-10-12/README.md` — the record; `docs/labs/d23-learning-law-design-2026-10-12/README.md` — the revised replacement program with the owner-supplied SpiralCore/murmurations deep review; `docs/integration/current-state.md` — the entries.

**Checks at exact head:** `cargo test -p uor-r4-training --lib` **938 passed / 0 failed** (with UOR_BUILD_SOURCE_COMMIT); `cargo check --examples --bins --tests` clean; `cargo fmt --check` clean; `check_claim_wording.py` PASS. **CUDA gate (the #2212 lesson):** full parity suite **38 passed / 0 failed on an RTX 4090** (pod `zkjg238gzdhyv3`, cap 89, nvcc 12.8, rustc 1.97.1) at `f1e9acb7a`, including `test_margin_objective_parity ... ok`; the gate caught two defects pre-merge (an out-of-bounds tie-break read, CUDA-only test-build literals), both fixed and re-verified; the pod is released and deleted.

**Review:** exact-head self-review (owner rule), scope the files above.

**Decision test:** if the margin objective is rejected, rung 0 and every D23-compliant training run changes; if it lands, the project has its first lawful training law and rung 0 runs next.

References #2029.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

