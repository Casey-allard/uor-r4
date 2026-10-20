# #934 — research/#821-G: audit the canonical 30% quality target and pre-register baseline remediation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T00:48:58Z
- **Updated:** 2026-08-25T02:10:20Z
- **Closed:** 2026-08-25T02:09:51Z
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/934
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Parent tracker / stage

#821 — S0 truth and inference closure. This is a foundational prerequisite to the canonical evidence phase of #933; it is not parity-harness work from #932 and it is not a tracker.

## Problem

The project has repeatedly referred to an approximately 30% canonical top-1 baseline or minimum, but the live records currently expose several non-interchangeable numbers and scopes:

- canonical TLA same-distribution comparator: 28.121%;
- canonical Rule 1+2 reference/off-serving row: 24.393%;
- #908 SKMX result: 29.702%, measured through `R4Engine` and therefore not deployed `R4G1Runtime` evidence;
- RF-31's frozen causal gate: a paired +20 per-mille lower bound over the exact sections-absent normative control, not an absolute 30% threshold.

It is not yet clear from one authoritative record whether “30%” is a formally required floor, a rounded historical target, a derived expectation, or a shorthand that has conflated different scorers, populations, and decision gates. Nor is there one complete account of why the canonical graph underperforms TLA, what prior research and experiments established, what mechanisms were ruled out and why, and which mechanisms remain live and reachable today.

No further expensive parity run or threshold tuning should substitute for this foundational diagnosis.

## Objective

Produce an evidence-genealogy and teacher-free causal diagnosis that answers:

1. Where did the 30% target come from, exactly?
2. Which metric, population, selector, decode mode, confidence interval, and release decision does it govern?
3. What historical hypotheses and interventions were tried, what did each measure, and why were they promoted, limited, retired, or left open?
4. Why do the canonical graph/reference rows trail the TLA comparator on the same positions?
5. Which quality mechanisms are actually present, reachable, and production-eligible in current code/artifacts?
6. What is the cheapest falsifiable remediation sequence most likely to raise the normative baseline without weakening gates?

Only after those questions have an authoritative answer may this issue pre-register a remediation plan and unblock #933's canonical evidence phase.

## Scope

### A. Threshold and claim provenance

- Find every live and historical assertion of `30%`, `0.30`, 29.702%, 28.121%, 24.393%, the RF-31 +20 per-mille gate, and related “minimum baseline” language in code, docs, issues, PRs, comments, and evidence artifacts.
- For each, record metric definition, numerator/denominator, corpus/partition, sample or census, scorer/selector, serving scope, decode mode, uncertainty, date, artifact/report identity, and whether it was a target, empirical result, acceptance floor, or informal shorthand.
- Determine and document whether an absolute 30% requirement exists. If it does not, correct current language rather than silently manufacturing one.
- Read and reconcile at minimum #509, #516, #783, #832, #833, #908, #910, #931, #932, #933, ADR-0001, BASELINE, RESEARCH, the score records, and the authoritative S0/S2 plan.

### B. Historical research ledger

- Build an append-only hypothesis/experiment table covering the plain TLA store, Rule 1/Rule 1+2, EXCT, cover/emission/transition changes, FWDA/right-context/latent arms, prompt-conditioned evidence, SKMX/PSIB, corpus/bundle refreshes, and any other intervention that materially affected the cited baseline.
- For every row, distinguish reference mathematics, compiler behavior, certifier evidence, dormant runtime code, normative runtime reachability, deployed-serving reachability, empirical evidence, and assumptions.
- Record the exact positive/negative/degenerate/unavailable verdict and why a mechanism was promoted, rejected, limited, retired, or remains unresolved. Do not rewrite historical records.

### C. Current live-path diagnosis

Using the already-attested canonical corpus/artifact/tokenizer/cover only:

