# #838 — semantics/#823-A: separate coverage novelty, answerability, contradiction, and calibrated confidence

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:36:31Z
- **Updated:** 2026-08-21T06:51:38Z
- **Closed:** 2026-08-21T06:49:41Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S2 — Selective prediction
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/838
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #823

## Context

#811 established parity for the D4 resolution policy, but all five semantic-OOD probes remained SERVABLE. D4 currently says something about representation/resolution coverage, not whether an answer is supported.

## Problem

Covered, novel, unsupported, contradictory, low-confidence, declined, and answered are not yet separate typed concepts. The five-probe canary cannot support a risk claim or distinguish false answers from useful service on answerable novelty.

## Goal

Freeze a typed selective-prediction contract and a powered, CID-bound answerability benchmark before fitting any threshold.

## Scope

- [ ] Define covered, distributionally novel, insufficient evidence, conflicting evidence, supported answer, low confidence, abstention, and hard incompatibility as separate statuses.
- [ ] State which statuses are format/runtime guarantees, assumptions, or empirical criteria under the formal vocabulary.
- [ ] Define evidence/provenance and confidence fields carried by the witness and production response.
- [ ] Specify deterministic mapping to CLI, native HTTP/API, OpenAI-compatible non-streaming/streaming responses, and supported WASM hosts.
- [ ] Specify backward-compatible semantics for artifacts without calibration data and fail-closed behavior for corrupt/incompatible data.
- [ ] Build categories for in-domain answerable, paraphrased answerable, novel-but-supported, missing evidence, private information, false premise, contradictory evidence, and unrelated OOD.
- [ ] Provide gold answerability/evidence annotations and document/domain/entity/template-disjoint partitions.
- [ ] Include always-serve, always-decline, current D4, distance-only, evidence-count-only, and trivial-prior baselines.
- [ ] Predeclare risk-coverage, false-answer, false-abstain, calibration, and category-confusion metrics with powered sample sizes and upper-confidence-bound targets.
- [ ] Freeze separate research and release operating points and bind generator/data/rubric/report identities.

## Non-goals

- Treat code distance as factual truth.
- Use five hand-written probes as a release benchmark.
- Hide abstention as an empty successful completion or generic server error.

## Dependencies and entry gate

Uses #830 status/scope semantics, #831 production reachability, #832 report constitution, and #833 canonical artifact. Draft taxonomy/generator work may proceed earlier.

## Acceptance criteria

- [ ] Every outcome has one typed meaning and deterministic cross-surface representation.
- [ ] The benchmark has sufficient n for the declared false-answer upper bound and retains category-level power.
- [ ] Answerable novelty and semantic unanswerability are explicitly separable.
- [ ] Baselines and controls are non-degenerate and the operating points are frozen before fitting.
- [ ] All data, annotations, generators, splits, rubrics, and reports are CID-bound and replayable.

## Verification

- [ ] Schema/status round-trip tests across CLI/API/server/WASM representations.
- [ ] Planted false-premise, contradiction, answerable-novel, missing-evidence, and corrupt-calibration fixtures.
- [ ] Power-analysis reproduction and split/leakage/tamper tests.
- [ ] Always-serve/always-decline/D4 baselines produce distinct expected confusion profiles.
- [ ] Claim-wording and conformance tests prevent a coverage result from being rendered as semantic PASS.

## Compatibility and migration

Add typed fields additively where protocols permit. Define legacy-client behavior explicitly; old artifacts may remain always-serve/current-D4 but cannot inherit a calibrated-answerability claim.

## Repository conformance

- **Execution scope:** Semantic contract, offline benchmark, and production response schema; no predictor is yet deployed; off-scope evidence is not a serving result.
- Preserve applicable P-4, zero-allocation, deterministic-byte, bounded-work/state, typed-error, no_std, and witness contracts.
- Empirical claims name distribution, protocol, n, uncertainty, provenance, operating point, and PASS/FAIL/UNAVAILABLE status.
- **Conformance mapping:** RF-01, RF-22, RF-23 and RF-29 for fixture-status discipline.. Add a new selective-prediction ID only when the typed capability is actually built.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- Generated conformance is never hand-edited; rejected/off-serving arms remain scoped in the ledger.
- Claim changes append an evidence record and reconcile every live public assertion.

## Claim status and next action

Completion freezes meanings and measurement. It explicitly records current semantic abstention as NOT ESTABLISHED.

## Comments (1)

### comment 5366227742 — 2026-08-21T06:51:38Z — Casey-allard

### Definition-of-Done evidence (issue closed via PR #892, merge `61501c8f`)

- **Merged deliverables (main `61501c8f`):** `docs/selective_prediction_spec_838.md` (the frozen typed contract + benchmark constitution), `crates/uor-r4-api/tests/selective_prediction_spec_838.rs` (executable reference model, 15 machine-checked properties, all green), `docs/RESEARCH.md` S2 item-A summary.
- **Contract frozen:** eight typed statuses with one meaning and one canonical label each; coverage/evidence/confidence axes + a total deterministic outcome function with fixed cause precedence; deterministic and injective representation across CLI, native HTTP, OpenAI-compatible non-streaming AND streaming, and WASM (abstention is never an empty success, an empty-`choices` completion, a silent stream end, or a generic error); integer evidence/provenance/confidence fields with witness parity; legacy coverage-only mode for artifacts without calibration data (no fabricated confidence, no inherited calibrated claim); corrupt calibration fails closed as `hard-incompatibility` and never degrades to legacy.
- **Benchmark constitution frozen (before any fitting):** 8 gold-annotated categories with document/domain/entity/template-disjoint partitions and CID-bound annotations; 6 baselines with distinct planted confusion signatures (current-D4 falsely serves false-premise / contradictory-evidence / missing-evidence — the #811 finding as a fixture; distance-only conflates the two novelty categories and is paraphrase-brittle; evidence-count-only serves conflicting evidence); pre-registered metrics (risk-coverage, false-answer, false-abstain, calibration, category-confusion); powered n = 600/category (4,800 total; 3,000 unanswerable → zero-failure UCB95 ≈ 1‰); frozen operating-point selection rules and targets (false-answer UCB95 ≤ 10‰ release / ≤ 50‰ research; false-abstain ≤ 200‰ / ≤ 300‰).
- **Key separations, machine-checked:** answerable novelty (`novel-but-supported`) is served, covered-but-unsupported abstains (`answerable_novelty_is_separable_from_unanswerability`); a coverage-only report can never render a semantic PASS (`coverage_result_cannot_render_semantic_pass`).
- **Verification:** issue items V1–V5 each map to named green tests (see PR body); local gates all green before push (fmt, claim-wording, clippy -D warnings workspace/all-targets/all-features, no_std ×2, wasm lib, xtask validate R1+R4+R5, repo-conformance R2+R3, cargo-deny bans R6, doc tests, workspace suite with 0 failures across every binary; the local-model bdd soak is CI-skipped by design); merge queue full ladder green in one pass.
- **Claim status recorded:** **current semantic abstention is NOT ESTABLISHED.** No ids.toml row, no CONFORMANCE regeneration (spec-leaf). Next: #837 (now unblocked) fits the artifact-only calibrator against this constitution; #839 executes/certifies across production surfaces.



---
_Comment/review ids in this file: 5366227742_

