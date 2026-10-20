# #830 — conformance: encode execution scope, serving reachability, and non-vacuous empirical verdicts (item B of #821)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:34:07Z
- **Updated:** 2026-08-20T01:31:29Z
- **Closed:** 2026-08-20T01:30:26Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/830
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #821

## Context

`model/ids.toml` and `model/ledger.toml` generate a consistent register, but the register cannot distinguish reference-only, compiler, certifier, dormant portable-runtime, normative-runtime, and deployed-serving evidence.

## Problem

This permits overreading. RF-08 uses an owned-string reference planner; RF-27/28 are reference/f32 models; RF-22 is a synthetic pathology filter; RF-29 can vacuously execute when fixtures are absent. The live dormant ledger also says the route-fit real run is unavailable despite its failed S1 verdict and describes R4G1Runtime patch ownership as production serving.

## Goal

Make evidence scope and reachability machine-visible, reconcile stale ledger truth, and make fixture-dependent empirical status impossible to report as PASS when inputs are absent.

## Scope

- [ ] Add generated execution-scope and serving-reachability fields to the conformance source model and renderer.
- [ ] Support at least: reference-only, offline compiler, certifier/instrument, dormant portable runtime, normative runtime, and deployed production path.
- [ ] Require an evidence pointer for every `build` claim and separate harness-built status from empirical PASS/FAIL/UNAVAILABLE.
- [ ] Audit RF-08, RF-22, RF-27, RF-28, RF-29 and any other scope-ambiguous row; narrow wording to implemented reachability.
- [ ] Update `route-fit-dormant` and `target-operator-certificate-dormant` with the real #605/#804 anti-vacuity failure.
- [ ] Correct `patch-overlay-dormant` so R4G1Runtime reachability is not called normative R4Engine serving.
- [ ] Make fixture absence emit `UNAVAILABLE`/`null`; add a negative fixture proving absence cannot mint an empirical certificate.
- [ ] Update `features/suites/README.md` to current suite/ID/scenario semantics and align the formal-vocabulary version header with its changelog.
- [ ] Regenerate `CONFORMANCE.md` from sources.

## Non-goals

- Promote a dormant route, patch, planner, or reference state model.
- Allocate one RF ID per GitHub issue.
- Hand-edit `CONFORMANCE.md`.

## Dependencies and entry gate

Coordinate field/schema changes with the generator, register tests, and consumers. This can proceed before the normative scorer decision but must describe that decision state explicitly.

## Acceptance criteria

- [ ] Every capability row renders an execution scope and explicit deployed-serving reachability.
- [ ] A non-production test cannot be cited as production evidence without a dedicated reachability assertion.
- [ ] Empirical status is exactly PASS, FAIL, or UNAVAILABLE and missing fixtures cannot produce PASS.
- [ ] The route-fit and patch-overlay ledger entries match live evidence and remain dormant behind activation gates.
- [ ] Generated files match their TOML/Gherkin/test sources and all existing valid capability IDs remain stable.

## Verification

- [ ] Run `cargo run -q -p xtask -- check-model`.
- [ ] Run `cargo test -q -p repo-conformance`.
- [ ] Run `cargo run -q -p xtask -- audit-deferral` and `cargo run -q -p xtask -- audit-limits`.
- [ ] Run the absent-fixture planted negative and verify serialized `UNAVAILABLE`.
- [ ] Run `python3 scripts/check_claim_wording.py` and inspect the regenerated register diff.

## Compatibility and migration

Add fields with deterministic defaults for every existing row. Preserve RF IDs and links; document any schema version/migration. Historical evidence stays intact while live summaries/ledger are corrected.

## Repository conformance

- **Execution scope:** Conformance model, generated register, structural harness, and explicit deployed-path reachability checks. Evidence outside this scope is not credited as a deployed-serving result.
- Preserve the applicable P-4, zero-allocation steady state, deterministic-byte, bounded-work, focused typed-error, `no_std`, and `forbid(unsafe_code)` contracts.
- Classify normative mathematics under `docs/formal_vocabulary.md`; empirical criteria name distribution, protocol, sample count, uncertainty, provenance identities, and status.
- **Conformance mapping:** RF-08, RF-09, RF-15, RF-21, RF-22, RF-23, RF-27, RF-28, RF-29.. Reuse/extend an existing capability unless a genuinely new capability requires a new RF ID.
- A new built capability lands in order: `model/ids.toml` row → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- `CONFORMANCE.md` is generated and must not be edited directly. Retained off-serving mechanisms stay `open` in `model/ledger.toml` with an activation gate.
- Measurements that revise a claim append a per-issue record and reconcile README, ROADMAP, RESEARCH, lifecycle/configuration, and ledger assertions.


## Claim status and next action

A green structural suite means the register/harness is built. Only a fixture-present, CID-bound empirical run may set empirical PASS.

## Comments (2)

### comment 5349772911 — 2026-08-20T00:44:12Z — Casey-allard

Taking this on as the next executable S0 item.

## Selection (from live GitHub state)

