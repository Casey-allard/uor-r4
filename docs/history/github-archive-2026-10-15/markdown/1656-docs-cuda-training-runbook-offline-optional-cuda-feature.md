# #1656 — docs: CUDA training runbook (offline, optional cuda feature)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T19:47:28Z
- **Updated:** 2026-10-09T13:17:06Z
- **Closed:** 2026-10-03T21:05:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1656
- **Merged:** 2026-10-03T21:05:31Z
- **Merge commit:** 31083b7cfb0dbe3dfd45f91c35a45fa077beb831
- **Base / Head:** main / UOR-Foundation:codex/ag-cuda-runbook
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

Adds docs/compute/cuda-training.md (docs only; no crates/ changes, no cargo run). Written against main at 63ec8388 after #1649 merged. Facts re-checked against crates/uor-r4-training/Cargo.toml (cuda feature), tests/cuda_stack_ops_parity.rs, examples/geometric-stack.rs (device=cpu|metal|cuda, no implicit fallback) and docs/evidence/cuda-delivery-2026-10-03.json (18 passed / 0 failed / 1 ignored at c17411b8, 2x RTX 4090). Throughput and parity figures are copied from that evidence, not re-measured. Offline training only; no serving or reproducibility claim. Claim-wording check passed. Not merged by me.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5973372052 — 2026-10-03T20:52:03Z — Casey-allard

Review fixes at 5a961181 (merged origin/main bc967c39). Line 3 now: merged at dc28b495; read at main 63ec8388. Section 3 and the build block: nvcc on PATH at build time (cudarc cuda-version-from-build-system), export PATH=/usr/local/cuda/bin:$PATH, CUDA_COMPUTE_CAP=89. Optional items added: read= defaults to lorentz (verified in geometric-stack.rs stack_config), ladder uses read=l2; UOR_CUDA_PROFILE=1 (per-kernel sync/timing, not cuBLAS or copies). Claim-wording check passed.

### review 5402738940 — 2026-10-03T21:04:38Z — Casey-allard — COMMENTED

**Exact-head review at `5a9611819d47bf1608ebbdd160f1cec648d47334` (Claude lab, independent of the Antigravity author).** Docs only: `docs/compute/cuda-training.md` (+72 vs main `bc967c39`).

**Checked against origin/main source**
- #1649 merged at `dc28b495`.
- The `cuda` feature and cudarc features. The build needs nvcc on PATH (`cuda-version-from-build-system`), and `CUDA_COMPUTE_CAP=89` is kept.
- The `UOR_REQUIRE_CUDA` skip/panic semantics and the test names.
- `read=` defaults to lorentz (`examples/geometric-stack.rs:1140-1141`); the ladder uses `read=l2`.
- `UOR_CUDA_PROFILE` matches `cuda_stack_kernels.rs:1088,1121-1136`. Nit: any value, including `0`, enables it.
- Evidence figures match `docs/evidence/cuda-delivery-2026-10-03.json`.

**Checks:** claim-wording passed, links are clean, and it merges cleanly. **APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5973372052, 5402738940_

