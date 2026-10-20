# #837 — research/#823-B: fit an artifact-only confidence calibrator against powered selective-risk controls

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:36:30Z
- **Updated:** 2026-08-21T07:44:35Z
- **Closed:** 2026-08-21T07:42:09Z
- **Milestone:** R4 Intelligence / S2 — Selective prediction
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/837
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #823

## Context

The current D4 signature mechanism did not abstain on any of five semantic-OOD probes. Useful evidence already available in artifacts may include candidate margins, support/provenance counts, route agreement, novelty, and contradiction signals.

## Problem

It is unknown whether these artifact-local signals calibrate answerability across documents/domains or merely re-express coverage density. Retrofitting a threshold after inspecting the test set would make the result uninterpretable.

## Goal

Compare simple and richer compiler-fit calibrators under equal deployed budgets, lower the best eligible candidate to deterministic fixed-point/table semantics, and select it only on a frozen selective-risk gate.

## Scope

- [ ] Partition documents/domains into fit, calibration, and untouched test sets; freeze sibling item A before fitting.
- [ ] Evaluate candidate margin, entropy proxy, evidence/support counts, provenance diversity, route/scale agreement, D4 novelty, contradiction indicators, and combinations.
- [ ] Compare monotone threshold/table models with richer compiler-side models under equal bytes, table reads, and operations.
- [ ] Include always-serve, always-decline, current D4, distance-only, count-only, shuffled-label, shuffled-feature, and constant-score controls.
- [ ] Choose thresholds only on the calibration split; never retune on held-out domains.
- [ ] Specify deterministic quantization, saturation, tie behavior, and missing-feature handling for each eligible model.
- [ ] Report risk-coverage curves, category confusion, calibration error, answerable-novel retention, transfer, ablations, bytes, and projected latency.
- [ ] Select at most one candidate for production integration; record every negative arm.

## Non-goals

- Call representation density answerability without risk evidence.
- Use a teacher, remote model, clock, or network at inference.
- Optimize aggregate accuracy while hiding false-answer or false-abstain categories.

## Dependencies and entry gate

Blocked on the frozen semantics/benchmark in sibling item A and the #833 baseline. Requires exact production-path features attributed by #831/#832.

## Acceptance criteria

- [ ] Fit/calibration/test identities are disjoint and frozen.
- [ ] All simple/rich arms have equal-budget comparisons and ablations.
- [ ] The selected model has a deterministic bounded integer/table reference form.
- [ ] Selection uses the predeclared untouched-domain risk/coverage rule; no candidate means `NO CALIBRATOR ESTABLISHED`.
- [ ] Raw scores and decision curves reproduce from the pinned record.

## Verification

- [ ] Small fixture verifies feature extraction, quantization, saturation, thresholding, and shuffled controls.
- [ ] Power and interval calculations reproduce independently.
- [ ] Float/compiler model versus fixed-point reference differential tests quantify any decision changes.
- [ ] Planted constant-score, label-leakage, missing-feature, and inverted-threshold models fail.
- [ ] Deterministic shard ordering/double-run check.

## Run contract

- **Metric/current value:** False-answer upper confidence bound at frozen useful coverage, with answerable-novel false abstention and per-category risk secondary; current semantic-OOD result is 0/5 abstained (all SERVABLE), which is a diagnostic only.
- **Reachability ceiling:** Compute the fraction of errors with observable artifact evidence and the fraction of serving steps exposing each feature; a feature cannot fix cases it cannot observe. Publish this ceiling before fitting.
- **Pinned identities:** Benchmark/generator/annotations/splits, #833 artifact, normative scorer/feature extractor, calibration model/config, quantization, seeds, and raw/report CIDs.
- **Nulls/falsifier:** Always serve/decline, current D4, distance-only, count-only, shuffled labels/features, constant score, and held-out domain transfer.
- **Binding cheap instrument:** Feature-availability/variance table, non-degenerate baseline curves, leakage check, and integer-lowering differential on a small fixture must pass; reachable ceiling must exceed the effect floor.
- **Exit rule:** Select only if the untouched-domain false-answer upper bound clears the frozen operating point while useful coverage and answerable-novel retention stay above their floors after integer lowering.
- **If positive:** Activate sibling item C for packed production integration without threshold retuning.
- **If negative:** Retain D4 as coverage-only, publish the failure, and redesign evidence acquisition rather than shipping semantic abstention.
- **Cost estimate:** Post wall time, peak RSS/storage, feature-extraction cost, model bytes/operations, and shared evaluation resources.

