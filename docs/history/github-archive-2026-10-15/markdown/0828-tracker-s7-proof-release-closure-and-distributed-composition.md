# #828 — TRACKER/S7: proof/release closure and distributed composition

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:32:24Z
- **Updated:** 2026-08-25T23:32:20Z
- **Closed:** 2026-08-25T23:32:20Z
- **Milestone:** R4 Intelligence / S7 — Proof, release, and distribution
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/828
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

## Objective

Bind claims to the reachable production call graph, prove the stable normative model, independently reproduce the offline serving product, and add authenticated distribution only after local composition is certified.

## Current evidence

- Source scans and executable proof models do not establish the instruction set of optimized release binaries.
- The current GNAF proof has a narrower claim boundary than the full serving path.
- The early #859 Lean package is reference mathematics; S7 remains responsible for proving or excluding its relationship to the stable normative serving implementation.
- Cross-platform deterministic-byte and offline-product claims require explicit artifact and platform boundaries.

## Entry criteria

The normative scorer/format/patch semantics are stable, required prior stage gates have final verdicts, and local composition is certified.

## Native dependencies

Blocked by #827.

## Promotion gate

The release trusted-computing boundary and claim map are complete; optimized x86_64/aarch64/WASM reachable kernels have scoped operation evidence; Kani/Lean obligations cover the stable model; a clean offline installation is independently reproduced; distribution reuses the certified local composition semantics.

## Kill / redesign criterion

Unproved, unscanned, non-reproducible, or off-serving components are excluded from the release claim. Network composition remains on hold until local composition and trust policy pass.

## Native children

- [ ] #855 — Define the release TCB and claim-to-evidence map
- [ ] #854 — Audit optimized x86_64/aarch64/WASM kernels
- [ ] #856 — Prove normative invariants with Kani and Lean
- [ ] #857 — Bind and independently reproduce the offline product
- [ ] #858 — Gate authenticated distribution and publish the final record

GitHub's native parent/sub-issue relationship is the hierarchy source of truth.

## Non-goals

- Equate source wording checks with target-machine proofs.
- Claim identical bytes across platforms where the contract only establishes platform-specific reproducibility.

## Repository conformance

Primary IDs: RF-06, RF-09, RF-10, RF-11, RF-13, RF-15, RF-17. Proof status is obligation-specific; binary operation claims require disassembly of the reachable production call graph.

Every leaf declares execution scope, preserves the applicable P-4/determinism/allocation/no_std/error/witness contracts, maps to existing RF capabilities or justifies a new one, and updates generated conformance only through its TOML/Gherkin/test sources. Empirical work posts the repository run contract before expensive execution.

## Closure rule

Close only after every native child has a completed, negative, not-triggered, or explicitly descoped verdict and this tracker records `PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE` with exact evidence links.

## Comments (1)

### comment 5418399403 — 2026-08-25T23:32:19Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Proving, auditing, reproducing, or distributing the current graph product before coherent generation is exactly the sequencing error being corrected. Historical proof and release assets remain intact. The successor release decision is #955, after the geometric decoder is product-viable. This is not completion evidence.


---
_Comment/review ids in this file: 5418399403_

