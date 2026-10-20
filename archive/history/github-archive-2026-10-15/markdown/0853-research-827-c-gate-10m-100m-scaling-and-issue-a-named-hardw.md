# #853 — research/#827-C: gate 10M/100M scaling and issue a named-hardware performance certificate

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:42:14Z
- **Updated:** 2026-08-25T23:32:08Z
- **Closed:** 2026-08-25T23:32:08Z
- **Labels:** graph-compiler, status: on-hold
- **Milestone:** R4 Intelligence / S6 — R4G1 scale and local composition
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/853
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #827

## Context

The Gemini proposal set a sub-50ms/10M target without fixing workload, hardware, cache state, query length, threads, percentile, artifact shape, or whether ExactContext dominates. Large compiles cost hours and must follow the first valid fixed-partition curve.

## Problem

A headline token count and latency can be made meaningless by cache warming, different partitions, truncated contexts, saturated structures, or a path the changed data never reaches.

## Goal

Remain on hold until sibling B demonstrates a non-saturated decision-relevant trend; then run only triggered 10M and 100M points and publish reproducible named-hardware compile and serving certificates.

## Scope

- [ ] Keep `status: on-hold` until sibling B posts a trigger with projected effect, non-saturated instrument, and resource budget.
- [ ] Freeze whether scale means source tokens, observed records, unique contexts, graph nodes/edges, or artifact bytes; report all relevant quantities.
- [ ] Freeze hardware/OS/toolchain, CPU governor, cores/threads, memory, storage/filesystem, artifact placement, cache/cold-warm policy, query/context/output lengths, concurrency, decoder, and repetitions.
- [ ] Use the identical untouched partition, scorer, compiler semantics, and workload across 2M/10M/100M points.
- [ ] Measure compile wall time, CPU time, peak RSS, temporary/final disk, checkpoint overhead, artifact bytes, load/startup, tokens/s, per-token p50/p95/p99, and path-specific table reads.
- [ ] Verify zero steady-state hot-path allocations and bounded lookup independent of corpus size within declared format capacities.
- [ ] Run 10M first; run 100M only if the 10M outcome can change a predeclared decision and resource projections remain inside limits.
- [ ] Publish raw samples/scripts/config CIDs and a certificate scoped to exact hardware/workload/cache state.
- [ ] Cancel a point on saturation, reachability, disk/RSS, runtime, or decision-value STOP.

## Non-goals

- Promise sub-50ms universally.
- Change hardware/workload/partition between points without treating it as a different curve.
- Run 100M simply because 10M completed.
- Use compiler/reference latency as production latency.

## Dependencies and entry gate

On hold until a positive trigger from sibling B and blocker/limit clearance from sibling A. Local composition item D need not wait for 100M.

## Acceptance criteria

- [ ] Every launched point had a posted positive trigger and all three long-run gates.
- [ ] 2M/10M/100M comparisons use identical partitions/semantics/workload and report all confounders.
- [ ] Certificate includes distribution/percentiles, raw samples, cold/warm state, hardware, binary/artifact identities, and path mix.
- [ ] Zero-allocation/P-4/bounded-state invariants pass on the actual production hot path.
- [ ] Verdict explicitly says which next scale or format decision changed.

## Verification

- [ ] Re-run benchmark harness calibration, timer overhead, warmup/cold-cache procedure, and raw-sample analysis.
- [ ] Actual production binary/request path with allocation census and operation/witness checks.
- [ ] Artifact/load corruption, resource-limit, too-large-count, and unsupported-format negatives.
- [ ] Fixed-partition/CID and scale-unit audit.
- [ ] Independent spot reproduction on the named hardware or a second declared platform where feasible.

## Run contract

- **Metric/current value:** Fixed-partition quality/path mix plus compile resources and production per-token latency percentiles; no current 10M/100M result. Baseline is sibling B's 2M point.
- **Reachability ceiling:** Recompute structure reachability and saturation before each point; projected quality movement and capacity headroom must exceed the frozen decision threshold.
- **Pinned identities:** All source/partition/scale units, compiler/scorer/artifacts, production binaries/configs, hardware/OS/toolchain/filesystem/cache/thread/workload, samples and reports.
- **Nulls/falsifier:** 2M baseline, cold/warm cache, fixed query lengths/concurrency, EXCT-on/miss/disabled, one-thread/frozen-thread settings, repeated runs.
- **Binding cheap instrument:** Capacity-scaling verdict NON-SATURATED, 2M trend supports the next point, format capacities fit, resource projection fits, and harness noise is below the minimum latency effect.
- **Exit rule:** Run a point only if it can alter freeze/R4G2/performance decisions; certify only exact workloads meeting posted quality/resource/latency bounds with confidence.
- **If positive:** Feed the measured headroom/blocker evidence into sibling D and optionally schedule the next triggered point.
- **If negative:** Stop at the last supported scale, publish the limiting resource/structure, and do not generalize the performance target.
- **Cost estimate:** Post expected/actual wall time, peak RSS/temp/final disk, CPU hours, hardware occupancy, checkpoints, and opportunity cost.

No expensive run starts until exact values are posted and the cheap structural instrument passes. Missing data is `UNAVAILABLE`/`null`; controls and workload must be non-degenerate.

## Compatibility and migration

Research/performance only unless a measured blocker triggers a separately specified format change. Certificates bind existing bytes/binaries; they do not redefine old artifacts.

## Repository conformance

- **Execution scope:** Offline large-scale compiler plus actual optimized normative R4Engine serving binary on named hardware; compiler/reference scaling is not a deployed latency or capacity result.
- Preserve deterministic artifacts/ordered reductions, bounded work, focused typed errors, format validation, backward compatibility, no_std where normative, P-4, and zero hot-path allocation.
- Empirical claims name fixed partitions/workloads, corpus sizes, hardware/cache/thread state, n/percentiles, uncertainty, identities, and status.
- **Conformance mapping:** RF-03, RF-04, RF-05, RF-14, RF-15, RF-17, RF-21.; do not create a new format capability merely to rename existing R4G1 multiresolution behavior.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Trigger-gated formats/scale runs stay open/on-hold with binding activation gates; generated conformance is never hand-edited.
- Append measurement/resource records and reconcile live scale/performance claims.
- A release latency claim requires the reachable production binary, not a certifier/reference microbenchmark.

## Claim status and next action

Current status is ON HOLD. A certificate is valid only for its exact artifact, hardware, workload, and cache/thread scope.

## Comments (1)

### comment 5418397524 — 2026-08-25T23:32:08Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Scaling or composing the current R4G1 representation cannot repair the missing causal language mechanism. The existing implementation and measurements remain preserved as comparators. Future scale work must target the decoder that survives #953 and the measured bottleneck in #954. This is not a completed capability claim.


---
_Comment/review ids in this file: 5418397524_