Do not launch the full run until exact values are posted and the cheap instrument passes. Missing data is `UNAVAILABLE`/`null`, never zero or PASS; controls must be non-degenerate.

## Compatibility and migration

All fitted outputs are experimental and identified separately. Production loaders reject them until sibling item C defines the validated optional section and admission rules.

## Repository conformance

- **Execution scope:** Offline compiler/certifier models and deterministic fixed-point reference lowering; production path is baseline only; off-scope evidence is not a serving result.
- Preserve applicable P-4, zero-allocation, deterministic-byte, bounded-work/state, typed-error, no_std, and witness contracts.
- Empirical claims name distribution, protocol, n, uncertainty, provenance, operating point, and PASS/FAIL/UNAVAILABLE status.
- **Conformance mapping:** RF-01, RF-22, RF-23; a new selective-prediction capability is justified only after a positive candidate exists.. Add a new selective-prediction ID only when the typed capability is actually built.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- Generated conformance is never hand-edited; rejected/off-serving arms remain scoped in the ledger.
- Claim changes append an evidence record and reconcile every live public assertion.

## Claim status and next action

A fitted curve is not a serving claim. Only the untouched-domain, fixed-point, production-path result may trigger promotion.

## Comments (3)

### comment 5366377094 — 2026-08-21T07:06:45Z — Casey-allard

### Run contract — artifact-only confidence-calibrator fit (pre-registered before the full run)

Executing this issue under the #838 frozen constitution. Harness `crates/uor-r4-api/tests/calibrator_fit_837.rs` at branch `issue-837-calibrator-fit` (off `61501c8f`), fixture teeth green (5/5), clippy clean. The binding cheap instrument (`calibrator_instrument_837`, FIT partition only — test untouched) has RUN and PASSED; its exact values are below.