- **Selected:** #830 — *conformance: encode execution scope, serving reachability, and non-vacuous empirical verdicts (item B of #821)*.
- **Why it is next:** Earliest eligible stage is **S0 (#821)**; within #821 the tracker child order is A→B→C→D→E = **#829 → #830 → #831 → #832 → #833**. #829 (item A) is **CLOSED** (PR #860, merged `d1d1f735`), so #830 (item B) is the next sequential child. It has no open native `blocked_by` dependencies (REST `dependencies/blocked_by` → `[]`), no sub-issues, and its body explicitly permits proceeding before the #831 normative-scorer decision "provided that decision state is described explicitly."
- **Parent tracker / milestone:** #821 (S0) / *R4 Intelligence / S0 — Truth and inference closure*.
- **Closed prerequisites:** #829. **Downstream dependents (unlocked toward):** #831 (item C, one normative scorer), #832 (item D, CID-bound suites), #833 (item E, re-ratify baseline), and the whole S1→S7 chain that reads the register.
- **Not already being worked:** no open PRs; no branch or PR references #830; issue unassigned before this claim. F0 (#859) is the parallel formal lane and does not block this sequence.

## Execution contract

**Scope**
- Add generated **execution-scope** + **serving-reachability** fields (and a per-row **evidence** pointer) to the conformance source model (`repo-model`) and renderer, supporting: reference-only, offline-compiler, certifier-instrument, dormant-portable-runtime, normative-runtime, deployed-production.
- Separate harness-built (structural `build`) status from empirical **PASS/FAIL/UNAVAILABLE**; make fixture absence emit UNAVAILABLE, never PASS.
- Audit RF-08 (owned-string reference planner), RF-22 (synthetic pathology filter), RF-27/28 (reference/f32 models), RF-29 (fixture-gated instrument) and narrow wording to implemented reachability.
- Reconcile ledger: **route-fit-dormant** and **target-operator-certificate-dormant** with the real #605/#804 S1 **FAIL — instrument vacuous** verdict (currently they still read "real run UNAVAILABLE"); correct **patch-overlay-dormant** so `R4G1Runtime` reachability is not described as normative `R4Engine` serving.
- Update `features/suites/README.md`; align the `docs/formal_vocabulary.md` version header with its changelog.
- Regenerate `CONFORMANCE.md` from sources.

**Non-goals:** promote any dormant route/patch/planner/reference model; one RF ID per issue; hand-edit `CONFORMANCE.md`.

**Negative controls / falsifiers**
1. Absent-fixture planted negative: an empirical status computed from a missing fixture serializes as `UNAVAILABLE`; no absent-fixture input can yield `PASS`.
2. A register row claiming `deployed-production` scope without a `deployed-serving` reachability assertion is rejected by the model check.

**Verification:** `cargo xtask check-model`; `cargo test -p repo-conformance`; `cargo xtask audit-deferral` + `audit-limits`; the absent-fixture negative; `python3 scripts/check_claim_wording.py`; full workspace gate block + register R1–R6 before push.

**Expected files:** `crates/repo-model/src/{registry,lib,codegen}.rs` (+ new empirical-status module), `model/ids.toml`, `model/ledger.toml`, `CONFORMANCE.md` (regenerated), `crates/repo-conformance/tests/*` (negative fixture), `docs/formal_vocabulary.md`, `features/suites/README.md`, and a per-issue record `docs/*_830.md`.

Branch: `issue-830-conformance-scope`.


### comment 5350075801 — 2026-08-20T01:31:28Z — Casey-allard

## Definition of Done — completed ✅

Merged to `main` via the protected merge queue.

- **PR:** #861 — merged 2026-08-20T01:30:25Z.
- **Merge commit:** `bc5a2a92cf066367c519b559cf8a28813f3e94c3` (verified `git branch --contains` → `main`; `main` HEAD).
- **Queue verdict:** merge_group CI **success** (full binding ladder, 17m33s).

### Landed on main
- Register schema: `scope` / `reachability` / `evidence` on all 29 `RF-*` rows (`crates/repo-model/src/registry.rs`), rendered into `CONFORMANCE.md` (regenerated, 29 ids, CM-01).
- `Model::check`: every `build` claim carries an evidence pointer; `deployed-production` scope requires a `deployed-serving` reachability assertion.
- `EmpiricalStatus` (PASS/FAIL/UNAVAILABLE) as a separate axis from the harness-built `build` level; a missing fixture is UNAVAILABLE, never PASS (`crates/repo-model/src/empirical.rs`).
- Audited RF-08 / RF-22 / RF-27 / RF-28 / RF-29 wording narrowed to implemented reachability.
- Ledger: `route-fit-dormant` + `target-operator-certificate-dormant` reconciled to the #804/#605 S1 **FAIL — instrument vacuous** verdict; `patch-overlay-dormant` corrected (R4G1Runtime patch-chain reachability ≠ normative R4Engine serving).
- Docs: `formal_vocabulary.md` header aligned to changelog (0.1.14); `features/suites/README.md` refreshed; per-issue record `docs/conformance_execution_scope_830.md`.

### Acceptance criteria
- [x] Every capability row renders an execution scope and explicit deployed-serving reachability.
- [x] A non-production test cannot be cited as production evidence without a dedicated reachability assertion (enforced + planted negative).
- [x] Empirical status is exactly PASS/FAIL/UNAVAILABLE and missing fixtures cannot produce PASS (planted negative fixture).
- [x] Route-fit and patch-overlay ledger entries match live evidence and remain dormant behind activation gates.
- [x] Generated files match their sources; all `RF-*` IDs stable.

### Verification (green)
check-model (R1) · repo-conformance R2/R3 · audit-deferral (R4) · audit-limits (R5) · R6 (register-conformance job) · claim-wording · fmt · clippy `--all-targets --all-features -D warnings` · no_std ladder · wasm router lib · full merge-queue ladder.

### Disposition
**Completed.** No dormant mechanism promoted; `deployed-production` scope remains unused (reserved for a deployed-path reachability audit); the single-normative-scorer designation remains #831 (item C of #821), named explicitly. Next sequential S0 child: **#831**.



---
_Comment/review ids in this file: 5349772911, 5350075801_

