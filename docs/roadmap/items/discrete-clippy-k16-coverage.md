---
id: discrete-clippy-k16-coverage
title: Track discrete Clippy drift, PR coverage and the K16 timing instrument
milestone: infra
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
scope: [crates/uor-r4-discrete/src/d11_export.rs, crates/uor-r4-discrete/src/e_means.rs, crates/uor-r4-discrete/src/kn_score.rs, crates/uor-r4-discrete/src/ngram_count.rs, crates/uor-r4-discrete/src/ngram_count_cuda.rs, crates/uor-r4-discrete/src/ngram_count_device.rs, crates/uor-r4-discrete/src/pick_read_cuda.rs, crates/uor-r4-discrete/tests/native_vector_cuda.rs, crates/uor-r4-discrete/src/training.rs, crates/uor-r4-discrete/src/kn_score_cuda.rs, docs/labs/k16-timing-2026-10-15/instrument/**, scripts/pod/uor-cuda-check-remote, .github/workflows/ci.yml, docs/roadmap/items/discrete-clippy-k16-coverage.md, docs/loop/tasks/discrete-clippy-k16-coverage/**]
---

## Why it matters

Drift on code paths outside PR checks can hide broken diagnostics.

## What is missing

PR #3424 handled earlier lint findings. PR #4059 addresses a subset of the next 23, leaving the training.rs source-pin lint and kn_score_cuda.rs outside its scope. K16 timing and a decision about PR check coverage remain distinct parts of the original report. [PR 3424](https://github.com/UOR-Foundation/uor-r4/pull/3424) · [PR 4059](https://github.com/UOR-Foundation/uor-r4/pull/4059) · [Issue 3168](https://github.com/UOR-Foundation/uor-r4/issues/3168#issuecomment-6953591450)

## Done when

Resolve the registered lint, CI-coverage and K16 timing findings with the corresponding verification for each, preserving source-pin integrity.

## How to start

Review the merged and pending partial fixes, then check the remaining lint, K16 instrument and PR coverage separately. A workflow edit requires the owner approval recorded for `.github/**`. [Owner paths](../../harness/owner-paths.json)

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/3168)

<details>
<summary>Original report from 2026-10-24</summary>

[claude] Found while finishing #3111 (tracked from #3165). Not fixed there, because both are outside its scope and older than it.

**1. Native clippy never gates a PR, and `uor-r4-discrete` has drifted.**
- **The PR job doesn't run clippy.** On PRs, CI's `fmt / clippy / tests / no_std / κ` job reports "Ruleset transport only; no formatting, Clippy, native tests, no_std, kappa or architecture-policy checks ran" (`.github/workflows/ci.yml:216`). So lint drift builds up on main.
- **The drift on main.** On main at `7923d3263`, with the pinned toolchain 1.97.1 and a stub `nvcc`, `NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo clippy -p uor-r4-discrete --all-targets --all-features --no-deps -- -D warnings` fails on:
  - `src/d11_export.rs:430`: `manual_is_multiple_of`;
  - `src/normalization_mechanism.rs:917`: `needless_range_loop`, plus a `type_complexity`;
  - `src/training.rs:1969`, in the lib-test target.
- **Without features.** Workspace clippy also fails in `crates/uor-r4-source-interface/src/lib.rs:383,397,416` (`write_literal`).
- **What #3111 adds.** Two `too_many_arguments` errors: `device.rs` `make_binding` and `Device::new_with_position`, each with 8 parameters. A config struct would fix both.

**2. `docs/labs/k16-timing-2026-10-15/instrument` no longer builds with `--features cuda`.**
- Its `include_str!` paths point at folders that have since moved to the archive: `propose-subphase-diagnostic-2026-10-16/diagnostic-pin.json` and `admission-redirect-packet-2026-10-15/configs/k16-A-seed1.json`.
- Its `Snapshot` literal is missing `normalization_mode`, and after #3111 also `position`.
- `scripts/pod/uor-cuda-check-remote` still lists this instrument as an admitted compile gate.

**Done when.**
- A real-feature clippy gate runs on PRs that touch `uor-r4-discrete`, and main passes it.
- The k16-timing instrument either builds again or is removed from the admitted compile-gate list, with a dated note.

</details>
