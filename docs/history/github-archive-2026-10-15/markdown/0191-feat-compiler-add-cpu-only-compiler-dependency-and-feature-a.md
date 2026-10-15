# #191 — feat(compiler): add CPU-only compiler dependency and feature audit (#174)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-25T04:35:26Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-07-25T06:23:46Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/191
- **Merged:** 2026-07-25T06:23:46Z
- **Merge commit:** cb666225dbce565efe3e3e6e4599e525581aea0e
- **Base / Head:** main / UOR-Foundation:issue-174-compiler-dependency-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Implements Issue #174: CPU-only compiler dependency and feature audit.

- **Normative Specification ([](file:///Users/casey.allard/uor-r4/docs/compiler_dependency_audit.md))**: Defines CPU-only compiler dependency denylist categories (CUDA, Metal, OpenCL, Vulkan, WebGPU, DirectML, SYCL, `tch`/`torch`, `candle-core`, `ort`, `openblas-sys`, `intel-mkl-sys`), zero-default GPU feature rules, teacher-backend isolation invariants, and CI machine runner enforcement.
- **Compiler Dependency Auditor ([)**: Implements `CompilerDependencyAuditor`, `CompilerDependencyAuditReport`, and `CompilerDependencyAuditError`.
- **CI Audit Script ([Pass: Audited 214 lockfile packages; 0 forbidden GPU/tensor/BLAS dependencies.
Compiler dependency and feature audit passed cleanly: 100% CPU-native.)**: Standalone Python script auditing workspace manifests and `Cargo.lock` against the compiler denylist.
- **Proof Model Verifier & Vocabulary**: Added `verify_compiler_dependency_audit_compliance` in `crates/uor-r4-proof-model/src/structural_guarantees.rs`; updated `docs/formal_vocabulary.md` to v0.1.12.

Closes #174.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5076956053 — 2026-07-25T04:35:30Z — Casey-allard

@copilot please review this PR

### comment 5076964261 — 2026-07-25T04:38:14Z — Copilot


> @copilot please review this PR

Reviewed the current PR changes on commit `f3f0b2e`. I verified the dependency-audit tests and proof-model structural guarantee tests locally, and the audit script passes (`Pass: Audited 214 lockfile packages; 0 forbidden GPU/tensor/BLAS dependencies.`). No additional code changes were required.


---
_Comment/review ids in this file: 5076956053, 5076964261_

