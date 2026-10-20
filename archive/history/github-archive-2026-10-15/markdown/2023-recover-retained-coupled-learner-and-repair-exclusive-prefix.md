# #2023 — Recover retained coupled learner and repair exclusive Prefix export

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:50:01Z
- **Updated:** 2026-10-09T14:33:16Z
- **Closed:** 2026-10-09T14:31:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2023
- **Merged:** 2026-10-09T14:31:38Z
- **Merge commit:** 8c78a27aae05cbcef92f217e0d95bbec207fa054
- **Base / Head:** main / UOR-Foundation:codex/coupled-export-recovery-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The completed coupled Prefix/Generate constructor could not finish export: the generic writer already created the Prefix directory and Cue-binding repair attempted a second exclusive save there. This restores the retained learner from the source archive on main, saves the rebound Prefix into a fresh directory, and retains the generic intermediate before publication. The strict library save contract, learned coefficients and serving decisions are unchanged.

Validation and outcome:

- Scientific execution source `a6be8f699eef3841a1a4dd7a51faefc06d83dd7c`: 116 focused tests, named formatting/diff checks and optimized CUDA-feature build passed. The new filesystem test covers success, destination refusal and failed-save preservation.
- All391 independent native reloads completed in190.2118 seconds, peak2,509,332KiB, preserving17 reference and380 original guard winners. Actual Rust complete-file/BLAKE3 checks passed.
- The separate saved-arithmetic reader passed all1,920 coordinate decisions,14,292 recorded alternatives,62 raw arrays and391 complete factor/action-pool reconstructions (729.6314 supervised seconds, peak1,178,352KiB). It does not replicate backwards or autoregressive execution.
- Conditional score remains9/15 (original5/15), including an incorrect EOS position. The candidate is unselected; actual-nine and full512 qualification are NOT_RUN. No new gradients, ordering, proposals or constructor repetition occurred.
- All578 completed outcome/evidence files passed forced-fresh archive-set/size/SHA verification at private store revision `fb1a0509de6a574def9472c5e33d4530f569a5b9`. Runtime preservation is separately pinned.

The complete small evidence directory, current-state update, source patch and branch-archive INDEX row are included. AGENTS records the later owner clarification: Codex uses its own worktree, keeps Casey checkout on main, limits active delivery branches, and manually removes GitHub/local branches and the owned worktree after verified merge. Casey checkout is already back on main; its existing token-optimizer material is preserved.

Final delivery head is `40e4ac037585767ee96626343f729835ba5eb2a0`. Only documentation/AGENTS changed after the measured source, including concurrent archive consolidation on main; the Rust source is byte-identical. Exact final-head checks passed: 116 focused tests, named Rust formatting and the optimized CUDA-feature build in 109.1254 seconds. The source/doc diff check passed excluding literal archived patch context; the exact retained patch passed reverse-application validation. Claim-wording checks passed. The receipt and exact-head self-review are posted below. Historical compatibility statuses are acknowledgements, not tests.

References #820. No successor starts before actual protected main merge, delivered-source verification and manual cleanup.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6082973321 — 2026-10-09T14:30:38Z — Casey-allard

Final delivery validation and exact-head self-review: `40e4ac037585767ee96626343f729835ba5eb2a0`.

PASS: 116 focused example tests, named Rust formatting and optimized CUDA-feature release build on the approved pod, 109.12539090588689 seconds total. All five Rust file SHA-256 values match the measured a6be source packet; changes since that scientific source are documentation/AGENTS only. No model experiment was repeated for this final delivery check.

PASS: source/documentation range whitespace check (excluding archived .patch files); the retained exact source patch passes `git apply --reverse --check`. Literal unified-diff blank context lines are retained in the patch. Claim-wording check passed.

