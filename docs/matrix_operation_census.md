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
- **certified-exact** — the host may use a hardware-native f64 fold only when
  a mechanical outward-error/rounding-cell witness proves that its binary32
  result is the same bit as the pinned `uor-matmul` exact owner; an ambiguous,
  exceptional, zero, or overflow-adjacent lane falls back to that exact owner.
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
separate dense sites are covered by their own certified-exact row below.
The migrated Llama result is correctly-rounded exact and byte-identical across
targets (no per-machine Accelerate variance) — a determinism improvement for
teacher outputs and the derived corpus/artifact bytes that bind those outputs.

| Site | Operation | Backend | Classification |
|---|---|---|---|
| `uor-r4-model-source/src/lib.rs` `matmul` | matrix–vector (`W·x`) | `uor_matmul::slice::gemm_float` | uor-matmul-owned |
| `uor-r4-model-source/src/lib.rs` `matmul_batched` | matrix–matrix (`X·Wᵀ`) | `uor_matmul::slice::gemm_float` | uor-matmul-owned |

No `cblas_*` symbol remains anywhere in the default production chain; the CI
guard now enforces zero library-BLAS use (only the two dependency-audit files,
which list BLAS crate names as denylist data, are exempt).

### #804 measurement-only BLAS exception (maintainer-approved 2026-08-18)

One additional, **opt-in** sanctioned site exists:
`uor-r4-model-source/src/observation_blas_exception.rs` routes the teacher
weight matmuls through Apple Accelerate (`cblas_sgemv`/`cblas_sgemm`) — but
ONLY when a build explicitly passes `--features observation-blas-exception`
on macOS, for corpus-scale teacher-forced **observation** passes whose
wall-clock is infeasible under the exact GEMM (~50–100× slower on the pinned
measurement Mac). Every default build keeps the uor-matmul owner everywhere;
the census gate (`observation_blas_exception_is_opt_in_and_never_default`)
pins the file, its cfg gating, its exact two dispatch sites, and that no
manifest default-enables the feature. Runtime provenance is loud: the
"teacher model ready" line reports
`Accelerate cblas (observation-only exception #804)` for every run built with
the feature, and any corpus produced under it must record the backend in its
run log / contract amendment (see #804/#605). Accelerate accumulation order
is machine-tuned, so logits differ from the exact GEMM in low-order bits —
corpora produced under the exception are compared against traces from the
SAME backend, never mixed silently with exact-GEMM corpora.

### certified-exact (migrated by #704)

Current-v2 source attention and GPT-2 dense execution use caller-owned
output/scratch storage. The certified-native path prepares a binary64 result,
refines ambiguous lanes, and falls back to the pinned exact owner whenever its
rounding certificate cannot establish the declared binary32 result. This is
host-side teacher arithmetic; it does not enter the deployed kernel.

| Site | Operation | Backend | Classification |
|---|---|---|---|
| `uor-r4-model-source/src/attention.rs` source-attention helpers | per-head Q·K and weighted-value column folds for standard, experimental, and GPT-2 learned-absolute v2 | certified-native f64 fold + mechanical cell witness; pinned `uor-matmul` exact fallback | certified-exact |
| `uor-r4-model-source/src/gpt2.rs` `conv1d` / `conv1d_batched` | fixed-weight Conv1D `x@W` for `c_attn`, `c_proj`, and MLP projections | certified-native prepared/refined exact-real dot; pinned exact fallback | certified-exact |
| `uor-r4-model-source/src/gpt2.rs` `finish_forward` / `forward_batch` | tied `lm_head` projection | certified-native prepared/refined exact-real dot; pinned exact fallback | certified-exact |

### conventional-to-migrate

No matrix-like production-chain site remains in this class after #704. The
historical GPT-2 binary32 left folds remain reproducible under
`gpt2-source-dense/1`; they are provenance history, not a current dispatch.

## Teacher-arithmetic eras (#655-B2, #704 A2)

Changing teacher arithmetic can change observation rows and derived artifact
CIDs. It does **not** change the source-file/snapshot κ: those addresses cover
source bytes, not teacher output, and there is no separate "teacher output κ"
with that meaning. Arithmetic eras are recorded by the typed attention/dense
records; downstream corpus, report, and artifact identities move when their
actual bytes move. No source κ is re-pinned for an executor-only change.

#704 appends second explicit eras for source attention and GPT-2 dense
execution. The current
`standard-source-attention/2`, `experimental-r4-source-attention/2`, and
`learned-absolute-source-attention/2` records use the certified-exact Q·K and
weighted-value folds described above. `gpt2-source-dense/2` names current
Conv1D, MLP, and tied-lm-head semantics. The immutable attention/1 and dense/1
records continue to identify prior bytes; existing bundles are never relabelled
or resumed across eras.

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
- #655-B2: Llama teacher weight matmuls (`matmul`, `matmul_batched`) migrated
  to `uor_matmul::slice::gemm_float`; `cblas_*` FFI + SIMD dot helpers removed;
  census guard tightened to zero library-BLAS. Teacher-arithmetic era change
  (see above); no source-κ re-pin required.
- #704 A2 (2026-08-15): source-attention Q·K and weighted-value folds migrated to the
  certified-exact path for all current `/2` families; `/1` provenance remains
  immutable.
- #704 dense (2026-08-15): GPT-2 fixed-weight `conv1d` / `conv1d_batched`,
  MLP, and tied `lm_head` folds migrated under `gpt2-source-dense/2`; dense/1
  remains immutable history. #655-B3 is closed by this row.
