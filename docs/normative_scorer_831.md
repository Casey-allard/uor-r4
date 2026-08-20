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
