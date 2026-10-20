# #180 — feat(certify): add runtime operation, allocation, and CPU portability certificates (#161)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-24T21:02:33Z
- **Updated:** 2026-10-09T14:16:53Z
- **Closed:** 2026-07-25T00:00:58Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/180
- **Merged:** 2026-07-25T00:00:58Z
- **Merge commit:** 991a0d7cdb98ce34ecd8431243111719d8033ba4
- **Base / Head:** main / UOR-Foundation:issue-161-performance-certificates
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Overview & Purpose
Resolves #161. Adds runtime operation, zero-allocation, and CPU portability certificates to `uor-r4-graph-certify`.

## Key Changes
1. **Evidentiary Certificate Extension (`crates/uor-r4-graph-certify/src/performance_certificate.rs`)**:
   - Implements 6 `EvidentiaryClass` variants (CompilerDeclaredBounds, RuntimeObservedCounters, DisassemblyVerifiedAbsence, AllocationInstrumentation, CpuFeatureRequirements, EmpiricalMetrics).
   - Implements `DeclaredZeroFields` with explicit evidence links for zero-multiply, zero-divide, zero-float, zero-allocation, etc.
   - Implements `CpuPortabilityRecord` confirming scalar fallback and cross-target byte equality.
   - Implements `RuntimePerformanceCertificate` and `verify_evidence_links()`.
2. **Proof Model Integration (`crates/uor-r4-proof-model/src/structural_guarantees.rs`)**: Added `verify_performance_certificate_compliance`.
3. **Formal Vocabulary (`docs/formal_vocabulary.md`)**: Updated to v0.1.5.
4. **Cucumber BDD Suite (`features/suites/performance_certificate.feature`)**: Added feature suite and step definitions in `tests/bdd.rs`.

## Verification
- `cargo fmt --check`: Clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: Clean
- `cargo test --workspace --offline`: All unit tests passed
- `cargo test --test bdd --offline`: All BDD scenarios passed
- `cargo check -p uor-r4-graph-format --no-default-features`: Clean
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`: Clean
- `python3 scripts/check_claim_wording.py`: Wording gate passed

## Timeline (8 entries; chronological: conversation comments, reviews, review comments)

### comment 5074378479 — 2026-07-24T21:02:37Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5074379162 — 2026-07-24T21:02:42Z — Casey-allard

@copilot review

### review-comment 3648151467 — 2026-07-24T21:06:14Z — Copilot

_on crates/uor-r4-graph-certify/src/performance_certificate.rs:229_

`verify_evidence_links()` is meant to validate that *all* declared-zero fields carry evidence links (per the BDD scenario text), but it currently checks only 4 of the 9 `DeclaredZeroFields` links. This can incorrectly certify certificates that are missing evidence for FMA/dot/matrix/GPU fields.

### review 4776732499 — 2026-07-24T21:06:15Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds a new “runtime performance certificate” surface area across the certifier, proof model, and BDD suite to support issue #161’s goal of certifying runtime operation/allocation properties and CPU portability with evidence links.

**Changes:**
- Extended `uor-r4-graph-certify` with `RuntimePerformanceCertificate`, declared-zero evidence links, and CPU portability record types.
- Integrated a new proof-model verifier entrypoint for performance-certificate compliance.
- Added a new Cucumber feature + Rust step implementations to exercise performance certificate expectations; updated formal vocabulary changelog.

### Reviewed changes

Copilot reviewed 6 out of 6 changed files in this pull request and generated 4 comments.

<details>
<summary>Show a summary per file</summary>

| File | Description |
| ---- | ----------- |
| tests/bdd.rs | Adds world state + step definitions for performance-certificate BDD scenarios. |
| features/suites/performance_certificate.feature | Introduces new enforced BDD scenarios for evidence-link integrity and CPU portability tier. |
| docs/formal_vocabulary.md | Changelog entry for formal vocabulary v0.1.5 reflecting #161 certificate terminology. |
| crates/uor-r4-proof-model/src/structural_guarantees.rs | Adds a new structural verifier function for performance-certificate compliance. |
| crates/uor-r4-proof-model/Cargo.toml | Adds `uor-r4-graph-certify` dependency to support the new verifier. |
| crates/uor-r4-graph-certify/src/performance_certificate.rs | Implements new certificate structs/enums and evidence-link verification logic. |
</details>








---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3648151494 — 2026-07-24T21:06:15Z — Copilot

_on crates/uor-r4-graph-certify/src/performance_certificate.rs:209_

`RuntimePerformanceCertificate::new()` hard-codes `certificate_cid` to a non-derived value (`kappa:blake3:perf_cert_100`). This makes the certificate non-content-addressed and inconsistent with the existing `Certificate` / `PerformanceCertificate` CID patterns in this crate.

### review-comment 3648151520 — 2026-07-24T21:06:15Z — Copilot

_on crates/uor-r4-graph-certify/src/performance_certificate.rs:154_

`CpuPortabilityRecord::default()` currently hard-codes `target_tier` and `minimum_isa_requirements` to x86-64 values. This will emit incorrect portability metadata when the certifier is built on/for AArch64 (the README explicitly calls out NEON on AArch64), and it makes the "CPU portability" record non-portable by construction.

### review-comment 3648151554 — 2026-07-24T21:06:15Z — Copilot

_on crates/uor-r4-proof-model/src/structural_guarantees.rs:525_

This report sets `status: ProofStatus::Verified` even when `valid` is false. That produces internally inconsistent reports (`status` says verified while `verified` is false) and can mislead downstream consumers that key off `status`.

### comment 5074461894 — 2026-07-24T21:14:01Z — Copilot


> @copilot review

Addressed the review findings in commit `955945e`:
- Runtime performance certificate CID is now content-addressed (derived via BLAKE3 over payload).
- `verify_evidence_links()` now checks all declared-zero evidence links.
- CPU portability defaults are architecture-aware (`x86_64`/`aarch64`/fallback) instead of hard-coded x86 values.
- Proof-model performance compliance report now keeps `status` consistent with `verified`.
- Added regression tests and updated the portability BDD expectation to be architecture-aware.


---
_Comment/review ids in this file: 5074378479, 5074379162, 5074461894, 4776732499, 3648151467, 3648151494, 3648151520, 3648151554_

