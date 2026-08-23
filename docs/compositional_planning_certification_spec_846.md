# Compositional-planning certification and claim boundary — execution contract (#846)

- **Issue:** #846 — "research/#826-D: certify compositional planning and bound the reasoning
  claim" (item D of S4 tracker #826, programme #820).
- **Date:** 2026-08-22 (America/New_York).
- **Status:** frozen execution contract. The source-of-record run contract was posted on #846
  before the access audit (`issuecomment-5383344816`). This document records that freeze and does
  not retroactively change it.
- **Claim language:** normative per [`formal_vocabulary.md`](formal_vocabulary.md).
- **Execution scope:** the candidate is the RF-33 **normative-runtime / deployed-serving** planner;
  this audit and any final measurement are **certifier-instrument / off-serving-path** evidence.
  Reference, packed, normative-runtime, and deployed-serving results remain separate.

## 1. Frozen candidate, identities, and inherited boundaries

**Definition (candidate).** The only candidate is RF-33 `bounded-breadth-first`, the non-geometric
arm selected and lowered by #843. `table-guided-beam` remains an exactly scoring sibling but is not
the selected mechanism. W(3,3) is excluded by #845's `NO GEOMETRIC ADVANTAGE` verdict; no geometry
arm is eligible for the final certification.

**Definition (source freeze).** Source baseline `ddf4a14236973fe3c8e7076b8e5c9e3f1c6cb2cb`.
The certifier emits these length-delimited BLAKE3 bindings:

- benchmark: `blake3:5ec9e6d885d8c211b3842e9a5d62a5e7a0c02ea5788ca9d079ecfc0c389debe2`;
- candidate: `blake3:5c16a08dd8684494ee7a68f081a6a215643d8e20eb6f1a9c69197245586c9f59`;
- audit inputs: `blake3:2ede56d0204570c42392efe14afdbf4da5440cf9194e5c3514e42ef15ba288c0`.

The benchmark binding covers the committed `s4-compositional-reasoning` manifest and generator.
The candidate binding covers the RF-33 identity, induction, packed-section, and deployed-planner
sources. The audit binding additionally covers the #843 full-grid source, the shared #845 episode
source, and the #843 measurement record.

Inherited boundaries do not move: teacher-forced planning only (#824 `LIMIT`); ordinal honest
decline, not calibrated confidence (#823 `REVISE`); five F1-F5 families; H in {1, 2, 4, 8};
`H_max = 16`; `W_max = 64`; n = 512 per cell; delta_min = 0.05; the intersection-union reading with
Holm-Bonferroni reported alongside; and the #843 witness, budget, allocation, P-4, deterministic,
and typed-decline guarantees.

## 2. Execution checklist and outcome branches

The issue body is translated without omission:

1. Freeze candidate, source, benchmark, thresholds, controls, and final partition identities.
2. Audit the final partition CID, access log, and disjointness before generating any final task.
3. If and only if item 2 passes, run every required generalization axis and horizon, report all
   terminal/intermediate/constraint/evidence/decline/failure/resource metrics, and recompute the
   statistics independently.
4. Replay every successful `PWIT` independently; plant an invalid intermediate path whose terminal
   state matches and require replay to reject it.
5. Measure retrieval-only, direct-continuation, memorized-trajectory, shuffled-state, trivial-prior,
   and the selected non-geometric arm under equal budgets. Geometry is reported as the completed
   negative predecessor result, not rerun.
6. Exercise prompt-state and selective-decision integration, including unsupported/no-plan decline,
   through the normative production route.
7. Repeat on the frozen second task family/domain and report the predeclared multiple-comparison
   reading.
8. Publish the task/horizon capability table, falsifiers, resource envelope, limits, and one of
   `PROMOTE TYPED PLANNING`, `LIMITED CAPABILITY`, or `REASONING NOT ESTABLISHED`.

**Empirical Criterion (positive branch). Status: Empirical.** `PROMOTE TYPED PLANNING` requires
every mandatory final-partition lower bound, witness rule, non-regression gate, and resource
envelope to pass without retuning.

**Empirical Criterion (limited branch). Status: Empirical.** `LIMITED CAPABILITY` requires a valid,
CID-bound final run that supports a strict subset and names every unsupported cell.

**Definition (invalid/unavailable branch).** If the final partition is absent, previously opened,
not CID-bound, not access-logged, or not materialized on every claimed split axis, the full grid is
`NOT_RUN` and the verdict is `REASONING NOT ESTABLISHED`. Evidence unavailable under that failure is
`UNAVAILABLE`, never a zero or a pass.

## 3. Binding cheap instrument

The instrument is `crates/uor-r4-graph-certify/tests/compositional_planning_certification_846.rs`.
It reads committed source and metadata only; it never calls the task generator. It checks:

- the seal is non-empty and exactly equals the cells untouched by fitting and prior evaluation;
- a partition CID and access-log binding exist in the committed suite identity;
- operator composition is a materialized split coordinate, not only a manifest boolean;
- predecessor access is reconstructed from the exact joint seed walk;
- benchmark, candidate, and audit inputs are content-bound;
- a synthetic valid seal passes and an overlapping seal fails, so the instrument is non-vacuous.

**Empirical Criterion (launch gate). Status: Empirical.** Every row above must pass and the sealed
sample ceiling must be non-zero before any final task is generated. A failure binds the negative
branch.

## 4. Run contract

    metric to move:       final sealed-cell lower bound of candidate minus strongest non-oracle
                          control on every family x horizon cell. #843 current: candidate 1.0000
                          in 20/20 prior joint cells; delta_min cleared in 12/20.
    reachability ceiling: number of CID-bound, access-logged, disjoint sealed cells x n=512.
                          Zero cells means zero certifiable samples and no final lower bound.
    instrument + verdict: section 3; every row must PASS before final access.
    exit rule:            section 2 outcome branches, without retuning.
    if positive:          freeze the exact typed-planning boundary for S5/release work.
    if negative:          preserve RF-33 LIMITED, publish the failed prerequisite, do not broaden
                          the claim, and close S4 with the limiting verdict.
    cost estimate:        audit under one minute; a valid full synthetic grid would take minutes
                          on one core, teacher-free and fixture-free. No grid runs after FAIL.

The branches cause different actions, so the audit has decision value.

## 5. Non-goals, compatibility, and conformance

- Do not choose new cells, extend the generator, or create a seal after inspecting the selected
  candidate. That would be post-selection retuning, not untouched certification.
- Do not revoke #843's RF-33 LIMITED claim when the final-certification prerequisite fails.
- Do not describe the task suite as universal/general intelligence, free-running coherence, or
  calibrated confidence.
- No artifact, runtime, API, format, or serving semantics change. Historical bundles remain valid.
- No new built capability is introduced. RF-33 keeps its exact LIMITED statement; RF-32 remains the
  certifier instrument. `CONFORMANCE.md` must remain generated and byte-identical.

## 6. Required evidence and verification

- content-bound JSON result: `compositional_planning_certification_846_result.json`;
- append-only verdict record: `compositional_planning_certification_846.md`;
- pass/fail-direction access audit and committed-result CID check;
- issue-specific RF-33 planner/witness/invalid-intermediate/allocation/P-4 regressions;
- repository conformance, claim wording, workspace tests/clippy/fmt/no_std, and applicable
  deterministic checks.

The full final grid, second-domain table, and multiple-comparison table are required only after the
binding access gate passes. On gate failure they are recorded `UNAVAILABLE`/`NOT_RUN` rather than
silently omitted.
