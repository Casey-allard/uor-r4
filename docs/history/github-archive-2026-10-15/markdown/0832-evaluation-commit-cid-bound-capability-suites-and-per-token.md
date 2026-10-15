# #832 — evaluation: commit CID-bound capability suites and per-token resolution attribution (item D of #821)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:34:09Z
- **Updated:** 2026-08-20T04:26:45Z
- **Closed:** 2026-08-20T04:25:07Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/832
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #821

## Context

Gate C, teacher-parity, canaries, OOD probes, and corpus replay answer different questions and use different slices. Some suites can skip without live fixtures, and aggregate scores hide which serving path produced each token.

## Problem

Without one evaluation constitution, document-disjoint partitions, pinned judges, powered samples, and path attribution, later improvements can be leakage, ExactContext dominance, decoder effects, vacuous controls, or benchmark drift.

## Goal

Commit versioned capability-suite manifests and a comparable report schema that binds every score to identities, execution scope, uncertainty, controls, and per-token resolution provenance.

## Scope

- [ ] Create CID-bound manifests for broad text, continuity text, assistant canaries, causal prompt pairs, answerability/OOD categories, free-running trajectories, compositional reasoning, instruction/retention, and scale workloads.
- [ ] Define document/domain/template/entity/topology split rules and leakage/tamper checks.
- [ ] Record teacher, tokenizer, corpus, compiler, artifact, decoder, seed, judge/rubric, hardware, and report identities.
- [ ] Report teacher, TLA/plain, R4G1 reference, and production serving results only on comparable slices.
- [ ] Emit per-token ExactContext, NGRAM, graph, root-prior, patch/delta, sampled-selection, and decline attribution.
- [ ] Add EXCT-disabled, prompt-swap, suffix-only, shuffled-emission/state, trivial-prior, always-serve, and always-decline controls as applicable.
- [ ] Specify sample-size/power, confidence intervals, multiple-seed aggregation, null/unavailable encoding, and primary versus secondary metrics.
- [ ] Provide replay commands and a schema/version policy for evaluate-report/capability records.

## Non-goals

- Choose final phase thresholds before the relevant power/baseline study.
- Replace deterministic primary metrics with an unversioned model judge.
- Merge incomparable teacher-forced and free-running scores.

## Dependencies and entry gate

Uses the normative-path decision in item C for production attribution and the conformance status vocabulary in item B. It may draft manifests before those land.

## Acceptance criteria

- [ ] Every stage can name a frozen primary suite, split, control set, report schema, and promotion statistic.
- [ ] Every reported production token is attributable to a normative resolution path and scorer identity.
- [ ] Fixture absence is visible as UNAVAILABLE, and a planted leakage/tamper case is rejected.
- [ ] Same-slice reports can compare teacher/reference/production without changing corpus partition or metric semantics.

## Verification

- [ ] Schema round-trip and deterministic-report tests.
- [ ] Planted document leakage, CID mismatch, missing fixture, degenerate control, and path-attribution negative tests.
- [ ] Replay a small committed fixture through reference and production paths.
- [ ] Run conformance/register and claim-wording checks.
- [ ] Document the distinction between Gate C held-out scoring and S6 recorded-corpus replay.

## Compatibility and migration

Version manifests/reports additively. Older reports remain readable but are marked incomplete for any newly required identity, scope, or attribution field.

## Repository conformance

- **Execution scope:** Offline evaluation/certification plus measured reachability to the normative deployed R4Engine path. Evidence outside this scope is not credited as a deployed-serving result.
- Preserve the applicable P-4, zero-allocation steady state, deterministic-byte, bounded-work, focused typed-error, `no_std`, and `forbid(unsafe_code)` contracts.
- Classify normative mathematics under `docs/formal_vocabulary.md`; empirical criteria name distribution, protocol, sample count, uncertainty, provenance identities, and status.
- **Conformance mapping:** RF-09, RF-15, RF-21, RF-22, RF-23, RF-29.. Reuse/extend an existing capability unless a genuinely new capability requires a new RF ID.
- A new built capability lands in order: `model/ids.toml` row → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- `CONFORMANCE.md` is generated and must not be edited directly. Retained off-serving mechanisms stay `open` in `model/ledger.toml` with an activation gate.
- Measurements that revise a claim append a per-issue record and reconcile README, ROADMAP, RESEARCH, lifecycle/configuration, and ledger assertions.


## Claim status and next action

This builds trustworthy measurement infrastructure; it does not itself pass any capability gate.

## Comments (2)

### comment 5350938806 — 2026-08-20T03:19:10Z — Casey-allard