Exact-head self-review covers exclusive Prefix publication, unchanged saved arithmetic and serving decisions, test coverage for success/refusal/failed-save retention, all evidence links, negative qualification scope, and owner main/worktree/manual-cleanup rules. Existing independent arithmetic reviews and the completed saved-reader audit retain their declared scope; this self-review is not a new independent mathematical replication. No blocking findings. Required GitHub compatibility statuses are acknowledgements, not executed tests.

Final check receipt (also retained on the pod at `sol-coupled-episode-learning/delivery-exact-source-checks.json`):

```json
{
  "schema": "uor-r4.coupled-exact-source-checks/1",
  "status": "PASS",
  "source_commit": "40e4ac037585767ee96626343f729835ba5eb2a0",
  "lane": "CPU-affinity0,1 Cargo tests and CUDA-feature optimized build, ownGPU0 job; no learned-model run",
  "started_utc": "2026-10-09T14:27:42.928630+00:00",
  "phases": [
    {
      "name": "rustfmt",
      "argv": [
        "/root/.cargo/bin/rustfmt",
        "--edition",
        "2021",
        "--config",
        "skip_children=true",
        "--check",
        "crates/uor-r4-training/examples/geometric-frozen-map-fit.rs",
        "crates/uor-r4-training/examples/geometric_frozen_map_fit/context_cue_coadapt.rs",
        "crates/uor-r4-training/examples/geometric_frozen_map_fit/prefix_fragment_learning.rs",
        "crates/uor-r4-training/examples/geometric_frozen_map_fit/generate_episode_learning.rs",
        "crates/uor-r4-training/examples/geometric_frozen_map_fit/coupled_episode_learning.rs"
      ],
      "elapsed_seconds": 0.15837817266583443,
      "exit_code": 0,
      "log": "/workspace/uor-r4/codex/sol-coupled-episode-learning/delivery-exact-source-checks-0001/rustfmt.log",
      "log_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "log_bytes": 0
    },
    {
      "name": "diff-check",
      "argv": [
        "git",
        "diff",
        "--check"
      ],
      "elapsed_seconds": 0.014921098947525024,
      "exit_code": 0,
      "log": "/workspace/uor-r4/codex/sol-coupled-episode-learning/delivery-exact-source-checks-0001/diff-check.log",
      "log_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "log_bytes": 0
    },
    {
      "name": "example-tests",
      "argv": [
        "taskset",
        "-c",
        "0,1",
        "/root/.cargo/bin/cargo",
        "test",
        "--locked",
        "--offline",
        "--release",
        "-p",
        "uor-r4-training",
        "--features",
        "cuda",
        "--example",
        "geometric-frozen-map-fit",
        "--",
        "--nocapture"
      ],
      "elapsed_seconds": 72.10997496172786,
      "exit_code": 0,
      "log": "/workspace/uor-r4/codex/sol-coupled-episode-learning/delivery-exact-source-checks-0001/example-tests.log",
      "log_sha256": "42f3570514a6b472776d4b003f17b8030932ac71681c02d710eae418fe6783f8",
      "log_bytes": 17542
    },
    {
      "name": "optimized-build",
      "argv": [
        "taskset",
        "-c",
        "0,1",
        "/root/.cargo/bin/cargo",
        "build",
        "--locked",
        "--offline",
        "--release",
        "-p",
        "uor-r4-training",
        "--features",
        "cuda",
        "--example",
        "geometric-frozen-map-fit"
      ],
      "elapsed_seconds": 26.564045142382383,
      "exit_code": 0,
      "log": "/workspace/uor-r4/codex/sol-coupled-episode-learning/delivery-exact-source-checks-0001/optimized-build.log",
      "log_sha256": "c55995e7d7e748225f2e6b2eb8f84556b4f2ae006a1008c5f011bd8ecaebb15e",
      "log_bytes": 5424
    }
  ],
  "source_files": [
    {
      "file": "crates/uor-r4-training/examples/geometric-frozen-map-fit.rs",
      "sha256": "8a29216221842522b37d6ccd010b34363db53fa5cadb8bbff1fc3d77b96e67ad",
      "bytes": 324244
    },
    {
      "file": "crates/uor-r4-training/examples/geometric_frozen_map_fit/context_cue_coadapt.rs",
      "sha256": "2efd3f31ca6192fed70edc5056550db2656dad1c9f27e177bb5493eda16773a5",
      "bytes": 84091
    },
    {
      "file": "crates/uor-r4-training/examples/geometric_frozen_map_fit/prefix_fragment_learning.rs",
      "sha256": "3ba46ee05eaaab90f50d00ab29866246c41566f6c9d7fa16318de630e258942c",
      "bytes": 137470
    },
    {
      "file": "crates/uor-r4-training/examples/geometric_frozen_map_fit/generate_episode_learning.rs",
      "sha256": "b6dc3a5967059f22dd246867131ff41826c7d9e75b382af135e4c83b9c8aa3d7",
      "bytes": 108694
    },
    {
      "file": "crates/uor-r4-training/examples/geometric_frozen_map_fit/coupled_episode_learning.rs",
      "sha256": "f16e814ab0f34cdbfefb723fd2203749d5b02d882c8f5ae043d138a33e367671",
      "bytes": 143294
    }
  ],
  "environment": {
    "CARGO_TARGET_DIR": "/root/codex/prototype-target",
    "CARGO_BUILD_JOBS": "2",
    "UOR_REQUIRE_CUDA": "1",
    "CUDA_COMPUTE_CAP": "89",
    "UOR_BUILD_SOURCE_COMMIT": "40e4ac037585767ee96626343f729835ba5eb2a0",
    "OMP_NUM_THREADS": "2",
    "RAYON_NUM_THREADS": "2",
    "CUDA_VISIBLE_DEVICES": "0"
  },
  "binary": {
    "file": "/root/codex/prototype-target/release/examples/geometric-frozen-map-fit",
    "sha256": "5af16bd6b5ce9d56af664d17a997f0ac49bee6303254a3007b8873aebf46bb6b",
    "bytes": 36362520
  },
  "elapsed_seconds": 109.12539090588689,
  "finished_utc": "2026-10-09T14:29:32.054022+00:00"
}
```

