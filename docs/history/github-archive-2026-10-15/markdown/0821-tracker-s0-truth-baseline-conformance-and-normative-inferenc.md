# #821 — TRACKER/S0: truth baseline, conformance, and normative inference closure

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:32:18Z
- **Updated:** 2026-08-25T06:12:44Z
- **Closed:** 2026-08-25T06:12:44Z
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/821
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

## Objective

Establish one authoritative post-v0.1 truth baseline: a canonical completion plan, execution-scope-aware conformance, one normative scorer, CID-bound evaluation, and a fresh source-complete attested bundle.

## Current evidence

- #784 refuted full-depth context-code collision and localized the failure to continuation-distribution convergence.
- #811 left all five semantic-OOD probes SERVABLE; D4 parity is not semantic answerability.
- The generated claim register does not distinguish reference, dormant, portable-runtime, normative, and deployed-serving reachability.
- R4Engine, GraphScorer, and R4G1Runtime evidence can currently be overread across different execution scopes.

## Entry criteria

Current v0.1/main is reproducible and the prior measurement records are retained as historical evidence.

## Native dependencies

None. #821 is the programme entry stage.

## Parallel formal foundation

#859 runs during the S0 milestone as a direct child of #820. It is deliberately not a child or promotion blocker of #821 because the current f64 prime router is outside the graph migration path. This stage must nevertheless make its reference-only proof scope representable in conformance.

## Promotion gate

All five children have final verdicts; one normative scoring path is named and differentially tested; benchmark identities and attribution are frozen; a fully attested post-#755 broad bundle passes actual production admission; M.V.G. thresholds are retained, revised, or retired with dated evidence.

## Kill / redesign criterion

If current claimed signals cannot be reproduced from pinned source inputs and the deployed path, downstream quality, geometry, and scale promotions remain unauthorized.

## Native children

- [x] #829 — Publish the completion plan and evidence-first issue forms — done (PR #860, merged d1d1f735)
- [ ] #830 — Encode execution scope, serving reachability, and non-vacuous verdicts
- [ ] #831 — Designate the normative R4G1 scorer
- [ ] #832 — Commit CID-bound suites and per-token attribution
- [ ] #833 — Rebuild the attested broad bundle and re-ratify the baseline

GitHub's native parent/sub-issue relationship is the hierarchy source of truth.

## Non-goals

- Implement prompt state, abstention, planning, or instruction behavior.
- Rewrite historical measurement records; corrections are appended and current summaries are reconciled.

## Repository conformance

Primary IDs: RF-09, RF-15, RF-21, RF-22, RF-23, RF-29. CONFORMANCE.md remains generated; fixture absence is UNAVAILABLE, never empirical PASS.

Every leaf declares execution scope, preserves the applicable P-4/determinism/allocation/no_std/error/witness contracts, maps to existing RF capabilities or justifies a new one, and updates generated conformance only through its TOML/Gherkin/test sources. Empirical work posts the repository run contract before expensive execution.

## Closure rule

Close only after every native child has a completed, negative, not-triggered, or explicitly descoped verdict and this tracker records `PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE` with exact evidence links.

## Comments (4)

### comment 5351353821 — 2026-08-20T04:26:46Z — Casey-allard

