# Interlock & Ratchet Hardening (D23 / D25 / D32, Commission 1) — 2026-10-16

- **Lab / Session:** `antigravity` (`ag1`)
- **Commission / Authority:** `#2028` comments [`6222315060`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6222315060), [`6222117146`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6222117146), and [`6222114728`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6222114728) (consolidated Commission 1; prior hold in `#2029` comment [`6222232136`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6222232136) released by PM).
- **Mode / Scope:** `native_geometric_ai` · source-only / host-only · `NOT_TRAINING` · `model_compute_executed: false` · `apply_calls: 0` · zero pods · $0 GPU cost.
- **Line / Count / Headline:** M1 qualification & enforcement tooling · count `0/3` unchanged · M1 headline `37/232` unchanged (suspended under D28/R14). Training, M2, and Stage B remain HELD.

---

## 1. Objective & Defects Remediated

The repository audit (`#2028` comment `6222114728`, Ranks 7–11) identified five enforcement gaps across the D23, D25, and D32 interlocks:

1. **Rank 7 (`ratchet d23` scanned only `D23_WORD`):** [`crates/uor-r4-harness/src/ratchet.rs`](../../../crates/uor-r4-harness/src/ratchet.rs) counted only case-insensitive occurrences of `D23_WORD`, leaving `softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, and `rotary` unguarded by `uor-harness ratchet d23`.
2. **Rank 8 (`ratchet d25` left tensor-core intrinsics `wmma` and `mma_sync` informational):** `TC_FRAGMENT` (`wmma`) and `TC_SYNC` (`mma_sync`) were counted in `tensor_core_sites` for an informational footer line (`informational: ... — not gated`) and excluded from `result.counts` and `result.detail`.
3. **Rank 9 (`SCAN_DIRS` omitted `docs/labs/*/instrument/`):** `const SCAN_DIRS: [&str; 1] = ["crates"]` skipped standalone instrument crates under `docs/labs/*/instrument/` (such as `docs/labs/k16-timing-2026-10-15/instrument/` and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/`).
4. **Rank 10 (`prlint.d23_interlock` ran only `Kind::NormExp`):** [`crates/uor-r4-harness/src/prlintrules.rs`](../../../crates/uor-r4-harness/src/prlintrules.rs) invoked `ratchet::scan` only for `Kind::NormExp` (`d23`) and never ran `Kind::Matmul` (`d25`) during `uor-harness lint check` or `uor-harness check`.
5. **Rank 11 (`r4-policy-check.rs` asserted stale pre-D25/D32 policy commitments):** [`crates/uor-r4-core/src/bin/r4-policy-check.rs`](../../../crates/uor-r4-core/src/bin/r4-policy-check.rs) asserted `/schema == "uor-r4.agent-execution-policy/5"` and `/architecture/training_matrix_multiplication_allowed == true`, failing against [`docs/integration/agent-execution-policy.json`](../../integration/agent-execution-policy.json) (which is at schema `/6` with `training_matrix_multiplication_allowed: false`).

---

## 2. Structural Changes

### 2.1 `crates/uor-r4-harness/src/ratchet.rs` and `crates/uor-r4-harness/src/main.rs`

