# #606 — certify: add target-operator recompilation certificates without duplicating Gate C

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T22:01:45Z
- **Updated:** 2026-08-12T21:26:15Z
- **Closed:** 2026-08-12T21:26:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/606
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The repository has mature certification pieces: Gate C scoring, teacher-parity BDD, runtime operation/allocation certificates, deterministic rebuilds, and witness checks. Target-operator recompilation needs one bounded certificate report that composes those existing gates without claiming that a compiled artifact is a quality success.

# Current evidence

- Merged PR #307 records teacher-forced top-1/top-8/Δbits, generation speed, zero-multiply/divide, zero-allocation, witness consistency, and corpus replay for existing legacy/R4G1 paths.
- `crates/uor-r4-graph-certify/src/score.rs`, `score_runtime.rs`, `certificate.rs`, and `performance_certificate.rs` already own Gate C, reference scoring, provenance, and runtime certificates.
- `crates/uor-r4-proof-model/src/proof_matrix.rs` and `deterministic_topk_proof.rs` own proof-status obligations; they must not be duplicated.
- #604/#605 introduce the missing target-operator semantics and fitting experiment; #603 introduces richer trace provenance.

# Problem

There is no target-operator-specific certificate that relates source parity, trace/fit provenance, progressive replacement, runtime bounds, witness replay, and model-quality measurements in one reproducible report.

# Goal

Extend existing certification surfaces with head/layer/model-level target-operator certificates for `R4RouteAttentionV1` fitting and packed lowering.

# Scope

- Add certificate rows for source tokenizer/config/adapter parity, attention-support overlap where meaningful, layer-output/final-hidden errors, logits/top-k agreement, bits/token, task/instruction evaluation, and artifact provenance.
- Reuse Gate C, teacher parity, performance, operation, allocation, deterministic rebuild, and proof-model checks; add only the target-operator joins and missing rows.
- Report bounded candidates, bytes read, operation count, witness replay, and exact absent/prerequisite states.
- Emit a versioned machine-readable certificate with source/corpus/trace/geometry/operator/compiler CIDs.

# Non-goals

- Do not create a second Gate C or teacher-parity harness.
- Do not turn empirical results into proof or exact-equivalence claims.
- Do not promote the operator, re-pin historical artifacts, or implement upstream integrations.

# Dependencies and blockers

- **BLOCKED BY #515** for activation/ledger policy.
- Depends on #597, #599, #602, #603, #604, and #605; consumes #531’s corpus-scale result.
- Proof obligations remain in #515/#589-aligned infrastructure and `uor-r4-proof-model`; no F1/Lean work starts before #589 unblocks.

# Proposed implementation shape

Add target-operator certificate sections and adapters to existing certifier reports. Keep source execution and fitting host-side, keep runtime checks independent, and ensure a certificate can say “not measured” or “blocked” rather than zero.

# Acceptance criteria

- [ ] A target-operator certificate links all input/output CIDs and operator/version.
- [ ] Head-, layer-, and model-scope rows distinguish source parity, fit, runtime, witness, and quality verdicts.
- [ ] Existing Gate C/teacher-parity/runtime/proof tests remain the owners of their existing checks.
- [ ] Missing source/trace/corpus prerequisites are represented as absent and cannot produce a passing quality claim.
- [ ] Certificate replay and deterministic rebuild produce identical canonical bytes for the same inputs.

# Verification

- Synthetic certificate fixtures for positive, negative, absent, and blocked rows.
- One pinned end-to-end target-operator report after #605 supplies valid inputs.
- Run existing Gate C/BDD/parity/performance/allocation/witness suites, `cargo fmt --check`, workspace clippy/tests, no-std checks, and the claim-wording checker.

# Compatibility and migration

Add a new certificate schema/version. Historical Gate C, parity, and proof fixtures remain valid and are not relabeled as target-operator certificates.

# Claim status

Definition, implementation, empirical criteria, and proof-obligation linkage. Certificate rows report measurements; they do not prove model equivalence.


## Comments (3)

### comment 5271803851 — 2026-08-12T19:28:05Z — Casey-allard

## Plan (#606 — composed target-operator recompilation certificate, no Gate C duplication)

**Prerequisite state.** #515 CLOSED (ledger/activation conventions in use since #600). #597/#599/#602/#603/#604/#605 all merged — #605's `route_fit_report` already embeds the existing `GateCMetrics` row type (serialized bytes unchanged) and carries RunContract-as-data, PASS/FAIL/UNAVAILABLE/NotRun stage records, and the deployed-kernel runtime checks. #606 is the *certificate* layer over those pieces: one versioned, machine-readable, canonical-bytes report that JOINS existing surfaces — it adds no second scorer, no second parity harness, no new proof obligations.

