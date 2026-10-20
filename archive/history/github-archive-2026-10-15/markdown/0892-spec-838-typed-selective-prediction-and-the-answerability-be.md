# #892 — spec(#838): typed selective prediction and the answerability benchmark constitution (S2 item A)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T06:31:05Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-21T06:49:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/892
- **Merged:** 2026-08-21T06:49:39Z
- **Merge commit:** 61501c8fb7cf24bdf37cc3d274dd30e34bddaabb
- **Base / Head:** main / UOR-Foundation:issue-838-selective-prediction-spec
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

#811 established D4 parity while all five semantic-OOD probes stayed SERVABLE: the deployed axis measures representation/resolution *coverage*, not whether an answer is *supported*. Covered, novel, unsupported, contradictory, low-confidence, declined, and answered were not separate typed concepts, and the five hand-written probes cannot support a risk claim. This PR freezes the S2 item-A contract: a typed selective-prediction status space with deterministic cross-surface representation, and a powered, CID-bound answerability benchmark constitution — **before any threshold is fitted**. It explicitly records **current semantic abstention as NOT ESTABLISHED**.

## Scope

- `docs/selective_prediction_spec_838.md` — the frozen contract + constitution:
  - Eight typed statuses (`covered`, `distributionally-novel`, `insufficient-evidence`, `conflicting-evidence`, `supported-answer`, `low-confidence`, `abstention`, `hard-incompatibility`), composed as coverage/evidence/confidence axes + a total deterministic outcome function with a fixed cause-precedence order. Answerable novelty (`novel-but-supported`) is separable from semantic unanswerability by construction.
  - Formal-vocabulary classification of every status (Guarantee vs Assumption vs Empirical Criterion, per `docs/formal_vocabulary.md`).
  - Evidence/provenance/confidence fields (integer-only) carried identically by witness and production response.
  - Deterministic mapping table for CLI, native HTTP, OpenAI-compatible non-streaming AND streaming, and WASM hosts. An abstention is never an empty success, an empty-`choices` completion, a silent stream end, or a generic server error.
  - Backward compatibility: no-calibration artifacts stay legacy/coverage-only (no fabricated confidence, no calibrated-answerability claim); corrupt/incompatible calibration **fails closed** as `hard-incompatibility` (corrupt never silently degrades to legacy).
  - Benchmark constitution for the frozen `s2-answerability-ood` suite (#832): 8 gold-annotated categories, document/domain/entity/template-disjoint partitions, 6 baselines (always-serve, always-decline, current-D4, distance-only, evidence-count-only, trivial-prior) with distinct expected confusion profiles, pre-declared metrics (risk-coverage, false-answer, false-abstain, calibration, category-confusion), powered n (600/category, 4,800 total) with UCB arithmetic, and frozen research/release operating-point selection rules + targets (false-answer UCB95 ≤ 50‰ / ≤ 10‰).
- `crates/uor-r4-api/tests/selective_prediction_spec_838.rs` — executable reference model: 15 machine-checked properties (totality/determinism, label round-trips, per-surface injectivity, abstention-never-hidden, legacy/fail-closed semantics with planted corrupt-calibration bytes, per-axis leakage + tamper rejection, planted category fixtures classifying per gold, pairwise-distinct baseline signatures incl. the #811 current-D4 false-serve fixture, power/UCB constants fixed before fitting, deterministic operating-point selection, the coverage-cannot-render-semantic-PASS claim gate, witness field parity, double-run/reordered determinism). Binds the #832 vocabulary (`ControlKind`, `MetricStatus`, `compute_cid`/`verify_cid`, `detect_document_leakage`, `is_degenerate_control`).
- `docs/RESEARCH.md` — S2 item-A summary paragraph appended.

## Non-goals

No predictor deployed, no threshold fitted (#837), no serving-surface changes (#839), no `model/ids.toml` row, no `CONFORMANCE.md` regeneration (spec-leaf per the #835/#832 precedent; a selective-prediction capability ID is added only when #839 builds the typed capability, in the required built-capability order). Code distance is not treated as factual truth; the five #811 probes are not a release benchmark.

## Acceptance-criteria status (issue #838)

- Every outcome has one typed meaning and deterministic cross-surface representation — DONE (spec §2/§5; injectivity + round-trip tests).
- Benchmark n sufficient for the declared false-answer upper bound with category-level power — DONE (spec §9: rule-of-three/UCB arithmetic, n=600/category; machine-checked constants).
- Answerable novelty vs semantic unanswerability explicitly separable — DONE (spec §2; `answerable_novelty_is_separable_from_unanswerability`).
- Baselines/controls non-degenerate; operating points frozen before fitting — DONE (spec §8/§9; distinct planted signatures; frozen selection rules + targets; `is_degenerate_control` bound).
- All data/annotations/generators/splits/rubrics/reports CID-bound and replayable — DONE (spec §7/§10; `compute_cid`/`verify_cid` fixtures; #830 UNAVAILABLE-never-PASS rule bound).

## Verification (issue items V1–V5 → named tests)

- V1 schema/status round-trip across CLI/API/server/WASM → `cross_surface_encodings_round_trip_and_are_injective`, `status_space_is_total_and_labels_round_trip`.
- V2 planted false-premise, contradiction, answerable-novel, missing-evidence, corrupt-calibration fixtures → `planted_category_fixtures_classify_per_gold`, `corrupt_calibration_fails_closed`.
- V3 power-analysis reproduction and split/leakage/tamper tests → `power_and_ucb_targets_fixed_before_fitting`, `partitions_reject_leakage_and_tamper`.
- V4 always-serve/always-decline/D4 baselines produce distinct expected confusion profiles → `baselines_produce_distinct_confusion_profiles`.
- V5 claim-wording/conformance tooth against rendering a coverage result as semantic PASS → `coverage_result_cannot_render_semantic_pass` (+ CI claim-wording gate green).

Local gates green before push: fmt --check; claim-wording; clippy -D warnings (workspace, all targets/features); no_std ladder; wasm lib; xtask validate (R1/R4/R5/gnaf); repo-conformance (R2/R3); cargo-deny bans (R6); doc tests; workspace test suite (the model-driven bdd soak is local-only and CI-skipped by design — all other binaries green).

## Compatibility and migration

Typed fields are additive where protocols permit; legacy-client and old-artifact behavior is explicitly defined (legacy-coverage mode); old artifacts cannot inherit a calibrated-answerability claim. No format or runtime bytes change in this PR.

## Conformance effects

None generated. Extends the evidence language of RF-01 / RF-22 / RF-23 / RF-29; no new built RF capability.

## Claim-boundary changes

`docs/RESEARCH.md` now records the frozen S2 contract and that semantic abstention is NOT ESTABLISHED — a narrowing, honesty-increasing change; no capability claim is added.

## Negative or unavailable results

None to report: this issue's deliverable is definitional infrastructure; the NOT-ESTABLISHED record is the required truthful claim status, not a measurement failure.

## Intentionally excluded

Pre-existing untracked user dirs (`obs-text/`, `tf1-pkg/`) are not part of this change and are not staged.

Closes #838.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01J2R2qs9kfcYpsjUua1SYmi


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

