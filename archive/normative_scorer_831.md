# Normative R4G1 scorer designation — evidence record (#831)

- **Issue:** #831 — "architecture: designate one normative R4G1 scorer for serving,
  certification, patches, and proofs" (item C of S0 tracker #821, programme #820).
- **Decision record:** [`docs/adr/0001-normative-r4g1-scorer.md`](adr/0001-normative-r4g1-scorer.md).
- **Date:** 2026-08-20.
- **Claim status:** Establishes semantic ownership + reachability + fail-closed divergence.
  Not a quality claim; the two scorers are **not** asserted exactly equivalent.

This record is append-only. It captures the inventory, the differential/reachability
evidence, and the acceptance-criteria status behind ADR-0001.

## Scorer inventory (as found)

| Surface | Symbol | Role | Scope |
|---|---|---|---|
| Deployed runtime scorer | `uor-r4-graph-runtime::R4G1Runtime` (+ `scoring`) | selects the served token; owns patch + witness | normative-runtime (served) |
| Normative spec | `uor-r4-graph-format::scoring_semantics` v1.0.0 | the rules the runtime realizes | normative-runtime (spec) |
| Reference / certifier | `uor-r4-graph-certify::score_runtime::GraphScorer` | witness-replayable reference; D4 policy resolver via `R4Engine` | certifier-instrument |
| Gate C harness | `uor-r4-graph-certify::score` | offline measurement | certifier-instrument |
| Legacy | `GraphScorer::score_candidates_legacy` | retired Σ-over-cloud formula | reference-only (retired) |
| Geometric router | `uor-r4-router` | exploratory f64 retrieval | off-serving |

The drift ADR-0001 removes: on the served path the token is chosen by `R4G1Runtime` while the
serve/abstain decision is chosen by `R4Engine`/`GraphScorer` on the same position — two
implementations, not exactly equivalent (the Cayley–Dickson `syntactic_morphism_score` term
runs only in the `R4G1Runtime` multi-candidate slice; see
`crates/uor-r4-graph-certify/tests/r4g1_cd_ab.rs`).

## Machine-checked evidence

`crates/uor-r4-graph-certify/tests/normative_scorer_831.rs` (default `cargo test` suite):

1. **Differential — spec vs deployed accumulator.** For duplicate-free residual sets
   (including saturation cases), the normative spec `ScoreAccumulator::accumulate`
   (graph-format) and the deployed runtime `scoring::accumulate_reference` (graph-runtime)
   produce identical totals. Two independent implementations in two crates agree on the
   normative accumulation semantics.
2. **Differential — spec vs deployed selector.** The runtime `scoring::select_best` winner
   matches the normative `ScoreAccumulator::compare_candidates` order (ScoreQ descending,
   id ascending), including exact ties.
3. **No-double-counting, per implementation.** The spec accumulator ignores a repeated
   contribution id (score unchanged); the runtime `accumulate_reference` rejects a set with a
   repeated evidence id (`None`). Both honor count-once.
4. **Planted negative (has teeth).** A divergent selector (higher-id-wins) and a
   non-saturating (wrapping) accumulator each **disagree** with the normative spec on a
   constructed case — proving the differential would catch a real scorer that violated the
   semantics.
5. **Single-source status space.** `uor-r4-api::ResolutionStatus` is the same type as
   `ScoreStatus`; `ScoringSemanticsVerifier::version() == 1.0.0` and
   `audit_scoring_compliance()` reports no violation.
6. **Reachability + fail-closed.** Both the deployed `R4G1Runtime::parse` and the reference
   `GraphScorer::from_artifact` are reachable from the same synthetic R4G1 bytes; both reject
   non-artifact bytes, and `R4G1Runtime::try_push_patch` rejects incompatible patch bytes
   (fail closed).

## Acceptance-criteria status

- [x] Exactly one implementation/specification named normative for deployed inference — the
  deployed `R4G1Runtime` scoring path, specified by `scoring_semantics` v1.0.0 (ADR §Decision.1).
- [x] All production entry points and certificates demonstrate reachability to that semantics
  — ADR §Reachability; test items 6.
- [x] Reference/certifier-only implementations explicitly scoped and differentially tested —
  ADR §Decision.2 table; test items 1–4.
- [x] Patch and witness semantics have one normative owner and incompatible artifacts fail
  closed — ADR §Decision.1 (patch/witness owner); test item 6.
- [x] A planted semantic divergence fails the relevant test — test item 4.

## Conformance / documentation reconciliation

- `model/ids.toml` RF-23 evidence extended to cite ADR-0001; `CONFORMANCE.md` regenerated
  (generated file, not hand-edited).
- `docs/RESEARCH.md` and `ROADMAP.md` note the S0 normative-scorer designation and link the ADR.
- Per-token CID-bound attribution **suites** remain scoped to #832 (documented exception).

## Append-only correction — 2026-08-24 (#933)

The ownership decision and the accumulator/selector differential evidence above
remain established. The broader reachability sentence in acceptance item 2 did
not remain true after #910: SKMX/PSIB was consumed by `R4Engine`, not by the
normative `R4G1Runtime`, and production decode surfaces did not share one
candidate list. That is an implementation/evidence-boundary failure, not a
superseding scorer decision.

Current status: ADR-0001 is reaffirmed; #821 is reopened with #933; RF-31 is
NOT ESTABLISHED at normative deployed-serving scope. The missing evidence is
exact `R4G1Runtime` SKMX/PSIB reachability plus unified production decode and a
CID-bound deployed-quality census. The planted spec-vs-accumulator negative in
this record remains valid structural evidence but cannot substitute for that
serving-path proof.

## Append-only resolution — 2026-08-25 (#933)

The missing serving-path evidence named in the preceding correction is now
present for one exact canonical bundle. `R4G1Runtime` is the sole candidate and
token selector across the schema-2 production envelope, and its greedy-decode
full census records **21,293 / 72,130 (29.5203%)** versus same-position TLA
**20,284 / 72,130 (28.1214%)**, paired **+13.988 permille, 95% CI [11.057,
16.919]**. The sections-absent control records **18,806 / 72,130 (26.0723%)**,
for **+34.479 permille [31.681, 37.277]** attributable to the bounded lane.

The full report records zero mismatch across 72,138 surface checks, zero
mismatch across 72,130 absent-section identity checks, and 64 / 64 witness
replays. Its CID is
`88ee8210e1f4c48dc26999f5685350b2d2343676cdbd6f9b1aee7c7f1c66146f`;
the graph CID is
`ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`.
After hardening, release-manifest raw BLAKE3
`c2025e9e507e8367993d78bd83ef099ce5851c838d3cc5cf01eda5560986ad33`
(SHA-256
`7572e07a1e3722f3ffc0ea749a67b4ac162221de79b5b4b8a315f4e4e6570fde`)
binds comparator-store CID
`c1749e62077758c4a098e2a02150b5455e1ca3c02c60b87e6d45fcbb9e2b4404`,
and strict production admission passed from an empty model store.

**Resolution: RF-31 RATIFY at this exact CID-bound bundle, held-out population,
`R4G1Runtime`, greedy-decode, schema-2 scope.** This closes the reachability
exception without broadening ADR-0001 into a universal quality claim. It does
not establish a 30% absolute floor, live-teacher parity, free-running quality,
instruction following, reasoning, factuality, or semantic abstention. The BDD
suite passed 124 / 124, but live-teacher parity fixtures were absent and those
scenarios vacuously skipped.
