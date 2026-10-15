# #918 — feat(#844): compositional-planning reference model (plan witness, generators, verifiers)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T06:27:55Z
- **Updated:** 2026-08-22T07:03:03Z
- **Closed:** 2026-08-22T06:49:25Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/918
- **Merged:** 2026-08-22T06:49:25Z
- **Merge commit:** a51af02f695bd65f72a4eaa9d92afce2b44499a9
- **Base / Head:** main / UOR-Foundation:issue-844-refmodel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

#844 (S4 item A) freezes the compositional-planning target and the byte-level typed
state/action semantics. The design (PR #916) and the benchmark constitution (PR #917) are on
main. This is **build increment 2 of 3**: the typed reference model — the plan witness,
evidence/provenance, ordinal confidence, typed decline, and the F1–F5 deterministic
generators/verifiers — that #843 (planner) and #846 (certification) consume.

## Scope and non-goals

**In scope:** `crates/uor-r4-graph-compiler/src/compositional_planning.rs`, extending the RF-27
`semantic_state` reference model and RF-08 planner (not duplicating them). Adds `PlanWitness`
(replayable, independently verifiable) with a total, deterministic `verify()`; `Evidence`/
provenance (F4); an ordinal `ConfidenceBand`; a typed `DeclineReason`; deterministic generators
+ a BFS reference solver producing replayable gold plans for the five frozen families; a
content-addressed sample id; a relabeling metamorphic transform; and `verify_submission`.

**Non-goals (this increment):** RF-32, the tagged Gherkin suite, and the marker/bdd wiring are
increment 3 (which closes #844); no deployed integer/table planner (#843); no geometry (#845);
no verdict (#846); no free-running generation (S3 #824 boundary).

## Acceptance-criteria status

Advances #844's "artifact-backed state/action semantics": deterministic verifier + replayable
gold per family; total reference semantics with deterministic typed decline; a
retrieval/memorized shortcut is foreclosed by the relabeling metamorphic (the action sequence,
not absolute positions, is what stays valid). RF-mapping/conformance registration is increment
3, so this PR does not close #844.

## Tests and verification

11 module tests: every family's gold verifies; the verifier rejects forbidden-intermediate,
missed-terminal, and (F4) missing-evidence witnesses; declines report the typed reason;
generation and verification are deterministic; relabeling preserves the action sequence and
changes the content id; confidence is ordinal. Full local ladder green: fmt; clippy
`-D warnings` (workspace/all-targets/all-features); `cargo test --workspace` (178 suites, 0
failures); no_std ladder; wasm lib build; `xtask validate` (R1 check-model 31 ids / R4 / R5 /
gnaf); `check_claim_wording.py`.

## Evidence artifacts

`crates/uor-r4-graph-compiler/src/compositional_planning.rs` (reference model + tests).

## Compatibility and migration

Purely additive: a new module behind `pub mod compositional_planning;`. No existing type,
signature, or artifact changes. f32/owned collections are confined to this reference-only /
off-serving-path module; the deployed integer/table form is #843's concern.

## Conformance effects

None this increment. No id/feature/marker change; `CONFORMANCE.md` unchanged (`xtask
check-model` reports it equal to the model, 31 ids). RF-32 registration is increment 3.

## Claim-boundary changes

None beyond the frozen doc's: reference-only / off-serving-path; establishes a falsifiable
target and byte-level meaning, not reasoning performance. Teacher-forced only (S3 LIMIT);
ordinal confidence, not calibrated (S2 REVISE); honest typed decline.

## Negative or unavailable results

None measured (reference model + generators only). Unsolvable instances are represented as an
honest `DeclineReason::NoPlan` gold, never a fabricated plan.

## Intentionally excluded work

RF-32 in `model/ids.toml`, `features/suites/compositional_planning_benchmarks.feature`, the
`repo-conformance` marker, the root `tests/bdd.rs` steps, and the regenerated `CONFORMANCE.md`
are increment 3 (closes #844). References #844.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

