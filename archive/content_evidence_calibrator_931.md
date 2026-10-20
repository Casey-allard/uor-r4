# RF-31 content-evidence selective-calibrator re-entry (#931)

- **Status:** Executed study; **`NO CALIBRATOR ESTABLISHED`**.
- **Date:** 2026-08-22.
- **Parent:** S2 tracker #823; the one redesigned B2 re-entry sanctioned by the
  2026-08-21 `REVISE` verdict.
- **Harness:** `crates/uor-r4-api/tests/selective_calibrator_reentry_931.rs`.
- **Result:** `docs/selective_calibrator_reentry_931_result.json`, result CID
  `blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`.
- **Claim language:** `docs/formal_vocabulary.md` is normative. This record is
  an evidence-backed negative on a teacher-grounded corpus-position proxy. It
  is not a semantic-answerability or production-abstention certificate.

## 1. Execution scope and frozen protocol

**Definition (execution scope).** Offline compiler/certifier analysis of
production-equivalent RF-31 SKMX/PSIB row semantics. The harness regenerates
the canonical base and skip-mix graphs, obtains the deployed `R4Engine` winner
and base candidate list, and independently extracts features through the
packed SKMX/PSIB readers. This work is off-serving-path and changes no deployed
selective decision, artifact format, protocol, or compatibility behavior.

**Definition (target and partitions).** Risk is the deployed skip-mix winner
differing from the canonical #833 corpus's recorded teacher argmax. The 72,130
held-out positions retain #837's story-disjoint folds: FIT 24,064 / 199 stories,
CAL 24,232 / 200, and reserved TEST 23,834 / 200. Table fitting reads FIT
labels only. The binding instrument runs on FIT; after it passes, operating
points are chosen on CAL. TEST may be evaluated once only after a selectable
arm clears the CAL release gate.

The contract clarification on #931 treats the already merged, CID-bound #837
and #908 all-heldout readings as predecessor evidence instead of re-reading
TEST merely to reproduce them. The harness verifies those predecessor records,
then reproduces the exact corpus and regenerated graph CIDs. Because no arm
qualified on CAL, TEST stayed untouched and the all-heldout recount remained
`null` in the result.

**Definition (frozen gates).** The gates did not move: release requires
false-answer UCB95 at most 10‰ with coverage at least 20‰; research requires
UCB95 at most 50‰ with coverage at least 50‰. The reference UCB arithmetic is
the frozen conservative ceiling `(1000k + 3000) / n`, where `k` is wrong among
served and `n` is served. A research-only result does not activate production.

## 2. Identity and binding instrument

**Guarantee (packed-row semantics). Status: Structural.** Fast fixture tests
cover primary/fallback precedence, injection, no-injection, missing-section
identity, tie behavior, saturation, and corrupt/truncated table rejection.
Primary-row presence suppresses PSIB fallback even when the candidate is
absent from that primary row; contributions add with saturation; supported
candidates outrank unsupported base candidates; ties go to the smaller token.

**Empirical Criterion (packed/deployed agreement). Status: Empirical.** On all
24,041 deployed-eligible FIT positions and all 24,222 deployed-eligible CAL
positions of the pinned bundle, the independently reconstructed RF-31 winner
equals the deployed engine winner and the complete independently extracted
feature vector equals the packed-path vector. When the lane promotes a token,
the independently summed winner contribution also equals the deployed witness
boost. Before replay, all 811,421 compiler SKMX rows and all 19,710 compiler
PSIB rows are checked against their validated packed representations. This
bundle-gated differential is empirical evidence, not a CI-run structural proof.

**Empirical Criterion (identity reproduction). Status: Empirical.** The
canonical identities reproduce exactly:

| Identity | CID / value |
|---|---|
| corpus metadata | `blake3:aa9d176779c1d2411e872c49c95ed585ee805ded5fa1b808ddf2f517a245b0ce` |
| regenerated base graph | `blake3:aaf98b68a78dd615f06dbb727a22dc4e170a152f055313fcc4fa574309f42d1e` |
| regenerated skip-mix graph | `blake3:19eb04d7dbf3fccd126069982ad8cbc1de31d536fff7e77ef2dacb26e64106cc` |
| #837 predecessor result | `blake3:1d1504aa06184b60ded96de366a5102a5d75c05441a3ff92eb86ca8fa8f1e549` |
| #908 predecessor result | `blake3:e32e4e33d70f342ae3c0913ba00d9aef0cf789b539b9e1b658a9366c51402a26` |

