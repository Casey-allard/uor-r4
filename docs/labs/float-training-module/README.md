# D37 Offline Float Training Module (`crates/uor-r4-float-train`)

**Issue:** `#2823` (`float-training-module`)  
**Order:** `docs/loop/tasks/float-training-module/antigravity-72/ORDER.md`  
**Notebook line:** [`docs/research/notebook/float-training.md`](../../research/notebook/float-training.md)  
**Rules:** CHARTER §3.1–§3.4, §3.7 (D37 offline float training), D23 (no softmax/squashing), D25/D30 (no dense matmul/GEMM), D29 (no NVRTC; ahead-of-time `nvcc` PTX with zero banned instructions), D32 (discrete model stays integer-only).

## 1. Crate Name and Workspace Placement

- **Crate name:** `uor-r4-float-train` (`crates/uor-r4-float-train/Cargo.toml`).
- **Root `Cargo.toml` placement:** Added as `"crates/uor-r4-float-train"` in the root `[workspace] members` list (line 6, between `"crates/uor-r4-training"` and `"crates/uor-r4-discrete"`).
- **Zero impact on other crates:** No other crate's `Cargo.toml` or `Cargo.lock` dependency entry is modified. `uor-r4-float-train` uses only dependencies already resolved in the workspace (`candle-core`, optional `cudarc` under `--features cuda`, `half`, `hex`, `safetensors`, `serde`, `serde_json`, `sha2`, and `uor-r4-core = { path = "../uor-r4-core" }`).

## 2. The 24-Kernel Extract and Source Provenance

Per `archive/audit/mechanisms/geometric-stack.json` (formerly `docs/labs/repo-renormalization-2026-10-19/audit/mechanisms/geometric-stack.json`) and [`docs/mechanisms/geometric-stack.md`](../../mechanisms/geometric-stack.md), the native geometric float training extract comprises **24 `__global__` CUDA kernels** across two ahead-of-time `nvcc`-compiled `.cu` translation units (`22` in `src/cuda_stack.cu` + `2` in `src/spiralcore_fbs.cu`) together with their Rust CPU/CUDA stack modules:

