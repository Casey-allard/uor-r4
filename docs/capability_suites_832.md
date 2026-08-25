# CID-bound capability suites and per-token resolution attribution (#832)

- **Status:** Committed evaluation infrastructure (item D of S0 tracker #821, programme #820).
- **Date:** 2026-08-20.
- **Scope of this record:** Offline evaluation/certification plus measured reachability to the
  normative deployed `R4Engine` path. Evidence outside this scope is not credited as a
  deployed-serving result.
- **Claim language:** follows `docs/formal_vocabulary.md` (normative). This record is a
  **Definition/Assumption-scope** design and infrastructure record. It builds trustworthy
  measurement infrastructure; it does **not** itself pass any capability gate, and it asserts no
  empirical capability result.
- **Relation to prior records:** uses the normative-scorer designation in ADR-0001
  (`docs/adr/0001-normative-r4g1-scorer.md`, #831) for production attribution and the
  conformance status vocabulary in #830 (`docs/conformance_execution_scope_830.md`). Records are
  appended, not rewritten.

## Why one constitution (the problem)

Gate C, teacher parity, the assistant canaries, the OOD probes, and corpus replay answer
different questions on different slices; some suites skip when a live fixture is absent; and an
aggregate score hides which serving path produced each token. Without document-disjoint
partitions, pinned identities, powered samples, declared negative controls, and per-token path
attribution, a later "improvement" can be leakage, ExactContext dominance, a decoder effect, a
vacuous control, or benchmark drift. The evaluation constitution makes each of those failure
modes a schema-level, testable object.

## What this lands

A versioned schema module, `crates/uor-r4-api/src/capability_suite.rs`, plus the committed
manifests under `crates/uor-r4-api/capability_suites/`. It mirrors the schema-first shape of
`release_bundle.rs` (#655-C0): versioned records and pure structural checks, filesystem-free at
the library edge (the manifests are embedded with `include_str!`), running no evaluation itself.

- **Suite manifests** (`SuiteManifest`) — one committed, versioned manifest per capability
  workload. Each freezes the suite's primary metric, promotion statistic, split rules, control
  set, target report schema, and the identity slots every report must pin.
- **Constitution index** (`Constitution`) — a stage → frozen-primary-suite map, so every
  programme stage S0–S7 *names* one primary suite, split, control set, report schema, and
  promotion statistic (acceptance criterion 1).
- **Capability report** (`CapabilityReport`) — the versioned record one run emits: suite
  identity, execution scope, pinned identities, metrics, controls, a resolution-path
  attribution histogram, and an optional bounded per-token attribution. It serializes
  deterministically (integer-only metrics, fixed field order).
- **Per-token resolution attribution** (`ResolutionPath`, `TokenAttribution`) — see below.
- **Content identity, leakage, tamper, and degenerate-control checks** — pure functions:
  `compute_cid`/`verify_cid` (blake3), `detect_document_leakage`, `is_degenerate_control`.

## Per-token resolution-path attribution

`ResolutionPath` enumerates the seven normative mechanisms a token can come from:
`exact-context`, `ngram`, `graph`, `root-prior`, `patch-delta`, `sampled-selection`, and
`decline`. This is the *path* (the mechanism), a separate axis from the D4 *status*
(`ExactContext`/`Graph`/`Novel`/`Contradictory`): a token has exactly one path.

The deployed `R4Engine` surfaces the served subset directly. `ResolutionPath::from_served`
maps its observable `(PolicyStatus, ngram_hit)` signals: an explicit NGRAM context row is
`ngram`, the EXCT probe is `exact-context`, a graph-tier selection is `graph`, and an
abstention is `decline` (#362 established the NGRAM/EXCT split under the shared `ExactContext`
status). The remaining categories carry their own explicit signals a report producer supplies —
`root-prior` (root base-prior fallback), `patch-delta` (a token from an active patch/delta
chain), and `sampled-selection` (the decode policy sampled from the distribution rather than
taking the argmax).

**Definition (normative scorer binding).** Every *served* production token binds
`NORMATIVE_SCORER_ID` = `uor-r4-graph-format::scoring_semantics@1.0.0`, the deployed R4G1
scoring path ADR-0001 designates. `TokenAttribution::validate` rejects a served token bound to
any other or unnamed scorer; the crate's tests pin the version tail against
`ScoringSemanticsVersion::V1_0_0`, so the identity cannot silently drift from the specification
it names (acceptance criterion 2).

## Identities and the absent-fixture contract

`SuiteIdentities` pins the teacher, tokenizer, corpus, compiler, artifact, decoder, seed,
judge/rubric, hardware, and report identities. **Empirical Criterion (absence is not a value).**
An absent required identity forces the metrics bound to it to `MetricStatus::Unavailable`;
`CapabilityReport::validate_against` rejects a `Measured` primary metric when a required
identity the manifest pins is absent. A missing fixture is `Unavailable`, never a vacuous
`Measured` zero — the #830 rule, enforced structurally here.

Metric values are exact integer fractions (`numerator`/`denominator`), never floats, so a
report serializes byte-reproducibly (the #609–#613 serde-float lesson); confidence intervals
are integer parts-per-thousand.

## Splits, controls, and comparability

Each manifest declares document/domain/template/entity/topology split axes and requires a
leakage check and at least one disjointness axis. `detect_document_leakage` rejects a document
present in both the train and eval partitions; `verify_cid` rejects a tampered fixture (a single
flipped byte fails). Controls (`ExctDisabled`, `PromptSwap`, `SuffixOnly`, `ShuffledEmission`,
`ShuffledState`, `TrivialPrior`, `AlwaysServe`, `AlwaysDecline`) are declared per suite;
`is_degenerate_control` flags a control that fails to separate from the primary (which would
mean the primary reading is not attributable to the capability under test).

**Comparability.** `CapabilityReport::comparable_to` permits a comparison only across identical
workload, identical scoring mode, and the same pinned slice partition. Teacher-forced and
free-running scores are different comparability classes and are **never** merged; a missing
slice identity makes two reports incomparable rather than assumed-equal (acceptance criterion 4,
and the #832 non-goals).

## The committed suites

One manifest per workload; the constitution names a stage's frozen primary suite:

| Stage | Primary suite (id) | Workload | Mode | Primary metric |
|---|---|---|---|---|
| S0 | `s0-broad-text` | broad-text | teacher-forced | held-out-top1 |
| S1 | `s1-causal-prompt-pairs` | causal-prompt-pairs | teacher-forced | causal-influence-delta |
| S2 | `s2-answerability-ood` | answerability-ood | teacher-forced | risk-coverage-auc |
| S3 | `s3-free-running` | free-running | free-running | frozen-horizon-agreement |
| S4 | `s4-compositional-reasoning` | compositional-reasoning | teacher-forced | held-out-composition-accuracy |
| S5 | `s5-instruction-retention` | instruction-retention | teacher-forced | base-capability-retention |
| S6 | `s6-scale` | scale | teacher-forced | throughput-bounded-accuracy |
| S7 | `s7-assistant-canaries` | assistant-canaries | teacher-forced | canary-exact-match |

A ninth workload, continuity-text (`s1-continuity-text`), is a committed secondary suite. The
thresholds in each `promotion_statistic` are described relative to a named control lower bound;
final numeric thresholds are deliberately deferred to each stage's power/baseline study (a #832
non-goal).

## Gate C held-out scoring versus S6 recorded-corpus replay

These two measurements are not the same slice and must not be compared as if they were. **Gate
C** scores a *held-out* partition of a bundle with the compiler-side plain baseline — the
`serving_eval` C row (#280) measures the deployed `R4Engine` on the bundle's own held-out split.
**S6 corpus replay** replays *recorded* corpus positions through the deployed paths against the
bundle's recorded teacher labels in `corpus.meta`/`corpus.records` — no live teacher. Because S6
replays recorded positions rather than scoring a disjoint held-out partition, its figures sit
above Gate C's held-out anchor by construction (the teacher-parity BDD suite documents the same
gap: the S6 corpus-replay scenario reports next to Gate C's anchors, and the two are read on
different slices). The constitution keeps them apart: a `CapabilityReport`'s
`slice_partition_cid` and `mode` bind the slice, and `comparable_to` refuses a cross-slice
comparison.

## Replay and verification commands

```bash
# The constitution's fail-closed tests (parse/validate, reference+production replay,
# leakage/tamper/CID, degenerate control, path-attribution and fixture-absence negatives):
cargo test -p uor-r4-api --offline --test capability_suites_832

# The module's schema unit tests (round-trip, determinism, comparability, scorer-id pin):
cargo test -p uor-r4-api --offline capability_suite

# Register/claim-wording/deferral gates:
cargo xtask check-model
python3 scripts/check_claim_wording.py
cargo xtask audit-deferral
```

The replay reads the committed teacher fixture
`crates/uor-r4-core/tests/fixtures/tless_artifacts.bin`, builds a small synthetic R4G1 bundle
(the recipe shared with `normative_scorer_831.rs`), and replays it through both the reference
(`R4G1Runtime`/`GraphScorer`) and production (`R4Engine`) paths, attributing every production
token. A fixture absent from a real bundle is reported `Unavailable`, so the suite reports its
own coverage rather than skipping into a vacuous green.

## Repository conformance mapping

This is measurement infrastructure that binds existing capabilities; it introduces no new built
RF capability and passes no capability gate, so it adds no `model/ids.toml` row. It maps to and
extends the evidence of RF-09 (graph invariant ownership / loader validation), RF-15 (parallel
reproducibility / deterministic bytes), RF-21 (R4G1 compile quality), RF-22 (R4G1 pathology
filter), RF-23 (R4G1 runtime selection/fallback — the deployed path the attribution binds via
ADR-0001), and RF-29 (teacher parity — the fixture-gated empirical instrument whose absent
fixtures are `UNAVAILABLE`, never PASS).

## Verification

- Schema round-trip and deterministic-report tests (`capability_suite` unit tests;
  `production_report_is_byte_deterministic`).
- Planted document leakage, CID mismatch, missing fixture → `Unavailable`, degenerate control,
  and path-attribution negatives (`capability_suites_832`).
- A committed synthetic fixture replayed through the reference and production paths, attributing
  every token to a normative resolution path and scorer identity.
- Register (`cargo xtask check-model`), claim-wording, and deferral gates.

## Non-goals (honored)

Final phase thresholds are not chosen before the relevant power/baseline study; a model judge is
not used as a primary metric; teacher-forced and free-running scores are never merged; and no
new RF built-capability row is asserted.

## Append-only execution-scope correction (2026-08-24, #933)

The schema, manifests, deterministic serialization, identity validation, leakage checks, and
fixture-absence contract above remain the historical #832 infrastructure result. A later
ADR-0001 call-graph audit found that the synthetic replay and the then-current `serving_eval`
row called `R4Engine`; they did not establish that `R4G1Runtime`, the sole normative
candidate/token selector, was mechanically reached by a deployed surface. Accordingly, the
opening references to a normative deployed `R4Engine`, the production-path description in
the attribution section, and the Gate C/replay descriptions are superseded as execution-scope
claims: those rows are **certifier/reference evidence at off-serving scope** unless a producer
separately binds the exact normative production adapter.

`NORMATIVE_SCORER_ID` remains a schema identity for reports; its presence alone is not
production evidence. A report may credit a served production token only when its producer
records `R4G1Runtime` candidate/token authority and the loaded release/report identities, and
when the relevant deployed surface is mechanically exercised. Historical `R4Engine` rows stay
readable and comparable within their pinned slice, but cannot be reinterpreted as
deployed-serving results.