FIT-only top-1 diagnostics were base 6,390/24,064, deployed skip-mix
6,958/24,064, and the locked suffix predictor 5,859/24,064.

**Empirical Criterion (binding cheap instrument). Status: Empirical.** The
post-review full-family instrument returned **PROCEED** in 22.3 seconds. The
deployed D4 policy made 24,041/24,064 FIT positions eligible and declined 23;
no candidate may resurrect those declines. Winner support was present on 999‰
of FIT positions, content margin was nonzero on 985‰, the deployed lane changed
the base winner on 546‰, and 331‰ selected an injected candidate outside the
base list. Swapping the content/last-token key changed 22,097/24,064 complete
feature vectors (918‰). There were 23,340 exact feature cells. The label-aware
upper-bound cell over the complete declared feature family reached:

- release: 6,962 served / 43 wrong at theta 333, UCB95 7‰;
- research: 7,161 served / 203 wrong at theta 195, UCB95 29‰.

Thus feature availability and the declared family did not make the target
mathematically unreachable; the CAL sweep had decision value. An initial
preflight used only the two-dimensional content-table cell as its oracle and
printed STOP. Review caught that it did not upper-bound the declared hybrid
family. It was corrected before any CAL/TEST access. A later independent review
then tightened deployed eligibility, effective candidate counting, the full
feature differential, and compiler-row/packed-row checks; the instrument was
rerun before the audited CAL sweep. No selectable candidate feature, threshold,
floor, or outcome branch changed. The #931 issue thread preserves the earlier
readings and marks the 22.3-second run as their append-only superseding result.

The exact ordered feature vectors are content-bound independently for FIT
(`blake3:a961e1b8b595885402d831cdb7679d050f5b2f3041939dcd8324889c93bd5b6d`)
and CAL
(`blake3:92690fc790e36d1e64d23fe91564739def7933968a1f6f4ab7bc92e723deede3`).
The machine-readable result preserves bounded summaries and histograms for
every required feature.

## 3. Features, arms, and budgets

**Definition (artifact-only features).** Each row records bounded integer
joint/fallback row counts, winner support, conflicting-row count,
winner/runner contributions and saturated margin, base agreement, injected
candidate status, base margin, candidate count, and the locked #837 suffix
presence/support/margin inputs. Candidate scores use no teacher label, network,
clock, RNG, division, or floating point. FIT may allocate while building the
tables; the candidate representation is a fixed bounded table suitable for a
later P-4 lowering if it had qualified.

Candidate order and projected incremental budget were frozen before CAL:

| Arm | Budget | Role |
|---|---:|---|
| `content-margin` | 4 B; 2 feature reads; 0 table reads; 3 projected operations | selectable |
| `content-support-margin` | 292 B; 2 feature reads; 1 table read; 7 projected operations | selectable |
| `hybrid-content-suffix` | 932 B; 4 feature reads; 2 table reads; 14 projected operations | selectable |
| locked #837 suffix table | 644 B; 2 feature reads; 1 table read; 7 projected operations | reference/control |

SKMX-only, PSIB-only, no-injection, and injection-without-confidence readings
are non-selectable ablations. Always-serve, always-decline, current D4,
winner-support-only, base-margin-only, constant, inverted content margin,
label shuffle, feature shuffle, and content-key shuffle are controls or
falsifiers.

## 4. CAL result and verdict

**Empirical Criterion (risk/coverage on CAL). Status: Empirical.** No
selectable arm met either frozen gate:

| Arm | 10‰ coverage slice: error / UCB | 20‰ release floor: error / UCB | 50‰ research floor: error / UCB | Verdict |
|---|---:|---:|---:|---|
| `content-margin` | 32‰ / 46‰ (243 served) | 72‰ / 79‰ (485) | 176‰ / 180‰ (1,226) | neither |
| `content-support-margin` | 580‰ / 582‰ (3,020) | same cell | same cell | neither |
| `hybrid-content-suffix` | 113‰ / 124‰ (299) | 117‰ / 124‰ (485) | 148‰ / 151‰ (1,269) | neither |
| locked #837 suffix predictor/reference | 11‰ / 23‰ (261) | 34‰ / 41‰ (549) | 125‰ / 129‰ (1,350) | neither |
| locked suffix score applied to skip-mix winner | 72‰ / 85‰ (261) | 78‰ / 84‰ (549) | 158‰ / 161‰ (1,350) | neither |

