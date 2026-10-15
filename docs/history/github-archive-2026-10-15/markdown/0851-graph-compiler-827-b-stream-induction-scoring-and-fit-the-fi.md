# #851 — graph-compiler/#827-B: stream induction/scoring and fit the first valid fixed-partition scale curve

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:42:12Z
- **Updated:** 2026-08-25T23:32:05Z
- **Closed:** 2026-08-25T23:32:05Z
- **Labels:** enhancement, graph-compiler
- **Milestone:** R4 Intelligence / S6 — R4G1 scale and local composition
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/851
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #827

## Context

#460 showed why scale experiments need reachability and saturation gates: a historical slice had about 97.9% ExactContext resolution, so cover-side headline movement was capped near 2.1pp; the cheap instrument reported about 36.02 records/full key and 0.9882 EXCT-supported fraction as saturated. Earlier scale curves also changed both training and held-out data.

## Problem

In-memory observation/induction/scoring limits corpus size, but a larger compile is scientifically useless if the measured path is saturated or the evaluation partition changes with scale.

## Goal

Implement deterministic disk-backed streaming stages and measure the first valid corpus-size curve through 2M tokens/records (exact unit frozen in the contract) against one fixed held-out partition and binding structural instruments.

## Scope

- [ ] Profile and document current peak RSS, disk, wall time, shard size, passes, hot allocations, and quality/path mix on the #833 baseline.
- [ ] Define content-addressed input shards, deterministic partitioning, ordered reductions/merges, stable dedup/ties, and canonical final packing.
- [ ] Stream observation, cover induction, edge/score aggregation, ExactContext/NGRAM construction, certification, and report generation as required.
- [ ] Add resumable checkpoints with manifest/CID validation, atomic completion, corruption detection, restart, and orphan cleanup.
- [ ] Bound file descriptors, mmap/windows, buffers, temporary disk, and peak RSS; fail early with resource estimates and typed errors.
- [ ] Use one fixed untouched held-out partition for all corpus prefixes/scales and freeze prefix-selection/dedup semantics.
- [ ] Measure at least a small/medium/2M ladder with identical compiler/scorer semantics, reporting quality by path, structural capacity, bytes, RSS/disk, wall time, and determinism.
- [ ] Run the cheap `capacity_scaling`/successor instrument before each expensive point and cancel saturated/unreachable arms.
- [ ] Double-build/checkpoint-replay representative points and compare canonical bytes with the non-streaming path where feasible.

## Non-goals

- Launch 10M/100M points in this issue.
- Change the held-out partition or metric per scale.
- Interpret more ExactContext memorization as graph generalization.
- Introduce nondeterministic parallel reductions.

## Dependencies and entry gate

Requires #833 canonical inputs, #832 fixed reports/path attribution, and sibling A limits. Exact run contract is posted before implementation-scale execution.

## Acceptance criteria

- [ ] Streaming and reference/non-streaming compiles produce identical canonical bytes on overlapping fixtures.
- [ ] Peak RSS and temporary disk are bounded/predicted and checkpoints resume deterministically.
- [ ] Every scale uses the same untouched partition and reports EXCT-on/miss/disabled plus structural saturation.
- [ ] No long point runs after a binding saturation/reachability STOP verdict.
- [ ] The through-2M curve supports a documented scale/quality/resource decision with uncertainty.

## Verification

- [ ] Deterministic shard/order/parallelism, interruption/resume, corruption, partial-write, duplicate, and resource-limit tests.
- [ ] Streaming versus in-memory byte/report differential on tractable fixtures.
- [ ] Workspace no-fail-fast, clippy, fmt, no_std/wasm checks for affected crates, deterministic rebuild.
- [ ] Capacity-scaling instrument and path-attribution replay at every planned point.
- [ ] Independent fixed-partition and resource-accounting audit.

## Run contract

- **Metric/current value:** Quality/bits and path-specific coverage versus corpus scale on one fixed held-out partition, plus peak RSS/disk/wall time/artifact bytes; current exact baseline comes from #833. Historical #460 saturation numbers are context, not reused blindly.
- **Reachability ceiling:** At each point compute EXCT/NGRAM/graph/root reachability and the maximum metric movement for the changed structure. If the ceiling is below the minimum effect, stop that arm.
- **Pinned identities:** Teacher/tokenizer/corpus ordering and fixed partition, prefix sizes, compiler/streaming config, shard/checkpoint scheme, normative scorer, hardware/filesystem, outputs, and reports.
- **Nulls/falsifier:** Current in-memory path on overlap, same fixed partition, no/new cover where relevant, reordered shards/parallelism, EXCT-disabled/miss slices.
- **Binding cheap instrument:** `cargo test -p uor-r4-graph-certify --test capacity_scaling -- --ignored` or its superseding committed instrument must report non-saturated on the structure being changed; preflight RSS/disk and small byte-equivalence must pass.
- **Exit rule:** Continue through 2M only while structural instrument is non-saturated and the next point can change the declared decision; success requires deterministic bytes and the predeclared resource/quality curve.
- **If positive:** Use the curve to trigger or reject sibling C's 10M/100M ladder and feed local composition/freeze item D.
- **If negative:** Stop scaling, publish the saturation/resource ceiling, and optimize representation/path reachability or retain the current scale rather than launching larger runs.
- **Cost estimate:** Post per-point wall time, peak RSS/temp/final disk, CPU/thread/filesystem, checkpoints, and which shared resources are blocked.

No expensive run starts until exact values are posted and the cheap structural instrument passes. Missing data is `UNAVAILABLE`/`null`; controls and workload must be non-degenerate.

## Compatibility and migration

Streaming is an offline implementation change and must preserve canonical R4G1 bytes. Checkpoint formats are explicitly versioned/non-normative; incompatible checkpoints fail and restart without altering released artifacts.

## Repository conformance

- **Execution scope:** Offline graph compiler/certifier with quality measured through the normative deployed R4Engine path; compiler/reference scaling is not a deployed latency or capacity result.
- Preserve deterministic artifacts/ordered reductions, bounded work, focused typed errors, format validation, backward compatibility, no_std where normative, P-4, and zero hot-path allocation.
- Empirical claims name fixed partitions/workloads, corpus sizes, hardware/cache/thread state, n/percentiles, uncertainty, identities, and status.
- **Conformance mapping:** RF-03, RF-04, RF-05, RF-14, RF-15, RF-17, RF-21.; do not create a new format capability merely to rename existing R4G1 multiresolution behavior.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Trigger-gated formats/scale runs stay open/on-hold with binding activation gates; generated conformance is never hand-edited.
- Append measurement/resource records and reconcile live scale/performance claims.

## Claim status and next action

A larger successful compile establishes a measured resource/quality point only. It does not imply a universal knee or a serving latency claim.

## Comments (1)

### comment 5418397076 — 2026-08-25T23:32:04Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Scaling or composing the current R4G1 representation cannot repair the missing causal language mechanism. The existing implementation and measurements remain preserved as comparators. Future scale work must target the decoder that survives #953 and the measured bottleneck in #954. This is not a completed capability claim.


---
_Comment/review ids in this file: 5418397076_

