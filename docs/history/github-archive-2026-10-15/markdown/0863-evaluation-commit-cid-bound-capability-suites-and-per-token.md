# #863 — evaluation: commit CID-bound capability suites and per-token resolution attribution (#832)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T04:00:12Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-20T04:25:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/863
- **Merged:** 2026-08-20T04:25:06Z
- **Merge commit:** 70586aa0ee4d104faaca2dfbe8cc88d645837fef
- **Base / Head:** main / UOR-Foundation:issue-832-cid-bound-capability-suites
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Gate C, teacher parity, the assistant canaries, the OOD probes, and corpus replay answer different questions on different slices; some suites skip when a live fixture is absent; and an aggregate score hides which serving path produced each token. Without one evaluation constitution — document-disjoint partitions, pinned identities, powered samples, declared controls, and per-token path attribution — a later "improvement" can be leakage, ExactContext dominance, a decoder effect, a vacuous control, or benchmark drift.

This PR commits that constitution as a versioned schema plus committed manifests, mirroring the schema-first shape of `release_bundle.rs` (#655-C0). It is offline evaluation/certification infrastructure and, per the issue's own claim status, **passes no capability gate**.

## Scope and non-goals

**In scope (all delivered):** CID-bound manifests for the nine workloads + a constitution index; document/domain/template/entity/topology split rules with leakage/tamper checks; the ten identity slots (teacher, tokenizer, corpus, compiler, artifact, decoder, seed, judge, hardware, report); a comparable report schema that reports teacher/reference/production only on comparable slices; per-token ExactContext/NGRAM/graph/root-prior/patch-delta/sampled-selection/decline attribution; the eight controls; sample-size, integer-CI, primary-vs-secondary, and null/unavailable encoding; replay commands and a schema/version policy.

**Non-goals (honored):** no final phase thresholds before the relevant power/baseline study; no unversioned model judge as a primary metric; teacher-forced and free-running scores are never merged; and no new RF built-capability row (this binds existing capabilities).

## Acceptance-criteria status

- **Every stage names a frozen primary suite, split, control set, report schema, promotion statistic** — ✅ `Constitution` maps S0–S7 to committed `SuiteManifest`s; `committed_constitution_validates_and_names_a_primary_suite_per_stage`.
- **Every reported production token is attributable to a normative resolution path and scorer identity** — ✅ `TokenAttribution::from_decision` binds `NORMATIVE_SCORER_ID` (ADR-0001 / #831, `scoring_semantics` v1.0.0); a served token bound to any other scorer is rejected; `production_replay_attributes_every_token_to_the_normative_scorer`.
- **Fixture absence is visible as UNAVAILABLE, and a planted leakage/tamper case is rejected** — ✅ `MetricStatus::Unavailable`; `validate_against` rejects a `Measured` primary with an absent required identity; `detect_document_leakage`/`verify_cid`.
- **Same-slice reports compare teacher/reference/production without changing partition or metric semantics** — ✅ `CapabilityReport::comparable_to` requires identical workload, mode, and slice-partition CID.

The report schema is delivered and enforced; populating teacher/TLA-plain/reference/production **numbers** on a real bundle is downstream measurement (later stages), not this infrastructure commit.

## Tests and verification

- Module unit tests (schema round-trip + byte-deterministic report, comparability, CID/leakage/degenerate-control, attribution mapping, and the scorer-id-pins-`ScoringSemanticsVersion::V1_0_0` guard).
- `crates/uor-r4-api/tests/capability_suites_832.rs`: constitution parse/validate + per-stage primary; a committed synthetic fixture replayed through the **reference** (`R4G1Runtime`/`GraphScorer`) and **production** (`R4Engine`) paths attributing every token; planted document leakage; CID mismatch; degenerate control; path-attribution negative; and missing-fixture → `Unavailable`.

Gates run locally, all green: `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -D warnings`; `cargo test` across all crates **except** the root live-teacher `bdd` teacher-parity slow mode (independent of this leaf-crate change; re-run as the binding verdict in the merge queue); `cargo check -p uor-r4-graph-format --no-default-features` and `--features alloc`; `python3 scripts/check_claim_wording.py`; `cargo xtask check-model` (register CM-01) and `cargo xtask audit-deferral` (R4). No `uor-r4-core`/router code touched, so no wasm-target change.

## Evidence artifacts

- `docs/capability_suites_832.md` — the record: attribution taxonomy + normative-scorer binding, identity/UNAVAILABLE contract, splits/controls/CID discipline, comparability, the committed-suites table, the **Gate C held-out vs S6 recorded-corpus replay** distinction, replay commands, schema/version policy, RF mapping, and claim status.
- `crates/uor-r4-api/capability_suites/*.json` — the committed manifests + constitution.

## Compatibility and migration

Additive. `CAPABILITY_SUITE_SCHEMA` and `CAPABILITY_REPORT_SCHEMA` version manifests and reports independently; new fields are `Option<T>` with `skip_serializing_if`. An older report stays readable but is marked incomplete for any newly required identity, scope, or attribution field. No existing type, format, or serving path changes.

## Conformance effects

No `model/ids.toml` change and no `CONFORMANCE.md` regeneration: this is measurement infrastructure that binds existing capabilities, not a new built capability, so it adds no RF row (`check-model` CM-01 stays green at 29 ids). It maps to and extends the evidence of RF-09, RF-15, RF-21, RF-22, RF-23 (the deployed path the attribution binds via ADR-0001), and RF-29 (the fixture-gated instrument whose absent fixtures are UNAVAILABLE, never PASS) — documented in the record.

## Claim-boundary changes

None strengthened. The record is Definition/Assumption-scope and states plainly that the constitution passes no capability gate and asserts no empirical result. Claim language follows `docs/formal_vocabulary.md`; the claim-wording gate passes.

## Negative or unavailable results

By design, the schema makes absence first-class: a missing fixture is `Unavailable`, a tampered fixture fails `verify_cid`, a leaked document fails `detect_document_leakage`, and a degenerate control is flagged — each covered by a negative test. The replay uses a small committed synthetic fixture (the #831 recipe); real teacher/TLA-plain/reference/production numbers on a live bundle are downstream and intentionally not produced here.

## Intentionally excluded files or work

Untracked `obs-text/` and `tf1-pkg/` are left unstaged (user-owned; `tf1-pkg` never goes to GitHub). No generated files edited. No new RF row / Gherkin suite / marker test (no new built capability).

Closes #832

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

