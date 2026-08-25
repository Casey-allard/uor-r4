# Artifact-only confidence calibrator — fit and verdict (#837)

- **Status:** Executed study; verdict recorded (item B of S2 tracker #823, programme #820).
- **Date:** 2026-08-21.
- **Claim language:** follows `docs/formal_vocabulary.md` (normative). This record asserts an
  evidence-backed **negative**: `NO CALIBRATOR ESTABLISHED` under the frozen gates. Current
  semantic abstention remains **NOT ESTABLISHED**; the deployed D4 policy stays coverage-only.
- **Harness:** `crates/uor-r4-api/tests/calibrator_fit_837.rs` (fixture teeth non-ignored;
  `calibrator_instrument_837` and `calibrator_fit_run_837` ignored/bundle-gated). Record:
  `docs/calibrator_837_result.json` (CID-bound). Run contract posted to #837 before the full
  run, with the instrument's exact values; outcome posted against it.
- **Relation to prior records:** executes under the #838 frozen constitution
  (`docs/selective_prediction_spec_838.md` §8–§9: gates, selection rule, UCB arithmetic) on
  the #833 canonical bundle, with the #875/#834 suffix baseline as the reference predictor
  (reproduced to the digit — 246.6‰ — before any reading was accepted).

## 1. Protocol

**Definition (execution scope).** Offline compiler/certifier reference (#830 vocabulary):
features are integer reads of the artifact tables (suffix and content co-occurrence, fitted
on the 288,794 TRAIN positions); the reference predictor is the suffix baseline; no teacher,
remote model, clock, or network is consulted at inference. The deployed engine is not
consulted (its public surface exposes the selected token, not candidate scores); production
integration is sibling item C (#839), which this verdict does **not** activate.

**Definition (partitions).** The 72,130 held-out positions split by story (`story % 3`) into
fit 24,064 / calibration 24,232 / test 23,834 (199/200/200 stories), document-disjoint by
construction and asserted. Thresholds were chosen ONLY on the calibration partition; the
test partition was reserved for a single evaluation of a selected candidate (none was
selected, so it stayed untouched — no test-set consumption on a negative).

**Definition (frozen gates, from #838 §9).** RELEASE: false-answer UCB95 ≤ 10‰ among served
at coverage ≥ 20‰. RESEARCH: UCB95 ≤ 50‰ at coverage ≥ 50‰. UCB95 is the frozen reference
arithmetic `(1000·k + 3000)/n` (ceiling). Risk is teacher-grounded: served-and-wrong vs the
recorded #833 teacher argmax.

**Empirical Criterion (binding cheap instrument — PASS, published before fitting).**
Feature exposure on the FIT partition: suffix key present 634‰, margin>0 542‰,
content-agreement 214‰, support>0 578‰, disagree>0 998‰ — all non-degenerate (asserted).
Observability ceiling: **17,882 / 18,205 fit-partition baseline errors (982‰) expose a
low-confidence signal** (absent key | margin < 500‰ | no content agreement), so the ceiling
clears any plausible effect floor — the study could have detected a qualifying calibrator.

## 2. Arms, controls, and results (calibration partition)

Candidate arms in pre-declared budget order; every score is integer/fixed-point native (the
deployed lowering is the model itself; the f64-shadow differential instrument detects a
planted fractional-weight drift and reads 0 on the integer-native arms).

| Arm | budget | release gate | research gate | risk–coverage highlights (err‰ @ cov‰) |
|---|---|---|---|---|
| `margin-threshold` | 4 B, 1 read | — | — | 520 @ 10 · 431 @ 200 · 752 @ 1000 |
| `top1rate-threshold` | 4 B, 1 read | — | — | 520 @ 10 · 440 @ 200 |
| `bucket-table` (16×10 fitted) | 644 B, 2 reads + 1 table | — | — | **11 @ 10 (UCB 23)** · 34 @ 20 (UCB 41) · 125 @ 50 · 230 @ 100 |
| `rich-combo` (6 features) | 28 B, 6 reads | — | — | 250 @ 10 · 327 @ 50 |
| `current-d4` (present) | 0 B, 1 read | — | — | flat 644 at every coverage ≤ 634 |
| `distance-only` | 0 B, 1 read | — | — | 520 @ 10 · 572 @ 200 |
| `count-only` | 0 B, 1 read | — | — | 492 @ 10 · 651 @ 200 |
| `constant-score` (planted) | — | — | — | single point (always-serve, 752) |
| `inverted-margin` (planted) | — | — | — | 925 @ 10 (worse than base, as planted) |
| shuffled-label null (bucket refit) | — | — | — | does not qualify (asserted) |

**Empirical Criterion (the verdict). Status: Empirical.** **`NO CALIBRATOR ESTABLISHED`** —
no arm met the frozen release gate, and none met the research gate either, on the
calibration partition; per the pre-registered rule no test-partition evaluation occurred and
no production activation (#839) is triggered. `docs/calibrator_837_result.json` carries the
full per-arm qualification points, curves, and budgets; result_cid `blake3:1d1504aa…`,
corpus_meta_cid `blake3:aa9d1767…`.

## 3. What the curves establish (the useful findings)

**Empirical Criterion (artifact signals do rank confidence). Status: Empirical.** The fitted
`bucket-table` (support-bucket × margin-decile) produces a cleanly monotone risk–coverage
curve — 11‰ error at 10‰ coverage rising to the 752‰ base error at full coverage — so
artifact-local evidence genuinely orders confidence. The frozen gates are nonetheless
missed: at the release floor (20‰ coverage) its UCB95 is 41‰ vs the required 10‰, and at
the research floor (50‰) its error is already 125‰ vs the required 50‰. A thin
high-precision memorized slice exists (~1% of positions at ~99% precision) but is too thin
for the frozen operating points.

**Empirical Criterion (margin alone is singleton-polluted). Status: Empirical.** The raw
`margin-threshold` arm is 520‰ WRONG in its most-confident decile-slice: a margin of 1000‰
is dominated by singleton suffix keys (total = 1 ⇒ top1 − top2 = total), which carry almost
no evidence. Support-conditioning (the bucket table) is what turns margin into signal — the
concrete reason `distance-only`/`count-only`-class signals cannot substitute for a fitted
calibrator, and a caution for any future threshold-on-margin shortcut.

**Empirical Criterion (answerable novelty is structurally discarded). Status: Empirical.**
2,454 of the 26,002 novel-suffix positions (94‰ of novelty; 34‰ of all held-out) are
content-answerable — the whole-window content aggregate alone predicts the recorded teacher
argmax — yet every suffix-feature calibrator serves 0 of them (margin = 0 on an absent
key). Read with #834 §6.3 (the content-evidence arm class is real but sub-floor), this
quantifies the redesign direction the negative branch names: **evidence acquisition**
(content-side features the suffix tables cannot see), not threshold tuning.

## 4. Controls and verification

Planted fixture teeth (non-ignored, CI-run): extraction/quantization/saturation and
monotone thresholding operate; an informative arm qualifies at a fixture-scale gate while
constant-score, inverted-margin, shuffled-label, and shuffled-feature all fail it
(non-degenerate, each changing real scores); a planted label-leakage "feature" is
detectable (no honest arm reaches zero-wrong at any fixture coverage); the integer-vs-f64
differential detects a planted 0.29-weight drift and reads 0 decision changes on the
integer-native arms; partitions reject a planted story leak; fit and sweep are double-run
and order-invariant. On the real run: the story-disjointness of the three partitions is
asserted, the base predictor reproduced 246.6‰ exactly, and the shuffled-label null failed
both gates (asserted).

## 5. Repository conformance and claim status

**Definition (RF mapping).** Research-leaf: no `model/ids.toml` row, no `CONFORMANCE.md`
regeneration. Extends the evidence language of RF-01 (behavioral probes), RF-22
(quality/pathology instruments), RF-23 (the deployed D4 path, used here as the baseline
policy), under the #830 scope discipline (off-scope evidence is not a serving result).

**Claim status and next action.** A fitted curve is not a serving claim, and no curve met
the frozen gates: the D4 policy remains **coverage-only**, semantic abstention remains
**NOT ESTABLISHED**, and #839 is **not activated**. The recorded next direction is
evidence-acquisition redesign — content-side features reaching the 2,454 discarded
answerable-novel positions and de-polluting the singleton-margin mass — after which a new
fit would re-enter under the same frozen constitution (the #838 gates do not move; #887
precedent). The S2 stage verdict against the full child set remains the maintainer's call
on #823.

## 6. Append-only content-evidence re-entry (#931, 2026-08-22)

This section appends the outcome of the single redesigned re-entry sanctioned by the S2
tracker; it does not revise the historical #837 measurement above. The follow-up harness
is `crates/uor-r4-api/tests/selective_calibrator_reentry_931.rs`, the readable record is
[`content_evidence_calibrator_931.md`](content_evidence_calibrator_931.md), and the
machine-readable record is
[`selective_calibrator_reentry_931_result.json`](selective_calibrator_reentry_931_result.json).

**Empirical Criterion (content-evidence re-entry). Status: Empirical.** The follow-up
reproduced the #833 corpus identity and the #908 base/SKMX+PSIB artifact identities, then
refit production-equivalent packed RF-31 content evidence on the original story-disjoint
FIT/CAL/TEST partition under the unchanged #838 gates. The FIT-only instrument qualified
the run, but no selectable content-margin, content-support-margin, or hybrid
content-plus-suffix arm met either the release or research gate on CAL. TEST therefore
remained untouched. The evidence-backed verdict is **`NO CALIBRATOR ESTABLISHED`**;
semantic abstention remains **NOT ESTABLISHED**, and #839 remains limited to its
legacy-coverage phase unless a later maintainer decision establishes a new gate-backed
path. The audited result CID is
`blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`.
The harness separately reproduces this record's original suffix-predictor curve
(3/261, 19/549, and 170/1,350 wrong at the three reported floors) and records the
different curve obtained when that locked score ranks the RF-31 skip-mix winner. The
#837 result remains true as its own historical suffix-evidence finding.