- **metric to move:** false-answer UCB95 (‰, frozen #838 reference arithmetic `(1000k+3000)/n` ceil) among served, at frozen useful coverage, on the untouched test partition — teacher-grounded (served-and-wrong vs the recorded #833 teacher argmax). Secondary: answerable-novel retention, per-arm curves. Current value: none exists (D4 is coverage-only; the 0/5 semantic-OOD abstention result is a diagnostic).
- **execution scope:** offline compiler/certifier reference (#830) — the reference predictor is the #875/#834 suffix baseline (harness must reproduce its 246.6‰ before any reading; it did in the instrument run), NOT the deployed engine (which exposes no candidate scores; production integration is #839). No teacher/remote/clock/network at inference: every feature is an integer artifact-table read.
- **partitions (document-disjoint by story, story % 3, disjointness asserted):** fit 24,064 / calibration 24,232 / test 23,834 positions (199/200/200 stories). Thresholds are chosen ONLY on calibration; the test partition is evaluated exactly once for the selected candidate. corpus_meta_cid `blake3:aa9d1767…`.
- **reachability ceiling (published before fitting, from the instrument):** 17,882 of 18,205 fit-partition baseline errors (**982‰**) expose a low-confidence signal (absent key | margin<500‰ | no content agreement) — an artifact-only calibrator can observe nearly every error, so the ceiling clears any plausible effect floor. Feature exposure (fraction of serving steps exposing each feature): suffix-key present 634‰, margin>0 542‰, content-agreement 214‰, support>0 578‰, disagree>0 998‰ — all non-degenerate (asserted).
- **arms (candidates, pre-declared budget order — simpler wins ties):** `margin-threshold` (4B, 1 read) → `top1rate-threshold` (4B, 1 read) → `bucket-table` (16×10 integer precision table fitted on the FIT partition only; 644B, 2 features + 1 table read) → `rich-combo` (fixed pre-declared integer weights over 6 features; 28B). All integer/fixed-point native — the deployed lowering is the model itself; the f64-shadow differential is quantified (fixture proves the instrument detects drift; planted 0.29-weight case counts >0 changes; integer-native arms count 0).
- **controls (never selectable):** always-serve/always-decline (curve endpoints), current-D4 (`present`, the novelty-only policy), distance-only (branching-factor analog), count-only, constant-score (single-point, planted fail), inverted-margin (planted fail), shuffled-label (bucket table refit on LCG-permuted labels — must not qualify, asserted), shuffled-feature (fixture-asserted). Non-degeneracy asserted throughout.
- **frozen gates (from #838 §9, unchanged):** RELEASE: UCB95 ≤ **10‰** at coverage ≥ **20‰**. RESEARCH: UCB95 ≤ **50‰** at coverage ≥ **50‰**. Selection rule: maximize calibration-partition coverage subject to the bound; ties to larger θ, then the simpler arm. **Exit rule:** SELECT iff the selected (arm, θ) — frozen on calibration — confirms on the single untouched-test evaluation at the RELEASE gate. Research-only qualification = recorded, but verdict stays NO CALIBRATOR ESTABLISHED for production (the release bar stands, #887 calibration). Nothing qualifying = NO CALIBRATOR ESTABLISHED.
- **if positive (SELECT):** activate #839 for packed production integration WITHOUT threshold retuning; close #837 with the record.
- **if negative:** retain D4 as coverage-only, publish the falsifier set (all arms + controls + curves), close #837 with `NO CALIBRATOR ESTABLISHED`; evidence-acquisition redesign (e.g., content-side features the suffix tables cannot see — the answerable-novel retention metric quantifies exactly what is being discarded) is the recorded next direction, not threshold tuning.
- **cost estimate:** single-process offline, ~1–3 min wall (instrument ran 1.16 s + table build), peak RSS ≲ 3 GB, 0 teacher evaluations, artifacts: 1 CID-bound JSON + 1 record doc; blocks nothing.

Launching the full run now; the outcome will be posted against this contract.


### comment 5366424280 — 2026-08-21T07:11:33Z — Casey-allard

### Outcome against the run contract — **`NO CALIBRATOR ESTABLISHED`** (no arm met either frozen gate)

The full run executed as contracted (`calibrator_fit_run_837`, ~1 s after table build; record `docs/calibrator_837_result.json`, write-up `docs/selective_calibration_837.md`). Base predictor reproduced 246.6‰ exactly; partitions fit 24,064 / cal 24,232 / test 23,834 (199/200/200 stories, disjointness asserted); shuffled-label null failed both gates (asserted). Identities: corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:1d1504aa…`.

| arm (budget) | release (UCB≤10‰ @cov≥20‰) | research (UCB≤50‰ @cov≥50‰) | curve highlights (err‰ @ cov‰) |
|---|---|---|---|
| margin-threshold (4B/1r) | — | — | **520 @ 10** · 431 @ 200 |
| top1rate-threshold (4B/1r) | — | — | 520 @ 10 · 440 @ 200 |
| **bucket-table** (644B/2f+1t) | — | — | **11 @ 10 (UCB 23)** · 34 @ 20 (UCB 41) · 125 @ 50 |
| rich-combo (28B/6f) | — | — | 250 @ 10 · 327 @ 50 |
| current-D4 / distance-only / count-only | — | — | 644 flat / 520 @ 10 / 492 @ 10 |
| constant-score / inverted-margin (planted) | — | — | fail as planted (single-point / 925 @ 10) |

**Verdict per the pre-registered rule: `NO CALIBRATOR ESTABLISHED`.** No candidate qualified on the calibration partition at either gate, so the untouched test partition was never consumed and **#839 is not activated**; the deployed D4 policy stays coverage-only and semantic abstention remains NOT ESTABLISHED.

**Three findings worth keeping (all in the record):**
1. **Artifact signals do rank confidence.** The fitted support×margin bucket table's risk–coverage curve is cleanly monotone (11‰ error at 10‰ coverage → 752‰ at full): a thin ~99%-precision memorized slice exists (~1% of positions) — but it is too thin for the frozen operating points (UCB 41‰ vs required 10‰ at the release floor; 125‰ vs 50‰ at the research floor).
2. **Raw margin is singleton-polluted.** `margin-threshold`'s most-confident slice is **520‰ wrong**: margin = 1000‰ is dominated by total=1 suffix keys. Support-conditioning is what turns margin into signal — a concrete caution against any future threshold-on-margin shortcut, and the reason distance/count-class signals cannot substitute for a fitted calibrator.
3. **Answerable novelty is structurally discarded.** 2,454 of 26,002 novel-suffix positions are content-answerable, and every suffix-feature calibrator serves 0 of them (margin = 0 on absent keys). Read with #834 §6.3: the redesign direction the negative branch names is **evidence acquisition** (content-side features), not threshold tuning; a redesigned feature set re-enters under the same frozen #838 gates.

Closing this issue with the record per the contract's negative branch. PR follows.


### comment 5366835121 — 2026-08-21T07:44:35Z — Casey-allard

### Definition-of-Done evidence (issue closed via PR #893, merge `d1215f55`)

- **Verdict: `NO CALIBRATOR ESTABLISHED`** under the pre-registered rule — no candidate arm met the frozen release gate (false-answer UCB95 ≤ 10‰ at coverage ≥ 20‰) or the research gate (≤ 50‰ at ≥ 50‰) on the calibration partition; the untouched test partition was never consumed; **#839 is not activated**; the deployed D4 policy stays coverage-only; semantic abstention remains NOT ESTABLISHED.
- **Merged deliverables (main `d1215f55`):** `crates/uor-r4-api/tests/calibrator_fit_837.rs` (fixture teeth + instrument + study), `docs/calibrator_837_result.json` (CID-bound: per-arm gates, risk–coverage curves, budgets; corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:1d1504aa…`), `docs/selective_calibration_837.md` (the record), `docs/RESEARCH.md` S2 item-B paragraph.
- **Process:** binding cheap instrument run FIRST and PASSED (feature exposure non-degenerate; observability ceiling 982‰ of fit-partition errors); run contract with those exact values posted BEFORE the full run; outcome posted against it; base predictor reproduced 246.6‰ exactly; partitions story-disjoint (asserted); shuffled-label null and every planted negative failed as required; thresholds touched only the calibration partition.
- **Findings recorded:** the fitted support×margin bucket table ranks confidence cleanly (monotone curve; 11‰ error at 10‰ coverage — a thin ~99%-precision memorized slice, too thin for the gates); raw margin is singleton-polluted (520‰ error in its most-confident slice; support-conditioning is what makes margin a signal); 2,454 content-answerable novel positions (94‰ of novelty) are structurally discarded by suffix features — the recorded redesign direction is evidence acquisition (content-side features), re-entering under the same frozen #838 gates.
- **Verification:** local gates all green before push (fmt / claim-wording / clippy -D warnings workspace / no_std ×2 / wasm / xtask validate / repo-conformance / deny bans / doc tests / workspace suite 0 failures; model-driven bdd soak CI-skipped by design); merge queue full ladder green in one pass. UNAVAILABLE items recorded truthfully (domain transfer on a single-domain corpus; #838 category metrics pending the gold benchmark dataset).



---
_Comment/review ids in this file: 5366377094, 5366424280, 5366835121_

