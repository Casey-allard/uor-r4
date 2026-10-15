# #850 — format/#827-A: audit R4G1 v1 freeze blockers and define the only valid R4G2 triggers

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:42:11Z
- **Updated:** 2026-08-25T23:32:03Z
- **Closed:** 2026-08-25T23:32:03Z
- **Labels:** graph-compiler, conformance
- **Milestone:** R4 Intelligence / S6 — R4G1 scale and local composition
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/850
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #827

## Context

The proposed roadmap treated hierarchical multiresolution compilation as an R4G2 milestone, but R4G1 already has multiresolution cover structures, packed validation, patch sections, borrowed views, and a no_std runtime. A new magic/version is warranted only by an irreducible measured blocker.

## Problem

Renaming the format before identifying an ABI/limit/semantics blocker would fragment tools, proofs, fixtures, and compatibility without improving scale. Conversely, latent offset/capacity/composition constraints must be found before v1 freezes.

## Goal

Audit every R4G1 section, invariant, consumer, numeric limit, extension mechanism, composition requirement, and scale projection; then publish binding criteria for freezing R4G1 v1 or activating R4G2.

## Scope

- [ ] Inventory header/version, sections, offsets/lengths/count widths, alignment, optional extensions, ScoreQ, cover hierarchy, ExactContext/NGRAM, patches/RTNX, witnesses, and content identity.
- [ ] Map parser/validator/compiler/certifier/runtime/API/CLI/server/WASM/proof/fuzz/golden-fixture ownership and feature combinations.
- [ ] Project maximum corpus/record/token/node/edge/cover/patch/composition sizes and detect overflow/addressability/alignment/streaming constraints.
- [ ] Audit mmap/borrowed access, endian/cross-platform identity, no_std/alloc ladders, unknown-section policy, forward/backward compatibility, and validation complexity.
- [ ] Test whether streaming and local composition requirements fit versioned optional R4G1 sections without ambiguous semantics.
- [ ] List every candidate blocker with current evidence, cheap reproduction, workaround, compatibility cost, proof impact, and go/no-go threshold.
- [ ] Define the only valid R4G2 triggers: an observed irreducible blocker, not size alone, aspiration, or naming preference.
- [ ] Publish a decision record and migration sketch for both outcomes; defer final freeze until sibling B/D evidence arrives.

## Non-goals

- Design a new packed format spec before a trigger passes.
- Call existing tiered covers flat.
- Change bytes or magic in this audit issue.

## Dependencies and entry gate

Uses #831 normative scorer/patch semantics and input from S5 composition requirements. Can start before large scale runs.

## Acceptance criteria

- [ ] Every normative byte/consumer/limit has an owner and tested maximum.
- [ ] Candidate blockers have measurable reproductions and R4G1-extension alternatives.
- [ ] R4G2 activation requires a predeclared blocker threshold and migration/proof cost.
- [ ] No current claim equates multiresolution with a future format version.
- [ ] Decision record is usable by sibling B/D and S7 proofs.

## Verification

- [ ] Run format test matrix, no_std/alloc checks, golden round-trips, fuzz corpus, maximum/boundary fixtures, and unknown-section negatives.
- [ ] Cross-check compiler/certifier/runtime/API/CLI/server/WASM readers against the inventory.
- [ ] Static/checked arithmetic audit for offsets/counts/sizes.
- [ ] Reproduce every asserted blocker with a focused test or mark it hypothetical/unavailable.
- [ ] Claim wording and conformance tests prevent premature R4G2 claims.

## Compatibility and migration

Audit-only. Preserve R4G1 bytes and behavior. Any later extension/version must include explicit reader matrix, old-artifact support or recompile decision, and deterministic migration.

## Repository conformance

- **Execution scope:** R4G1 format and all readers/writers across compiler, certifier, portable runtime, normative R4Engine, and WASM; compiler/reference scaling is not a deployed latency or capacity result.
- Preserve deterministic artifacts/ordered reductions, bounded work, focused typed errors, format validation, backward compatibility, no_std where normative, P-4, and zero hot-path allocation.
- Empirical claims name fixed partitions/workloads, corpus sizes, hardware/cache/thread state, n/percentiles, uncertainty, identities, and status.
- **Conformance mapping:** RF-03, RF-04, RF-05, RF-14, RF-15, RF-17, RF-21.; do not create a new format capability merely to rename existing R4G1 multiresolution behavior.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Trigger-gated formats/scale runs stay open/on-hold with binding activation gates; generated conformance is never hand-edited.
- Append measurement/resource records and reconcile live scale/performance claims.

## Claim status and next action

This issue defines triggers; it does not authorize R4G2. Final freeze/version decision belongs to sibling item D.

## Comments (1)

### comment 5418396828 — 2026-08-25T23:32:02Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Scaling or composing the current R4G1 representation cannot repair the missing causal language mechanism. The existing implementation and measurements remain preserved as comparators. Future scale work must target the decoder that survives #953 and the measured bottleneck in #954. This is not a completed capability claim.


---
_Comment/review ids in this file: 5418396828_

