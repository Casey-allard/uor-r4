# #839 — runtime/serving: execute and certify selective prediction across production surfaces (item C of #823)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:36:32Z
- **Updated:** 2026-08-25T19:23:43Z
- **Closed:** 2026-08-25T18:52:36Z
- **Labels:** enhancement, conformance
- **Milestone:** R4 Intelligence / S2 — Selective prediction
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/839
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #823

## Context

A positive calibrator from sibling item B must be encoded and executed in the normative R4Engine path. Reference/certifier success cannot establish identical CLI, server, SDK-compatible, streaming, and WASM behavior.

## Problem

Missing or corrupt calibration, surface-specific fallback, threshold drift, or bypass of the normative scorer could silently turn an intended decline into a generated answer.

## Goal

Implement a validated, bounded selective-decision section and return the same typed, witnessed result on every supported production surface; certify the frozen risk/coverage operating point without retuning.

## Scope

- [ ] Record the exact positive activation report; close `NOT TRIGGERED` if sibling item B selects no model.
- [ ] Define a versioned optional calibration section containing feature schema, quantization, threshold/operating point, source/report CIDs, and bounds.
- [ ] Implement structural/semantic validation and fail closed on missing-required, corrupt, incompatible, overflowed, or identity-mismatched calibration.
- [ ] Execute feature extraction and decision in the #831 normative scorer using bounded fixed-point/table operations.
- [ ] Extend witnesses with evidence counts/provenance, feature values, threshold identity, decision, and saturation/missing flags.
- [ ] Map the same typed outcome across CLI, native API/server, OpenAI-compatible streaming/non-streaming, and supported WASM.
- [ ] Prevent hidden teacher/network fallback and prove streaming cannot emit answer tokens before a required decline decision.
- [ ] Run the untouched-domain benchmark at frozen thresholds and issue the production-scope certificate.

## Non-goals

- Retune thresholds on release/test data.
- Return untyped empty text for decline.
- Allow older artifacts to inherit a calibration claim.

## Dependencies and entry gate

Triggered only by a positive sibling item B verdict; blocked by #831 normative path and sibling item A surface/status contract.

## Acceptance criteria

- [ ] Packed/reference and all production surfaces return identical decisions and materially equivalent witnesses.
- [ ] Malformed/missing-required/incompatible calibration fails closed with focused errors.
- [ ] P-4, allocation, no_std, bounded-state, determinism, and latency budgets pass.
- [ ] Frozen production risk/coverage and answerable-novel retention thresholds pass with declared confidence.
- [ ] Capability manifests and public claims name exact operating point, artifact/report CIDs, scope, and limitations.

## Verification

- [ ] Format round-trip, two-stage validation, corrupt/truncated/range/identity negatives, and fuzz smoke.
- [ ] Reference/packed/production differential suite with planted threshold, feature, and bypass mutations.
- [ ] Workspace no-fail-fast tests, clippy, fmt, no_std ladder, wasm target, deterministic rebuild.
- [ ] Allocation census, P-4 scan, witness replay, streaming-before-decision and no-network tests.
- [ ] Frozen untouched-domain benchmark replay with no retuning and independent interval recomputation.

## Compatibility and migration

Old artifacts retain their documented legacy decision behavior and are explicitly uncalibrated. Protocol additions are backward-compatible where possible; incompatible clients receive a documented typed fallback/error, never an invented confidence.

## Repository conformance

- **Execution scope:** Normative deployed R4Engine path across CLI, native/OpenAI-compatible server, supported SDK surface, and WASM; off-scope evidence is not a serving result.
- Preserve applicable P-4, zero-allocation, deterministic-byte, bounded-work/state, typed-error, no_std, and witness contracts.
- Empirical claims name distribution, protocol, n, uncertainty, provenance, operating point, and PASS/FAIL/UNAVAILABLE status.
- **Conformance mapping:** RF-01, RF-22, RF-23 plus the new selective-prediction capability created by the implementation.. Add a new selective-prediction ID only when the typed capability is actually built.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- Generated conformance is never hand-edited; rejected/off-serving arms remain scoped in the ledger.
- Claim changes append an evidence record and reconcile every live public assertion.
- Verification must prove real production-path reachability; R4G1Runtime or certifier-only tests are insufficient.

## Claim status and next action

Final disposition is `PROMOTE`, `REVISE`, `D4-COVERAGE-ONLY`, or `NOT TRIGGERED`. State the measured risk/coverage boundary, not “understands unknowns.”

## Comments (4)

### comment 5368815523 — 2026-08-21T10:42:27Z — Casey-allard

### Maintainer re-scope (2026-08-21): execute the typed contract in legacy-coverage mode now; calibrated mode stays gated

The pre-registered activation condition — a selected calibrator from #837 — did not occur (`NO CALIBRATOR ESTABLISHED`, PR #893). Rather than idle this issue, the maintainer decision splits it:

