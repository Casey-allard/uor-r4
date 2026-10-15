# #823 — TRACKER/S2: evidence-grounded selective prediction and calibrated abstention

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:32:20Z
- **Updated:** 2026-08-25T19:23:45Z
- **Closed:** 2026-08-25T19:23:08Z
- **Milestone:** R4 Intelligence / S2 — Selective prediction
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/823
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent programme: #820

## Objective

Separate representation coverage from answerability and add a calibrated, artifact-only decision that declines unsupported responses at an explicit risk/coverage operating point.

## Current evidence

- #811 showed D4 runtime parity but 0/5 semantic-OOD abstentions; all probes were SERVABLE.
- Five probes are neither powered nor category-balanced enough to support a semantic safety claim.
- Distance or signature novelty alone cannot distinguish answerable novelty, missing evidence, false premises, and contradiction.

## Entry criteria

S0 has frozen serving-path attribution, benchmark identity rules, and non-vacuous empirical verdict semantics.

## Native dependencies

Blocked by #821. It may proceed in parallel with #822 after S0 closes.

## Promotion gate

A fixed threshold achieves the predeclared false-answer upper confidence bound at useful coverage on held-out domains while preserving answerable novelty, and the same typed result is returned across production surfaces.

## Kill / redesign criterion

If calibration fails document/domain transfer or useful coverage collapses, retain D4 only as a representation-coverage indicator and explicitly decline the semantic-answerability claim.

## Native children

- [ ] #838 — Freeze typed selective-prediction semantics and powered benchmark
- [ ] #837 — Fit an artifact-only confidence calibrator
- [ ] #839 — Execute and certify selective prediction across production surfaces

GitHub's native parent/sub-issue relationship is the hierarchy source of truth.

## Non-goals

- Claim factual truth from code-space distance alone.
- Use a network or live teacher fallback at inference.

## Repository conformance

Primary IDs: RF-01, RF-22, RF-23; add a new capability ID only when a typed selective predictor is built. Missing calibration evidence is UNAVAILABLE, not zero or PASS.

Every leaf declares execution scope, preserves the applicable P-4/determinism/allocation/no_std/error/witness contracts, maps to existing RF capabilities or justifies a new one, and updates generated conformance only through its TOML/Gherkin/test sources. Empirical work posts the repository run contract before expensive execution.

## Closure rule

Close only after every native child has a completed, negative, not-triggered, or explicitly descoped verdict and this tracker records `PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE` with exact evidence links.

## Comments (4)

### comment 5366227879 — 2026-08-21T06:51:39Z — Casey-allard

### S2 item A complete — #838 closed (PR #892, merge `61501c8f`)

The stage's lettered-first child is done: the typed selective-prediction contract and the `s2-answerability-ood` benchmark constitution are frozen before any threshold fitting, and **current semantic abstention is explicitly recorded as NOT ESTABLISHED** (`docs/selective_prediction_spec_838.md` + `crates/uor-r4-api/tests/selective_prediction_spec_838.rs`, 15 machine-checked properties).

S2 chain state:

- **A #838:** CLOSED — contract + constitution frozen (statuses, cross-surface mappings, fail-closed calibration, categories, baselines, powered n, frozen research/release operating-point rules and UCB targets).
- **B #837:** now UNBLOCKED (its native blocker was #838) — fit the artifact-only confidence calibrator against the frozen constitution: the §8 baselines, §9 targets (false-answer UCB95 ≤ 50‰ research / ≤ 10‰ release), calibration-partition-only fitting, one evaluation pass on the frozen eval partition.
- **C #839:** blocked on #837 — execute and certify the typed contract across CLI / native HTTP / OpenAI-compatible / WASM production surfaces.


### comment 5366835367 — 2026-08-21T07:44:36Z — Casey-allard

### S2 item B complete — #837 closed with `NO CALIBRATOR ESTABLISHED` (PR #893, merge `d1215f55`)

S2 child state after tonight:

