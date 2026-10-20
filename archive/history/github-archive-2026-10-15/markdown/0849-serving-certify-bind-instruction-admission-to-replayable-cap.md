# #849 — serving/certify: bind instruction admission to replayable capability and retention evidence (item C of #825)

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:40:36Z
- **Updated:** 2026-08-25T23:32:00Z
- **Closed:** 2026-08-25T23:31:59Z
- **Labels:** enhancement, conformance
- **Milestone:** R4 Intelligence / S5 — Instruction and delta covers
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/849
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #825

## Context

Even a positive base+delta experiment needs production admission, cross-surface semantics, capability discovery, rollback, and a certificate that binds the exact base, delta chain, benchmark, and retention result.

## Problem

A server may load a patch with the wrong base, advertise unsupported capabilities, stream before checking constraints, or continue serving after report/manifest mismatch. Aggregate certification can hide failed strata.

## Goal

Admit instruction-capable bundles only when compatible patch/full artifacts and all mandatory capability/retention evidence are present; return consistent witnessed behavior across production surfaces.

## Scope

- [ ] Record the exact positive trigger from sibling B; close `NOT TRIGGERED` if delta/full instruction behavior does not clear the frozen gate.
- [ ] Define capability manifest fields for task strata, limits, base/delta chain CIDs, template/data/report identities, operating points, retention, and expiry/version policy.
- [ ] Require admission to validate artifact compatibility, certificate/report root, mandatory strata, retention floors, and normative scorer/binary identity where applicable.
- [ ] Expose supported capabilities and limits consistently through CLI/model discovery, readiness, native API/server, OpenAI-compatible endpoints, and supported WASM.
- [ ] Ensure streaming honors instruction/selective decisions before irreversible output and witnesses bind active layers, capability, evidence, constraints, and decline.
- [ ] Test deterministic activation, restart, rollback, downgrade prevention, multiple compatible layers, and fail-closed mismatch/corruption.
- [ ] Run the untouched unseen-template/task/retention suite without retuning and publish per-stratum intervals and resource envelopes.
- [ ] Issue `PROMOTE`, `LIMITED INSTRUCTION CAPABILITY`, `FULL-RECOMPILE ONLY`, or `NOT TRIGGERED`.

## Non-goals

- Advertise “aligned” without naming tasks/limits.
- Let an unverified remote report or patch mutate local capability state.
- Hide a failed mandatory stratum inside a passing macro average.

## Dependencies and entry gate

Triggered by sibling B and requires S1/S2 plus the exact S3/S4 capability boundaries. Local composition rules must be compatible with later S6 work.

## Acceptance criteria

- [ ] An empty model root can install the exact admitted base+instruction artifacts offline and reproduce discovery/readiness/prediction.
- [ ] All surfaces expose the same capability set, decline semantics, active-layer provenance, and limits.
- [ ] Wrong/missing/corrupt/stale reports, bases, deltas, or mandatory strata fail admission.
- [ ] Untouched capability and retention gates pass per stratum with frozen thresholds or the limitation is published.
- [ ] Capability record binds exact artifact/report roots and can be independently replayed.

## Verification

- [ ] Admission/discovery/readiness/non-streaming/streaming/restart/rollback end-to-end tests.
- [ ] Wrong-base/report-root/binary-id/capability/version/corrupt/downgrade planted negatives.
- [ ] Actual production route witness replay and no-network/offline install test.
- [ ] Frozen untouched benchmark with independent verifier/statistics recomputation.
- [ ] Regression gates for P-4, allocation, determinism, prompt causality, selective risk, generation, planning, and wasm.

## Run contract

- **Metric/current value:** All mandatory per-stratum unseen-template capability lower bounds plus every retention floor; no aggregate compensation. Baselines come from sibling B.
- **Reachability ceiling:** Use patch/full path attribution to state the fraction of evaluated decisions affected; admission cannot claim capabilities absent from the bound report.
- **Pinned identities:** Base and active layers, capability manifest, benchmark/splits/verifiers, reports, normative binaries/scorer, decoder/seeds, hardware, install root.
- **Nulls/falsifier:** Base-only, delta-disabled, full-recompile, wrong base/report, stale capability manifest, offline empty-root install, second template family.
- **Binding cheap instrument:** Admission dry run, report-root replay, per-stratum completeness, production-path attribution, and offline canary all pass before untouched certification.
- **Exit rule:** Promote only if every mandatory capability/retention bound and production admission/cross-surface invariant passes without retuning.
- **If positive:** Publish the scoped instruction capability and hand stable patch/composition semantics to S6/S7.
- **If negative:** Publish LIMITED/FULL-RECOMPILE-ONLY, reject the failing layer at admission, and retain only measured capabilities.
- **Cost estimate:** Post benchmark wall time, hardware, report/fixture storage, startup/load latency, and release resources.

Exact values are posted before execution; long runs require a passing cheap instrument and non-degenerate controls. Missing results are `UNAVAILABLE`/`null`.

## Compatibility and migration

Capability fields are versioned/additive; old artifacts are explicitly unqualified. Admission rejects incompatible base/layer/report eras rather than guessing. Rollback preserves the prior admitted state.

## Repository conformance

- **Execution scope:** Normative deployed R4Engine across CLI/discovery/native/OpenAI-compatible server and supported WASM; patch bytes or reference behavior are not credited as production instruction capability without R4Engine reachability.
- Preserve P-4, zero-allocation, deterministic bytes/reductions, bounded work/state, typed errors/declines, no_std, and witness replay where applicable.
- Empirical claims bind tasks/templates, retention suites, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-09, RF-22, RF-23 plus built patch/instruction/admission capabilities.; introduce patch-lifecycle/instruction IDs only for built capabilities.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Dormant patch mechanisms remain open until production reachability and activation gates pass; generated conformance is never hand-edited.
- Append evidence and reconcile all current claims when instruction or patch status changes.
- Production reachability, offline install, and planted bypass/mismatch negatives are required.

## Claim status and next action

The certificate states exact task strata, templates, limits, retention, artifacts, and confidence. It does not claim universal alignment.

## Comments (1)

### comment 5418396298 — 2026-08-25T23:31:59Z — Casey-allard

Closed as not planned by the geometric-decoder architecture reset #948 / #949. This scope assumes that R4G1 instruction and delta-cover machinery is the product language path, but coherent free-running generation was not established. The issue remains preserved as design history; instruction work may be re-scoped against the working geometric decoder only after #953. This is a RETIRE disposition, not completion evidence.


---
_Comment/review ids in this file: 5418396298_

