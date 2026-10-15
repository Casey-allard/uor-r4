# #857 — release/#828-D: bind and independently reproduce the complete offline serving product

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:43:56Z
- **Updated:** 2026-08-25T23:32:16Z
- **Closed:** 2026-08-25T23:32:16Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S7 — Proof, release, and distribution
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/857
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #828

## Context

Deterministic graph bytes, reports, proofs, binaries, manifests, and offline server behavior currently have separate evidence chains. A production release needs one root and a clean-room reconstruction/load test.

## Problem

A reproducible component can coexist with a non-reproducible package, missing tokenizer/adapter, mismatched report, platform-sensitive binary, network-dependent startup, or unsupported release claim.

## Goal

Create a unified content-bound release evidence root and independently rebuild/install/serve the complete product from pinned inputs in an isolated, network-disabled environment on every supported platform scope.

## Scope

- [ ] Define a release manifest/root binding source commits/dependencies/toolchains, teacher/tokenizer/corpus/config, artifact sections, reports, capability records, proof artifacts, target binaries/modules, SBOM/licenses, resource envelopes, and sibling A–C evidence.
- [ ] State precisely which bytes are cross-platform identical, platform-specific reproducible, or intentionally outside identity; include canonical math/toolchain boundaries.
- [ ] Build from pinned source inputs in a clean isolated environment with logged hardware/OS/filesystem/toolchain and no unrecorded workspace caches.
- [ ] Compare compiler outputs/checkpoints/reports/manifests/proofs and target-specific binaries against their declared identities.
- [ ] Install into an empty model root with network disabled and verify discovery, readiness, non-streaming, streaming, decline, instruction/patch/local-composition, corruption, incompatibility, and rollback canaries.
- [ ] Verify no hidden network/teacher/source checkout is required after installation.
- [ ] Generate and validate SBOM, licenses, provenance/attestation, signatures/trust roots if used, and documented resource requirements.
- [ ] Run an independent second-person/environment reproduction and publish raw logs plus PASS/FAIL/UNAVAILABLE per component/platform.
- [ ] Make release admission reject any missing/mismatched mandatory evidence node.

## Non-goals

- Assume cross-platform identical binaries.
- Reuse an unverified local `.uor-models` directory as the independent install.
- Hide unavailable hardware/fixtures as passing tests.

## Dependencies and entry gate

Requires sibling A–C and stable final artifacts/capability verdicts from prior stages. Network composition is not required for the offline product.

## Acceptance criteria

- [ ] One root traverses to every mandatory source, artifact, report, proof, binary, SBOM, and capability identity.
- [ ] Clean rebuild outputs match the exact identity contract for each platform/component.
- [ ] Empty-root network-disabled install and full serving canary pass on supported scopes.
- [ ] Independent reproduction has complete logs and no unavailable mandatory node.
- [ ] Tampered/missing/mismatched evidence prevents release admission.

## Verification

- [ ] Clean isolated rebuild twice plus independent environment/person replay.
- [ ] Hash/root traversal and planted artifact/report/proof/binary/SBOM mismatch negatives.
- [ ] Network-disabled empty-root install and production endpoint canaries.
- [ ] Cross-platform/platform-specific identity comparison according to the declared contract.
- [ ] Release admission, rollback, corruption, incompatible-era, and resource-limit tests.

## Compatibility and migration

Release manifest/root is versioned and retains artifact-era/platform identity. Older releases remain verifiable under their roots; unsupported eras are explicit. No silent re-pin or fixture substitution.

## Repository conformance

- **Execution scope:** Complete offline compile/package/install/admit/serve product on each supported platform, with exact component scopes from sibling A; each proof/check/report is credited only to the exact reachable bytes, target, build mode, and obligation it covers.
- Preserve P-4, zero-allocation, deterministic artifacts, bounded work/state, typed errors, no_std, unsafe prohibitions, witness replay, and compatibility where normative.
- Guarantee/assumption/empirical/proof-status language follows the formal vocabulary and includes identities, platform scope, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-06, RF-09, RF-10, RF-11, RF-13, RF-15, RF-17 plus all promoted capability IDs.; do not mark a capability built merely because a reference model or harness exists.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Proof and release evidence updates the source model; generated conformance is never hand-edited.
- Append reproduction/measurement records and reconcile every public claim to the final claim map.
- A fixture-dependent mandatory node must be present; absence is UNAVAILABLE and blocks the corresponding release claim.

## Claim status and next action

This establishes reproducibility/offline-product scope only for the published root and platforms. It does not broaden empirical intelligence claims.

## Comments (1)

### comment 5418398898 — 2026-08-25T23:32:16Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Proving, auditing, reproducing, or distributing the current graph product before coherent generation is exactly the sequencing error being corrected. Historical proof and release assets remain intact. The successor release decision is #955, after the geometric decoder is product-viable. This is not completion evidence.


---
_Comment/review ids in this file: 5418398898_