**Phase 1 — executable now (this issue's live scope):** lower the #838 frozen contract's **legacy-coverage mode** onto the production surfaces:

- typed abstention records on the CLI and native HTTP surfaces (outcome `abstention`, cause `distributionally-novel`, coverage axis populated, **no fabricated confidence or evidence fields** — spec §6);
- the OpenAI-compatible surface migrates the generic `engine_declined` error to the typed `uor_abstention_*` / `uor_incompatible_artifact` codes — non-streaming structured error bodies (422/409), streaming terminal typed event then `[DONE]`; never an empty-`choices` success, never a silent stream end (spec §5);
- the typed WASM boundary; **fail-closed** corrupt-calibration handling (`hard-incompatibility`; corrupt never silently degrades to legacy);
- schema/status round-trip certification across all surfaces per spec §5 — this also binds the #841 streaming/non-streaming trace-equivalence Assumption to the transports themselves.

If phase 1 creates a genuinely new served capability, the built-capability order applies (ids.toml → tagged Gherkin → failing marker/behavior test → implementation → regenerated CONFORMANCE.md) — the first S2 item that is NOT a research leaf; judged at implementation time.

**Phase 2 — gated, unchanged:** calibrated-mode execution activates only when a redesigned calibrator clears the frozen release gate (false-answer UCB95 ≤ 10‰ at coverage ≥ 20‰) on the #838 constitution. The gate does not move.


### comment 5369769133 — 2026-08-21T12:24:37Z — Casey-allard

### Phase-1 execution contract (per the 2026-08-21 re-scope): lowering the #838 legacy-coverage mode onto the production surfaces

**Scope checklist (this PR series):**

1. **Built-capability judgment: phase 1 DOES create a new served capability** — the typed selective-prediction surface contract (legacy-coverage mode) on the deployed serving path. The required order applies: new `model/ids.toml` row (scope `normative-runtime`, reachability `deployed-serving`) → tagged Gherkin suite (`features/suites/selective_prediction_surfaces.feature`) → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
2. **CLI (`r4 chat`/`ask`, `src/chat.rs`):** `ChatAbstention` extended with the typed §5 record — outcome `abstention`, cause `distributionally-novel` (the only legacy cause; the D4 `Novel` reading), coverage populated; raw D4 label and `widened` retained as compatibility fields; abstention stays exit 0 (honest success). Hard incompatibility (calibration seam, below) is a typed error, nonzero exit.
3. **Native HTTP (`src/server.rs`):** additive §5 fields on the served and abstention responses — `status`, `coverage`, `cause`, `confidence_permille: null`, `evidence: null` in legacy mode (never fabricated); existing `outcome`/`r4g1.*` fields retained for compatibility. Hard incompatibility → `409` with `status: "hard-incompatibility"`. Recorded limitation: `coverage` is populated from the D4 signal when the policy engine produced a reading and is `null` on auxiliary tiers that bypass D4 (absence is absence).
4. **OpenAI-compatible (`src/server.rs`):** abstention declines migrate from the generic 503 `engine_declined` to **HTTP 422, `error.type = "uor_selective_prediction"`, `error.code = "uor_abstention_distributionally_novel"`** (snake rewrite of the kebab cause) + coverage; never an empty-`choices` success. Streaming: an abstention emits **no content chunk** — one terminal typed SSE `error` event with the same code, then `data: [DONE]`; never a silent stream end. Hard incompatibility → 409 / `uor_incompatible_artifact` on both forms. Non-abstention declines (unrecognized engine, profile restriction, model-changed races, all-tiers-failed-without-abstention) are NOT abstentions and keep their existing envelopes.
5. **WASM boundary:** an additive typed export carrying the canonical labels (`served`/`abstained{cause, coverage}`/`incompatible`) — a typed value, never a trap; the legacy `Option<String>` export is retained unchanged.
6. **Fail-closed calibration seam (§6):** the versioned optional selective-prediction calibration sidecar identity is defined (`selective_calibration.bin` beside the active artifact — DISTINCT from the compiler-side `hamming_calibration.json`, which is a hamming-radius conversion input and is untouched). ABSENT → legacy-coverage mode (today's D4 behavior through the typed schema). PRESENT → `hard-incompatibility` on every surface in phase 1 (no executable calibrated mode exists yet; corrupt or unknown-version especially must never silently degrade to legacy). Phase 2's valid calibrated mode replaces the present-arm when its gate clears.
7. **§5 round-trip certification tests** on the production encoders: injectivity + byte-determinism per surface, streaming/non-streaming typed-content equivalence for the same decision (binding the #841 transport Assumption), and the migrated G4-family expectations (the old `engine_declined`-on-abstention tests encode the pre-#838 contract and are updated to the typed one — that update IS the migration, per spec §5's "migration ancestor" note).

**Non-goals:** calibrated-mode execution (phase 2; the 10‰ @ ≥20‰ release gate does not move); evidence/confidence fields in legacy mode (never fabricated); threshold work; any change to D4 policy semantics (served output stays byte-identical on Serve); touching `hamming_calibration.json` semantics.

**Verification plan:** new Gherkin-tagged behavior tests + marker test; migrated server in-file tests; `cargo test --workspace --no-fail-fast`; clippy all-targets; fmt; **wasm build check** (the root lib is the wasm-pack surface and is directly modified); `status_policy` + `status_policy_census` (deployed policy block touched — allocation/P-4 census must stay green); claim-wording; R1 `check-model` + regenerated `CONFORMANCE.md` determinism; no_std ladder untouched.

**Claim boundary:** this phase certifies the typed SCHEMA (Guarantee-class structural properties: typed, deterministic, fail-closed, injective per surface) in legacy-coverage mode. It makes NO semantic-answerability claim (coverage is not semantic PASS, spec §12) and mints no confidence.


### comment 5371007955 — 2026-08-21T14:13:35Z — Casey-allard

### Phase 1 delivered (2026-08-21): typed selective-prediction surfaces live in legacy-coverage mode — RF-30

Merged via PR #900 → main `c205eb7b` (first queue pass failed on unmatched cucumber steps for the new suite — all 1,666 unit tests were green; fixed with executable step definitions `9ab21093`, rebased `b4c7afe5`, second pass fully green).

Delivered against the phase-1 execution contract (comment above):

- **Built-capability order end-to-end:** `model/ids.toml` **RF-30** (scope `normative-runtime`, reachability `deployed-serving`) → `features/suites/selective_prediction_surfaces.feature` (6 scenarios with executable steps) → marker `selective_prediction_surfaces_rf_30` + behavior tests → implementation → regenerated `CONFORMANCE.md` (30 ids, CM-01).
- **CLI:** typed `ChatAbstention` (outcome/cause/coverage labels; abstention stays exit 0) + fail-closed load seam (typed error, nonzero exit).
- **Native HTTP:** additive spec-§5 `selective` block on served and declined bodies; 409 typed hard-incompatibility; coverage `null` when the D4 signal is absent (absence is absence, never fabricated).
- **OpenAI-compatible:** abstentions = HTTP 422 `error.type "uor_selective_prediction"` / `error.code "uor_abstention_distributionally_novel"` (+ coverage/cause); streaming declines = one terminal typed SSE `error` event then `[DONE]` (chat completions) or a terminal typed event (responses); never an empty-`choices` success, never a silent stream end; non-abstention declines keep their historical `engine_declined` envelopes.
- **WASM:** `typed_r4g1_response` typed boundary export (never a trap; the abstained arm is declared vocabulary, reachable when the policy engine reaches the wasm path — recorded limitation).
- **Fail-closed calibration (spec §6):** `selective_calibration.bin` present ⇒ `hard-incompatibility` on every surface; absent ⇒ legacy-coverage mode; the compiler-side `hamming_calibration.json` is untouched.

**Verification:** full queue ladder green on the speculative merge (workspace tests incl. the RF-30 bdd scenarios, no_std ladder, deterministic rebuild, κ, wasm, fuzz smoke, Gate C trend, audit) plus local fmt / clippy `-D warnings` all-targets / lib (270) + bin (8) suites / `status_policy` (14) + census / wasm target / R1–R3 / `xtask validate` / claim wording.

**Claim boundary:** the typed SCHEMA is certified (Guarantee-class structural properties: typed, deterministic, injective, fail-closed) in legacy-coverage mode. No semantic-answerability claim is made and no confidence is minted (spec §12).

**Phase 2 (calibrated mode) stays gated, unchanged:** it activates only when a redesigned calibrator clears the frozen release gate (false-answer UCB95 ≤ 10‰ at coverage ≥ 20‰) on the #838 constitution — the S1-redesign-fed re-entry per RFC §5 / #897. Unassigning; this issue remains open as the phase-2 home.


### comment 5415178300 — 2026-08-25T18:52:35Z — Casey-allard

### Final disposition — phase 1 delivered; calibrated phase `NOT TRIGGERED`

Closing this leaf after the one sanctioned S2 re-entry completed.

- **Phase 1: DELIVERED.** PR #900 shipped RF-30 typed selective-prediction surfaces in legacy-coverage mode across CLI, native HTTP, OpenAI-compatible streaming/non-streaming, and WASM, including fail-closed `selective_calibration.bin` handling. Its protected queue ladder passed.
- **Phase 2: `NOT TRIGGERED`.** #931 / PR #938 returned **`NO CALIBRATOR ESTABLISHED`** under the unchanged #838 gates. No candidate met the release or research gate on CAL; TEST remained sealed (`test_opened=false`); semantic abstention remains **NOT ESTABLISHED**. Result CID: `blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`.
- **Production claim boundary:** RF-30 legacy-coverage mode remains the only selective-prediction mode. It reports coverage and typed decline/incompatibility outcomes; it does not mint calibrated confidence, evidence, or semantic answerability. Present calibration remains fail-closed.
- **No hidden follow-on:** this closure does not authorize another calibrator attempt or move the frozen thresholds. Any re-entry requires a new explicit maintainer decision.

Evidence: #838 contract/constitution; #837 initial negative fit; #931 content-evidence re-entry; PR #900 phase-1 delivery; PR #938 final audited verdict and full protected-queue verification.

Disposition: phase 1 complete, phase 2 `NOT TRIGGERED`; the parent #823 can now record its final stage verdict.


---
_Comment/review ids in this file: 5368815523, 5369769133, 5371007955, 5415178300_

