# #917 — feat(#844): freeze the S4 compositional-reasoning benchmark constitution

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T05:38:26Z
- **Updated:** 2026-08-22T06:03:36Z
- **Closed:** 2026-08-22T05:59:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/917
- **Merged:** 2026-08-22T05:59:54Z
- **Merge commit:** f631a133487e4905dbb3201534f8c1d285e51afd
- **Base / Head:** main / UOR-Foundation:issue-844-compositional-build
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

#844 (S4 item A) freezes the compositional-planning benchmark target and the typed
state/action contract for S4. Its design was frozen in `docs/compositional_planning_spec_844.md`
(PR #916). This is **build increment 1 of 2**: it lands the frozen **benchmark constitution**
in the #832 capability-suite framework, converting the `s4-compositional-reasoning` placeholder
into the frozen declaration the stage is judged on. Increment 2 lands the typed reference model
+ generators/verifiers + RF-32 + Gherkin.

## Scope and non-goals

**In scope:** expand `capability_suites/compositional_reasoning.json`; add the three new
generalization split axes (`by_vocabulary`, `by_operator_composition`, `by_horizon`) to
`SplitRules`; add the four planning-baseline `ControlKind` variants (`retrieval-only`,
`direct-continuation`, `memorized-trajectory`, `shortest-path-oracle`); freeze the primary
metric, promotion statistic, controls, and pinned identities; add the machine-checked record
`tests/compositional_planning_spec_844.rs`.

**Non-goals (this increment):** no typed reference model, generators, verifiers, RF-32, or
Gherkin (increment 2); no planner (#843); no geometry (#845); no verdict (#846); no
free-running generation (S3 #824 boundary).

## Acceptance-criteria status

Advances #844's "freeze compositional benchmarks" scope: the frozen split axes prevent
entity/topology/vocabulary/composition memorization, and the six baselines (incl. the two
memorization nulls) are frozen. The reference-semantics acceptance items are increment 2. This
PR does not close #844.

## Tests and verification

New: 5 tests in `compositional_planning_spec_844.rs` (manifest validates + is the named S4
primary; frozen valid-plan-rate metric; six split axes; six baselines; kebab-case labels + ALL
exhaustiveness). Existing `capability_suites_832.rs` (9) still green. Full local ladder green:
`cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
`cargo test --workspace --no-fail-fast` (178 suites, 0 failures); no_std ladder
(`uor-r4-graph-format` `--no-default-features` ± `alloc`); wasm lib build; `xtask validate`
(R1 check-model 31 ids / R4 / R5 / gnaf); `check_claim_wording.py`.

## Evidence artifacts

`crates/uor-r4-api/capability_suites/compositional_reasoning.json` (frozen manifest);
`crates/uor-r4-api/tests/compositional_planning_spec_844.rs` (record).

## Compatibility and migration

Additive. The three new `SplitRules` fields are `#[serde(default)]`, so the other nine committed
manifests deserialize unchanged. The four new `ControlKind` variants extend the enum without
altering existing ones. `CAPABILITY_SUITE_SCHEMA`/`CAPABILITY_REPORT_SCHEMA` stay at 1. The
primary metric is renamed from the placeholder `held-out-composition-accuracy` to
`held-out-valid-plan-rate`; no committed report references the old name (S4 has not run).

## Conformance effects

None. No id/feature/marker change this increment; `CONFORMANCE.md` unchanged (`xtask check-model`
reports it equal to the model, 31 ids). RF-32 registration is increment 2.

## Claim-boundary changes

None beyond the frozen doc's: reference-only / off-serving-path; establishes a falsifiable
target, not reasoning performance. Teacher-forced only (S3 LIMIT); honest decline, not calibrated
confidence (S2 REVISE).

## Negative or unavailable results

None (schema/constitution freeze). The manifest fixes that an absent benchmark fixture is
UNAVAILABLE, never PASS (framework invariant).

## Intentionally excluded work

Typed reference model, F1–F5 generators/verifiers, RF-32 + Gherkin + marker/bdd steps, and the
metamorphic/tamper/power/witness-replay tests are increment 2. References #844.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

