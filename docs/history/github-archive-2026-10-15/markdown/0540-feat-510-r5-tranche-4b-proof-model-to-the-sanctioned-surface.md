# #540 — feat(#510): R5 tranche 4b — proof-model to the sanctioned surface

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T10:32:41Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-08-09T10:53:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/540
- **Merged:** 2026-08-09T10:53:30Z
- **Merge commit:** a63c0eecf6fbb0637559bfb2400223854c32d03d
- **Base / Head:** main / UOR-Foundation:r5/proof-model
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Part of the #510 R5 rearchitecture. Converts `uor-r4-proof-model` to the sanctioned Result<> surface, clearing all 31 audit-limits violations for the crate.

A proof-validation crate reports whether an obligation holds; under R5 that report is a value, not a raised error.

## What changed
- **structural_guarantees.rs** — the 22 `StructuralGuaranteeVerifier::verify_*`/`audit_*` methods return `ProofVerificationReport` directly. A failed obligation is a report with `verified=false`, `status=Unverified`, reason in `details` (new `ProofVerificationReport::unverified` helper). `ProofValidationError` removed. Calls into still-`Result` graph-compiler/graph-certify APIs match-and-return an unverified report instead of `?`.
- **inference_audit.rs** — `audit_disassembly`/`audit_dependencies → Option<usize>`; `audit_all → InferenceAuditReport` (total; `verdict`/`is_certified` carry the finding). `AuditVerdict` kept as report data.
- **pdf_traceability.rs** — `audit_traceability_matrix → TraceabilityAuditReport` (total; no early `Err`). `TraceabilityValidationError` removed.
- **Option<String>** (None = holds): `proof_matrix::verify_all`, `range_bounds_proof::verify_range_bounds`, `deterministic_topk_proof::verify_canonical_order`, `theorem7_proof::verify_theorem_7_proof`. `allocation_proof::verify_zero_allocation → Option<R>`.
- Callers updated: crate unit tests, `tests/proof_model_tests.rs`, `tests/proof_matrix_audit.rs`, and root `tests/bdd.rs` proof/PDF steps.

## Verification
- `cargo build --workspace --all-targets` — clean
- proof-model unit + integration tests pass; **BDD 101/101 scenarios, 337/337 steps** pass
- `clippy -D warnings` clean (proof-model + root); `cargo fmt --check` clean
- `audit-deferral` (R4) passes; `audit-limits` proof-model violations: **0**

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

