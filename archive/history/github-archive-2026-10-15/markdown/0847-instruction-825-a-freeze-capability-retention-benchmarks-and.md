# #847 — instruction/#825-A: freeze capability/retention benchmarks and versioned instruction records

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:40:34Z
- **Updated:** 2026-08-25T23:31:56Z
- **Closed:** 2026-08-25T23:31:56Z
- **Labels:** enhancement, conformance
- **Milestone:** R4 Intelligence / S5 — Instruction and delta covers
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/847
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #825

## Context

Instruction-conditioned delta covers are plausible because R4G1 has patch-related structures, but neither a generic instruction record contract nor a powered capability/retention benchmark exists. An aggregate “75% adherence” target lacks task strata, templates, controls, and uncertainty.

## Problem

Prompt-template memorization, base-model leakage, decoder behavior, selective decline, and catastrophic retention loss can all be hidden by a single adherence percentage. Raw chat strings also do not define deterministic compiler semantics.

## Goal

Freeze versioned instruction examples/templates, task and retention suites, split rules, objective verifiers, and admission thresholds before compiling a delta cover.

## Scope

- [ ] Define records for roles/turns, instruction, inputs/context/evidence, target/acceptable outputs, constraints, capability/task IDs, provenance/license, weights, negatives, and base compatibility.
- [ ] Define deterministic prompt/template compilation and canonicalization with versioned template identity; preserve the semantic record independently of rendering.
- [ ] Specify instruction precedence/conflict, multi-turn continuation, no-answer/decline, evidence requirements, and interaction with prompt state, selective prediction, and planning.
- [ ] Create task strata for transformations, extraction, classification, constrained formatting, evidence-grounded QA, multi-turn constraints, and any promoted typed planning behavior.
- [ ] Split by instruction template, task instance, document/domain, surface vocabulary, and composition; include unseen templates and paraphrases.
- [ ] Create retention suites for base language/continuity, prompt causality, selective risk, free-running coherence, planning, latency, bytes, and deterministic behavior.
- [ ] Include no-instruction, base-only, template-only, shuffled labels, conflicting instruction, wrong evidence, delta-disabled, and full-recompile baselines.
- [ ] Freeze per-stratum metrics, macro aggregation, confidence intervals, minimum n, capability and retention floors, and claim language.
- [ ] CID-bind records, templates, splits, verifiers/rubrics, base artifact requirements, and report schema.

## Non-goals

- Set a universal 75% target without per-task baselines and power.
- Encode inference-time hidden system prompts as the capability.
- Use training templates or instances in final evaluation.

## Dependencies and entry gate

Requires promoted S1/S2 and the exact S3/S4 scopes that instruction records may invoke. Uses #832 evaluation constitution.

## Acceptance criteria

- [ ] Semantic instruction records are independent of prompt rendering and have deterministic versioned serialization.
- [ ] Unseen-template/task splits and retention suites are leakage-tested and powered.
- [ ] Every primary task has an objective verifier or explicitly versioned rubric with judge ownership.
- [ ] Admission requires all mandatory capability strata and all retention floors, not a compensating aggregate.
- [ ] Old/base-only artifacts have a defined benchmark status and cannot inherit instruction capability.

## Verification

- [ ] Record/template round-trip and deterministic canonicalization tests.
- [ ] Template/task/document split leakage and tamper tests.
- [ ] Planted template memorizer, shuffled-label, base-only, always-decline, and retention-regression models fail intended gates.
- [ ] Power/interval and macro/per-stratum aggregation reproduce independently.
- [ ] Claim-wording/conformance checks map each benchmark statement to exact capability scope.

## Compatibility and migration

Instruction records and benchmark manifests are offline/versioned. They do not change production bytes. Any later artifact sections must declare base compatibility and preserve old-artifact semantics.

## Repository conformance

- **Execution scope:** Offline data/template/evaluation contract; no deployed instruction capability; patch bytes or reference behavior are not credited as production instruction capability without R4Engine reachability.
- Preserve P-4, zero-allocation, deterministic bytes/reductions, bounded work/state, typed errors/declines, no_std, and witness replay where applicable.
- Empirical claims bind tasks/templates, retention suites, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-09, RF-22, RF-23 and the promoted S1–S4 capabilities used in retention.; introduce patch-lifecycle/instruction IDs only for built capabilities.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Dormant patch mechanisms remain open until production reachability and activation gates pass; generated conformance is never hand-edited.
- Append evidence and reconcile all current claims when instruction or patch status changes.

## Claim status and next action

Completion freezes a falsifiable target. It does not establish alignment or instruction following.

## Comments (1)

### comment 5418395663 — 2026-08-25T23:31:55Z — Casey-allard

Closed as not planned by the geometric-decoder architecture reset #948 / #949. This scope assumes that R4G1 instruction and delta-cover machinery is the product language path, but coherent free-running generation was not established. The issue remains preserved as design history; instruction work may be re-scoped against the working geometric decoder only after #953. This is a RETIRE disposition, not completion evidence.


---
_Comment/review ids in this file: 5418395663_

