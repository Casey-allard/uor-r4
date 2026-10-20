# #1812 — Reconcile native geometric CUDA context and frozen credit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T23:05:57Z
- **Updated:** 2026-10-09T13:17:51Z
- **Closed:** 2026-10-06T23:18:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1812
- **Merged:** 2026-10-06T23:18:09Z
- **Merge commit:** 291205ab68399ba46bdae45dfe37bdb2d75afc77
- **Base / Head:** main / UOR-Foundation:codex/reconcile-native-cuda-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Move the native geometric context's full 120-state action credit and frozen consumer gradients onto CUDA while retaining the CPU numerical references. The same authentic native hard trace binds state/action metadata; device graphs carry larger adjoints and temporal utility without reducing them to four ambient coordinates.

Replacement 2 of 3 for the KEEP work in #1781/#1792. Five training-source files (+2,233/-29), based directly on main. Current main's bf16, flash-read and chunked recurrence work is preserved. Ordered joint-cue and bank Generate learning integration follows separately. This prerequisite is not a trained attention/chat result or a fully GPU-resident preparation claim.

References #820. Exact-head package checks, focused CPU tests and five real CUDA parity tests will be recorded before queueing. Historical probe branches and artifacts remain preserved.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6027202267 — 2026-10-06T23:17:16Z — Casey-allard

Exact-head self-review: `e74f7d687c280b223ca7877cb2e14af524ef9c07`, clean committed source, directly based on main including #1810, #1757 and #1809. Five training files (+2,233/-29); existing stack bf16/flash/chunked implementation unchanged by this PR.

Actual checks:
- `cargo check --release --locked --offline -p uor-r4-training --lib`: PASS.
- Same check with `--features cuda`: PASS.
- CUDA-feature release library test build: PASS.
- CPU `context_post_state_`: 2 passed; `context_hard_state_`: 2 passed. GPUs masked for CPU checks.
- RTX5090/sm120/CUDA12.8 pod `p0zi15px4cf0u7`, under uor-pod GPU lock: `native_context_cuda_` 4 passed; `cuda_frozen_no_read_potential_and_cue_root_credit_match_cpu_reference` 1 passed. Job exit 0. The tests propagate CUDA-device failure, and the actual nonzero counts were checked; no skip or zero-match success is counted.
- All five touched files: rustfmt PASS. Workspace cargo fmt FAIL is byte-identical to separately executed main `99b1f1e4`; no new formatting failure. Not claimed as workspace PASS.

Built on Linux x86_64 canonical pod `ianty3o30vhgew` with four CPU jobs and own shared target, executed the same hashed binary on the leased GPU. SHA256 `e41a788d571a6cfd59dcf8a46b5b4380f78b3a49cbb1ff7e509eeda9025e5442`; independently verified before CUDA tests. Build/CPU receipts `/workspace/uor-r4/codex/reconciliation/cuda-e74f7d68-attempt1/`; actual GPU receipts `cuda-e74f7d68-gpu-attempt1/`; tool job `/workspace/uor-r4/jobs/codex/20261006T231632Z-reconcile-cuda-gpu0.log`. Pod bootstrap used cached a5e132b3 binaries; that bootstrap parity is NOT this PR's validation.

Source review covers authentic native trace admission, shape/device/dtype boundaries, full120 temporal/factual carry, separate ambient channels, frozen numerical anchors, alias reduction before pullback, double rounded kernel arithmetic and explicit tolerance-based adjoint parity. Another agent re-reviewed the extraction it prepared; this is not counted as a non-author independent review. Root exact-head self-review finds no blocking defect. Metal NOT_RUN on Linux. Host admission/download/packing remain explicit; this is offline learning infrastructure, not fully GPU-resident preparation or an attention/chat result.

APPROVE for the declared CUDA prerequisite integration.



---
_Comment/review ids in this file: 6027202267_

