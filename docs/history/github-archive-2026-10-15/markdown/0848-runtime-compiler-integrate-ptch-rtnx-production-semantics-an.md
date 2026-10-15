# #848 — runtime/compiler: integrate PTCH/RTNX production semantics and A/B delta covers against full recompiles (item B of #825)

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:40:35Z
- **Updated:** 2026-08-25T23:31:58Z
- **Closed:** 2026-08-25T23:31:58Z
- **Labels:** enhancement, graph-compiler
- **Milestone:** R4 Intelligence / S5 — Instruction and delta covers
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/848
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #825

## Context

R4G1Runtime owns patch-chain functionality, but the repository documentation says this is not automatically the normative R4Engine serving path. Patch mechanics must be generic and proven reachable before instruction data can rely on them.

## Problem

Undefined precedence, base compatibility, tombstones, conflicts, rollback, witness attribution, or chain bounds can produce nondeterministic behavior. A small delta may also underperform a same-data full recompile while adding runtime complexity.

## Goal

Make PTCH/RTNX semantics normative and production-reachable, compile instruction examples into deterministic sparse deltas, and compare base+delta against equal-data full recompilation and strong controls.

## Scope

- [ ] Audit exact PTCH/RTNX bytes, parser/validator, R4G1Runtime behavior, certifier behavior, and R4Engine reachability; write an ADR for normative ownership.
- [ ] Specify base/delta identity, compatibility, ordering, precedence, conflicts, tombstones, saturation/ties, maximum chain depth/bytes, activation, rollback, and witness attribution.
- [ ] Add structural then semantic validation and fail closed on wrong base, cycles, duplicate/conflicting entries, over-limit chains, corrupt offsets, and unsupported versions.
- [ ] Compile sibling A records into content-addressed sparse delta covers with deterministic shards, ordered reductions, deduplication, and provenance.
- [ ] Compile an equal-data full-recompile arm plus base-only, delta-disabled, shuffled-label, wrong-base, reordered-chain, and equal-byte controls.
- [ ] Execute patch composition through the #831 normative R4Engine path and expose active layer/capability provenance in witnesses.
- [ ] Measure task strata, unseen templates, retention, bytes, compile cost, load/activation latency, per-token latency, table reads, and chain depth.
- [ ] Prove deterministic activation/rollback and historical behavior when no patch is active.

## Non-goals

- Implement instruction-specific special cases in the generic patch engine.
- Allow arbitrary/unbounded patch chains or silent conflict resolution.
- Treat R4G1Runtime tests alone as production reachability.

## Dependencies and entry gate

Blocked on sibling A and the promoted base capabilities. Requires #831 normative scorer and #830 corrected patch ledger/scope.

## Acceptance criteria

- [ ] One normative PTCH/RTNX semantics is documented and reached by actual R4Engine serving.
- [ ] Malformed/incompatible/over-limit chains fail closed and rollback is deterministic.
- [ ] Compiler outputs reproduce byte-for-byte and witnesses attribute every patched decision.
- [ ] Base+delta clears the frozen capability/retention gate and compares favorably with full recompile under the posted decision rule, or the negative path is recorded.
- [ ] P-4, allocation, no_std, deterministic composition, latency, and chain bounds pass.

## Verification

- [ ] Parser/two-stage validation/fuzz and wrong-base/cycle/conflict/chain-limit/tombstone negatives.
- [ ] R4G1Runtime/reference versus normative R4Engine differential with planted patch-bypass/precedence mutations.
- [ ] Deterministic compile, reordered-shard/chain, activation/rollback, restart, streaming, and witness replay tests.
- [ ] Workspace no-fail-fast, clippy, fmt, no_std ladder, wasm target, deterministic rebuild, allocation census, P-4 scan.
- [ ] Frozen capability/retention A/B report with independent resource and metric recomputation.

## Run contract

- **Metric/current value:** Per-stratum unseen-template capability with all retention floors mandatory; current base, full-recompile, and delta values are measured on sibling A before delta fitting.
- **Reachability ceiling:** Compute the fraction of target decisions reachable by patched tables/edges and the maximum possible task movement; separately bound retention exposure by the fraction of base decisions touched.
- **Pinned identities:** Base bundle, instruction records/templates/splits, patch compiler/config, delta/full artifacts, chain order, normative scorer/binaries, decoder/seeds, hardware, raw reports.
- **Nulls/falsifier:** Base only, delta disabled, same-data full recompile, shuffled labels, wrong base, reordered chain, equal-byte additional base data, template-only.
- **Binding cheap instrument:** Small deterministic patch fixture, real R4Engine reachability, non-vacuous touched-decision rate, conflict/rollback tests, and reachability ceilings must pass.
- **Exit rule:** Promote delta semantics only if every mandatory capability/retention bound passes and base+delta meets the frozen advantage/size/compile-cost rule against full recompile.
- **If positive:** Proceed to sibling C production admission and capability attestation; retain generic patch semantics.
- **If negative:** Keep generic patch mechanics only if independently useful; compile instruction data into full bundles and do not claim delta-cover alignment.
- **Cost estimate:** Post compile wall time, peak RSS/storage, delta/full bytes, load/activation latency, per-token cost, and shared resources.

Exact values are posted before execution; long runs require a passing cheap instrument and non-degenerate controls. Missing results are `UNAVAILABLE`/`null`.

## Compatibility and migration

Use versioned optional PTCH/RTNX semantics and explicit base CIDs. No-patch behavior stays byte/behavior compatible. Incompatible eras or bases fail closed; document chain and recompile migration.

## Repository conformance

- **Execution scope:** Offline graph compiler plus normative deployed R4Engine patch/composition path; patch bytes or reference behavior are not credited as production instruction capability without R4Engine reachability.
- Preserve P-4, zero-allocation, deterministic bytes/reductions, bounded work/state, typed errors/declines, no_std, and witness replay where applicable.
- Empirical claims bind tasks/templates, retention suites, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-09, RF-22, RF-23 and justified patch-lifecycle/instruction capability IDs.; introduce patch-lifecycle/instruction IDs only for built capabilities.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Dormant patch mechanisms remain open until production reachability and activation gates pass; generated conformance is never hand-edited.
- Append evidence and reconcile all current claims when instruction or patch status changes.
- Actual R4Engine CLI/server/WASM reachability plus reference/packed differential and a planted negative are mandatory.

## Claim status and next action

A positive delta experiment establishes only the measured task/retention scope. Patch availability alone is not instruction alignment.

## Comments (1)

### comment 5418395987 — 2026-08-25T23:31:57Z — Casey-allard

Closed as not planned by the geometric-decoder architecture reset #948 / #949. This scope assumes that R4G1 instruction and delta-cover machinery is the product language path, but coherent free-running generation was not established. The issue remains preserved as design history; instruction work may be re-scoped against the working geometric decoder only after #953. This is a RETIRE disposition, not completion evidence.


---
_Comment/review ids in this file: 5418395987_