| # | Kernel entry symbol | Source file & line (`uor-r4-float-train`) | Geometric role |
|---|---|---|---|
| 1 | `straight_through_fwd` | `src/cuda_stack.cu:160` | Identity forward copy for straight-through discrete/float bridge paths |
| 2 | `hamilton_gate_fwd` | `src/cuda_stack.cu:172` | $S^3$ unit-quaternion rotation gate forward (`(g / |g|) ⊗ u` per 4-block via `sqrtf`) |
| 3 | `hamilton_gate_bwd` | `src/cuda_stack.cu:192` | Exact analytical adjoint of $S^3$ unit-quaternion gate with respect to `g` and `u` |
| 4 | `rate_select_fwd` | `src/cuda_stack.cu:259` | Exact dyadic rate-table forward lookup (`RATE_TABLE[idx]`, `KEEP_TABLE[idx]`) |
| 5 | `rate_select_bwd` | `src/cuda_stack.cu:269` | Exact dyadic rate-table backward zero-gradient pass (`0.0f` per element) |
| 6 | `rms_norm_fwd` | `src/cuda_stack.cu:297` | Row-wise RMS sphere projection forward (`x * (1.0 / sqrt(mean(x^2) + eps)) * w`) |
| 7 | `rms_norm_bwd_dx` | `src/cuda_stack.cu:316` | Row-wise RMS sphere projection backward with respect to input $x$ and inverse RMS `row_r` |
| 8 | `rms_norm_dw_partial` | `src/cuda_stack.cu:344` | Partial chunk reduction for RMS scale gradient $d\gamma$ |
| 9 | `rms_norm_dw_reduce` | `src/cuda_stack.cu:362` | Cross-chunk final reduction for RMS scale gradient $d\gamma$ |
| 10 | `quaternion_scan_fwd` | `src/cuda_stack.cu:377` | Causal sequential quaternion recurrence forward ($h_t = q_t \otimes h_{t-1} + d_t$) |
| 11 | `quaternion_scan_bwd` | `src/cuda_stack.cu:394` | Reverse-time exact adjoint for quaternion scan (`dq`, `db` transition and drive gradients) |
| 12 | `margin_rank_rows` | `src/cuda_stack.cu:508` | Piecewise-linear rank-margin hinge row loss ($\text{mean}_{k} \max(0, 1 - z_{\text{target}} + z_k)$ over top-`rivals`) |
| 13 | `margin_rank_grad` | `src/cuda_stack.cu:514` | Exact step subgradient of rank-margin hinge loss with respect to candidate logits |
| 14 | `recurrence_scan_fwd` | `src/cuda_stack.cu:587` | Chunked serial causal quaternion recurrence scan (`held_t = q_t ⊗ held_{t-1} + keep_t c_t`, one thread per `(window, lane)`) |
| 15 | `recurrence_tile_fold` | `src/cuda_stack.cu:646` | Pass 1 of tiled parallel scan: per-tile affine summary reduction $(A_{\text{tile}}, B_{\text{tile}})$ |
| 16 | `recurrence_tile_carry` | `src/cuda_stack.cu:674` | Pass 2 of tiled parallel scan: cross-tile prefix carry propagation |
| 17 | `recurrence_tile_expand` | `src/cuda_stack.cu:693` | Pass 3 of tiled parallel scan: intra-tile state expansion from tile incoming carry |
| 18 | `recurrence_bwd_params` | `src/cuda_stack.cu:724` | Fused backward recurrence parameter reduction (`d_tap0..d_tap3`, `d_bias`, `d_log_a`, `d_branches`) |
| 19 | `poison_words` | `src/cuda_stack.cu:778` | Deterministic `0xFFFFFFFFu` (all-ones NaN bit pattern) buffer poisoning for scratch verification |
| 20 | `adam_update` | `src/cuda_stack.cu:787` | On-device first/second moment AdamW parameter update |
| 21 | `sq_lanes` | `src/cuda_stack.cu:813` | Per-lane squared-norm accumulation for gradient norm monitoring |
| 22 | `sq_tree` | `src/cuda_stack.cu:845` | Tree reduction of squared norms across lanes |
| 23 | `spiralcore_fbs_fwd` | `src/spiralcore_fbs.cu:91` | 8D $\text{Cl}(0,6)$ Finite Bivector Set (`240` ternary signed permutation binders) forward routing |
| 24 | `spiralcore_fbs_bwd` | `src/spiralcore_fbs.cu:124` | Exact secant-pullback backward routing for $\text{Cl}(0,6)$ FBS binders and weights |

## 3. Zero Dependency Edge on `crates/uor-r4-training`

The chief architect's D37 audit (`docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`, Risk 1 at line 179) identified the primary structural hazard: reusing or depending on `crates/uor-r4-training`, which retains historical dense/sigmoid modules and legacy baselines.

`crates/uor-r4-float-train` eliminates any dependency edge on `crates/uor-r4-training`:
1. **Self-contained extract modules:** `src/cuda_stack.cu`, `src/spiralcore_fbs.cu`, `src/cuda_stack_kernels.rs`, `src/geometric_stack_native.rs`, `src/geometric_stack_native_cuda.rs`, `src/spiralcore_fbs.rs`, `src/native_vector_writer.rs`, and `src/native_starling_spec.rs` are hosted directly in `crates/uor-r4-float-train/src/`.
2. **Core geometry from `uor-r4-core`:** The $\text{Cl}(0,6)$ bivector algebra and 240-element composition table (`cl06_bivectors`, `cl06_finite_composition_table`, `SignedMatrix8`) are imported directly from `uor_r4_core::spiralcore_operator`.
3. **Minimal local error and digest glue:** The only symbols those modules previously took from `uor-r4-training` were `crate::TrainError` (`invalid`) and `sha256_bytes`. Both are defined locally in `crates/uor-r4-float-train/src/lib.rs`.
4. **Automated graph guard:** `crates/uor-r4-float-train/tests/no_training_crate_dep.rs` runs `~/.cargo/bin/cargo tree --offline -p uor-r4-float-train` and asserts that `uor-r4-training` and `candle-nn` are completely absent from the dependency tree.