- **A #838:** CLOSED — typed contract + benchmark constitution frozen; semantic abstention recorded NOT ESTABLISHED (PR #892 / `61501c8f`).
- **B #837:** CLOSED — the calibrator study ran under the frozen gates and returned the pre-registered negative: no arm met release (UCB95 ≤ 10‰ @ ≥ 20‰ coverage) or research (≤ 50‰ @ ≥ 50‰) on the calibration partition; test partition untouched. Evidence-backed findings: artifact signals rank confidence (bucket-table curve, 11‰ err @ 10‰ cov) but the memorized slice is too thin; raw margin is singleton-polluted (520‰ err in its top slice); 2,454 content-answerable novel positions are structurally discarded by suffix features. (PR #893; records `docs/selective_calibration_837.md` + `docs/calibrator_837_result.json`.)
- **C #839:** native blocker (#837) is closed, but per #837's pre-registered exit rule the **activation condition did not occur** — there is no selected calibrator to integrate, so #839's required input does not exist and it is **not executable**. It stays open awaiting either a maintainer decision (retire / re-scope) or a positive re-entry after evidence-acquisition redesign (content-side features) under the same frozen #838 gates.

**The S2 stage verdict against the full child set is the maintainer's call on this tracker.** The evidence set for it is complete: contract frozen, calibrator study negative with the falsifier set and the redesign direction quantified. Downstream, S3 (#824) entry is gated on the S1 (#822) stage verdict, which is likewise pending as a maintainer call — the roadmap's executable frontier now waits on those two stage verdicts (plus the parallel F0 #859 Lean lane).


### comment 5368815388 — 2026-08-21T10:42:26Z — Casey-allard

### Maintainer stage verdict (2026-08-21): **REVISE** — semantics frozen and honest; calibration returns for one redesigned re-entry under the same gates

Recorded on the maintainer's decision this morning:

- **Item A (#838):** the typed selective-prediction contract and the `s2-answerability-ood` benchmark constitution are FROZEN; current semantic abstention is recorded **NOT ESTABLISHED**.
- **Item B (#837):** `NO CALIBRATOR ESTABLISHED` — no arm met the release (false-answer UCB95 ≤ 10‰ @ coverage ≥ 20‰) or research (≤ 50‰ @ ≥ 50‰) gate on the calibration partition. The diagnosis is **evidence acquisition, not thresholds**: the fitted bucket table ranks confidence cleanly but its memorized slice is too thin; raw margin is singleton-polluted; 2,454 content-answerable novel positions are structurally invisible to suffix features.
- **Item C (#839):** the activation condition did not occur; per the maintainer re-scope recorded on its thread this morning, it now executes the typed contract in **legacy-coverage mode** (phase 1, executable now), with calibrated mode behind the unchanged release gate (phase 2).

**Verdict meaning:** the stage neither passes nor is limited/retired. One redesigned calibrator re-entry — content-side evidence features, aligned with the S1 (#822) representation-redesign direction — is sanctioned under the **SAME frozen gates**; the bars do not move (the #887 discipline). Stage closure is re-evaluated after that re-entry, or after phase-1 #839 certification plus an explicit maintainer LIMIT call, whichever the maintainer takes first.


### comment 5415555367 — 2026-08-25T19:23:41Z — Casey-allard

### Definition of Done — S2 closed `LIMIT`

PR #939 merged through the protected queue as `681573a94f9b1ee987c6167c725f8d306e33919d`; the final verdict is now mirrored in `docs/r4_intelligence_completion_plan.md` and `docs/RESEARCH.md`.

- **#838: CLOSED** — typed contract and powered benchmark constitution frozen.
- **#837: CLOSED / `NO CALIBRATOR ESTABLISHED`** — original artifact-only fit did not clear the frozen gates.
- **#931: CLOSED / `NO CALIBRATOR ESTABLISHED`** — the one sanctioned RF-31 content-evidence re-entry also failed both gates on CAL; TEST remained sealed; result CID `blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`.
- **#839: CLOSED** — RF-30 legacy-coverage phase delivered through PR #900; calibrated phase `NOT TRIGGERED`.

**Final S2 disposition: `LIMIT`.** Deterministic typed coverage/decline/incompatibility outcomes are deployed, and unsupported/present calibration fails closed. Calibrated confidence, semantic answerability, and semantic abstention remain **NOT ESTABLISHED**. This is not a promotion and does not authorize another fit or move the frozen #838 gates.

Verification: proportional local docs checks passed (`git diff --check`; claim-wording gate). Protected merge-group run #32886870431 passed the binding ladder: workspace nextest, BDD, doc tests, fmt/clippy, no-std, deterministic rebuild, κ reproduction, Gate C tracking/alarm, all-features, WASM, fuzz, audit, and conformance.

PR: https://github.com/UOR-Foundation/uor-r4/pull/939
Queue evidence: https://github.com/UOR-Foundation/uor-r4/actions/runs/32886870431


---
_Comment/review ids in this file: 5366227879, 5366835367, 5368815388, 5415555367_

