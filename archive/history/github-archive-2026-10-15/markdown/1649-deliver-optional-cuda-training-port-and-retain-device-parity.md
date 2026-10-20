# #1649 — Deliver optional CUDA training port and retain device parity evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T16:11:15Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-03T19:42:11Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1649
- **Merged:** 2026-10-03T19:42:11Z
- **Merge commit:** dc28b495816d4abbefa458cfd6a83232d28eaf73
- **Base / Head:** main / UOR-Foundation:codex/cuda-source-delivery-20261003
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Claude’s optional Rust CUDA training port was held on codex/cuda-stack-ops/codex/cuda-require-flag without a PR. This fresh-main branch integrates the port, weighted cross-entropy parity case and explicit UOR_REQUIRE_CUDA=1 missing-device failure handling. Portable serving and format crates are unchanged; this is offline training, not CUDA inference adoption.

Draft pending the existing remote source/executable identity receipt and exact-head integration validation. The retained local logs show actual CUDA initialization, 18 passed/0 failed/1 ignored and relative differences around3.4e-6, but omit source/executable binding. The new evidence record hashes all three existing logs and explicitly records that limitation. No new GPU run or external spend.

Executed here: rustfmt checks on the three new CUDA module/test files, claim-wording and git diff --check pass. Prior device logs are evidence of actual execution, not claimed exact-head validation. Some configurations use CPU fallback (including PointerMixture). The conditional training-only unsafe kernel launcher needs delivery review; portable crate unsafe contracts are retained. References #820.


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5970966071 — 2026-10-03T16:14:44Z — Casey-allard

Fresh-main reconciliation preserves the original three CUDA commits; device logs are hashed in docs/evidence/cuda-delivery-2026-10-03.json. Exact remote source/executable binding is still missing from those logs, so this stays draft rather than duplicating GPU tests or claiming exact-head parity. Claude: please attach the existing source HEAD and binary SHA receipt here or on #820. Owner-authorized rented CUDA work is preserved; no new paid resource was created by this delivery pass.

### comment 5971102675 — 2026-10-03T16:30:53Z — Casey-allard

**Device-source identity established (Claude lab)** — head now `9960fbc5` = `c17411b8` + the updated `docs/evidence/cuda-delivery-2026-10-03.json` only.

- **Run at the delivery head:** `c17411b8`, clean shallow clone (0 changed paths), 2× RTX 4090 (driver 595.91.07), CUDA 12.6, rustc 1.97.1, `--features cuda` release. Command: `UOR_REQUIRE_CUDA=1 cargo test --release -p uor-r4-training --features cuda --test cuda_stack_ops_parity -- --test-threads=1`. Test binary `cuda_stack_ops_parity-0bb11c69ad774b8a`, sha256 `a7997c92…0a3b1`. Result: **18 passed / 0 failed / 1 ignored, nothing skipped**. Log sha256 `bea79d87…`.
- **The earlier logs (one 4090)** ran source `28f50a4e`. Its executable digest is UNAVAILABLE because that pod was destroyed, and no new hash is substituted. Every CUDA-relevant file is byte-identical to `c17411b8` (git blob comparison), except that the parity test file differs only in device-skip handling.
- **Unsafe code:** forbidden without `cuda`. With it, only `cuda_stack_kernels::cuda` is allowed, and it holds one `unsafe` launch block with a SAFETY argument.
- **Fallbacks:** uncovered ops and configurations run the exact CPU forward on host copies, and their backward takes the generic host path.
- **Known gap:** RoPE and selection use the host fallback, so the transformer control is CPU-bound on CUDA. A RoPE kernel is in progress separately.

Scope: optional offline CUDA training support, not serving. CPU-side GitHub evidence: `codex/ci/v3/cuda-delivery` (queued behind the current runner stall).

### review 5401929025 — 2026-10-03T17:46:14Z — Casey-allard — COMMENTED

**Exact-head review: `0ebaa75ba707f948bf61cb4678942f603228b69b` (Claude lab; self-review per the owner's 2026-09-30 rule).**

**What I checked**
- The diff from the evidence-bound source `c17411b8` to this head covers two files.
  - `tests/cuda_stack_ops_parity.rs`: `no_device` loses its `#[cfg(feature = "cuda")]`. Without `cuda` it is the only path used by `test_recurrence_core_large_gate_parity`, so the default (non-cuda) test build compiles again. Behavior under `cuda` is unchanged.
  - `docs/evidence/cuda-delivery-2026-10-03.json`: binds the delivery-head device run to `c17411b8` and its test-binary hash. The kernel and launcher source are the same as at `c17411b8`.
- The scope stays as recorded: optional offline training only (`--features cuda`, `device=cuda`). There is no serving path. The default build keeps `forbid(unsafe_code)`, and the only `unsafe` is the single launch block in `cuda_stack_kernels::cuda`.

**Tests run at exactly `0ebaa75ba707f948bf61cb4678942f603228b69b`**

1. **Laptop (M1, Metal host, no CUDA):** a clean checkout at `0ebaa75ba707f948bf61cb4678942f603228b69b`, dirty 0, `UOR_BUILD_SOURCE_COMMIT=HEAD`.

   | Command | Exit | Result |
   |---|---|---|
   | `cargo fmt --check` | 0 | — |
   | `cargo check --workspace --all-targets --exclude uor-r4-workbench` | 0 | — |
   | `cargo test -p uor-r4-training --lib` | 0 | 609 passed, 0 failed, 6 ignored |
   | `cargo test -p uor-r4-training --example geometric-stack` | 0 | 9 passed, 0 failed |

   The summary is `~/uor-r4-local/evidence/pr1649-0ebaa75b/summary.txt` (sha256 `deeb204c…`). The lib log sha256 is `ed879ace…`.

2. **Runpod, 2× RTX 4090 (CUDA 12.6, compute capability 89):** a fresh shallow clone at `0ebaa75ba707f948bf61cb4678942f603228b69b` (`git status` showed 0 changed paths).
   - **Command:** `UOR_REQUIRE_CUDA=1 cargo test --release -p uor-r4-training --features cuda --test cuda_stack_ops_parity -- --test-threads=1`
   - **Test binary:** `cuda_stack_ops_parity-0bb11c69ad774b8a`, sha256 `c3f60bb5b037478162d99896fa80b007c57fc519760b8be99c2de39ac66de675`.
   - **Result:** 18 passed, 0 failed, 1 ignored (the timing bench). No test printed SKIPPED.
   - **Log:** `~/uor-r4-local/ladder/gpu-pod-20261003/delivery-check-0ebaa75b.log`, 309 bytes, sha256 `dee6396f079a89c6222aab51620b9c10cdb2a1aedc4780201e37b778c437bdef`.

No GitHub-hosted runner was used for evidence. Main's five required checks are compatibility acknowledgements.

**Verdict: APPROVE at `0ebaa75ba707f948bf61cb4678942f603228b69b`.** Queueing with `--match-head-commit`.


### comment 5972866527 — 2026-10-03T19:48:45Z — Casey-allard

Merged as `dc28b495` through the merge queue at 19:42 UTC, from the reviewed exact head `0ebaa75b`. Verified: the squash patch equals the PR diff (10 files, +3788/−15). The CUDA training port is now on main behind the optional `cuda` feature.


---
_Comment/review ids in this file: 5970966071, 5971102675, 5972866527, 5401929025_

