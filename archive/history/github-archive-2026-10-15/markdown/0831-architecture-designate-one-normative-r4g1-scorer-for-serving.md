# #831 — architecture: designate one normative R4G1 scorer for serving, certification, patches, and proofs (item C of #821)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:34:08Z
- **Updated:** 2026-08-20T02:52:40Z
- **Closed:** 2026-08-20T02:50:16Z
- **Labels:** enhancement, conformance
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/831
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #821

## Context

The repository contains `uor-r4-api::R4Engine`, compiler/certifier scoring, and `uor-r4-graph-runtime::R4G1Runtime`. Their useful reference and portable roles are not currently expressed as one unambiguous normative production semantics.

## Problem

Quality results, patch behavior, witnesses, and proof obligations can be measured on one scorer and discussed as though they automatically apply to another. Silent semantic drift would invalidate downstream research gates.

## Goal

Choose and document one normative step/state/witness semantics, route every production consumer through it or label alternatives reference-only, and make divergence fail closed.

## Scope

- [ ] Inventory R4Engine, GraphScorer, R4G1Runtime, certifier, CLI/server/WASM, patch, witness, and proof-model consumers with exact reachability.
- [ ] Write an ADR selecting the normative scorer, state transition, tie-break, saturation, decline, patch precedence, and witness semantics.
- [ ] Converge duplicate production semantics or give non-equivalent implementations distinct identities and claim boundaries.
- [ ] Add per-token resolution/contribution attribution for ExactContext, NGRAM, graph evidence, root prior, patch/delta, decoder selection, and typed decline.
- [ ] Create differential fixtures for each path, boundary capacities, corrupt/incompatible sections, and deterministic ties.
- [ ] Add planted-negative mutations that prove the differential and reachability tests detect divergence.
- [ ] Route serving, certification, patch interpretation, witness replay, and proof targets through the decision or document scoped exceptions.

## Non-goals

- Improve model quality or change the selected decoding policy.
- Remove valuable reference implementations when clear scoping is sufficient.
- Call bit-different scorers equivalent.

## Dependencies and entry gate

Conformance scope fields from sibling item B should land with or immediately after the ADR. Coordinate with format/runtime owners before any ABI change.

## Acceptance criteria

- [ ] Exactly one implementation/specification is named normative for deployed inference.
- [ ] All production entry points and certificates demonstrate reachability to that semantics.
- [ ] Reference/certifier-only implementations are explicitly scoped and differentially tested.
- [ ] Patch and witness semantics have one normative owner and incompatible artifacts fail closed.
- [ ] A planted semantic divergence fails the relevant test.

## Verification

- [ ] Run focused scorer differential, patch-chain, validation, and witness-replay suites.
- [ ] Run `cargo test --workspace --no-fail-fast --offline`, clippy, fmt, graph-format no_std ladder, and deterministic rebuild checks.
- [ ] Run `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib` for affected core/runtime code.
- [ ] Run allocation census and P-4 operation scan on the normative path.
- [ ] Exercise actual CLI/API/server admission and prediction, not only R4G1Runtime or certifier helpers.

## Compatibility and migration

Prefer an ADR and convergence without format changes. If bytes or semantics change, version the relevant section/operator, preserve historical artifact behavior where possible, and publish an explicit migration/recompile decision.

## Repository conformance

- **Execution scope:** Normative deployed R4Engine serving path; alternate scorers only as explicitly named reference/certifier scopes. Evidence outside this scope is not credited as a deployed-serving result.
- Preserve the applicable P-4, zero-allocation steady state, deterministic-byte, bounded-work, focused typed-error, `no_std`, and `forbid(unsafe_code)` contracts.
- Classify normative mathematics under `docs/formal_vocabulary.md`; empirical criteria name distribution, protocol, sample count, uncertainty, provenance identities, and status.
- **Conformance mapping:** RF-09, RF-15, RF-21, RF-22, RF-23.. Reuse/extend an existing capability unless a genuinely new capability requires a new RF ID.
- A new built capability lands in order: `model/ids.toml` row → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- `CONFORMANCE.md` is generated and must not be edited directly. Retained off-serving mechanisms stay `open` in `model/ledger.toml` with an activation gate.
- Measurements that revise a claim append a per-issue record and reconcile README, ROADMAP, RESEARCH, lifecycle/configuration, and ledger assertions.
- Differential tests cover candidate/reference and packed/deployed forms plus a planted negative.