The raw content margin contains a non-degenerate high-confidence slice: 8
wrong among 243 served at 10‰ coverage. It is not better than #837's
suffix-predictor reference slice (3/261), and the predictors differ. It is too
risky and too thin. At the required 20‰ release floor,
its UCB is 79‰, almost eight times the frozen 10‰ bound; at the 50‰ research
floor it is 180‰ versus 50‰. Support bucketing collapses heterogeneous content
rows into a poor 3,020-position cell, while the hybrid does not recover the
lost separation.

Every real null failed both gates as required. Label shuffling changed
9,974 FIT / 10,190 CAL label pairings, content-key shuffling changed 22,097 /
22,335 feature pairings, and feature shuffling changed 24,061 / 24,228; all
three null reports evaluated every selectable arm. The constant control is the
deployed-D4 endpoint (16,980 wrong among 24,222 eligible positions; 701‰
observed error), while the separate always-serve control is 16,990/24,232. The
inverted-margin release-floor slice is 931‰ wrong, demonstrating directionality.
The CAL top-1 ablations were: base 6,518; deployed skip-mix 7,242; suffix
5,995; SKMX-only 7,248; PSIB-only 2,361; no-injection 7,261. Aggregate accuracy
movement therefore does not supply calibrated confidence; notably, removing
candidate injection slightly raises this CAL top-1 count while neither
confidence family qualifies.

There were 517 CAL positions where the suffix key was absent, the deployed
skip-mix winner differed from the base winner, and skip-mix matched the recorded
target. This is the predeclared current-run content-caused novelty proxy; it is
not the historical #837 whole-heldout count of 2,454 content-answerable novel
positions. Because no model reached a release point, candidate admission did
not evaluate a served novelty slice and selected retention is recorded as
`null`, never as measured zero.

**Empirical Criterion (content-evidence candidate selection). Status:
Empirical.** **`NO CALIBRATOR ESTABLISHED`** — no selectable arm met release or
research on CAL. TEST was not opened, no threshold was retuned, and #839 phase
2 is not activated. The deployed RF-30 legacy-coverage schema remains the only
selective-prediction mode.

## 5. Controls, determinism, and availability

**Guarantee (falsifier and determinism teeth). Status: Structural.** The fast
suite detects planted label leakage, feature shuffling, inverted confidence,
fractional/reference drift, partition overlap, table corruption, and missing
sections. Fit, curves, and the fixture report are input-order independent.

**Empirical Criterion (bundle-run determinism). Status: Empirical.** Two
complete pinned-bundle runs produced identical result CID
`blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`
and identical result-file SHA-256
`838b4591f126f6ab0d2f767a976f24c07782b9932910831e6c9dff3a019d7ade`.
The full run took 41.2–41.4 seconds. Peak RSS was not instrumented and is
`UNAVAILABLE`, not zero.

**Empirical Criterion (semantic benchmark availability). Status: Empirical.**
The real powered `s2-answerability-ood` annotation fixture is **`UNAVAILABLE`**.
The repository contains its constitution and planted reference populations;
those are structural tests, not a 4,800-item, eight-category, four-axis-
disjoint empirical reading. The result binds the suite manifest as
`blake3:19e6c0f9051567f3ecb421aa92b78515598ecfe49168e5b9d67ff9cff4c8b31e`
and the constitution as
`blake3:37f770b346ad03471a4757cd21adbcac7c5ac39bb1b6aa51e7c8d8b378ed2e9b`;
the generator, annotations, rubric, and split-assignment identities remain
`null`. `UNAVAILABLE` is not `PASS`.

## 6. Compatibility, conformance, and next action

**Definition (compatibility and RF mapping).** This research leaf changes no
runtime, artifact bytes, loader, protocol, or migration behavior. It adds no
`model/ids.toml` row and requires no generated `CONFORMANCE.md` change. It
extends the evidence records of RF-01, RF-22, RF-23, RF-29, RF-30, and RF-31;
it does not build a new served capability.

Current semantic abstention remains **NOT ESTABLISHED**. The one sanctioned
content-evidence re-entry is now exhausted with a negative result. #839 stays
legacy-only and its calibrated phase is not triggered. The next action is a
maintainer S2 verdict on #823 (`LIMIT`, `REVISE`, or another explicitly
authorized scope); this issue does not silently create another calibration
attempt or lower the frozen bars.
