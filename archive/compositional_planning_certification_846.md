# Compositional-planning certification — access audit and S4 verdict (#846)

- **Issue:** #846, item D of S4 tracker #826.
- **Date:** 2026-08-22 (America/New_York). **Records are append-only.**
- **Frozen contract:** [`compositional_planning_certification_spec_846.md`](compositional_planning_certification_spec_846.md),
  source-of-record pre-access comment `issuecomment-5383344816`.
- **Result:** [`compositional_planning_certification_846_result.json`](compositional_planning_certification_846_result.json).
- **Execution scope:** certifier-instrument / off-serving-path audit of the RF-33 normative-runtime /
  deployed-serving candidate.

## 1. Verdict — REASONING NOT ESTABLISHED

**Empirical Criterion (S4 final-certification outcome). Status: Empirical.**

> The required untouched final partition does not exist in the committed benchmark identity. The
> #846 access audit therefore fails with a sealed sample ceiling of **0**, the final grid is
> `NOT_RUN`, and the S4 promotion claim is **REASONING NOT ESTABLISHED**. This does not revoke the
> narrower RF-33 result: bounded semantic-transition planning remains **LIMITED** on the 12 of 20
> previously evaluated joint-split cells established by #843.

No new partition was selected after the candidate was known. Doing so would turn a final
certification into post-selection benchmark design and would violate #846's no-retuning rule.

## 2. Binding access-audit result

The audit used committed metadata/source only and did not generate a final instance.

| Required precondition | Result | Evidence |
|---|---|---|
| Non-empty declared seal | **FAIL** | both predecessor harnesses instantiate `sealed_topologies` as empty |
| Untouched topology cell | **FAIL** | fitting uses 0–3; #843/#845 joint evaluation uses 4–7; union = all 8 cells |
| Untouched operator-composition cell | **FAIL** | `SplitCell` has entity/vocabulary/topology/template/horizon but no composition coordinate; the manifest boolean is not a materialized seal |
| CID-bound final partition | **FAIL** | the committed suite manifest has no `sealed_partition_cid` or `slice_partition_cid` |
| Access-log binding | **FAIL** | the committed suite manifest names no access record |
| Non-zero final sample ceiling | **FAIL** | 0 sealed cells x 512 = **0** certifiable samples |
| Instrument can pass and fail | PASS | synthetic disjoint seal passes; overlapping seal fails |
| Candidate/benchmark/audit content identities | PASS | three BLAKE3 bindings in the result JSON, checked by the test |

The apparent contract handoff in #843 — "sealed composition and topology cells, which this issue
never opened" — is corrected by source and access arithmetic. `compositional_planning_measurement_843.rs`
ran its full joint grid with an empty seal, and its 512-sample high-half walk covers topology cells
4–7. The fitting walk covers 0–3. There is no ninth frozen topology cell, and operator composition
was never represented as an independently sealable coordinate.

## 3. Why the full run did not launch

**Definition (reachability ceiling).** The maximum number of samples that can contribute to an
untouched-partition statistic is `|sealed disjoint cells| x 512`. Here that is `0 x 512 = 0`.
Therefore no confidence bound, effect-floor comparison, or multiple-comparison table exists for the
claimed final scope.

The issue and repository long-run policies are explicit: unavailable identities are
`UNAVAILABLE`, never `PASS`; a failed cheap instrument stops the expensive/final run; and positive
and negative branches must cause different actions. Those rules apply even though the synthetic
grid would take minutes rather than hours, because the invalidity is epistemic, not computational.

## 4. Requirement and acceptance-criteria disposition

| Requirement | Disposition |
|---|---|
| Candidate and source identities frozen before audit | MET — RF-33 breadth-first and exact source/CIDs frozen in the issue comment |
| Final partitions frozen before access | **NOT MET — prerequisite absent**; the negative verdict is the evidence-backed outcome |
| Entity/vocabulary/topology/composition/template/relabeling/counterfactual/horizon final slices | `UNAVAILABLE`; no final partition |
| Terminal/intermediate/constraint/evidence/decline/failure/resource table | `UNAVAILABLE` at final scope; #843 tables remain prior-scope evidence |
| Equal-budget nulls and geometry | `NOT_RUN`; #843 controls remain prior-scope evidence and #845 geometry remains negative |
| Independent final witness census | `UNAVAILABLE`; existing PWIT replay and planted invalid-intermediate tests remain Structural evidence |
| Prompt-state/selective-decision final integration | `UNAVAILABLE`; no final queries may be opened |
| Second task/domain replication | `UNAVAILABLE`; no frozen second-domain partition identity exists |
| Multiple-comparison handling | `UNAVAILABLE`; no final cell statistics exist |
| Verdict and precise public boundary | MET — `REASONING NOT ESTABLISHED`; RF-33 LIMITED preserved |

Thus the positive acceptance criteria do not pass, but the issue's allowed negative outcome is
complete and fully evidenced. No difficult requirement is silently dropped.

## 5. Content identities

- benchmark: `blake3:5ec9e6d885d8c211b3842e9a5d62a5e7a0c02ea5788ca9d079ecfc0c389debe2`;
- candidate: `blake3:5c16a08dd8684494ee7a68f081a6a215643d8e20eb6f1a9c69197245586c9f59`;
- audit inputs: `blake3:2ede56d0204570c42392efe14afdbf4da5440cf9194e5c3514e42ef15ba288c0`.

The committed test recomputes all three and refuses a stale result JSON.

## 6. Compatibility, conformance, and claim boundary

- No compiler, artifact, runtime, API, CLI, server, or format behavior changes.
- `CONFORMANCE.md` remains at 33 ids and is not edited. RF-33 keeps its exact 12/20 LIMITED
  statement; RF-32 remains certifier-instrument / off-serving-path.
- Existing Structural evidence remains in force: reference/packed/production differential, PWIT
  replay and invalid-intermediate rejection, allocation census, P-4 scan, bounded counters, typed
  declines, absent-section identity, and deterministic bytes.
- No claim of general/universal intelligence, free-running coherence, or calibrated confidence is
  made. The only positive planning phrase remains the #843-scoped "bounded typed state-transition
  planning on the measured tasks and horizons."

## 7. Re-entry condition

Re-entry is a **new versioned benchmark programme**, not a rerun of this final candidate: materialize
operator composition as a real split coordinate; commit non-empty partition CIDs and an access-log
binding before candidate fitting/selection; reserve cells outside both fitting and all development
evaluation; freeze a second domain and the multiple-comparison family; then repeat candidate
selection and final certification under the unchanged honesty rules. A post-hoc seal over this
already-evaluated eight-cell axis cannot satisfy the condition.
