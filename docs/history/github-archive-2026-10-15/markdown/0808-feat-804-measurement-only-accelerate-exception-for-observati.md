# #808 — feat(#804): measurement-only Accelerate exception for observation passes — opt-in, pinned, loud

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T23:12:44Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-18T23:33:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/808
- **Merged:** 2026-08-18T23:33:44Z
- **Merge commit:** a54e12dc70e6004ffa43568f4e3127e7462437d3
- **Base / Head:** main / UOR-Foundation:issue-804-blas-exception
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #804 — implements the maintainer's measurement-only BLAS exception decision (2026-08-18), unblocking the S1 overnight corpus pass.

**Why.** The S1 smoke run measured the current exact-GEMM teacher (#655-B2) at >18 minutes for ONE 128-position traced article on the pinned M1 — ~150× slower than needed; the full 360,924-position `smollm2-360m-broad` traced pass would be 100+ hours. Under this exception the same article observes in **7 s** (~18.3 pos/s), putting the full corpus at ≈ 5.5 h single-stream.

**What.** A feature-gated (`observation-blas-exception`, macOS-only) Accelerate `cblas_sgemv`/`cblas_sgemm` path for the teacher weight matmuls, in one new sanctioned file (`observation_blas_exception.rs`), dispatched from exactly two sites in `matmul`/`matmul_batched`. Passthrough features on `uor-r4-graph-cli` and the root crate so observation builds opt in with one flag.

**Why this is an exception, not a rollback of #655-B2:**

- **Never a default** — every default build keeps the pinned `uor-matmul` exact GEMM byte-identically. The census gate gains `observation_blas_exception_is_opt_in_and_never_default`, pinning the exact cfg-gate count, the two dispatch sites, the FFI living only in the sanctioned file, and that no workspace manifest default-enables the feature.
- **Loud provenance** — every exception build's teacher-ready line reports `matmul=Accelerate cblas (observation-only exception #804)`; the S1 contract amendments record the backend for the produced corpus.
- **Correctness witness** — a feature-gated parity test bounds Accelerate vs the owned exact GEMM on fixtures (divergence is low-order accumulation bits; the caveat is stated in the census doc: exception corpora are compared against traces from the same backend, never mixed silently with exact-GEMM corpora).
- **Serving unaffected by construction** — the deployed transformerless runtime carries no dependency on these teacher matmuls (P-4 contract).

Gates: fmt; clippy `-D warnings` default (workspace, all targets) AND under the feature; model-source default suites incl. the census (3/3) green; feature-build census green; the parity witness green; smoke-verified end to end on the real 360M teacher with the trace pipeline from #807.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