S0 progress: item D (#832) merged via #863 (`70586aa0`, queue full-ladder green) and closed COMPLETED. S0 now 4/5 children done (#829, #830, #831, #832); remaining: item E #833 (rebuild attested broad bundle + re-ratify baseline), now unblocked.

### comment 5358327757 — 2026-08-20T15:49:58Z — Casey-allard

## S0 stage verdict — **PROMOTE** (2026-08-20)

All five native children of this tracker are closed with recorded final verdicts, and every
condition of the S0 promotion gate is met. Per the closure rule ("close only after every
native child has a completed/negative/not-triggered/descoped verdict and this tracker records
`PROMOTE`/`REVISE`/`LIMIT`/`RETIRE` with exact evidence"), S0 is **PROMOTED**. Downstream
stages **S1 (#822)** and **S2 (#823)** are unlocked.

### Promotion gate — condition-by-condition evidence

1. **All five children have final verdicts** — all CLOSED/COMPLETED:
   - #829 — completion plan + evidence-first issue forms — PR #860 (`d1d1f735`)
   - #830 — execution scope / serving reachability / non-vacuous verdicts — PR #861 (`bc5a2a92`)
   - #831 — normative R4G1 scorer designation — PR #862 (`1c377df7`)
   - #832 — CID-bound capability suites + per-token attribution — PR #863 (`70586aa0`)
   - #833 — attested post-#755 broad rebuild + re-ratify — PR #867 (`1b3e46f2`), determinism/provenance chain #864 (`9c1f2c10`), #866 (`19465441`)
2. **One normative scoring path named and differentially tested** — the deployed R4G1 runtime
   scoring path is the single normative scorer per [ADR-0001](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/adr/0001-normative-r4g1-scorer.md) (#831); spec-vs-deployed
   accumulator/selector differentials plus a planted-negative control in
   `crates/uor-r4-graph-certify/tests/normative_scorer_831.rs`. Record: `docs/normative_scorer_831.md`.
3. **Benchmark identities and attribution frozen** — CID-bound capability suites (blake3
   `compute_cid`/`verify_cid`), a stage→primary-suite constitution, and 7-way per-token
   `ResolutionPath` attribution in `crates/uor-r4-api/src/capability_suite.rs` with committed
   manifests (#832). Record: `docs/capability_suites_832.md`.
4. **Fully attested post-#755 broad bundle passes actual production admission** — the
   source-complete, #755-native, byte-reproducible bundle `smollm2-360m-broad-clean` passes
   deployed R4Engine admission + the offline prediction canary (#833). Record:
   `docs/attested_broad_baseline_833.md`.
5. **M.V.G. thresholds retained/revised/retired with dated evidence** — **RETAINED**. Gate C
   RATIFY (RETAIN), 2026-08-20, held-out 72,130: Rule 1+2 24.30% → 24.39%, best-live
   31.48% → 31.11%, TLA-3 28.21% → 28.12% (all within the predeclared `<~0.5 pp` reachability
   bound); witness replay 64/64. Teacher floor 3.6015 bits RETAINED by invariance.

### Kill / redesign criterion — not triggered

The claimed signals reproduce from pinned source inputs through the deployed path
(byte-reproducible rebuild, admitted bundle, Gate C within bound), so downstream quality,
geometry, and scale promotions are authorized.

### Recorded / carried forward (non-blocking)

- Full-population **teacher-floor re-measurement is UNAVAILABLE** at scale under the current
  exact-GEMM teacher path (~24 h for the held-out split); RETAINED by invariance. Recommended
  follow-up: a faster teacher forward path or a persisted floor cache — not a promotion blocker.

### Definition-of-Done

- Readable mirror reconciled: `docs/r4_intelligence_completion_plan.md` (S0 PROMOTE record) +
  `ROADMAP.md` (S1 now the entry stage) via **PR #871**, merged to `main` as **`43b7b9e9`**.
- GitHub remains the source of record for stage status; this comment is the tracker's recorded
  verdict.

Closing as **completed (PROMOTE)**.


### comment 5401402160 — 2026-08-24T21:08:24Z — Casey-allard

Reopening S0 after the 2026-08-24 foundation audit established that the prior PROMOTE record did not prove strict R4G1Runtime deployed-serving reachability: #833 used the explicit quality-bypass loader, #908 measured an off-serving R4Engine harness, and #910 did not lower SKMX/PSIB into the ADR-designated normative selector. #933 is the concrete corrective child and owns the append-only truth correction, single-selector restoration, CID-bound admission evidence, canonical broad re-emission, and teacher-free verdict. This tracker is a governance container and remains unassigned; #933 is the only actively assigned implementation issue.

### comment 5406310189 — 2026-08-25T06:12:43Z — Casey-allard

## Corrected S0 stage verdict — **PROMOTE** (2026-08-25)

This append-only verdict supersedes the 2026-08-20 PROMOTE comment where that comment treated #833's bypass-loader / `R4Engine` canary as strict normative production admission. The 2026-08-24 reopening was warranted; #933 and #934 have now resolved the identified evidence gap.

### Native child audit

Every native S0 child is closed with an explicit outcome: #829, #830, #831, #832, #833, corrective child #933, and baseline-genealogy child #934. #933 closed via protected PR #936 at merge commit `e346816cb40b089583cbd3fff2cdd84924362c65`; its binding merge-group run passed: https://github.com/UOR-Foundation/uor-r4/actions/runs/32814445068

### Corrected promotion evidence

- one production candidate/token owner: `R4G1Runtime` across the shared native, HTTP, library, install, and WASM/static envelope;
- schema-2 admission binds the exact graph, population, positions, tokenizer, corpus, selector/decode configuration, full deployed-quality report, raw parity/witness evidence, and parsed TLA comparator store;
- strict production startup admits the exact bundle from an empty model store and fails closed on substituted, mutated, wrong-version, wrong-position, or malformed-but-rebound comparator evidence;
- exact 72,130-position normative census: 21,293 / 72,130 = 29.5203% versus same-position TLA 20,284 / 72,130 = 28.1214%, paired +13.988 permille [11.057, 16.919];
- same-generation sections-absent control: 18,806 / 72,130 = 26.0723%, giving +34.479 permille [31.681, 37.277] and clearing RF-31's unchanged +20 permille lower-bound gate;
- collapsing rotated-label null, 0 / 72,130 absent-identity mismatches, 0 / 72,138 cross-surface mismatches, 64 / 64 witness replay, and byte-identical 4/8-worker semantic artifacts;
- the protected queue passed the full test/BDD/doc/no-std/determinism/κ/Gate C/WebAssembly/fuzz/security/conformance/all-features ladder.

**S0 verdict: PROMOTE at the corrected, exact evidence scope.** This establishes the truth boundary and production-admitted teacher-forced-position result for the bound #933 envelope. It does not establish a universal 30% floor, live-teacher parity, instruction following, reasoning, factuality, semantic abstention, or free-running coherence.

#908's 29.702% remains reference/off-serving evidence; it is not relabeled. The BDD run was 124 / 124, but live-teacher fixtures were absent, so parity is NOT_RUN rather than PASS.

The native downstream chain remains truthful: #932 is open and unassigned now that its sole blocker #933 is closed; #931 remains open and blocked by #932. No downstream parity or confidence result is implied by this S0 closure.

Canonical record: https://github.com/UOR-Foundation/uor-r4/blob/main/docs/normative_r4g1_quality_933.md
PR: https://github.com/UOR-Foundation/uor-r4/pull/936

Closing this governance tracker as completed (**PROMOTE**).


---
_Comment/review ids in this file: 5351353821, 5358327757, 5401402160, 5406310189_

