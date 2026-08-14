# Matrix-operation census (production chain)

Issue #655 sub-A. This is the generated ownership report for every matrix-like
operation reachable in the production chain
`uor-r4-model-source → source/graph compiler → certifier → bundle → graph/TLA runtime → API adapters`.

Each site is classified as one of:

- **eliminated-at-compile** — no matrix operation runs in deployed inference;
  the table-native R4G1/TLA runtime routes over compiled tables under the P-4
  zero-multiply/divide/float contract.
- **uor-matmul-owned** — the operation is performed by the pinned
  `uor-matmul` exact/coded surface (`b13c98449948174f590e337c4dc25dfc394a07d0`).
- **conventional-to-migrate** — a project-owned conventional BLAS / hand-rolled
  matrix implementation that must be replaced by `uor-matmul` (or eliminated)
  under #655-B. No such site may remain reachable in the production chain once B
  lands.
- **dev-only-oracle** — a differential-verification oracle that must be
  structurally unreachable from the release build/serve path.

The CI guard (`crates/uor-r4-model-source/tests/matrix_operation_census.rs`)
enforces the mechanically-detectable half of this census: no library-BLAS
matrix FFI/crate (`cblas_*`, `matrixmultiply`, `openblas`, `dgemm`) may appear
in any production-chain crate `src/` outside the sanctioned teacher site below.
The crate/manifest audits (`uor-r4-graph-compiler::dependency_audit`,
`uor-r4-proof-model::inference_audit`) remain the guard for forbidden BLAS/GPU
*crate dependencies*; this census closes the FFI-symbol gap those cannot see.

## Deployed inference — eliminated-at-compile

| Component | Matrix operation | Classification |
|---|---|---|
| `uor-r4-graph-runtime` (R4G1 engine) | none — byte-oriented table lookups, typed D4 policy, bounded witness replay; P-4 forbids multiply/divide/remainder/float in the runtime kernel | eliminated-at-compile |
| `uor-r4-graph-format` | none — container parse/verify only | eliminated-at-compile |
| `uor-r4-api` (`R4Engine`) | none — adapts the table-native runtime to the request/response contract | eliminated-at-compile |

Deployed inference performs no matrix multiplication. This is the property
#655's P-4 clause protects and does not need migration.

## Compile / observe teacher

The teacher/source executors run at observe/compile time (teacher-forcing to
generate the corpus and oracle logits). They are **not** deployed inference, but
they are inside the production chain (`model-source → compiler`), so their
conventional arithmetic is a migration target for #655-B.

### uor-matmul-owned (migrated by #655-B2)

The shared Llama teacher **weight matmuls** now call the pinned `uor-matmul`
exact GEMM (`uor_matmul::slice::gemm_float`). The Accelerate `cblas_sgemv` /
`cblas_sgemm` FFI and all hand-rolled SIMD dot helpers are removed. GPT-2's
tied lm-head reuses `matmul_batched`, so it is migrated too. The result is
correctly-rounded exact and byte-identical across targets (no per-machine
Accelerate variance) — a determinism improvement for teacher-side κ.

| Site | Operation | Backend | Classification |
|---|---|---|---|
| `uor-r4-model-source/src/lib.rs` `matmul` | matrix–vector (`W·x`) | `uor_matmul::slice::gemm_float` | uor-matmul-owned |
| `uor-r4-model-source/src/lib.rs` `matmul_batched` | matrix–matrix (`X·Wᵀ`) | `uor_matmul::slice::gemm_float` | uor-matmul-owned |

No `cblas_*` symbol remains anywhere in the production chain; the CI guard now
enforces zero library-BLAS use (only the two dependency-audit files, which list
BLAS crate names as denylist data, are exempt).

### conventional-to-migrate (remaining — #655-B3)

GPT-2 keeps its own hand-rolled f32 arithmetic, a separate follow-up:

| Site | Operation | Backend today | Classification |
|---|---|---|---|
| `uor-r4-model-source/src/gpt2.rs` `conv1d` / `conv1d_batched` | Conv1D `x@W` | hand-rolled f32 | conventional-to-migrate |
| `uor-r4-model-source/src/{lib.rs,gpt2.rs}` attention scoring | per-head `Q·K` / `·V` accumulation | hand-rolled f32 | conventional-to-migrate |

These are not library-BLAS (no `cblas_*`), so the mechanical guard does not flag
them; they are tracked here for #655-B3.

## Teacher-arithmetic era (#655-B2)

Switching the teacher weight matmuls from Accelerate/hand-rolled f32 to the exact
`uor-matmul` GEMM changes teacher output bytes, so compiled artifacts produced
after B2 have different CIDs than pre-B2 ones. This is recorded by the teacher's
own κ (blake3 over teacher output), which changes accordingly, and surfaced in
the "teacher model ready" diagnostic (`matmul=uor-matmul exact GEMM`). No test
pins a pre-B2 teacher-derived κ as a constant (verified across the full suite),
so no fixture re-pin is required; post-B2 CIDs are a new teacher-arithmetic era
and are never retroactively assigned to pre-B2 artifacts.

## Out of census scope

| Site | Why |
|---|---|
| `uor-r4-core/src/transformerless/compiler.rs:60` `vvexpf` (Accelerate vForce) | vector `exp`, not a matrix operation. Teacher-side softmax backend, disabled under canonical math. If a later issue wants full Accelerate removal it is tracked there, not here. |

## uor-matmul as a production dependency

`uor-matmul` (`rev = b13c9844`) is a **production dependency** of
`uor-r4-model-source` (promoted from the dev-only parity pin in
`crates/uor-r4-graph-cli`, #622 → #655-B1). It is `no_std`, `forbid(unsafe)`,
zero-heap. It enters only the **compile-time teacher**, not the deployed R4G1
runtime kernel, so it adds nothing to the P-4 deployed-inference audit surface;
the crate-dependency audits confirm it pulls in no BLAS/GPU crate.

## Status

- #655-A: census + guard landed (#699). No behavior/CID change.
- #655-B1: `uor-matmul` promoted to production dep + exact-matmul parity harness
  (#701). No behavior/CID change.
- #655-B2 (this): Llama teacher weight matmuls (`matmul`, `matmul_batched`;
  GPT-2 lm-head reuse included) migrated to `uor_matmul::slice::gemm_float`;
  `cblas_*` FFI + SIMD dot helpers removed; census guard tightened to zero
  library-BLAS. Teacher-arithmetic era change (see above); no fixture re-pin
  required.
- #655-B3: migrate GPT-2 `conv1d` / `conv1d_batched` and the attention-scoring
  accumulations (`Q·K` / `·V`) — the remaining hand-rolled f32 matrix ops.