Proceeding through protected delivery; no successor before verified main and cleanup.

### comment 6083022016 — 2026-10-09T14:33:16Z — Casey-allard

Protected delivery and manual cleanup complete.

PR #2023 merged at 2026-10-09T14:31:38Z as `8c78a27aae05cbcef92f217e0d95bbec207fa054`. Fresh `origin/main` contains that commit. All 28 changed source, policy, patch and evidence paths have identical Git blobs at the reviewed head, actual merge commit and refreshed main.

Deleted `codex/coupled-export-recovery-20261009` on GitHub, locally and in the owned pod checkout. Removed the managed local delivery worktree after verification. Casey checkout is on current main with no tracked modifications; the pre-existing `.token-optimizer/` material is preserved. No local Codex research branches remain; remote Codex has only the allowed `codex/lab-state`. Local free space is approximately 33 GiB. Foreign DeepSeek branches/worktree remain under their owner.

Released codex/sol-prototype-20261008 GPU0 lease. The pod remains running for DeepSeek's live GPU1 lease. No new model job is running for this deliverable. Final pod ledger and `delivery-main-cleanup-receipt.json` record completion and cumulative elapsed preparation/build/model/audit/preservation/delivery time. Final-head checks: https://github.com/UOR-Foundation/uor-r4/pull/2023#issuecomment-6082973321 .

The complete retained negative is in `docs/labs/coupled-export-recovery-2026-10-09/`; source patch plus INDEX row are in `docs/history/branch-archive/`. Native391 and saved-arithmetic audit passed, but conditional construction remains 9/15; candidate unselected, actual-nine/full512 NOT_RUN. The next dependency on main is saved-observation analysis of the six remaining conditional errors to distinguish erased representation from excluded useful learning changes before choosing further compute. No successor was stacked on the unmerged branch.



---
_Comment/review ids in this file: 6082973321, 6083022016_