- **Expanded D23 (`Kind::NormExp`) pattern families (`D23_IDENT_FAMILIES`):**
  - Preserves raw case-insensitive substring counting for `D23_WORD` (`concat!("soft", "max")`).
  - Adds `pub const D23_IDENT_FAMILIES: [&str; 8]` assembled via `concat!` (`softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, `rotary`).
  - `d23_sites(text)` strips comments (`//`, nested `/* */`) and plain string literals (`"..."`) while preserving raw strings (`r"..."`, `r#"..."#`) via `strip_comments_and_strings(text)` and splits on non-ASCII-alphanumeric boundaries (`!c.is_ascii_alphanumeric()`).
  - Each non-empty ASCII-alphanumeric code segment `seg` is tested by `is_d23_ident_segment(seg)`:
    - For `softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, and `rotary`, any case-insensitive substring match inside the alphanumeric segment counts (catching standalone identifiers, C float intrinsics such as `tanhf`, and CamelCase/PascalCase compounds such as `SigmoidGate`, `RotaryEmbedding`, `SoftplusLoss`, `GeluActivation`, `SiluGate`, `SiLU`, `GeLU`).
    - For `rope` and `ropei`, a segment counts when its lowercased form starts with `rope` (catching `rope`, `ropei`, `RopeMode`, `RopeTheta`, `RoPE`, `RoPEConfig`) or its case-preserved form contains `Rope`, `RoPE`, or `ROPE` (catching prefixed PascalCase/camelCase compounds such as `PreRopeProjectionObservations`, `PreRopeProjectionProbe`, `applyRope`), while producing zero false positives on English words containing `r-o-p-e` (`proper`, `Proper`, `property`, `Property`, `properties`, `Properties`, `is_proper_prefix`, `WriterProperties`) and never double-counting `ropei`.
- **Gated tensor-core intrinsics in D25 (`Kind::Matmul`):**
  - `matmul_sites` now scans stripped code lines for non-overlapping case-insensitive occurrences of `TC_FRAGMENT` (`wmma`) and `TC_SYNC` (`mma_sync`), pushing `Site { line, kind: "tensor-core", text }` into the gated site list (`result.counts` and `result.detail`) alongside `.matmul(`, `MatMul`, `cublas`, and whole-word `gemm`.
  - `result.tensor_core` and `result.tensor_files` are computed from the gated `"tensor-core"` sites, and `cmd_ratchet` in [`crates/uor-r4-harness/src/main.rs`](../../../crates/uor-r4-harness/src/main.rs) now prints `gated: N tensor-core intrinsic sites (fragment and sync intrinsics) across M files.`
- **Expanded `SCAN_DIRS` to cover `docs/labs/*/instrument/`:**
  - `const SCAN_DIRS: [&str; 2] = ["crates", "docs/labs/*/instrument"];`
  - `scan_roots(repo)` expands the `/*/` wildcard across non-symlink subdirectories of `docs/labs/`, scanning `.rs`, `.cu`, `.cuh`, and `.wgsl` files inside each `<lab-record>/instrument/` tree (skipping `target/`) while leaving dated prose/evidence outside `instrument/` untouched.

### 2.2 `crates/uor-r4-harness/src/prlintrules.rs`

- Added `pub const INTERLOCK_D25: &str = "uor-harness ratchet d25";`.
- Updated `interlock_holds` to run both `(Kind::NormExp, "D23", INTERLOCK)` and `(Kind::Matmul, "D25", INTERLOCK_D25)` in process and emit an `error` finding if either ratchet cannot run, lacks a readable baseline, or reports any file above baseline.

### 2.3 `crates/uor-r4-core/src/bin/r4-policy-check.rs`

- Updated `/schema` commitment to `"uor-r4.agent-execution-policy/6"` (per D25/D32 in [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md)).
- Updated `/architecture/training_matrix_multiplication_allowed` commitment to `false`.
- Added explicit commitment checks for:
  - `/architecture/serving_matrix_products_allowed == false`
  - `/architecture/dense_matrix_multiplication_allowed == false`
  - `/architecture/model_floating_point_allowed == false`
  - `/architecture/floating_point_model_instruments_allowed == false`
  - `/architecture/floating_point_gradients_and_optimizer_allowed == false`
- Added unit tests `real_repo_policy_passes` and `stale_training_matmul_true_is_rejected`.

---

## 3. Baseline Audit & Active-Path Findings

### 3.1 Active-Path Findings (Highlighted for PM / Lab Disposition)

1. **FINDING 1 — `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs` (`0 -> 1` in `scripts/no-softmax-baseline.json`):**
   - **Location:** Line `1254`, inside `#[test] fn a_duplicate_cell_and_an_unknown_policy_are_refused()`.
   - **Site:** Negative-policy test fixture string literal verifying that `validate()` rejects an unknown/prohibited policy name (`0` stripped-code hits; `1` raw `D23_WORD` hit). This negative-policy fixture is a rejection test, not a model operation.
   - **Disposition:** Left untouched on that narrow basis so `#2550`'s merged instrument source SHA is preserved for the upcoming diagnostic transcript chain (`#2028` comment `6222315060`); recorded at `1` in `scripts/no-softmax-baseline.json` and flagged for follow-up string-split cleanup after the diagnostic completes.
2. **FINDING 2 — `crates/uor-r4-training/src/lib.rs` (`4 -> 10` in `scripts/no-softmax-baseline.json`, `3` in `scripts/no-matmul-baseline.json`):**
   - **Location:** D23 lines `328, 329, 346, 347, 351` (`rope_theta`, `rope_interleaved`) and `537, 608` (`apply_rope_f32`, `silu_fwd`); D25 lines `467, 581, 587` (`.matmul(`).
   - **Classification:** Inactive / quarantined `#1017` dense reference scaffolding. Every one of the 6 newly counted D23 sites, the 4 pre-existing `D23_WORD` sites, and all 3 `.matmul(` sites sit inside `#[cfg(any())] pub struct ReferenceConfig` (line `314`) and `#[cfg(any())] impl ReferenceModel` (line `361`), which are never compiled in any build profile.
3. **FINDING 3 — `crates/uor-r4-training/src/cuda_stack_kernels.rs` (`5 -> 5` in `scripts/no-softmax-baseline.json`, `0` in `scripts/no-matmul-baseline.json`):**
   - **Location:** Lines `12, 44, 45` (module/item doc-comments describing D29 exclusions), line `70` (`EXCLUDED_KERNEL_NAMES` rejection list entry), and line `362` (refusal error string).
   - **Classification:** Active kernel launcher module with `0` stripped-code hits; all 5 hits are raw `D23_WORD` occurrences in documentation and the D29 kernel rejection guard.
4. **FINDING 4 — PTX admission guard string literals in `uor-r4-discrete` and `admission-rank-fixture-runner` (`0` sites in `scripts/no-matmul-baseline.json`):**
   - **Location:** `crates/uor-r4-discrete/build.rs:48`, `crates/uor-r4-discrete/src/prime_zeta.rs:634`, and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/build.rs:45`.
   - **Classification:** Clean (`0` gated sites). Those lines contain `"wmma"` inside a plain string literal in `admit_ptx` (the PTX admission guard that rejects `wmma` instructions in compiled PTX). Because `Kind::Matmul` scans `strip_comments_and_strings(&text)`, plain string literals are stripped while raw strings (`r#"..."#` inline kernels) remain gated.

### 3.2 Complete `scripts/no-matmul-baseline.json` Census (All 26 Non-Zero Files, 93 Sites, 0 Tensor-Core Sites)

| File | Gated Sites | Site Kind | Justification |
|---|---:|---|---|
| `crates/uor-r4-model-source/src/lib.rs` | 6 | `.matmul(` (`matmul-method`) | Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-training/examples/aerm-keys.rs` | 3 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-binding.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-potential-native.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-integer-qk.rs` | 2 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/geometric-stack.rs` | 3 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/bin/d4-float-reference.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/bin/s2-behavior-codec.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/bin/s2-behavior-distortion.rs` | 2 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/cache_memory.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_context_credit.rs` | 3 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_event_credit.rs` | 2 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_no_read.rs` | 2 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_potential_q4.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_read_feedback.rs` | 2 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_stack.rs` | 22 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_value_producer.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/joint_model.rs` | 12 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/kappa_llama.rs` | 6 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/lib.rs` | 3 | `.matmul(` (`matmul-method`) | **FINDING 2:** quarantined `#1017` dense reference scaffolding inside `#[cfg(any())] ReferenceConfig` / `ReferenceModel` (never compiled) |
| `crates/uor-r4-training/src/lut_export.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_aerm.rs` | 4 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_export.rs` | 3 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_tracking.rs` | 7 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/whole_path_learning.rs` | 2 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/tests/backprop_first_contribution_parity.rs` | 1 | `.matmul(` (`matmul-method`) | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| **Total (26 files)** | **93** | `0` `tensor-core` | **Unchanged from prior baseline** |

### 3.3 `scripts/no-softmax-baseline.json` Changed-File Census (`140 -> 154` files, `1,641 -> 2,253` sites, `+612` sites across `41` files)

| File | Old | New | Delta | Newly Counted Tokens & Justification |
|---|---:|---:|---:|---|
| `crates/uor-r4-core/src/bin/addressed-attention-arith-ste.rs` | 0 | 1 | +1 | `sigmoid`:1 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/kvar-recall.rs` | 4 | 8 | +4 | `sigmoid`:4 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/ordinary-lexical.rs` | 65 | 66 | +1 | `tanh`:1 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/helm_d_r4_attention.rs` | 39 | 41 | +2 | `rope`:2 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/learned_reference/numerics.rs` | 7 | 9 | +2 | `gelu`:2 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/addressed_attention/ste.rs` | 1 | 3 | +2 | `sigmoid`:2 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/integrated_attention/training_support.rs` | 3 | 5 | +2 | `sigmoid`:2 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/tests/helm_d_learned_manifold_r4_construction_973.rs` | 14 | 188 | +174 | `rope`:174 — Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-graph-compiler/src/observation_text.rs` | 3 | 5 | +2 | `rope`:2 — Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-integer/src/config.rs` | 2 | 7 | +5 | `sigmoid`:2, `tanh`:3 — Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/model.rs` | 9 | 25 | +16 | `sigmoid`:8, `tanh`:8 — Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/stack/format.rs` | 13 | 17 | +4 | `gelu`:2, `silu`:2 — Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/stack/kernels.rs` | 0 | 1 | +1 | `sigmoid`:1 — Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/stack/session.rs` | 9 | 34 | +25 | `gelu`:11, `sigmoid`:3, `silu`:11 — Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/tables.rs` | 0 | 20 | +20 | `sigmoid`:10, `tanh`:10 — Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-lut/src/format.rs` | 2 | 21 | +19 | `gelu`:8, `rope`:1, `silu`:10 — Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-model-source/src/attention.rs` | 59 | 60 | +1 | `rope`:1 — Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-model-source/src/conformance.rs` | 2 | 32 | +30 | `rope`:30 — Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-model-source/src/gpt2.rs` | 4 | 11 | +7 | `gelu`:6, `tanh`:1 — Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-model-source/src/lib.rs` | 18 | 101 | +83 | `rope`:83 — Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-model-source/tests/adapter_conformance.rs` | 0 | 7 | +7 | `rope`:7 — Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-source-interface/src/lib.rs` | 0 | 4 | +4 | `rope`:4 — Host source-ingestion adapter boundary (`AdapterFeature::RopeMode` / `RopeTheta` rejection enum for `uor-r4-model-source`) |
| `crates/uor-r4-training/examples/geometric_frozen_map_fit/cross_state_completion.rs` | 1 | 11 | +10 | `sigmoid`:5, `softplus`:5 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/velocity-info.rs` | 0 | 3 | +3 | `gelu`:2, `tanh`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/bin/load-candidate-forward-report.rs` | 0 | 21 | +21 | `gelu`:10, `silu`:10, `tanh`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/cache_memory.rs` | 2 | 3 | +1 | `sigmoid`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_read.rs` | 1 | 3 | +2 | `tanh`:2 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_stack.rs` | 159 | 223 | +64 | `gelu`:6, `rope`:24, `rotary`:8, `sigmoid`:17, `silu`:1, `softplus`:5, `tanh`:3 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_stack/cuda_ops.rs` | 3 | 4 | +1 | `rope`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/joint_evaluation.rs` | 0 | 3 | +3 | `rope`:3 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/joint_integer_tables.rs` | 0 | 5 | +5 | `sigmoid`:2, `tanh`:3 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/joint_model.rs` | 7 | 13 | +6 | `sigmoid`:3, `tanh`:3 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/joint_rounding.rs` | 0 | 5 | +5 | `sigmoid`:5 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/kappa_llama.rs` | 11 | 25 | +14 | `rope`:13, `silu`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/lib.rs` | 4 | 10 | +6 | `rope`:5, `silu`:1 — **FINDING 2:** quarantined `#1017` dense reference scaffolding inside `#[cfg(any())] ReferenceConfig` / `ReferenceModel` (never compiled) |
| `crates/uor-r4-training/src/lut_export.rs` | 0 | 16 | +16 | `rope`:5, `silu`:11 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/metal_stack_kernels.rs` | 5 | 23 | +18 | `gelu`:9, `softplus`:3, `tanh`:6 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_aerm.rs` | 0 | 1 | +1 | `sigmoid`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_export.rs` | 15 | 37 | +22 | `gelu`:10, `rope`:1, `silu`:10, `tanh`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/tests/backprop_first_contribution_parity.rs` | 0 | 1 | +1 | `tanh`:1 — Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs` | 0 | 1 | +1 | `D23_WORD`:1 — **FINDING 1:** negative-policy fixture string literal in `a_duplicate_cell_and_an_unknown_policy_are_refused()` (line 1254; a rejection test, not a model operation); untouched to preserve `#2550` instrument source SHA |

### 3.4 `scripts/no-softmax-baseline.json` Unchanged Non-Zero Entries (`113` files, `1,179` sites)

| File | Sites | Justification |
|---|---:|---|
| `crates/uor-r4-core/src/bin/ablate-prose.rs` | 2 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/addressed-attention-arithmetic-reach.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/cold-prior-pilot.rs` | 4 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/competitive-reader.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/head-projection.rs` | 4 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/occurrence-reader.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/prefix-recover.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/r4-zoology-frame-export.rs` | 2 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/readout-diagnostic.rs` | 9 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/shortlist-recall.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/bin/support/integrated_attention_a2_data.rs` | 3 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/direct_causal_geometric_attention.rs` | 17 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/learned_reference/adapter.rs` | 2 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/lib.rs` | 2 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/cold_prior.rs` | 4 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/embedding.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/geometric_attention.rs` | 7 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/jepa_trainer.rs` | 7 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/lowbit_attention.rs` | 3 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/lowbit_core.rs` | 6 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/lut4/mod.rs` | 6 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/policy_feasibility.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/prefix_state.rs` | 7 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/prior_learning.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/query_read.rs` | 7 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/relational.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/learner/transferable_lexical.rs` | 3 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/shared_core.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/shared_core/tests.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/shared_core/tied_training.rs` | 12 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/native_geometric/vsa/attention.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/r4_softmax_trace_state_student.rs` | 338 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/r4_softmax_trace_student.rs` | 304 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/src/transformerless/compiler.rs` | 7 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/tests/connection_gauge_covariance_v4_973.rs` | 2 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/tests/direct_causal_geometric_attention_973.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/tests/helm_d_r4_softmax_decoder_973.rs` | 11 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/tests/intrinsic_lorentz_r4_attention_973.rs` | 20 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/tests/prime_route_geometric_attention_958.rs` | 1 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-core/tests/softmax_topk.rs` | 9 | Preserved pre-D23/D32 `uor-r4-core` reference, learner, and evaluation scaffolding (inactive historical evidence) |
| `crates/uor-r4-graph-certify/src/certify.rs` | 1 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-certify/src/msa_ab_harness.rs` | 1 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-certify/src/route_fit_report.rs` | 1 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-certify/src/score.rs` | 2 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-cli/tests/marginal_inconsistency_457.rs` | 1 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-cli/tests/marginal_reconstruction_457.rs` | 8 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-cli/tests/tropical_2hop_626.rs` | 4 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-compiler/src/observation.rs` | 1 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-graph-compiler/src/route_fit.rs` | 4 | Declared Rank-7 legacy graph-compiler / certification comparator scaffolding (preserved historical evidence) |
| `crates/uor-r4-integer/src/capability_api.rs` | 1 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/geometric_potential.rs` | 1 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/geometric_source_realizer.rs` | 2 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/stack/flock.rs` | 1 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/stack/mod.rs` | 1 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/src/stack/tests.rs` | 3 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/tests/adversarial_m2_softmax_matrix_fuzz.rs` | 33 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-integer/tests/challenger_m3_hardware_invariants.rs` | 1 | Preserved pre-D32 Q31 integer / LUT table format and conformance scaffolding (inactive historical evidence) |
| `crates/uor-r4-model-source/src/exact_probe.rs` | 2 | Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-model-source/src/geometric_training.rs` | 6 | Declared Rank-7 legacy teacher/comparator crate (`uor-r4-model-source`, outside serving tree; preserved evidence per PM ruling `6222315060`) |
| `crates/uor-r4-router/src/lib.rs` | 1 | Preserved historical router comment/diagnostic reference (inactive evidence) |
| `crates/uor-r4-training/examples/aerm-keys.rs` | 36 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-composition-fit.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-event-age-check.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-event-native.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-no-read-fit.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-no-read.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-value-q4-fit.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/attention-geometric-value-q4.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/geometric-source-realizer.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/geometric-stack.rs` | 23 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/geometric_frozen_map_fit/context_cue_coadapt.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/geometric_frozen_map_fit/coupled_episode_learning.rs` | 2 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/kappa-conversion.rs` | 8 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/lut-tool.rs` | 4 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/m-world.rs` | 8 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/mqar-bench.rs` | 13 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/mqar_bench_step2/fact.rs` | 3 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/examples/tracking-lanes.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/clause_probe.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/cuda_stack_kernels.rs` | 5 | **FINDING 3:** active kernel launcher module; all 5 hits are raw `D23_WORD` in doc-comments and `EXCLUDED_KERNEL_NAMES` rejection list (`0` stripped-code hits) |
| `crates/uor-r4-training/src/e8_memory.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/flock.rs` | 20 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/flock_transport.rs` | 10 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_bank_generate.rs` | 7 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_composition.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_context.rs` | 12 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_context_credit.rs` | 5 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_context_cuda.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_continuation_learning.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_event.rs` | 3 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_event_credit.rs` | 4 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_generate_learning.rs` | 10 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_no_read.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_occurrence_consumer.rs` | 2 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_potential_native.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_potential_q4.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_read_feedback.rs` | 3 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_read_state_bridge.rs` | 5 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_source_realizer.rs` | 4 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_span.rs` | 4 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_stack/cuda_ops_bf16.rs` | 2 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_transport_state_credit.rs` | 2 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_turn_compiler.rs` | 9 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/geometric_value_producer.rs` | 4 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/reference_eval.rs` | 3 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/relation_compiler.rs` | 28 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_dialogue.rs` | 1 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_memory.rs` | 3 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/stack_snap_parity.rs` | 7 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/temporal_compiler.rs` | 8 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/src/whole_path_learning.rs` | 3 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs` | 2 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |
| `crates/uor-r4-training/tests/stack_softmax_free_export.rs` | 11 | Preserved pre-D32 `uor-r4-training` module/target excluded from compilation (`#[cfg(any())]` in `src/lib.rs` or `autobins/autoexamples/autotests = false` in `Cargo.toml`) |

---

## 4. Verification Receipt

All checks executed on host in `/Users/casey.allard/uor-r4/.worktrees/antigravity-interlock-ratchet-hardening` (`model_compute_executed: false`, `apply_calls: 0`):

- `cargo fmt -p uor-r4-harness -- --check` & `cargo fmt -p uor-r4-core -- --check`: **PASS (`0` diff)**.
- `CARGO_TARGET_DIR=local/harness-target cargo clippy -p uor-r4-harness --locked --all-targets -- -D warnings`: **PASS (`0` warnings)**.
- `cargo test -p uor-r4-harness`: **543 unit tests + 13 integration tests + 1 doctest PASS (`0 failed`)**.
- `cargo test -p uor-r4-core --bin r4-policy-check`: **2 unit tests PASS (`0 failed`)**.
- `cargo run -p uor-r4-core --bin r4-policy-check`: `Native geometric architecture policy: PASS (no model behavior evaluated)`.
- `python3 scripts/harness/parity.py --json`: **`"ok": true` (`0` findings)**.
- `uor-harness ratchet d23`: `OK: no softmax added. 2253 remaining across 154 files (baseline 2253).`
- `uor-harness ratchet d25`: `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93). gated: 0 tensor-core intrinsic sites (fragment and sync intrinsics) across 0 files.`
- `uor-harness check`: **PASS (`0 errors`)**.
- `python3 scripts/check_claim_wording.py`: **PASS**.
