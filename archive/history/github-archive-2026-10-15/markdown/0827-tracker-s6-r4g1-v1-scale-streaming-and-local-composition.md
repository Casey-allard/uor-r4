# #827 — TRACKER/S6: R4G1 v1 scale, streaming, and local composition

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:32:23Z
- **Updated:** 2026-08-25T23:32:10Z
- **Closed:** 2026-08-25T23:32:10Z
- **Milestone:** R4 Intelligence / S6 — R4G1 scale and local composition
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/827
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

## Objective

Scale the existing R4G1 multiresolution compiler/runtime with valid fixed-partition curves, bounded resources, and deterministic local multi-cover composition before deciding whether R4G2 is necessary.

## Current evidence

- #460 showed cover-side changes had little serving reach because ExactContext dominated; one subdivision arm was negative.
- The existing scale knee was confounded by changing both training and held-out partitions.
- R4G1 already supplies multiresolution structures; a new format name is not itself a scaling result.

## Entry criteria

S0 provides exact path attribution and an attested baseline; S3/S5 identify which evidence and overlay forms must survive scaling.

## Native dependencies

Blocked by #825.

## Promotion gate

Streaming compilation and inference meet predeclared memory/latency/byte envelopes on fixed held-out partitions; local composition is deterministic, bounded, conflict-defined, and certified. R4G1 v1 is frozen unless a measured blocker satisfies a written R4G2 trigger.

## Kill / redesign criterion

If reachability arithmetic or the cheap saturation instrument caps the target effect below the decision threshold, the long run does not launch. R4G2 is not created without an irreducible measured blocker.

## Native children

- [ ] #850 — Audit R4G1 v1 blockers and R4G2 triggers
- [ ] #851 — Stream compilation and fit the fixed-partition scale curve
- [ ] #853 — Gate 10M/100M named-hardware certification
- [ ] #852 — Certify local composition and issue the R4G1/R4G2 decision

GitHub's native parent/sub-issue relationship is the hierarchy source of truth.

## Non-goals

- Promise sub-50 ms at 10M tokens without a workload, hardware, cache state, and percentile definition.
- Fetch remote covers during ordinary inference.

## Repository conformance

Primary IDs: RF-03, RF-04, RF-05, RF-14, RF-15, RF-17, RF-21. Deterministic partitioning, ordered reductions, no hot-path allocation, bounded lookup, format validation, and backward compatibility remain release gates.

Every leaf declares execution scope, preserves the applicable P-4/determinism/allocation/no_std/error/witness contracts, maps to existing RF capabilities or justifies a new one, and updates generated conformance only through its TOML/Gherkin/test sources. Empirical work posts the repository run contract before expensive execution.

## Closure rule

Close only after every native child has a completed, negative, not-triggered, or explicitly descoped verdict and this tracker records `PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE` with exact evidence links.

## Comments (1)

### comment 5418397747 — 2026-08-25T23:32:09Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Scaling or composing the current R4G1 representation cannot repair the missing causal language mechanism. The existing implementation and measurements remain preserved as comparators. Future scale work must target the decoder that survives #953 and the measured bottleneck in #954. This is not a completed capability claim.


---
_Comment/review ids in this file: 5418397747_