Claiming this (item D of #821). Branch `issue-832-cid-bound-capability-suites`.

**Execution contract (scope → deliverable).** This lands the evaluation *constitution* as versioned schema + committed manifests + per-token attribution + fail-closed tests, mirroring the `release_bundle.rs` schema-only pattern (#655-C0). It is offline evaluation/certification infrastructure and **passes no capability gate** (per this issue's own claim status).

- **Schema module** `crates/uor-r4-api/src/capability_suite.rs` — versioned (`CAPABILITY_SUITE_SCHEMA=1`), `deny_unknown_fields`, `validate() -> Option<String>` (no new error type, R5), additive `Option<T>` fields. Filesystem-free at the library edge.
- **Committed manifests** `crates/uor-r4-api/capability_suites/*.json` for the nine workloads (broad text, continuity, assistant canaries, causal prompt pairs, answerability/OOD, free-running, compositional reasoning, instruction/retention, scale) + a `constitution.json` stage index (S0–S7 → frozen primary suite/split/control/schema/promotion-statistic). Embedded via `include_str!`.
- **Per-token attribution** `ResolutionPath` ∈ {ExactContext, Ngram, Graph, RootPrior, PatchDelta, SampledSelection, Decline}, derived from the deployed `R4Engine` `PredictDecision`/`PolicyStatus`/`ngram_hit` and bound to the normative scorer identity from #831 (ADR-0001; `scoring_semantics` v1.0.0). A production token with no normative scorer id is rejected.
- **Discipline as data/enums:** document/domain/template/entity/topology split rules + leakage/tamper/CID checks (blake3); eight controls (EXCT-disabled, prompt-swap, suffix-only, shuffled-emission/state, trivial-prior, always-serve, always-decline) + degenerate-control detection; metrics as integer rationals with sample-n/CI(permille)/primary-vs-secondary; `MetricStatus::Unavailable` for absent fixtures (never a vacuous value); comparability requires identical slice-partition CID + scoring mode (teacher-forced and free-running never merged).
- **Tests** `crates/uor-r4-api/tests/capability_suites_832.rs`: schema round-trip + deterministic report; planted document leakage; CID mismatch; missing fixture → UNAVAILABLE; degenerate control; path-attribution negative; and a replay of a small committed synthetic fixture through the **reference** (`R4G1Runtime`/`GraphScorer`) and **production** (`R4Engine`) paths, attributing each token. Manifest+constitution coverage and comparability tests.
- **Docs** `docs/capability_suites_832.md` (record, RF-09/15/21/22/23/29 mapping, replay commands, schema/version policy, Gate C held-out vs S6 recorded-corpus replay distinction, claim status) + a one-line reconcile in `docs/RESEARCH.md`.

**Non-goals honored:** no final phase thresholds; no unversioned model judge as a primary metric; teacher-forced and free-running scores never merged; no new RF built-capability row (this binds existing capabilities).

**Verification:** `cargo test --workspace --no-fail-fast`, `clippy --all-targets --all-features -D warnings`, `fmt --check`, graph-format no-default/alloc checks, `scripts/check_claim_wording.py`, `cargo xtask check-model` (register), `cargo xtask audit-deferral`. No core/router code touched, so no wasm-target change.


### comment 5351353656 — 2026-08-20T04:26:45Z — Casey-allard

## Definition of Done — completed

Merged via the protected merge queue: PR #863, squash merge commit `70586aa0ee4d104faaca2dfbe8cc88d645837fef` on `main` (2026-08-20T04:25:06Z). Full merge-group ladder green (tests, no_std, deterministic rebuild, κ-reproduction, Gate C trend, wasm, fuzz; 16m56s). PR-side register conformance (R1–R6), fmt/clippy/tests/no_std/κ, cargo audit, and Kani all passed.

**Acceptance criteria (all met, at infrastructure scope — this passes no capability gate, per the issues claim status):**
- Every stage S0–S7 names a frozen primary suite, split, control set, report schema, and promotion statistic — committed `Constitution` + nine `SuiteManifest`s.
- Every reported production token is attributable to a normative `ResolutionPath` and scorer identity — `TokenAttribution` binds `NORMATIVE_SCORER_ID` (ADR-0001 / #831, `scoring_semantics` v1.0.0); a served token bound to any other scorer is rejected.
- Fixture absence is `Unavailable`, never a value; planted leakage/CID tamper rejected.
- Same-slice comparability enforced (workload + mode + slice-partition CID); teacher-forced and free-running never merged.

**Evidence artifacts:**
- `crates/uor-r4-api/src/capability_suite.rs` (schema + attribution + checks), `crates/uor-r4-api/capability_suites/*.json` (nine workload manifests + constitution index).
- `crates/uor-r4-api/tests/capability_suites_832.rs` — reference (`R4G1Runtime`/`GraphScorer`) + production (`R4Engine`) replay of a committed synthetic fixture attributing every token; planted leakage, CID mismatch, missing-fixture→`Unavailable`, degenerate-control, and path-attribution negatives. Module unit tests cover round-trip/determinism, comparability, and the scorer-id-pins-`ScoringSemanticsVersion::V1_0_0` guard.
- `docs/capability_suites_832.md` — record (RF-09/15/21/22/23/29 mapping, Gate C held-out vs S6 recorded-corpus replay distinction, replay commands, schema/version policy, claim status); reconcile line in `docs/RESEARCH.md`.

**Conformance:** no new RF row / no `CONFORMANCE.md` change — this binds existing capabilities (register CM-01 stays green at 29 ids). **Compatibility:** additive schema versioning; older reports readable, marked incomplete for newly required fields. No new RF built-capability row; teacher/reference/production numbers on a live bundle are downstream measurement, intentionally not produced here.

Closing verdict: **COMPLETED**. Next sequential S0 item is #833 (item E), now unblocked (its blockers #830/#831/#832 are all closed).



---
_Comment/review ids in this file: 5350938806, 5351353656_