## Claim status and next action

This establishes semantic ownership and reachability, not quality. Downstream reports must name the normative scorer identity.

## Comments (1)

### comment 5350621837 — 2026-08-20T02:52:40Z — Casey-allard

## Definition of Done — completed

Merged via **PR #862** → `main` merge commit **`1c377df7fcc3fffca30f32db5aeec4df32d89131`** (merge queue, full ladder green, 2026-08-20). Verified present on `main`: the ADR, the test, and the regenerated `CONFORMANCE.md` RF-23 evidence.

**Outcome:** one normative scorer is designated and the reference/certifier scorers are given distinct identities with fail-closed divergence — removing the silent-drift hazard where a certifier (`GraphScorer`/Gate C) measurement was read as a served-path result while `R4G1Runtime` actually selects the served token.

**Decision (`docs/adr/0001-normative-r4g1-scorer.md`):** the deployed **R4G1 runtime scoring path** (`uor-r4-graph-runtime` `R4G1Runtime` + `scoring`), specified by `uor-r4-graph-format::scoring_semantics` v1.0.0 and constrained by `docs/inference_contract.md`, is the one normative scorer for deployed inference. Tie-break (ScoreQ desc, id asc), saturating accumulation, no-double-counting, the single `ScoreStatus` scorer-status space, the patch owner (`try_push_patch`, fail-closed), and the witness owner (replay mismatch fail-closed) are named. `GraphScorer` (reference/certifier + D4 policy resolver), the Gate C harness, the retired `score_candidates_legacy`, and `uor-r4-router` are explicitly scoped out of the served-selection role.

**Acceptance criteria — all met:**
- ✅ Exactly one implementation/specification named normative for deployed inference (ADR §Decision.1).
- ✅ All production entry points + certificates demonstrate reachability (ADR §Reachability; test `reference_and_normative_reachable_from_same_artifact`).
- ✅ Reference/certifier-only implementations explicitly scoped and differentially tested (ADR §Decision.2; `spec_and_runtime_accumulators_agree_on_disjoint_sets`, `spec_and_runtime_selectors_agree`, `no_double_counting_enforced_by_both`).
- ✅ Patch and witness semantics have one normative owner; incompatible artifacts fail closed (`incompatible_artifacts_fail_closed`).
- ✅ A planted semantic divergence fails the relevant test (`planted_divergence_is_detected`).

**Evidence:** `crates/uor-r4-api/tests/normative_scorer_831.rs` (7 tests, default suite); `docs/normative_scorer_831.md` (per-issue record). Conformance: `model/ids.toml` RF-23 evidence extended to cite the ADR; `CONFORMANCE.md` regenerated (generated, not hand-edited). Docs reconciled: `ROADMAP.md`, `docs/RESEARCH.md`.

**Verification:** `fmt --check`; `clippy --workspace --all-targets --all-features -D warnings`; graph-format no_std ladder; `check_claim_wording.py`; register conformance R1/R2/R3/R4/R5/R6; wasm-router lib check; full `cargo test --workspace` — all green locally, and the merge-queue full ladder (the binding verdict) passed.

**Scope boundary / claim status:** establishes semantic ownership + reachability, not model quality; does not claim the scorers are exactly equivalent. No format/ABI change (convergence is a non-goal). Per-token **CID-bound attribution suites** remain scoped to sibling **#832** (the ADR names the attribution owner; the suites are its deliverable).

S0 tracker **#821** progress: item C complete → **3/5 children done** (#829, #830, #831); next sequential item is **#832** (item D).



---
_Comment/review ids in this file: 5350621837_