- Reproduce the relevant teacher-free baseline rows or record UNAVAILABLE with the missing identity; do not run a teacher forward, new observation job, download, or hours-class parity suite.
- Decompose same-position errors and headroom by ExactContext/Graph/Novel status, probe depth/support, candidate recall, TLA hit/miss cross-tab, active section, SKMX versus PSIB contribution, sections-absent identity, and any other already-recorded causal lane.
- Quantify reachability ceilings before proposing work. A mechanism that cannot touch enough errors to close the declared gap is not a remediation candidate.
- Verify which mechanisms exist in the current source, current artifact, `R4G1Runtime`, production surfaces, and strict admission. “Implemented somewhere” is not “live.”
- Identify whether the deficit arises primarily from evidence coverage, candidate recall, score/ranking, exact-context dominance, partition mismatch, stale artifact generation, unreachable runtime sections, or a comparator/claim-boundary error.

### D. Pre-registered remediation plan

After A–C are complete, rank bounded interventions by reachable improvement per unit cost. For each retained intervention declare:

- metric/current value and target;
- reachability arithmetic;
- cheapest binding instrument and required verdict;
- positive and negative next actions;
- falsifier/control;
- maximum wall time and resources;
- affected implementation/claim boundary; and
- why earlier evidence did not already decide it.

Split implementation into concrete child issues only after this ordering exists. Prefer fixes to wasted computation, stale/unreachable code, evidence coverage, and ranking logic over raising worker counts or launching broad searches. No proposed phase may exceed one projected hour without a separately approved run contract.

## Non-goals

- No lowering, post-hoc raising, or relabeling of a threshold to obtain a pass.
- No live teacher inference, source download, observation rebuild, multi-hour parity run, or speculative hyperparameter sweep.
- No production-code change before the causal diagnosis and remediation order are reviewed.
- No crediting `R4Engine` reference results as `R4G1Runtime` deployed-serving evidence.
- No changing #932's exact-parallel harness contract; that issue remains performance/observability infrastructure.
- No replacing #933's structural normative-selector work; this issue supplies the missing quality rationale and remediation ordering.

## Required deliverables

- `docs/canonical_quality_baseline_<issue>.md`, append-only, containing the threshold provenance, historical experiment ledger, current live-path map, reproduced counts/CIDs, error decomposition, reachability arithmetic, and pre-registered remediation order.
- Corrections to current README/ROADMAP/RESEARCH/BASELINE/plan language wherever it presently treats incomparable metrics as one baseline.
- A machine-readable teacher-free diagnostic artifact with versioned schema and content-bound input identities, or an exact UNAVAILABLE record.
- Native dependency updates so #933 cannot claim a canonical quality verdict until this audit is resolved; #932 remains downstream of #933.
- Concrete follow-up implementation issues only for interventions that survive the cheap instruments.

## Acceptance criteria

- [ ] The repository has one unambiguous answer to whether 30% is required and why.
- [ ] Every cited baseline number is mapped to its exact metric/population/selector/scope and source artifact.
- [ ] Prior work is classified with evidence-backed positive, negative, degenerate, unavailable, limited, or retired status; no significant attempt is silently omitted.
- [ ] The current canonical deficit versus TLA is reproduced teacher-free on identical positions or recorded UNAVAILABLE.
- [ ] Error attribution and reachability ceilings explain which live mechanisms can and cannot close the gap.
- [ ] Current source, artifact, runtime, production-surface, and admission reachability agree.
- [ ] The remediation plan is ordered, bounded, falsifiable, and gives different next actions for positive and negative results.
- [ ] No hours-class run is authorized; missing evidence is UNAVAILABLE, never PASS.
- [ ] Current documentation and native dependency state agree with the audit.

## Verification

Issue-specific provenance/link/schema tests and teacher-free diagnostic reproduction first, followed by claim wording, conformance/register validation, generated-file determinism, documentation/link checks, and any repository gates touched by the final documentation or diagnostic harness. A later implementation child owns its own full code gates.

## Claim boundary

This issue is research/specification and teacher-free certification evidence. It may conclude that the absolute 30% premise is unsupported, that the current normative path is quality-negative, or that one or more mechanisms have reachable headroom. Each is a legitimate evidence-backed result. It does not itself ratify RF-31 or production admission.


## Comments (3)

### comment 5403692745 — 2026-08-25T01:14:11Z — Casey-allard

