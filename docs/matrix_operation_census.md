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

## Compile / observe teacher — conventional-to-migrate (targets of #655-B)

The teacher/source executors run at observe/compile time (teacher-forcing to
generate the corpus and oracle logits). They are **not** deployed inference, but
they are inside the production chain (`model-source → compiler`), so their
conventional arithmetic is a migration target for #655-B.

| Site | Operation | Backend today | Classification |
|---|---|---|---|
| `uor-r4-model-source/src/lib.rs` `matmul` | matrix–vector (`W·x`) | hand-rolled f32 loop | conventional-to-migrate |
| `uor-r4-model-source/src/lib.rs` `matmul_fast` (macOS) | matrix–vector | Accelerate `cblas_sgemv` FFI | conventional-to-migrate |
| `uor-r4-model-source/src/lib.rs` `matmul_fast` (non-macOS) | matrix–vector | hand-rolled (`dot_fast`) | conventional-to-migrate |
| `uor-r4-model-source/src/lib.rs` `matmul_batched` (macOS) | matrix–matrix (`X·Wᵀ`) | Accelerate `cblas_sgemm` FFI | conventional-to-migrate |
| `uor-r4-model-source/src/lib.rs` `matmul_batched` (non-macOS) | matrix–matrix | hand-rolled (`dot_fast` reuse) | conventional-to-migrate |
| `uor-r4-model-source/src/lib.rs` `dot`/`dot_fast` | dot product | hand-rolled f32 | conventional-to-migrate |
| `uor-r4-model-source/src/gpt2.rs` GPT-2 executor | Conv1D `x@W`, attention accumulation (reuses `matmul_batched`) | hand-rolled + shared matmul | conventional-to-migrate |

The library-BLAS FFI declaration lives once, in
`uor-r4-model-source/src/lib.rs` (`#[link(name = "Accelerate", kind = "framework")]`
→ `cblas_sgemv`, `cblas_sgemm`). That file is the **only** sanctioned location
for a `cblas_*` symbol; the CI guard fails if one appears anywhere else in the
production chain.

## Out of census scope

| Site | Why |
|---|---|
| `uor-r4-core/src/transformerless/compiler.rs:60` `vvexpf` (Accelerate vForce) | vector `exp`, not a matrix operation. Teacher-side softmax backend, disabled under canonical math. If a later issue wants full Accelerate removal it is tracked there, not here. |

## Dev-only oracles

`uor-matmul-core` is currently pinned as a **dev-dependency** parity oracle in
`crates/uor-r4-graph-cli/Cargo.toml` (`rev = b13c9844`). #655-B promotes it to
the production arithmetic owner and makes any dev-only comparison oracle
structurally unreachable from release compilation/serving.

## Status

- #655-A (this): census + guard landed. No behavior or CID change.
- #655-B: replace every `conventional-to-migrate` site with the pinned
  `uor-matmul` exact/coded surface (or eliminate it), delete duplicate local
  arithmetic, and extend the P-4 audit to the pinned dependency source. That is
  the first CID-changing step (new artifact era + κ re-pin, pre-approved).
