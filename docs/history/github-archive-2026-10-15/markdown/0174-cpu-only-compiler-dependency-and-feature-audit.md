# #174 — CPU-only compiler dependency and feature audit

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T18:44:21Z
- **Updated:** 2026-07-25T06:23:47Z
- **Closed:** 2026-07-25T06:23:47Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/174
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on: #165 (executor introduces the only new concurrency dependency).
Relates to: #160 (runtime-side dependency audit — this issue is the compiler-side twin; the two dependency graphs are distinct and separately owned), #157 §6 (CPU-only normative stance).
Source: workspace `Cargo.toml`; AGENTS.md (pinned git dependencies).

## Motivation

#160 Layer 4 audits the *normative runtime* dependency graph for GPU/tensor/BLAS contamination. The **compiler** dependency graph is a different graph — `uor-r4-graph-compiler`, `uor-r4-graph-certify`, `uor-r4-graph-cli`, `uor-r4-model-source`, and their transitive trees — and adding Rayon plus teacher-backend integrations is exactly the moment GPU or accelerator crates sneak in transitively. Today the compiler tree is clean (verified: no CUDA/ROCm/Metal/OpenCL/WebGPU/tensor/BLAS deps; rayon is declared-but-unused pending #165), but nothing keeps it clean. This issue owns the compiler-side audit; #160 keeps the runtime side.

## Scope

Audit, in CI, the compiler dependency graph:

- default Cargo features; optional features; target-specific dependencies; transitive dependencies; build scripts; CI images; benchmark tooling; documentation and examples.

The normative compiler must not require:

- CUDA; ROCm; Metal compute (Performance Shaders or compute kernels); OpenCL; Vulkan compute; WebGPU; DirectML; oneAPI/SYCL; GPU drivers; GPU runtime detection; GPU memory; GPU-specific artifact formats; accelerator-specific graph semantics; GPU shader compilation; GPU command queues; GPU synchronization; GPU fallback logic; tensor accelerators; neural engines; vendor BLAS requiring accelerator runtimes; GPU-resident intermediate artifacts.

Teacher-backend isolation rule: if any optional external teacher integration supports a GPU, it must be isolated behind a **non-default interface** and must not affect compiler correctness, artifact compatibility, or supported CPU-only operation. The compiler's CPU-native orchestration and analysis (per #170) function fully without any teacher backend present.

Mechanism options (choose in implementation): `cargo deny` bans, a `cargo tree`-based denylist script, and/or lockfile diff gates — pinned, versioned, in-repo; plus a CPU-only CI job that builds and tests the full workspace on a machine with no GPU runtime installed.

## Non-goals

- The runtime-side audit (#160 owns it).
- Removing Rayon (it is the intended CPU concurrency dependency per #165 — pinned, one version, one home).
- Prohibiting operators from using GPU teacher backends externally (allowed; just never required).
- Auditing teacher backends themselves (external to the repo).

## Affected crates/modules/docs

- `crates/uor-r4-graph-compiler`, `crates/uor-r4-graph-certify`, `crates/uor-r4-graph-cli`, `crates/uor-r4-model-source`, workspace `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, CI images (`.devcontainer/`), benchmark tooling, docs/examples.

## Dependencies

- #165 (rayon becomes real there); #168 (no GPU scheduling logic may enter the config surface — verified here).
- Feeds: #175 (CPU-only benchmark environments), #157/#160 (shared CPU-only policy wording).

## Deliverables

1. GPU/tensor/BLAS dependency denylist gate for the compiler tree (direct, optional-enabled, transitive, build scripts).
2. Feature-unification check (workspace builds must not enable GPU features).
3. CPU-only CI job: full build + test on a GPU-free runner.
4. Documentation statement: GPU hardware is not required for compilation, testing, or supported operation.

## Acceptance criteria

- CPU-only CI job passes (compile + full tests) on a machine with no GPU runtime installed.
- Dependency-tree checks fail CI on any denylisted crate entering the compiler graph by any path (including optional features and build scripts); negative fixture proves the gate can fail.
- No default GPU features anywhere in the workspace.
- Optional teacher integrations, if any, are non-default and proven removable without affecting compiler correctness or artifact compatibility.
- Docs state GPU hardware is not required.
- Four gates green; claim-wording check passes.

## Formal claim classification

- "The normative compiler builds, tests, and operates without any GPU dependency": **Guarantee**, status **Witnessed** (CI gate per commit).

## Test requirements

- **Unit:** denylist rule evaluation on fixture lockfiles.
- **Property / Reproducibility:** none new.
- **BDD/Cucumber:** none (CI gate).
- **Benchmark:** none.

## Memory considerations

None (audit tooling).

## Determinism considerations

Feature unification affects behavior, not canonical bytes; but the gate protects #167's cross-environment reproducibility by keeping the dependency surface fixed.

## CPU portability and no-GPU statement

This issue *is* the compiler-side no-GPU statement, enforced: no CUDA, ROCm, Metal, OpenCL, Vulkan compute, WebGPU, DirectML, oneAPI/SYCL, GPU drivers/runtimes/memory, GPU detection or fallback, tensor accelerators, neural engines, vendor BLAS, GPU-resident intermediates, or accelerator-specific formats — in architecture, default features, CI, tests, and production workflows.

## Traceability

- #157 §6 (policy source); plan §4.1; monograph (#133); traceability matrix (#137).

## Related issues and PRs

#58, #122, #157, #160, #165, #166, #167, #168, #169, #170, #171, #172, #173, #175.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

