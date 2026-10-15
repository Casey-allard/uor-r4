# #2262 — OP2: the fused Hamilton projection — matmul-free quaternion linear maps, with CUDA kernels

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T01:57:49Z
- **Updated:** 2026-10-13T02:03:20Z
- **Closed:** 2026-10-13T02:01:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2262
- **Merged:** 2026-10-13T02:01:22Z
- **Merge commit:** 19500bf2282e0f87cbb56ef9f2ff7f69a2231d61
- **Base / Head:** main / UOR-Foundation:kimi/hamilton-linear-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 1/3 · headline 37/232 → 37/232 (no move claimed)

## OP2: the fused Hamilton projection — the quaternion maps without any matrix product

Head `75b8141a21605b3aa930578665f8b67e1837fa20`, branched from `origin/main` @ `7567cf725` (which contains the
merged OP1, #2260). Branch `kimi/hamilton-linear-2026-10-13`.
Record: `docs/labs/hamilton-linear-2026-10-13/README.md`.
Decision: **NOT YET PROMOTED** — source only, no run and no device receipt; the training hold stands.

### Why

The owner's amended order ([#2029 comment 6135511346](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6135511346),
carried into D25 and the operator card) puts OP2 first because `quaternion_weights=all` expressed its Hamilton
blocks through a dense GEMM: `params()` materialized each map as the `[4·out, 4·in]` left-multiplication matrix
and `linear` ran it through cuBLAS. OP2 makes that path a structured geometric product.

### What landed

- **`HamiltonLinear`** (`CustomOp2`): `y_j = Σ_k W_jk ⊗ x_k` for a compact weight `[out/4, in/4, 4]` and an
  activation `[.., in]`. One Hamilton product per term (16 products, 12 sums); `f64` accumulation in the CPU
  reference, parallel over output blocks. Both adjoints implemented: `dW_jk = Σ_rows dy_j ⊗ conj(x_k)` and
  `dx_k = Σ_j conj(W_jk) ⊗ dy_j`, with the public entry point `hamilton_linear`.
- **CUDA kernels** `hamilton_linear_fwd`, `hamilton_linear_bwd_w`, `hamilton_linear_bwd_x`: one warp per output
  quaternion / weight block / activation block, lanes striding the reduced dimension, shuffle-down reduce of
  the four accumulators, `act_t`-typed so one source serves the f32 and bf16 builds (f32 accumulation, round on
  store). Launchers in `cuda_ops.rs` + `cuda_ops_bf16.rs`; they bail (never `via_host`) for a non-contiguous,
  partial-block or mismatched input.
- **Dispatch**: `StackModel::linear` sends a compact quaternion map (3-D weight) to the fused op and leaves
  every other map on the existing dense path, byte for byte. The dense/SwiGLU path is untouched.
- **Metal** refuses the op (no kernel) — a quaternion-weights model no longer runs on Metal, where it used the
  banned dense GEMM.
- **One new refusal**: the compiled native value projection needs a dense `read.value.weight` and now says so
  for a compact one instead of failing on a shape mismatch.

### The weight-storage decision (investigated before choosing)

**Nothing new is stored: the compact form already is the artifact.** `set_quaternion_weights(All)` *replaces*
each dense map with its compact variable; `save` writes those variables by name and `load` rebuilds the
expected shapes with `quaternion_shapes` and errors on any dim mismatch — so the saved format has always been
`[out/4, in/4, 4]`, and the dense form existed only as a per-forward-pass materialization. OP2 therefore reads
the compact variables directly (after the same `quaternion_snap_straight_through` the old path applied, so the
snap and its straight-through gradient are unchanged) and **deletes the per-forward expansion**. No config
field, tensor name, shape or file changes: existing artifacts load unchanged, and the forward no longer
allocates `[4·out, 4·in]` per map per pass.

Honest scope: the *executed arithmetic* changed (fused Hamilton product with `f64` CPU accumulation instead of
an expanded GEMM), so values agree to `< 1e-5` rather than bit for bit. Save → load reproduction is still
bit-identical (the crate's round-trip test still passes).

### Checks at this exact head (offline, owner's laptop)

- [x] Checks run at this exact head, offline, on the owner's laptop
- [x] `cargo test -p uor-r4-training --lib --offline` — 949 passed, 0 failed, 7 ignored (base 944 + 5 new)
- [x] `cargo check -p uor-r4-training --lib --offline` — clean (pre-existing warnings only)
- [x] `cargo check -p uor-r4-training --examples --bins --offline` — clean
- [x] `cargo check -p uor-r4-training --all-targets --offline` — 0 errors
- [x] `cargo fmt -p uor-r4-training --check` — clean
- [x] `python3 scripts/check_no_softmax.py` — OK, 1649 remaining (baseline 1649)
- [x] `python3 scripts/check_claim_wording.py` — PASS

The five new lib tests are fixtures of the arithmetic, not model evidence: a hand-computed two-block case,
agreement with the dense expansion (`quaternion_dense_weight`) to `< 1e-5`, central-difference gradients for
both the weight and the activation, the refusals (partial block, mismatched width, dtype), and a
quaternion-weights model's forward and backward with a finite gradient on all 13 compact maps of the fixture.
The crate's pre-existing quaternion tests all still pass, including the whole-model central-difference gradient
check and the save/load round trip.

Device parity tests, skipped on this host (no CUDA device) and failing a missing device under `UOR_REQUIRE_CUDA=1`:
`test_hamilton_linear_parity` (forward + both gradients, CPU vs CUDA) and `test_bf16_hamilton_linear_parity`
(the bf16 trunk against its own f32 trunk).

### Scope of the local evidence

No CUDA toolkit here (`cudarc` fails on `nvcc`), so device qualification is pod work. Substitute, with its
scope: the three kernels were compiled with `clang++` (builtin stubs) and **executed on the host from the same
source text**, one lane at a time with the warp reduce emulated by summing the lanes' pre-reduce partials;
they match an independent `f64` implementation of the forward and both adjoints to `<= 8.4e-7` peak-scaled.
Kernel arithmetic and indexing only — nothing about device launch, the shuffle primitive on hardware, the bf16
storage path, or model behavior.

### Limitations, and what the pod run should watch

1. **A sign error was caught this way before delivery** — two components of `dy ⊗ conj(x)` in
   `hamilton_linear_bwd_w` were wrong until the host execution against the reference exposed them. Run that
   kernel against analytic values first in the pod session rather than trusting the parity test's tolerance.
2. Device launch geometry: `rows*out_blocks*32`, `out_blocks*in_blocks*32` and `rows*in_blocks*32` threads (a
   whole number of warps in 256-thread blocks).
3. bf16: the twins are the same source with `act_t`; a "requires F32" error would mean the bf16 storage path
   did not reach the op.
4. The forward's values changed relative to the pre-OP2 dense expansion (`< 1e-5`, not bit-identical).
5. Metal is refused, and the compiled native value projection refuses compact weights — both by design here.
6. No capability, serving or model-quality claim. Export/D11 still refuse quaternion weights; the card calls
   that an unfinished dependency, not compliance.

This PR also carries the lead's harness receipt line (`docs/harness/ledger/2026-10.jsonl`).

Pre-registration: none is needed for this PR, which runs no compute. The kernel qualification this source
feeds needs its own prospective kernel card and `TEST FITNESS` verdict first (the amended card fixes no
tolerance or speed bar of its own, and the freeze holds it).

Cost: one implementation session on the owner's laptop, no paid compute, no pod leased.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

