# #946 — observation/compiler/S3 re-entry: compile and compose full-prefix-signature routing regions

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T21:41:04Z
- **Updated:** 2026-08-25T22:56:27Z
- **Closed:** 2026-08-25T22:56:27Z
- **Labels:** enhancement, graph-compiler, conformance
- **Milestone:** R4 Intelligence / S3 — Free-running coherence
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/946
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #824
Predecessors: #942 and #944
Canonical admitted artifact/input provenance: #933

## Context

#942 carried a deterministic fixed-size signature of the complete admitted prompt plus committed generated prefix into every normative R4G1 production decision. #944 then proved that path is behaviorally inert on the exact admitted #933 artifact: across 512 deterministic held-out cases, 492 resolved through explicit context rows and all remaining 20 context-signature probes admitted; the secondary session probe was attempted 0 times, with 0 candidate-list changes and 0 served-token changes.

The representation mismatch is exact: graph-compiler observations and ROUT prototypes use bounded context-bundle signatures, while production supplies a rolling TokenHistorySignature over the complete trajectory. The runtime also gives context routing strict precedence and currently applies session_signature.or(context_signature) as the affinity input without identifying which signature lane produced a node.

## Goal

Compile full-prefix trajectory signatures into explicit normative R4G1 routing regions and deterministically compose those regions with the existing context lane under a fixed shared budget, so earlier-prefix memory can change at least one admitted production candidate set or served token.

## Scope

- [ ] Extend compiler observations with a 288-bit trajectory signature produced incrementally once per story using the exact TokenHistorySignature algorithm used by production.
- [ ] Induce trajectory-signature prototypes, masks, calibrated radii, and emission shortlists from recorded corpus positions.
- [ ] Define and validate explicit context-route and trajectory-route node flags. Legacy flag-zero nodes remain context nodes.
- [ ] Pack both lanes under one fixed node/ROUT/EMIT budget equal to the context-only control; do not obtain an effect by unboundedly doubling the artifact.
- [ ] Build lane-specific route indexes during graph admission, outside steady-state prediction.
- [ ] During graph fallback, probe both lanes and compose a deterministic fixed-capacity active set. First implementation point: at most four context nodes plus four trajectory nodes, stable distance/node-id ordering, deduplicated.
- [ ] Score prototype affinity using the signature corresponding to each node's declared lane, not unconditional session_signature.or(context_signature).
- [ ] Preserve exact-context and suffix-DFA precedence, old-artifact admission, allocation-free steady-state prediction, no_std runtime compatibility, deterministic bytes, and P-4 operation restrictions.
- [ ] Re-run the bounded #944 canary against the resulting production-admitted candidate artifact.

## Non-goals

- Run the frozen #841 free-running protocol in this issue.
- Load a live teacher, regenerate teacher observations, or add inference-time learning.
- Tune decoding, prompts, repetition penalties, or surface heuristics.
- Promote a coherence or prompt-relevance claim from routing non-vacuity alone.
- Add a new graph format generation unless the reserved node-flag semantics cannot safely express the lane distinction.

## Equal-budget control

The treatment and context-only control must use the same total region, ROUT, EMIT, candidate, and active-node ceilings. The initial composition is frozen at 4 context + 4 trajectory active nodes within the existing maximum of 8. Artifact bytes, per-lane nodes, candidates, and operation counts are recorded. If the treatment needs more bytes or work, report the delta explicitly and do not call it equal-budget.

## Run contract

metric to move: count of paired held-out cases whose normative candidate set or served token changes only when earlier-prefix trajectory memory changes

current value: 0/512 on #944; session probes attempted 0/512

reachability ceiling: #944 exposed 20/512 graph-fallback positions after 492 explicit context-row resolutions. The new composition can affect at most those fallback positions under the preserved precedence rules.

binding cheap instrument: before a production artifact build, run a teacher-free in-memory preflight over at most 4,096 recorded held-out positions using the fixed equal-budget lane allocation. It must show at least one context-fallback position with a trajectory-region admission and a candidate set different from the context-only control. Planted positive and inert fixtures must classify correctly.

if the cheap instrument is negative: stop without the candidate artifact build; close with REPRESENTATION-NOT-ESTABLISHED and retain the report

if the cheap instrument is positive: publish the candidate-build cost estimate from the measured compiler stages, perform one teacher-free candidate graph build from the existing #933-bound inputs, admit it through the normative schema-2 boundary, and run the 512-case #944 product canary

product exit rule: EFFECT_ESTABLISHED requires at least one session/trajectory-attributed admitted route plus a changed normative candidate set or served token. Otherwise the verdict is INERT.

if product-positive: authorize a separate later issue for only a small production-scope slice of the frozen #841 protocol

if product-negative: do not run #841 and close with the retained negative artifact

cost estimate: cheap preflight is seconds to low minutes and blocks nothing. The candidate build is not authorized until its wall-clock projection is posted from measured stages; any hours-scale projection requires a new explicit long-run contract before launch.

## Acceptance criteria

- [ ] Incremental compiler trajectory signatures are byte-identical to production TokenHistorySignature for every prefix and reset exactly at story boundaries.
- [ ] Same newest WINDOW tokens with different earlier prefixes produce distinct trajectory signatures and can address distinct trajectory regions in a planted artifact.
- [ ] Unknown node-lane flags fail admission; legacy flag-zero artifacts retain byte-identical serving behavior.
- [ ] Context queries cannot select trajectory-only nodes and trajectory queries cannot select context-only nodes.
- [ ] Lane composition is deterministic, bounded, allocation-free in steady state, and operation-census compatible.
- [ ] No-session calls and old artifacts return the same candidates, token, score, and witness as before.
- [ ] Double compilation from identical pinned inputs produces identical artifact bytes.
- [ ] Planted positive and inert controls prevent a vacuous product verdict.
- [ ] The exact production canary returns EFFECT_ESTABLISHED, INERT, or UNAVAILABLE with bound artifact/input identities; signature inequality alone is never credited as an effect.
- [ ] README/RESEARCH/roadmap claims remain limited unless a later frozen trajectory evaluation supports promotion.

