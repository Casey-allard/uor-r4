# OP2 — the fused Hamilton projection, implemented (2026-10-13)

**Question.** Per the owner's amended build order
([#2029 comment 6135511346](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6135511346),
carried into [D25](../integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction) and the
[operator card](operator-phase-2026-10-12/README.md)): can the quaternion-valued projection maps
(`quaternion_weights=all`) execute as their own structured Hamilton products — forward **and** gradient — with
no dense matrix product on any path the optimizer reaches?

**State: implemented and locally checked. No run, no device receipt, no model numbers.**

**Decision: NOT YET PROMOTED.** Source only: nothing here is a capability, a headline or an acceptance
result, and the training hold is untouched. Per the card, OP2's device qualification
(parity against analytically specified values) still needs its own prospective kernel card, and the parity
receipt waits for TEST FITNESS under the rule freeze.

## Why this was needed

`quaternion_weights=all` already stored every projection as one quaternion per 4-lane block, but its
*execution* materialized each map as the dense `[4·out, 4·in]` Hamilton left-multiplication matrix and ran it
through `matmul` — cuBLAS, i.e. the banned dense contraction. OP2 replaces that execution with the fused
Hamilton product, so the whole quaternion path is a structured geometric product.

## What landed

- **`HamiltonLinear`** (`crates/uor-r4-training/src/geometric_stack.rs`, `CustomOp2`): the compact weight
  `[out/4, in/4, 4]` times an activation `[.., in]` read as `in/4` quaternion blocks,
  `y_j = Σ_k W_jk ⊗ x_k`. Each term is one Hamilton product (16 products, 12 sums); the CPU reference
  accumulates in `f64` and is parallel over output blocks; the CUDA path accumulates in `f32`.
  Forward and backward (`dW_jk = Σ_rows dy_j ⊗ conj(x_k)`, `dx_k = Σ_j conj(W_jk) ⊗ dy_j`) are both
  implemented; `pub fn hamilton_linear` is the public entry point.
- **CUDA kernels** (`cuda_stack_kernels.rs`): `hamilton_linear_fwd`, `hamilton_linear_bwd_w` and
  `hamilton_linear_bwd_x`, one warp per output quaternion / weight block / activation block, lanes striding
  the reduced dimension, `act_t`-typed so one source serves the f32 and bf16 builds (f32 accumulation, rounding
  on store). Launchers in `cuda_ops.rs` + `cuda_ops_bf16.rs`: they bail (never `via_host`) for a
  non-contiguous input, a partial block or a mismatched width.
- **Dispatch** (`StackModel::linear`): a compact quaternion map (a 3-D weight) takes the fused route; every
  other map keeps the existing dense product, byte for byte. The dense/SwiGLU path is untouched.
- **Metal**: refused (`bail`, the established pattern). A quaternion-weights model therefore no longer runs on
  Metal; the dense GEMM it used there is banned in any case, and a Metal kernel for this op is not in OP2.
- **One new refusal**: the compiled *native value projection* validates a dense `read.value.weight` and now
  refuses a compact one with a clear message instead of a confusing shape error. That combination needed a
  dense weight; nothing else about that path changed.

## The weight-storage decision

**Investigated first, then decided: store nothing new — the compact form already *is* the artifact.**

- `set_quaternion_weights(All)` **replaces** each dense map with its compact variable
  (`layers.NN.mlp.gate.weight` → `layers.NN.mlp.gate.quaternion`, shape `[out/4, in/4, 4]`); the dense tensor
  is not kept.
- `save` writes `self.variables` by name to `model.safetensors`, and `load` builds the expected shapes with
  `quaternion_shapes` and **errors if a saved tensor's dims differ**. So the saved format has always been the
  compact form; the dense form existed only inside `params()` as a per-forward-pass materialization
  (`quaternion_dense_weight`).
- Therefore OP2 reads the compact variables directly (after the same
  `quaternion_snap_straight_through` the old path applied, so the snap and its straight-through gradient are
  unchanged) and **deletes the per-forward dense expansion entirely**. No config field, no tensor name, no
  shape and no file changes: every existing quaternion-weight artifact loads unchanged, and the model saves a
  quarter-size map set as before. As a side effect the forward no longer allocates the `[4·out, 4·in]`
  expansion per map per pass.

Honest scope of that choice: the *executed* arithmetic did change from `L(W) @ x` (a GEMM) to the fused
Hamilton product with `f64` CPU accumulation, so a model's values are no longer bit-identical to the old
path — they agree to `< 1e-5` on the CPU (asserted below). Existing artifacts still load and reproduce their
own forward exactly (save → load is bit-identical; the crate's round-trip test still passes).

## Checks actually executed (owner's laptop, 2026-10-13)

| Check | Result |
|---|---|
| `cargo test -p uor-r4-training --lib --offline` | **949 passed / 0 failed / 7 ignored (base 944 + 5 new)** |
| `cargo check -p uor-r4-training --lib --offline` | clean, pre-existing warnings only |
| `cargo check -p uor-r4-training --examples --bins --offline` | clean |
| `cargo check -p uor-r4-training --all-targets --offline` | 0 errors |
| `cargo fmt -p uor-r4-training --check` | clean |
| `python3 scripts/check_no_softmax.py` | OK: 1649 remaining (baseline 1649) |
| `python3 scripts/check_claim_wording.py` | PASS |
| `python3 scripts/harness/cli.py lint check` | PR body lints with 0 errors |

New lib tests, all CPU fixtures of the arithmetic rather than model evidence: a hand-computed two-block case,
agreement with the dense expansion (`quaternion_dense_weight`) to `< 1e-5`, central-difference gradients for
the weight and the activation, the refusals (partial block, mismatched width, dtype), and a quaternion-weights
model's whole forward and backward with a finite gradient on all 13 compact maps of the fixture.

The crate's existing quaternion tests all still pass, including the whole-model central-difference gradient
check and the save/load round trip.

Device parity tests are in place and **skipped here** (no CUDA device on the laptop):
`test_hamilton_linear_parity` (forward + both gradients, CPU vs CUDA) and `test_bf16_hamilton_linear_parity`
(the bf16 trunk against its own f32 trunk) in `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs`.
`UOR_REQUIRE_CUDA=1` turns a missing device into a failure on the pod.

## What the local checks cover, and what they do not

No CUDA toolkit on this host (`cudarc` fails on `nvcc`), so device qualification is pod work. Substitute with
a stated scope: the three kernels were compiled with `clang++` (CUDA builtin stubs) and **executed on the
host from the same source text**, one lane at a time with the warp's shuffle reduce emulated by summing the
lanes' pre-reduce partials; the results match an independent `f64` implementation of the forward and both
adjoints to <= 8.4e-7 peak-scaled on a `(rows 3, out/4 2, in/4 3)` case. That covers the kernels' arithmetic
and indexing only; it says nothing about device launch, the warp primitive on hardware, the bf16 storage path,
or any model behavior. The CPU reference and the per-forward dispatch are covered by the lib tests above.

## Findings for the lead

1. **A sign error was caught before delivery** in `hamilton_linear_bwd_w` (two components of
   `dy ⊗ conj(x)`), by comparing the host-executed kernels against the reference — the kind of defect the pod
   parity test is there to catch, and it argues for running `hamilton_linear_bwd_w` against analytic values
   early in the pod session.
2. **The forward's arithmetic changed** (fused Hamilton product instead of an expanded GEMM): values agree to
   `< 1e-5`, not bit for bit. Any retained quaternion-weight result should be re-derived if exact reproduction
   matters.
3. **Metal is refused** for quaternion-weight models now (they used the dense GEMM there).
4. **The remaining dense/fallback sites on this path**: the head (`embedding.weight`) and `read.null.weight`
   are not quaternion maps by design ([`QUATERNION_MAPS`]); the re-addition of the MLP gate's own matmul is
   OP1/OP2's business, and OP3 covers the head. Export/D11 still refuse quaternion weights
   (`quaternion_served_refusal`) — the card calls that an unfinished dependency, not compliance.

## Next

1. OP3a (the product-key head), then the rest of the lawful path, per the amended order; rung 0 resumes only
   after the whole executed path is lawful and the owner releases the hold.
2. The pod parity receipt for these three tests needs a prospective kernel card and TEST FITNESS first: this
   record fixes no tolerance or speed bar of its own.

**Cost:** one implementation session on the owner's laptop; no paid compute, no pod leased.
