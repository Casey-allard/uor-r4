# #893 — research(#837): artifact-only confidence calibrator — NO CALIBRATOR ESTABLISHED

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T07:20:33Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-21T07:42:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/893
- **Merged:** 2026-08-21T07:42:08Z
- **Merge commit:** d1215f5532696a02912ba7eef38aff31b851866d
- **Base / Head:** main / UOR-Foundation:issue-837-calibrator-fit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Whether artifact-local signals (candidate margins, support/provenance counts, route agreement, novelty, contradiction indicators) calibrate answerability — or merely re-express coverage density — was unknown, and a threshold retrofitted after seeing the test set would be uninterpretable. This PR executes the #837 study under the #838 frozen constitution and records the pre-registered verdict: **`NO CALIBRATOR ESTABLISHED`** — no candidate arm met the frozen release gate (false-answer UCB95 ≤ 10‰ at coverage ≥ 20‰) or the research gate (≤ 50‰ at ≥ 50‰) on the calibration partition. The untouched test partition was never consumed, #839 is not activated, the deployed D4 policy stays coverage-only, and semantic abstention remains NOT ESTABLISHED.

Three substantive findings are recorded: the fitted support×margin bucket table ranks confidence cleanly (monotone risk–coverage; 11‰ error at 10‰ coverage — a thin ~99%-precision memorized slice, too thin for the gates); raw margin is singleton-polluted (its most-confident slice is 520‰ wrong: margin 1000‰ is dominated by total=1 suffix keys — support-conditioning is what makes margin a signal); and 2,454 content-answerable novel positions (94‰ of novelty) are structurally discarded by every suffix-feature calibrator — quantifying the evidence-acquisition redesign the negative branch names.

## Scope

- `crates/uor-r4-api/tests/calibrator_fit_837.rs` — the study harness:
  - Integer artifact-only features (suffix presence/total/margin/top1-rate/branching; content support/disagreement/agreement) over the #833 tables; the #875/#834 suffix baseline as the reference predictor, reproduced to the digit (246.6‰) before any reading.
  - Document-disjoint fit/calibration/test partitions (story % 3; 24,064/24,232/23,834; disjointness asserted). Thresholds chosen ONLY on calibration; test reserved for a single selected-candidate evaluation (not consumed on this negative).
  - Candidate arms under pre-declared budget order: margin-threshold, top1rate-threshold, fitted 16×10 bucket table (644 B), rich-combo (6-feature fixed integer weights). Controls: always-serve/decline endpoints, current-D4, distance-only, count-only, constant-score, inverted-margin, shuffled-label (bucket refit on LCG-permuted labels; asserted non-qualifying), shuffled-feature.
  - Frozen #838 gates and selection rule; UCB95 reference arithmetic `(1000k+3000)/n`; per-arm risk–coverage curve summaries and budget accounting in the record.
  - Non-ignored fixture teeth (CI-run): extraction/quantization/saturation/thresholding; planted constant/inverted/shuffled/leakage negatives (each non-degenerate and failing); integer-vs-f64 differential (detects a planted 0.29-weight drift; 0 changes on the integer-native arms); partition leakage detection; double-run/order-invariance.
  - Ignored, bundle-gated: `calibrator_instrument_837` (the binding cheap instrument — feature exposure/variance + the observability ceiling, run FIRST and posted in the run contract) and `calibrator_fit_run_837` (the full study).
- `docs/calibrator_837_result.json` — CID-bound record (corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:1d1504aa…`): per-arm gates, curves, budgets, partitions, answerable-novel accounting, verdict.
- `docs/selective_calibration_837.md` — the per-issue record (protocol, instrument, results table, findings, controls, claim status).
- `docs/RESEARCH.md` — S2 item-B summary paragraph appended.

## Non-goals

No serving-path changes, no ids.toml row, no CONFORMANCE regeneration (research-leaf). Representation density is not called answerability (the verdict is precisely that it did not qualify); no teacher/remote/clock/network at inference (features are artifact-table reads); no aggregate-accuracy optimization hiding false-answer/false-abstain categories (the record carries the curves and the answerable-novel discard count).

## Acceptance-criteria status (issue #837)

- Fit/calibration/test identities disjoint and frozen — DONE (story-keyed; asserted; #838 constitution frozen first).
- Simple/rich arms with equal-budget comparisons and ablations — DONE (pre-declared budget order + per-arm byte/read accounting; the bucket table vs raw margin IS the support-ablation, and cr-narrow-style single-feature arms bracket the combo).
- Selected model has a deterministic bounded integer/table reference form — the arms are integer/fixed-point native by construction; no model was selected (negative verdict), and the differential instrument is in place for any future candidate.
- Selection by the predeclared untouched-domain rule; no candidate ⇒ `NO CALIBRATOR ESTABLISHED` — DONE (exactly this outcome; test partition untouched).
- Raw scores and decision curves reproduce from the pinned record — DONE (integer counts + curves in the CID-bound JSON; double-run/order-invariance asserted).

## Verification

- Run contract posted to #837 BEFORE the full run with the instrument's exact values (feature exposure; observability ceiling 982‰ of fit-partition errors); outcome posted against it.
- Fixture teeth green (5 non-ignored tests); instrument + full run green; shuffled-label null failed both gates (asserted).
- Local gates before push: fmt, claim-wording, clippy -D warnings (workspace/all-targets/all-features), no_std ×2, wasm lib, xtask validate (R1/R4/R5), repo-conformance (R2/R3), cargo-deny bans (R6), doc tests, workspace suite (0 failures across every binary; the local-model bdd soak is CI-skipped by design).

## Compatibility and migration

None: no serving or format changes. All fitted outputs are experimental, identified separately, and none is promoted.

## Conformance effects

None generated. Extends RF-01 / RF-22 / RF-23 evidence language under the #830 scope discipline.

## Claim-boundary changes

`docs/RESEARCH.md` and the record now assert the negative verdict and its three findings. No capability claim is added; the D4 coverage-only status is reaffirmed.

## Negative or unavailable results

The verdict IS the negative result, pre-registered and evidence-backed. Domain-transfer metrics are UNAVAILABLE on this single-domain corpus (recorded as such, never PASS); the #838 category-level metrics await the gold-annotated benchmark dataset (future work under the frozen constitution).

## Intentionally excluded

Pre-existing untracked user dirs (`obs-text/`, `tf1-pkg/`) are not staged.

Closes #837.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01J2R2qs9kfcYpsjUua1SYmi


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