## Expected implementation areas

- crates/uor-r4-graph-compiler/src/induction.rs
- crates/uor-r4-graph-compiler/Cargo.toml and Cargo.lock
- crates/uor-r4-graph-format/src/records.rs and stage2.rs
- crates/uor-r4-graph-runtime/src/engine.rs and vp_tree.rs
- crates/uor-r4-api/tests/production_prefix_memory_canary_944.rs or a focused successor
- docs/transformerless/R4G1.md
- docs/RESEARCH.md and docs/r4_intelligence_completion_plan.md
- one append-only issue result record

## Focused verification

Run only issue-relevant local gates:

- compiler observation/signature extraction tests
- graph-format node-flag validation tests
- runtime lane-isolation and deterministic-composition tests
- planted positive/inert classifier tests
- the cheap preflight and, only if authorized, the exact 512-case product canary
- cargo fmt --check
- cargo check/test for touched packages
- python3 scripts/check_claim_wording.py
- graph-runtime no-default-features check
- WASM boundary check when shared runtime/format code changes

Workspace library compilation/tests remain the protected required CI context. BDD, teacher parity, full Gate C, fuzz, audit, Kani, and other certification suites are nightly/manual and are NOT_RUN locally unless the change directly targets one of those contracts.

## Repository conformance and claim boundary

Primary existing capabilities: RF-14, RF-21, RF-22, RF-29. Extend existing IDs unless the lane semantics genuinely require a new built capability; if a new ID is required, follow ids.toml → tagged Gherkin → failing behavior test → implementation → regenerated CONFORMANCE.md.

This issue may establish only that compiled full-prefix memory changes at least one normative production decision under the exact bound artifact and control. It cannot establish coherent generation, semantic relevance, planning, or clearance of the frozen #841 bar. A negative verdict is valid completion evidence.


## Comments (2)

### comment 5417448565 — 2026-08-25T22:05:23Z — Casey-allard

Binding cheap-instrument result: PREFLIGHT_POSITIVE.

Exact #933 inputs, teacher-free, bounded to 4,096 train + 4,096 held-out positions. Treatment/control used identical ceilings: 64 regions, 65 nodes, 1,232 edges, 8,280 ROUT bytes, 30,796 EMIT bytes, 66,508 total bytes, and 4 context + 4 trajectory active nodes. Result: 1,549 graph-fallback positions, 1,549 trajectory admissions, and 1,549 candidate-list changes. Double compilation was byte-identical. Total wall time was 8.386 s (observation extraction 0.455 s, induction 4.243 s, emission 1.315 s). No teacher was opened.

Candidate-build projection before launch: the exact #933 cover has 288,794 train positions, 72,130 held-out positions, k0=8, depth cap 3, 256-region ceiling, and actually induced 34 regions. A deliberately conservative linear projection from the bounded induction is 4.243 s * (288,794 / 4,096) = 299 s; add up to 60 s for full observation extraction and 60 s for emission/validation, for a projected cover-build wall of under 7 minutes. The production k0 is 4x smaller than the preflight k0, so this projection intentionally does not claim a speedup from the lower search width. One exact teacher-free candidate build is authorized. I will abort and return to the issue if the cover build exceeds a 15-minute hard wall; no hours-scale run is authorized.

### comment 5418005104 — 2026-08-25T22:56:26Z — Casey-allard

## Terminal result — implementation complete, product verdict INERT

Merged via protected queue in PR #947 at `43edd2de1639651a66d94e75c717a0e6d827136b`.

Delivered:
- production-identical incremental full-prefix `TokenHistorySignature` compiler observations with exact story resets;
- explicit validated trajectory-route node flags and ROUT prototype/radius metadata;
- lane-specific admission indexes and deterministic allocation-free 4 context + 4 trajectory composition;
- lane-specific affinity, legacy flag-zero compatibility, no-session preservation, `no_std`, and P-4 compliance;
- focused preflight and product evidence in `docs/trajectory_routing_946.md` and `docs/trajectory_routing_946_product_result.json`.

Binding preflight: `PREFLIGHT_POSITIVE` over 4,096 train + 4,096 held-out positions, with identical treatment/control ceilings and 66,508 bytes each; 1,549 trajectory admissions and 1,549 candidate-list effects; double emission byte-identical; no teacher loaded.

Exact candidate: schema-2 production admission PASS. Full 72,130-position census PASS at 29.5203% selector top-1 versus 28.1214% same-position TLA, 64/64 witness replays, zero internal/cross-surface mismatches.

Exact 512-case product canary: 492 exact-context rows; all remaining 20 fallback cases attempted and admitted trajectory routing; 0 candidate-list changes and 0 served-token changes when earlier-prefix memory changed. Verdict: `INERT`.

Per the predeclared contract, #841, BDD, teacher parity, and broader certification were `NOT_RUN`. No coherence, semantic relevance, planning, or generative-engine claim is promoted. This negative verdict is valid completion evidence.


---
_Comment/review ids in this file: 5417448565, 5418005104_

