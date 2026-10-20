# Typed selective prediction and the answerability benchmark constitution (#838)

- **Status:** Frozen contract + benchmark constitution (item A of S2 tracker #823, programme #820).
- **Date:** 2026-08-21.
- **Claim language:** follows `docs/formal_vocabulary.md` (normative). This record freezes
  *meanings and measurement*. It deploys no predictor, fits no threshold, and asserts no
  empirical capability result. **Current semantic abstention is NOT ESTABLISHED** (§13).
- **Companion reference model:** `crates/uor-r4-api/tests/selective_prediction_spec_838.rs` —
  the executable realization of every table in this document, with planted negatives.
- **Relation to prior records:** uses the #830 execution-scope/verdict vocabulary
  (`docs/conformance_execution_scope_830.md`), the ADR-0001 normative-scorer designation
  (#831), and the #832 evaluation constitution (`docs/capability_suites_832.md`, suite
  `s2-answerability-ood`). Records are appended, never rewritten.

## 1. Problem and scope

#811 established parity for the D4 resolution policy, and all five semantic-OOD probes
remained SERVABLE: the deployed D4 axis (`exact_context` / `graph` / `novel` /
`contradictory`) reports *representation and resolution coverage*, not whether an answer is
*supported by evidence*. Covered, novel, unsupported, contradictory, low-confidence,
declined, and answered are conflated today across surfaces: the CLI and the native server
surface a D4 abstention (#811 `ChatAbstention`), the OpenAI-compatible surface reports a
generic `engine_declined` error, and no surface carries evidence or calibrated confidence.
The five hand-written probes cannot support a risk claim.

**Definition (execution scope of this record).** Semantic contract, offline benchmark
constitution, and production response *schema* — `reference-only` / `off-serving-path` in
the #830 vocabulary. No predictor is deployed by this issue; evidence produced under this
record is not a serving result. The calibrator fit is sibling item B (#837); deployed
execution and certification across production surfaces is sibling item C (#839).

## 2. The typed status space

**Definition (the eight selective-prediction statuses).** Each is a separate typed concept
with exactly one meaning and one canonical kebab-case label. No status is ever inferred
from another; no surface may merge two of them into one representation.

| # | Status | Label | Kind | One meaning |
|---|---|---|---|---|
| 1 | Covered | `covered` | coverage-axis value | the artifact's calibrated representation resolves this input (D4 serve-side reading) |
| 2 | Distributionally novel | `distributionally-novel` | coverage-axis value | no calibrated region covers the input (D4 `novel` reading) |
| 3 | Insufficient evidence | `insufficient-evidence` | evidence-axis value | the input resolves, but no candidate answer has supporting evidence above the floor |
| 4 | Conflicting evidence | `conflicting-evidence` | evidence-axis value | active evidence materially disagrees between incompatible answers |
| 5 | Supported answer | `supported-answer` | outcome | an answer is served, bound to its supporting evidence and calibrated confidence |
| 6 | Low confidence | `low-confidence` | confidence reading | evidence supports an answer but calibrated confidence sits below the operating point |
| 7 | Abstention | `abstention` | outcome | the system declines to answer, with a typed cause — never an empty success, never a generic error |
| 8 | Hard incompatibility | `hard-incompatibility` | outcome | the request cannot be validly served by this artifact/surface at all (corrupt or version-incompatible calibration data, unsupported protocol form); fail-closed |

**Definition (three axes plus an outcome).** The statuses compose on orthogonal axes rather
than one flat scale — this is the separation #811 showed is missing:

- **Coverage axis** `V ∈ {covered, distributionally-novel}` — a structural reading of the
  artifact's calibrated geometry (today's D4 signal, `PolicyStatus` in
  `crates/uor-r4-api/src/engine.rs`).
- **Evidence axis** `E ∈ {supported, insufficient-evidence, conflicting-evidence}` — an
  evidential reading over candidate answers and their provenance. The deployed classifier
  for this axis does not exist yet (#837 fits it; #839 executes it).
- **Confidence reading** `q ∈ [0, 1000]` (integer parts-per-thousand) with a frozen
  operating point `θ`; `q < θ` reads `low-confidence`. Defined only when calibration data
  is present (§6).
- **Compatibility** `C ∈ {compatible, hard-incompatibility}` — a fail-closed structural
  gate evaluated before everything else.

**Definition (deterministic outcome function).** The served outcome is the total function
`decide(C, V, E, q, θ, calibrated)`:

| Rule (first match wins) | Outcome | Cause carried |
|---|---|---|
| `C = hard-incompatibility` | `hard-incompatibility` | — |
| `E = conflicting-evidence` | `abstention` | `conflicting-evidence` |
| `E = insufficient-evidence` | `abstention` | `insufficient-evidence` |
| `E = supported ∧ calibrated ∧ q < θ` | `abstention` | `low-confidence` |
| `E = supported ∧ (¬calibrated ⇒ legacy mode §6)` | `supported-answer` | — |

Cause precedence is the row order above: `hard-incompatibility ≻ conflicting-evidence ≻
insufficient-evidence ≻ low-confidence` (and, only in legacy mode, `distributionally-novel`
— §6). The coverage value `V` is always *reported* alongside the outcome and is never
itself a cause outside legacy mode: **novelty alone is not unanswerability**. A
distributionally-novel input with supported evidence and passing confidence is served —
this is answerable novelty, and it is separable from semantic unanswerability by
construction (Verification V3).

**Guarantee (totality and determinism). Status: Structural** (reference model;
`decision_table_is_total_and_deterministic`). `decide` is defined for every input in the
product space and identical inputs produce identical outcomes and identical wire bytes on
every surface.

## 3. Vocabulary classification of each status

**Definition (what kind of claim each status makes).** Under `docs/formal_vocabulary.md`:

- `covered` / `distributionally-novel` — readings of a **Guarantee**-class structural
  signal (the D4 classification is deterministic integer arithmetic over the artifact;
  proof-matrix statuses per #830). Whether coverage *correlates with answerability* is an
  **Empirical Criterion**, measured by this benchmark, never assumed.
- `insufficient-evidence` / `conflicting-evidence` / `low-confidence` — **Empirical
  Criterion**-class readings: they are meaningful only relative to a fitted, calibrated
  classifier with declared distribution, n, and uncertainty (#837). Until that exists they
  are **Unproven** for the deployed system, and no deployed surface may emit them (§6).
- `supported-answer` / `abstention` / `hard-incompatibility` — outcome semantics are
  **Guarantee**-class (structural response-schema properties: typed, deterministic,
  fail-closed); *the rates at which they are correct* are **Empirical Criteria** with the
  §9 metrics.
- **Assumption:** gold answerability annotations (§7) are treated as ground truth by every
  metric; annotation quality is bounded by the rubric identity they pin, not established by
  this repository.

## 4. Evidence, provenance, and confidence fields

**Definition (response evidence block).** A served `supported-answer` and every witness
record carry, additively:

- `evidence.supporting: u32` / `evidence.conflicting: u32` — bounded counts of evidence
  units for/against the served answer at the operating point.
- `evidence.paths` — the per-token `ResolutionPath` histogram of the served span (#832
  vocabulary: `exact-context`, `ngram`, `graph`, `root-prior`, `patch-delta`,
  `sampled-selection`, `decline`).
- `provenance` — the identity set: `artifact_cid`, `calibration_cid` (absent in legacy
  mode), `witness_cid`, and the ADR-0001 normative scorer id for every served token.
- `confidence_permille: u32 ∈ [0, 1000]` — the calibrated confidence reading; present iff
  calibration data is present and valid; **never fabricated** (§6).

All fields are integers or content identifiers — no floats on any wire or witness surface
(the #609–#613 serde-float lesson; #832 integer-fraction rule).

**Guarantee (witness parity). Status: Structural** (reference model;
`witness_carries_the_same_typed_fields`). The witness record carries the same status,
cause, coverage, evidence, and confidence fields as the production response, so an
independent verifier replays the abstention decision without the teacher.

## 5. Deterministic cross-surface representation

**Definition (surface mapping table).** One normative mapping per surface; the reference
model implements each encoder and round-trips it. Canonical labels are the §2 kebab-case
strings everywhere; `snake_case` variants appear only inside OpenAI-compatible `error.code`
values, by the deterministic rewrite `s/-/_/g`.

| Outcome | CLI (`r4 chat`/`ask`) | Native HTTP/API | OpenAI-compatible non-streaming | OpenAI-compatible streaming | WASM host |
|---|---|---|---|---|---|
| `supported-answer` | answer text; typed record carries status, coverage, `confidence_permille`, evidence | `200`, JSON `status: "supported-answer"` + §4 fields | normal `choices`; §4 fields under the vendored `uor` extension key | content chunks, then normal termination | `Ok(Served{…})` boundary enum, same labels |
| `abstention` | no answer text; typed abstention record (extends #811 `ChatAbstention`) with `cause`, coverage, `confidence_permille?`; exit code 0 (an abstention is a *successful, honest* outcome) | `200`, JSON `status: "abstention"`, `cause`, coverage, §4 fields | HTTP 422, `error.code = "uor_abstention_<cause_snake>"`, `error.type = "uor_selective_prediction"`; **never** an empty-`choices` success | **no** content chunk; one terminal typed SSE `error` event with the same code, then `[DONE]`; never a silent stream end | `Ok(Abstained{cause, …})` |
| `hard-incompatibility` | typed error, nonzero exit | `409`, JSON `status: "hard-incompatibility"`, `reason` | HTTP 409, `error.code = "uor_incompatible_artifact"` | terminal typed SSE `error` event, then `[DONE]` | `Err(Incompatible{reason})` — typed, never a trap |

**Guarantee (deterministic, injective representation). Status: Structural** (reference
model; `cross_surface_encodings_round_trip_and_are_injective`). For every surface the
mapping status → wire form is injective (no two statuses share a representation), total
over the §2 space, and byte-deterministic for identical inputs.

**Guarantee (abstention is never hidden). Status: Structural** (reference model;
`abstention_is_never_an_empty_success`). On no surface does an abstention serialize as an
empty successful completion, a zero-length `choices` success, or a generic server error —
the #838 non-goal made a falsifier. The current `/v1/*` `engine_declined` error body is the
migration ancestor of the typed `uor_abstention_*` codes; #839 executes the migration.

## 6. Backward compatibility and fail-closed calibration

**Definition (legacy-coverage mode).** An artifact **without** calibration data (no
calibration section/sidecar identity) serves under today's D4 policy, surfaced through the
same typed schema: coverage axis populated; abstentions carry cause
`distributionally-novel` (the only legacy cause); `confidence_permille` and the evidence
axis are **absent** — never fabricated, never defaulted to a value. Legacy artifacts remain
always-serve/current-D4 in behavior.

**Guarantee (no calibrated claim without calibration). Status: Structural** (reference
model; `legacy_artifact_without_calibration_stays_coverage_only`). Legacy-mode responses
cannot mint `insufficient-evidence`, `conflicting-evidence`, `low-confidence`, or any
confidence value, and a legacy-mode report cannot satisfy the §12 claim gate. Old artifacts
do not inherit a calibrated-answerability claim.

**Guarantee (fail-closed on corrupt calibration). Status: Structural** (reference model;
`corrupt_calibration_fails_closed`). Calibration data that is *present but invalid* —
CID-mismatched bytes, an unknown schema version, or a truncated table — is
`hard-incompatibility` for every request that would consult it. Corrupt is distinguished
from absent: absent means legacy mode (above); corrupt **never** silently degrades to
legacy mode, because a tampered calibration must not reopen the always-serve surface.

## 7. Benchmark categories and gold annotations

**Definition (the eight categories).** The answerability benchmark populates the frozen
`s2-answerability-ood` suite (#832) with eight input categories, each with a gold
answerability annotation and a gold status:

| Category | Answerable? | Gold outcome at a working operating point |
|---|---|---|
| `in-domain-answerable` | yes | `supported-answer` |
| `paraphrased-answerable` | yes | `supported-answer` (paraphrase-stable) |
| `novel-but-supported` | yes | `supported-answer` (answerable novelty; `V = distributionally-novel`) |
| `missing-evidence` | no | `abstention` / `insufficient-evidence` |
| `private-information` | no | `abstention` / `insufficient-evidence` (the evidence is absent from the artifact by construction) |
| `false-premise` | no | `abstention` / `insufficient-evidence` (no evidence supports the presupposed entity/fact) |
| `contradictory-evidence` | no | `abstention` / `conflicting-evidence` |
| `unrelated-ood` | no | `abstention` (legacy cause `distributionally-novel` permitted here and only here) |

**Definition (gold annotation schema).** Each item carries: category, `answerable ∈ {yes,
no}`, gold outcome + cause, evidence references into the corpus (present for answerable
items, absent-by-construction for `missing-evidence`/`private-information`), and the four
disjointness keys `document / domain / entity / template`. The full annotation set, the
generator configuration, and the rubric are CID-bound (`compute_cid`) and pinned in every
report (#832 `SuiteIdentities`).

**Definition (partitions).** Calibration, validation, and evaluation partitions are
disjoint on **all four axes** — document, domain, entity, and template — checked by the
#832 leakage machinery (`detect_document_leakage` extended per-axis in the reference
model). A leaked key on any axis rejects the split; a tampered fixture fails `verify_cid`.

## 8. Baselines and their expected confusion profiles

**Definition (the six baselines).** Every benchmark run reports, on identical items and
identities: `always-serve`, `always-decline`, `current-D4` (coverage-only policy),
`distance-only` (monotone in Hamming distance to the nearest calibrated region),
`evidence-count-only` (support count threshold, ignoring conflict structure), and
`trivial-prior` (the no-context floor). The first two and last one bind to the #832
`ControlKind` vocabulary (`always-serve`, `always-decline`, `trivial-prior`).

**Empirical Criterion (distinct confusion profiles). Status: Structural for the planted
populations; Empirical on the real benchmark.** The baselines are non-degenerate and
pairwise distinguishable by their category-confusion signatures: `always-serve` maximizes
false answers on every unanswerable category with zero false abstains; `always-decline`
inverts it; `current-D4` abstains on `unrelated-ood` but **falsely serves**
`false-premise`, `contradictory-evidence`, and `missing-evidence` (the #811 finding, made a
fixture); `distance-only` cannot separate `novel-but-supported` from `unrelated-ood` and is
paraphrase-brittle (it falsely abstains on `paraphrased-answerable`, whose surface distance
exceeds its threshold while the item stays covered and supported); `evidence-count-only`
cannot separate `contradictory-evidence` from strongly `in-domain-answerable`;
`trivial-prior` is the floor. The reference model plants each
signature (`baselines_produce_distinct_confusion_profiles`); a degenerate control
(`is_degenerate_control`) invalidates a reading.

## 9. Predeclared metrics, powered n, and upper-confidence-bound targets

**Definition (metric set).** All metrics are integer fractions (#832 `MetricStatus`):

- **Risk–coverage**: the curve of (coverage = served fraction, risk = error among served)
  as `θ` sweeps; primary summary `risk-coverage-auc` (the `s2-answerability-ood` primary
  metric), reported with the always-serve / always-decline endpoints.
- **False-answer rate**: served ∧ unanswerable, over unanswerable items, with a one-sided
  95% upper confidence bound (UCB95).
- **False-abstain rate**: abstained ∧ answerable, over answerable items, with UCB95.
- **Calibration error**: integer-bucketed (10 confidence deciles) expected calibration
  error in parts-per-thousand.
- **Category confusion**: the 8×3 outcome matrix per category, per policy.

**Definition (powered sample sizes — fixed before any fitting).** `N_PER_CATEGORY = 600`,
`N_TOTAL = 4,800` evaluation items (with calibration and validation partitions at least the
same size). Arithmetic behind the choice, from the declared bounds: at zero observed false
answers over `n` unanswerable items, the rule-of-three one-sided UCB95 is `≈ 3/n`; the
release false-answer target (below) needs UCB95 ≤ 10‰, so `n ≥ 300` unanswerable items
suffice at zero failures and `n = 5 × 600 = 3,000` unanswerable items give headroom for
non-zero counts (UCB95 at 3,000 with up to ~24 failures stays ≤ 10‰ by the same
approximation, reported exactly by the Clopper–Pearson bound in the real report).
Category-level resolution: at `n = 600` per category a 100‰ confusion rate carries a 95%
half-width of ±24‰, so ≥ 50‰ profile differences between baselines are resolvable
per-category.

**Definition (frozen UCB targets — the operating-point constitution).**

| Operating point | Selection rule (frozen now, fitted by #837) | Targets on the frozen eval partition |
|---|---|---|
| `research` | maximize coverage subject to UCB95(false-answer) ≤ **50‰** on the calibration partition | UCB95(false-answer) ≤ 50‰; false-abstain ≤ 300‰ |
| `release` | maximize coverage subject to UCB95(false-answer) ≤ **10‰** on the calibration partition | UCB95(false-answer) ≤ 10‰; false-abstain ≤ 200‰ |

Thresholds `θ` are **fitted only on the calibration partition** and evaluated **once** on
the evaluation partition; the selection rule and the targets above are frozen by this
record *before* any fitting (`power_and_ucb_targets_fixed_before_fitting`,
`operating_points_frozen_and_selection_rule_deterministic`).

## 10. Identity binding

**Definition (bound identities).** Every report pins, per #832: teacher, tokenizer, corpus,
compiler, artifact, decoder — plus this benchmark's generator configuration, annotation
set, rubric, split assignment, and report CIDs. A missing required identity forces the
metric to `Unavailable` (never a vacuous zero); an absent fixture is `UNAVAILABLE`, never
`PASS` (#830).

## 11. Reference model and verification

**Definition (executable reference model).** The companion test realizes: the §2 status
space with canonical labels and parsers; the §2 decision table (total, exhaustively
checked); the §5 encoders for CLI, native HTTP, OpenAI-compatible non-streaming and
streaming, and the WASM boundary, with round-trip and injectivity checks; the §6
legacy/fail-closed semantics with planted corrupt-calibration bytes; the §7 category and
partition schema with per-axis leakage and tamper rejection; the §8 baseline signatures on
planted populations; the §9 power constants and UCB arithmetic; and the §12 claim gate.
Verification items V1–V5 of the issue map to named tests:

- **V1** schema/status round-trips across surfaces → `cross_surface_encodings_round_trip_and_are_injective`.
- **V2** planted false-premise / contradiction / answerable-novel / missing-evidence /
  corrupt-calibration fixtures → `planted_category_fixtures_classify_per_gold`,
  `corrupt_calibration_fails_closed`.
- **V3** answerable novelty vs semantic unanswerability →
  `answerable_novelty_is_separable_from_unanswerability`.
- **V4** baseline confusion distinctness → `baselines_produce_distinct_confusion_profiles`.
- **V5** power reproduction, split/leakage/tamper →
  `power_and_ucb_targets_fixed_before_fitting`, `partitions_reject_leakage_and_tamper`,
  plus the determinism check `double_run_and_reordered_input_determinism`.

## 12. Coverage is not semantic PASS (the claim gate)

**Guarantee (a coverage result cannot render a semantic verdict). Status: Structural**
(reference model; `coverage_result_cannot_render_semantic_pass`). The reference claim gate
accepts a "calibrated semantic abstention" claim only from a report that (a) binds the
`s2-answerability-ood` suite identity, (b) carries measured evidence-axis metrics from a
calibrated (non-legacy) run, and (c) meets the §9 targets at a frozen operating point.
A D4/coverage-only report — however green — returns **NOT ESTABLISHED**. This is the
conformance tooth against re-reading representation coverage as answerability.

## 13. Repository conformance and claim status

**Definition (RF mapping).** This record freezes contracts and measurement; it builds no
deployed capability and adds **no** `model/ids.toml` row and **no** `CONFORMANCE.md`
regeneration (generated conformance is never hand-edited). It extends the evidence
language of RF-01 (behavioral probes), RF-22 (pathology/quality instruments), RF-23 (the
deployed D4 selection/fallback path this schema wraps), and RF-29 (fixture-status
discipline: absent is `UNAVAILABLE`, never `PASS`). A new selective-prediction capability
ID is added only when #839 actually builds the typed capability, in the required order
(IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated
`CONFORMANCE.md`).

**Claim status and next action.** This completion **freezes meanings and measurement** and
explicitly records: **current semantic abstention is NOT ESTABLISHED** — the deployed D4
policy is a coverage policy; all five #811 semantic-OOD probes remained SERVABLE; no
evidence-axis classifier exists. Next actions: #837 fits the artifact-only confidence
calibrator against the §8 baselines and §9 targets on this constitution; #839 executes and
certifies the typed contract across the production surfaces. A negative or sub-target
outcome there is a legitimate, recordable result under this constitution, not a reason to
move the frozen targets.