**Deliverables.**
1. `uor-r4-graph-certify`: new `target_operator_certificate.rs` — schema **`uor-r4-target-operator-certificate/1`** (versioned record, #600 discipline: canonical pinned-line bytes + blake3 declared-identity digest; registry refusing unknown (id,version)). The certificate composes, by reference and embedded rows: source parity (tokenizer/config/adapter — from existing model-source validation results), trace/fit provenance (the #605 FitManifest's eight CIDs + report κ), progressive-replacement rows per scope (head/layer/range/model) with the five separated verdict families (source parity / target-operator fit / runtime contract / witness replay / model quality), runtime bounds (bounded candidates, bytes read, op census), and artifact provenance. Every input/output CID + operator id/version linked.
2. Reuse map (in code comments, enforced by review): Gate C rows = embedded `GateCMetrics`; teacher parity = existing #307 surfaces referenced by κ; performance/operation/allocation = existing certificate types referenced, not reimplemented; proof-status = `uor-r4-proof-model` obligations linked by id, never restated; fit evidence = #605 `RouteFitReport` embedded/referenced by κ.
3. Absence semantics: every row supports **NotMeasured / Blocked(reason) / Unavailable(reason)** distinct from FAIL and from zero — a certificate with missing prerequisites CANNOT compose into a passing quality claim (machine-checked: quality verdict derives only from present, valid rows; tests prove the derivation refuses).
4. Witness-row semantics note (measured justification recorded in-doc): replacement comparisons bind to **outcomes under replacement**, never per-step path agreement — the historical program measured equal task outcomes at ~0.1–0.2 argmax path agreement, so path-agreement rows would be a false instrument.
5. Fixtures/tests: synthetic certificates for positive / negative / absent / blocked rows; deterministic double-run (byte + κ equality); replay-equality (rebuilding the certificate from the same inputs reproduces identical canonical bytes); one pinned end-to-end certificate over #605's synthetic-ladder outputs with real arms Blocked/Unavailable as recorded facts.
6. Ledger + docs: dormant lane `target-operator-certificate-dormant` (activation gate: a certificate over the real-teacher/#531-corpus ladder), CONFORMANCE regen, MODEL_LIFECYCLE #606 section.

**Non-goals honored:** no second Gate C; no proof/equivalence claims (rows report measurements); no operator promotion; no re-pins; historical fixtures keep their labels.


### comment 5272381238 — 2026-08-12T20:23:05Z — Casey-allard

## Delivery record — PR #646 (plan: comment 5271803851)

**Acceptance criteria:**
- ✅ *Certificate links all input/output CIDs + operator/version.* Identity block: 13 typed-absence fields (source snapshot, tokenizer, adapter, trace, geometry, operator id+version+record, corpus, compiler, fit-manifest κ, fit-report κ, fitted-params κ). Schema `uor-r4-target-operator-certificate/1` in the #600 discipline (pinned-line spec bytes, blake3 declared-identity digest, registry refusing unknown (id,version)).
- ✅ *Head/layer/model rows distinguish source parity, fit, runtime, witness, quality.* FIVE separated verdict families per scope row, states PASS / FAIL / NOT_MEASURED / BLOCKED(reason) / UNAVAILABLE(reason) — absence is three distinct states, never zero, never FAIL.
- ✅ *Existing owners keep their checks.* Gate C = embedded `GateCMetrics` verbatim; fit evidence = `RouteFitReport`/`FitManifest`/fitted params by κ; performance/empirical certificates + parity harness + `RunContract` by typed identity where no κ exists (typed absence, never invented digests); proof obligations = id+status links with a **proof-model-side mirror test** (dependency direction: proof-model → certify, so the mirror is pinned where both sides import).
- ✅ *Missing prerequisites cannot produce a passing quality claim.* `derive_overall_quality` is pure; the passing value carries a sealed token constructible only inside the derivation — a missing/blocked/unavailable prerequisite makes a passing claim **unrepresentable**, tested by refusal tests incl. tampered-κ detection.
- ✅ *Replay + deterministic rebuild identical bytes.* Double-run byte+κ equality and serialize→parse→re-serialize identity tested.

**End-to-end fixture (over #605's synthetic-ladder outputs):** head/layer/layer-range/model rows record source-parity/target-fit/runtime-contract/witness-replay **PASS** with model-quality **UNAVAILABLE**; real-teacher and real-corpus rows **UNAVAILABLE** with the #605 reason strings; overall quality verdict **NOT_PASSING**. Failing-ladder composition also covered (head target_fit=Fail → later scopes Blocked("exited at stage one-head")).

**Design rule recorded in-module:** replacement comparisons bind to outcomes under replacement, never per-step path agreement (historical program measurement: equal task outcomes at ~0.1–0.2 per-step argmax agreement — a path-agreement row would be a false instrument).

Ledger lane `target-operator-certificate-dormant` (activation gate: a certificate over the real-teacher + #531-corpus ladder with a valid instrument deriving a passing verdict). MODEL_LIFECYCLE #606 section added. Runtime kernel, P-4 scan lists, artifact surfaces untouched. Gates: validate ✅ wasm×5 ✅ fmt ✅ clippy -D warnings ✅ full workspace tests ✅ claim-wording ✅.


### comment 5272978674 — 2026-08-12T21:26:14Z — Casey-allard

Closing: PR #646 merged (2026-08-12T21:16:50Z). Delivery record: https://github.com/UOR-Foundation/uor-r4/issues/606#issuecomment-5272381238 — uor-r4-target-operator-certificate/1 shipped dormant (target-operator-certificate-dormant): composes Gate C (embedded GateCMetrics), #605 fit report/manifest by κ, proof obligations by id with proof-model-side mirror test; five separated verdict families per scope; sealed passing token makes a quality claim with missing/blocked/unavailable prerequisites unrepresentable; E2E fixture over #605 synthetic outputs reads NOT_PASSING with real arms unavailable.


---
_Comment/review ids in this file: 5271803851, 5272381238, 5272978674_

