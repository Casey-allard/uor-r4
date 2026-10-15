# #916 — docs(#844): freeze compositional-planning benchmarks + typed state/action semantics

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T04:54:34Z
- **Updated:** 2026-08-22T05:17:21Z
- **Closed:** 2026-08-22T05:13:29Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/916
- **Merged:** 2026-08-22T05:13:29Z
- **Merge commit:** 3c691c337b3ca7113028b8dc98b96f53cbe44cde
- **Base / Head:** main / UOR-Foundation:issue-844-compositional-spec
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S4 (#826) needs a frozen, intervention-resistant benchmark target and a byte-level typed
state/action contract before a planner or geometry can be chosen — otherwise multi-hop
continuation, retrieval, memorized paths, or f64 routing can masquerade as reasoning (the
#845 route-attention lesson), and witnesses/capacity bounds/proofs cannot be pinned.

This PR lands **item A's design-freeze** as a reference specification:
`docs/compositional_planning_spec_844.md`. It freezes the `s4-compositional-reasoning`
benchmark constitution and the versioned typed artifact/reference model. It is the reviewed,
signed-off design; the implementation (generators, verifiers, reference model, expanded suite
JSON, conformance) follows in a build pass.

## Scope and non-goals

**In scope (frozen):** five task families (graph navigation, symbolic transformation,
constraint satisfaction, multi-hop evidence composition, counterfactual intervention) each with
a deterministic verifier + replayable gold; six split axes; four metamorphic controls; six
baselines and the promotion statistic; frozen primary/secondary metrics, horizon progression
(H∈{1,2,4,8}, H_max=16, W_max=64), effect floor (δ_min=0.05), sampling (n=512/cell) and
Holm–Bonferroni handling; the typed 10-object reference model with total, deterministic
transition/decline semantics; the deployed-form projection constraints; and the plan-witness
schema.

**Non-goals:** no planner induction/lowering (#843), no geometry mapping (#845), no
untouched-partition verdict (#846), no free-running generation (S3 #824 boundary). No
nearest-neighbor/paraphrase/smoothness is treated as reasoning.

## Acceptance-criteria status

This PR is the **design-freeze** step of #844; it does not itself close the issue's build
checkboxes. Each #844 acceptance criterion is mapped to the frozen artifact/test that the build
pass will establish (spec §6). No acceptance criterion is silently dropped.

## Tests and verification

Docs-only diff (one file). Claim-wording gate (`scripts/check_claim_wording.py`) passes
locally — the PR-side fast path for a docs-only change. The queue re-verifies the merged
content in full. No code, so no clippy/test/no_std/wasm surface is touched.

## Evidence artifacts

`docs/compositional_planning_spec_844.md` (frozen experimentable contract, claim-labeled per
`docs/formal_vocabulary.md`).

## Compatibility and migration

The capability-suite manifest expansion planned for the build pass is additive
(`CAPABILITY_SUITE_SCHEMA = 1` preserved; placeholder metric name kept as a report alias).
Historical artifacts remain valid and are not reportable as planning-capable without the new
sections/report identities. This PR changes no schema or code.

## Conformance effects

None in this PR (docs-only; `CONFORMANCE.md` unchanged). The spec maps to existing
RF-01/08/12/13/27/28 and proposes registering a new certifier-instrument **RF-32**
`compositional_planning_benchmarks` in the build pass (fixture-gated empirical; absent fixture
is UNAVAILABLE, never PASS).

## Claim-boundary changes

Establishes a falsifiable target and byte-level meaning only — **no** reasoning-performance or
deployed-planning claim. Written to the S4 entry-reconciliation boundary: S1 promoted; S2
REVISE honest abstention (no calibrator); S3 LIMIT (teacher-forced only, no free-running
generation). Each public "reasoning" phrase is mapped to an exact measured task/horizon and a
claim status (spec §4).

## Negative or unavailable results

None measured here (specification only). The spec fixes that an absent benchmark fixture is
`UNAVAILABLE`, never `PASS`, and that honest decline (not a calibrated confidence) is the S2
boundary-respecting uncertainty signal.

## Intentionally excluded work

All implementation (ids.toml/RF-32, Gherkin, marker/behavior tests, reference model,
generators/verifiers, expanded `compositional_reasoning.json`, `ControlKind` extensions,
regenerated `CONFORMANCE.md`) is deferred to the build pass by design (maintainer chose
design-freeze first). References #844.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

