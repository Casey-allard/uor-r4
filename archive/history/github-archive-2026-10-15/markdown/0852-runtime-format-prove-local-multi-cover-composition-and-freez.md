# #852 — runtime/format: prove local multi-cover composition and freeze R4G1 v1 or trigger R4G2 (item D of #827)

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:42:13Z
- **Updated:** 2026-08-25T23:32:06Z
- **Closed:** 2026-08-25T23:32:06Z
- **Labels:** enhancement, graph-compiler, conformance
- **Milestone:** R4 Intelligence / S6 — R4G1 scale and local composition
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/852
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #827

## Context

Domain covers and instruction deltas need deterministic local composition before authenticated network distribution. The final R4G1/R4G2 decision should be made with real streaming, size-limit, and composition evidence rather than aspiration.

## Problem

Multiple bundles can disagree on tokenizer/base/operator/score eras, precedence, duplicate IDs, patches, constraints, and capability reports. Unbounded or order-dependent composition would violate deterministic and allocation/latency contracts.

## Goal

Define and implement bounded, content-addressed local cover composition through the normative R4Engine path, certify conflicts/limits/witnesses, and issue the evidence-based R4G1 v1 freeze or R4G2 trigger decision.

## Scope

- [ ] Define a local composition manifest binding base/cover/delta CIDs, tokenizer/operator/format/capability compatibility, explicit order/precedence, trust source, limits, and expected composed root.
- [ ] Specify duplicate/conflict/tombstone/score/tie/constraint/decline semantics and make order dependence either impossible or explicit/content-bound.
- [ ] Set fixed maximum layers, total bytes, per-section counts, lookup work, scratch/state, and deterministic fail-closed overflow/resource behavior.
- [ ] Compose without network access during ordinary inference and without copying/unbounded allocation on the prediction hot path.
- [ ] Expose active cover/layer/provenance and contribution attribution in witnesses and capability discovery.
- [ ] Add atomic activation, restart recovery, rollback, partial/missing/corrupt layer, incompatible tokenizer/operator/base, and malicious-size tests.
- [ ] Compare physical precomposition versus bounded logical composition where relevant under equal semantics and resource budgets.
- [ ] Exercise CLI/API/server/WASM local loading through the normative path.
- [ ] Apply sibling A triggers using sibling B/C scale evidence and composition results; publish `FREEZE R4G1 v1`, `EXTEND R4G1`, or `TRIGGER R4G2` with exact blocker.
- [ ] If R4G2 triggers, open a new bounded format epic with migration/proof plan; do not design it inside this issue.

## Non-goals

- Fetch bundles over HTTP.
- Infer precedence from filesystem order or discovery time.
- Trigger R4G2 solely for a cleaner design or new name.
- Allow composition to bypass capability/retention admission.

## Dependencies and entry gate

Requires sibling A trigger criteria, sibling B fixed-partition evidence, and stable S5 patch/admission semantics. Sibling C may be NOT TRIGGERED.

## Acceptance criteria

- [ ] Same manifest/layers produce identical composed identity, decisions, and witnesses regardless of filesystem/discovery ordering.
- [ ] Incompatible/conflicting/over-limit/missing/corrupt inputs fail closed before serving.
- [ ] Actual production path meets fixed lookup/allocation/P-4/no_std/latency bounds and reports layer provenance.
- [ ] Supported local composition works offline across CLI/server/WASM or the supported-surface limitation is explicit.
- [ ] Format decision cites a passed trigger or freezes R4G1 with remaining limits documented.

## Verification

- [ ] Manifest/root determinism, order permutation, duplicate/conflict/tombstone, wrong-base/tokenizer/operator/version, partial/corrupt/oversize negatives.
- [ ] Logical versus physical/reference versus production differential with planted precedence/bypass mutation.
- [ ] Restart/atomic activation/rollback/concurrent request and witness replay tests.
- [ ] Workspace no-fail-fast, clippy, fmt, no_std, wasm, deterministic rebuild, allocation census, P-4 scan, fuzz.
- [ ] Offline empty-root multi-cover install/load/capability/prediction canary.

## Compatibility and migration

Prefer versioned R4G1 optional composition manifests and exact old-artifact behavior. A triggered R4G2 requires explicit dual-reader/recompile/migration, deterministic conversion, proof, and deprecation plan in a new epic.

## Repository conformance

- **Execution scope:** Normative deployed R4Engine local composition across supported CLI/server/WASM surfaces; compiler/reference scaling is not a deployed latency or capacity result.
- Preserve deterministic artifacts/ordered reductions, bounded work, focused typed errors, format validation, backward compatibility, no_std where normative, P-4, and zero hot-path allocation.
- Empirical claims name fixed partitions/workloads, corpus sizes, hardware/cache/thread state, n/percentiles, uncertainty, identities, and status.
- **Conformance mapping:** RF-03, RF-04, RF-05, RF-09, RF-14, RF-15, RF-17, RF-21 plus built composition capability.; do not create a new format capability merely to rename existing R4G1 multiresolution behavior.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Trigger-gated formats/scale runs stay open/on-hold with binding activation gates; generated conformance is never hand-edited.
- Append measurement/resource records and reconcile live scale/performance claims.
- Production reachability and planted semantic mutations are mandatory; directory order or network timing may never define semantics.

## Claim status and next action

This owns the R4G1 v1/R4G2 decision. Network distribution remains outside scope and on hold until S7.

## Comments (1)

### comment 5418397309 — 2026-08-25T23:32:06Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Scaling or composing the current R4G1 representation cannot repair the missing causal language mechanism. The existing implementation and measurements remain preserved as comparators. Future scale work must target the decoder that survives #953 and the measured bottleneck in #954. This is not a completed capability claim.


---
_Comment/review ids in this file: 5418397309_