## 4. Why Nothing in the Extract Uses Softmax/Squashing (D23) or Dense Matmul (D25/D30)

- **No softmax, log-softmax, softplus, sigmoid, tanh, GELU, or normalized exponential (CHARTER §3.2 / D23):**
  - **Gating:** `hamilton_gate_fwd` / `hamilton_gate_bwd` apply left unit-quaternion multiplication $(g / |g|) \otimes u$ on $S^3$ using algebraic `sqrtf` normalization — zero sigmoid, tanh, GELU, or SiLU gates.
  - **Rate selection:** `rate_select_fwd` / `rate_select_bwd` index an exact 16-entry dyadic power-of-two table (`RATE_TABLE = [1/2, 1/4, ..., 2^{-16}]`) rather than a continuous squashing gate.
  - **Normalization:** `rms_norm_fwd` / `rms_norm_bwd_dx` project onto the RMS sphere via algebraic `1.0 / sqrt(mean(x^2) + eps)`.
  - **Loss & gradients:** `margin_rank_rows` / `margin_rank_grad` compute the piecewise-linear rank-margin hinge loss $\text{mean}_{k} \max(0, 1 - z_{\text{target}} + z_k)$ and its exact step subgradient — zero exponential (`ex2.approx`), logarithm (`lg2.approx`), or partition function.
- **No dense matrix multiplication or GEMM (CHARTER §3.3 / D25 / D30):**
  - **Channel mixing:** `spiralcore_fbs_fwd` / `spiralcore_fbs_bwd` route 8D octonion blocks through the 240-element $\text{Cl}(0,6)$ Finite Bivector Set signed permutation lookup tables (`ScFbsTables`: `nodes`, `frontier`, `axes`, `compose`, `root_of_state`, `bfs_actions`, `branch_targets`, `roots`), replacing $O(d^2)$ dense matrix multiplication with sparse ternary basis-action accumulation.
  - **Sequence mixing:** `quaternion_scan_fwd` / `recurrence_scan_fwd` / `recurrence_tile_*` combine tokens via cell-wise Hamilton products ($q_t \otimes h_{t-1}$) and $O(T \cdot d)$ associative prefix scans.
  - **Readout / vector writing:** `native_vector_writer.rs` scores vocabulary candidates via shared deterministic sign/cell tables (`shared_vector_table`) and cell routing without any dense $[d_{\text{model}}, V]$ GEMM or `cublasGemm`.

## 5. Mapping D37 Item 6 ("Own Scanned Allowlist Entry") onto `ratchet.rs`

`crates/uor-r4-harness/src/ratchet.rs` enforces the repository's three token/operator ratchets (`d23` / `NormExp`, `d25` / `Matmul`, and `d32` / `DiscreteFloat`). To implement D37 item 6 in Rust (with no new Python):
1. `ratchet.rs` defines `pub const D37_SCANNED_FLOAT_CRATES: [&str; 1] = ["crates/uor-r4-float-train/"];` and `pub fn is_d37_float_scope(rel: &str) -> bool`.
2. `scan_roots(repo_root)` explicitly registers `D37_SCANNED_FLOAT_CRATES` as its own scanned root entry alongside `crates` and `docs/labs/*/instrument` (deduplicating file paths so `crates/uor-r4-float-train/` is guaranteed scanned under its own entry without double counting).
3. Under `Gate::NormExp` (`d23`) and `Gate::Matmul` (`d25`), every source file in `crates/uor-r4-float-train/` is scanned with a strict **zero-site ceiling**: `growth()` clamps `base = 0` for `is_d37_float_scope` so any `d23` or `d25` hit fails immediately even against a forged baseline, and `update_baseline` refuses to record nonzero baseline entries for any path matching `is_d37_float_scope`.
4. Under `Gate::DiscreteFloat` (`d32`), `is_discrete_scope(rel)` explicitly excludes `is_d37_float_scope(rel)`, codifying that offline float training is allowlisted in `crates/uor-r4-float-train/` while `crates/uor-r4-discrete/` remains strictly integer-only.