### Pre-registered teacher-free audit contract\n\n- Metric/current state: reconcile the canonical broad-clean Rule 1+2 and same-position TLA full-census rows, then bind the historical skip-mix result to its actual selector and artifact scope. Current records say 17,595/72,130 versus 20,284/72,130; strict normative admission is not established.\n- Reachability arithmetic: the same-report deficit is 2,689 positions. A mechanism restricted to the 19,732 non-EXCT positions must recover at least 2,689/19,732 = 136.28 permille of that slice to tie TLA. No universal 30 percent target is assumed.\n- Binding instrument: verify canonical file CIDs, parse the committed full-census score report, validate its crosstab/status/candidate arithmetic, verify the release graph lacks SKMX/PSIB, and rerun the existing #908 teacher-free full-census harness without changing thresholds or inputs.\n- Exit rule: any identity, count, arithmetic, sections-absence, or deterministic-result mismatch is FAILED or UNAVAILABLE. No teacher, observation rebuild, download, threshold change, or substitute bundle is permitted.\n- If positive: retain #933 as the first bounded implementation step, requiring its own normative R4G1Runtime sample before any census; then let the measured branch determine whether ranking work is reachable.\n- If negative: do not launch #933 empirical work or #932 live parity; record the exact failed prerequisite and revise the remediation order.\n- Cost: expected under ten minutes, teacher-free, current canonical files only, no phase authorized above one hour.

### comment 5403953162 — 2026-08-25T01:47:28Z — Casey-allard

## Pre-registered audit outcome — 2026-08-24

**Verdict:** audit criteria PASS at teacher-free certifier/reachability scope; canonical production quality remains **NOT ESTABLISHED**.

- Threshold genealogy: there is no universal absolute 30% requirement. The fixed 29.7% tolerance is scoped to pinned/legacy reports; the broad-clean profile is `relative_tla`.
- Canonical bound census: Rule 1+2 = 17,595/72,130 (24.393456%); same-position TLA = 20,284/72,130 (28.121447%); deficit = 2,689 hits / -37.2799 permille.
- Identity audit: source, corpus, tokenizer, teacher artifact, cover, graph, score report, and release manifest CIDs all matched. The current canonical graph has neither SKMX nor PSIB, so that lane is unreachable.
- Scope: the existing #833 schema-26 census was identity-verified, not rerun. Canonical Gate C, teacher forward, and observation/corpus rebuild are explicitly `NOT_RUN`.
- #908 reproduction: PASS in 128.4 s under its original `R4Engine` reference/off-serving scope; base 19,372, skip 21,424, shuffled null 2,281; result CID `blake3:e32e4e33d70f342ae3c0913ba00d9aef0cf789b539b9e1b658a9366c51402a26` reproduced exactly.
- Decision: #933 remains the first structural step. Only after bound section emission and normative reachability may the 6,000-position teacher-free sample authorize any isolated retrieval/ranking child or a full census. #932 remains downstream.

Evidence and machine-readable diagnostic: PR #935.

Local final-state verification passed: full workspace tests, full warnings-as-errors lint, formatting, both `no_std` modes, live canonical-bundle binding, non-vacuous canonical kappa reproduction, wasm32 check, claim wording, repository registers, and dependency policy.


### comment 5404133082 — 2026-08-25T02:10:20Z — Casey-allard

## Closure evidence

Resolved by PR #935 through the protected merge queue.

- branch commit: `98427edc82f8047a6e35d58fae2c892cfd8198df`
- speculative merge and `main` commit: `c29fa0e003aaa2176d7b4024508540d883490141`
- binding merge-group run: https://github.com/UOR-Foundation/uor-r4/actions/runs/32799087119 — **SUCCESS**
- issue auto-closed at 2026-08-25T02:09:51Z

The queue passed the full Rust ladder, standalone BDD suite, doc tests, both `no_std` modes, deterministic rebuild, non-vacuous kappa reproduction, Gate C trend, wasm packaging, nightly fuzz, all-features dormant tests, repository conformance/dependency policy, and audit context.

Native hierarchy was re-read after merge: #934 still blocks #933 as a resolved dependency; #933 remains open and blocks #932; #932 remains open and blocks #931. No downstream issue was started in this one-issue execution.



---
_Comment/review ids in this file: 5403692745, 5403953162, 5404133082_

