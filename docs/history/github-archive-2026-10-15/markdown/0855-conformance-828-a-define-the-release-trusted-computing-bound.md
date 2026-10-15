# #855 — conformance/#828-A: define the release trusted-computing boundary and claim-to-evidence map

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:43:54Z
- **Updated:** 2026-08-25T23:32:13Z
- **Closed:** 2026-08-25T23:32:13Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S7 — Proof, release, and distribution
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/855
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #828

## Context

UOR-R4 has source scans, executable proof obligations, conformance IDs, runtime invariants, CI gates, and empirical reports, but final release claims span different scopes. The current register does not by itself show the complete reachable production boundary.

## Problem

“Machine verified,” “zero multiply,” “deterministic,” “no allocation,” and capability claims can be overextended from a reference model, source subset, absent-fixture test, or one target to the complete product.

## Goal

Freeze the exact release trusted-computing boundary (TCB), reachable call graph, assumption set, artifact/binary/platform matrix, and claim-to-evidence/proof matrix before final proof and release execution.

## Scope

- [ ] Inventory source inputs, compiler/certifier, format/parser/validator, normative scorer/state/patch/composition, API/server/WASM adapters, decoder/tokenizer, manifests, build toolchain, OS/runtime libraries, and external assumptions.
- [ ] Draw reachable production call graphs for compile/load/admission/predict/stream/decline/patch/compose on each supported target.
- [ ] Classify each component as trusted, proved, source-checked, binary-checked, differentially tested, empirically measured, reference-only, dormant, or out of scope.
- [ ] Map RF IDs and every README/release claim to exact code/bytes, test/proof/report, target, artifact era, and evidence status.
- [ ] Map every #859 theorem ID as reused, refined, reference-only, excluded, or still open in the release TCB; a reference theorem cannot satisfy deployed-serving admission by itself.
- [ ] Separate whole-product, inference-kernel, offline-compiler, platform-specific, cross-platform, and conditional-fixture statements.
- [ ] Define threat model/assumptions for compiler correctness, Rust/LLVM, CPU/WASM semantics, storage, content hashes/signatures, supply chain, and network-disabled operation.
- [ ] List proof gaps and assign each to sibling B/C/D/E or explicitly narrow the claim.
- [ ] Define release admission so missing/unavailable evidence cannot satisfy a mandatory claim.
- [ ] Update proof-status matrix and formal vocabulary references without claiming more than the artifacts show.

## Non-goals

- Prove implementation properties inside this mapping issue.
- Describe the complete server as multiplication-free if only the inference kernel is scoped.
- Treat Rust source scans as target-machine instruction evidence.

## Dependencies and entry gate

Requires stable normative scorer/format/patch/local-composition semantics and final prior-stage capability boundaries. Draft inventory can begin earlier; freeze follows S6.

## Acceptance criteria

- [ ] Every release claim resolves to exact evidence, target, execution scope, and status.
- [ ] The TCB and assumption set cover every reachable production surface and dependency relevant to the claim.
- [ ] Reference/dormant/unavailable evidence cannot satisfy deployed release admission.
- [ ] Every proof/evidence gap has a named child issue or narrows/removes the claim.
- [ ] A reviewer can start from a release binary/bundle and traverse to all supporting identities without ambiguity.

## Verification

- [ ] Automated completeness check between claim register, proof matrix, release manifest, public claims, and supported surfaces.
- [ ] Reachability review against actual binaries/configurations and negative feature combinations.
- [ ] Planted unregistered component/claim/missing-evidence cases fail admission.
- [ ] Claim-wording, register, deferral, limits, and documentation-link checks.
- [ ] Independent maintainer review of the TCB/assumption matrix.

## Compatibility and migration

Governance/evidence only. Preserve IDs and historical records. Any unsupported era/target or removed claim is explicitly documented rather than silently reinterpreted.

## Repository conformance

- **Execution scope:** Entire release pipeline with per-component execution-scope distinctions; deployed kernel/product claims remain separate; each proof/check/report is credited only to the exact reachable bytes, target, build mode, and obligation it covers.
- Preserve P-4, zero-allocation, deterministic artifacts, bounded work/state, typed errors, no_std, unsafe prohibitions, witness replay, and compatibility where normative.
- Guarantee/assumption/empirical/proof-status language follows the formal vocabulary and includes identities, platform scope, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-06, RF-09, RF-10, RF-11, RF-13, RF-15, RF-17 plus all promoted capability IDs.; do not mark a capability built merely because a reference model or harness exists.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Proof and release evidence updates the source model; generated conformance is never hand-edited.
- Append reproduction/measurement records and reconcile every public claim to the final claim map.

## Claim status and next action

Completion freezes what must be proved/measured, not that it has passed. Mandatory gaps block release or narrow the claim.

## Comments (1)

### comment 5418398311 — 2026-08-25T23:32:12Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Proving, auditing, reproducing, or distributing the current graph product before coherent generation is exactly the sequencing error being corrected. Historical proof and release assets remain intact. The successor release decision is #955, after the geometric decoder is product-viable. This is not completion evidence.


---
_Comment/review ids in this file: 5418398311_

