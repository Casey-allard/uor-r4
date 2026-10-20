# #911 — serving(#910): activate the promoted skip-mix lane (RF-31) — S1 PROMOTE follow-through

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T23:37:24Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-22T00:01:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/911
- **Merged:** 2026-08-22T00:01:07Z
- **Merge commit:** eee8f2353fd95ac74ff6d62fa23165de8c51d3ba
- **Base / Head:** main / UOR-Foundation:issue-910-skipmix-activate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

The S1 skip-mix lowering cleared the frozen 20‰ end-to-end causal floor (#908 / PR #909: deployed **+28.45‰ [25.57, 31.32]**, null −236.95‰), and the maintainer S1 verdict is **PROMOTE + activate** (#822). This PR turns the promoted lane on: the deployed serving decision routes through it, compiled bundles carry its sections, and the capability is registered as **RF-31**.

## Scope / non-goals

Deployed serving path + offline compile path + the RF register. No teacher, no corpus change, no wire-format change (the SKMX/PSIB sections exist since #897). Every change is **byte-identical on a bundle without the sections** (absent-section identity), so this alters what serving does on a bundle that carries the lane, not behavior on today's sections-absent fixtures. The segment lane (#836), the `serving_eval` Gate C readiness probe, and the `cover_sweep` analysis tool are intentionally left on the base decision.

## What changed (three parts)

1. **Serving reroute** — `crates/uor-r4-api/src/engine.rs`: `R4Engine::predict_decision` now routes through `predict_decision_candidates_with_skipmix`, so every serving entry (generation loop, chat single-step, HTTP) applies the lane at one point. Safe by construction: `predict_decision` and `predict_decision_candidates` share one implementation (`predict_signature_status_with_recent_candidates`, sink `None` vs `Some`) → identical served token/status/widening; and the lane returns the base decision unchanged when neither section is present. The lane re-selects only the served **token** — never the status, never overriding an abstention — so status-reading scan paths are unaffected. `StepCandidates` is a fixed-capacity stack buffer ⇒ hot path stays allocation-free.

2. **Compile-path wiring** — `crates/uor-r4-graph-cli/src/lib.rs`: the release score stage now fits (`skipmix_fit::fit_skipmix_tables`) and emits the SKMX/PSIB sections, so compiled bundles carry the lane. Teacher-free; re-splits the corpus to the same TRAIN partition the other stages use.

3. **RF-31 registration** — `model/ids.toml` row (scope `normative-runtime`, reachability `deployed-serving`) → `features/suites/skipmix_serving_lane.feature` (`@RF-31 @build`, 3 scenarios) → marker in `crates/repo-conformance/tests/registered.rs` + executable steps in `tests/bdd.rs` → regenerated `CONFORMANCE.md` (31 ids, CM-01, via `check-model --write`; never hand-edited).

## Acceptance-criteria status

- Serving routes through the lane; byte-identical on sections-absent bundles — **met** (root serving tests + `status_path_allocation_census` green).
- Compiled bundles carry SKMX/PSIB — **met** (score stage fits + emits; the #908 harness demonstrated fit→emit→`skipmix_tables_present() == (true, true)`).
- RF-31 registered, CONFORMANCE regenerated, marker green — **met**.
- Invariants preserved: allocation-free hot path, P-4/gnaf (no multiply/divide/float in the deployed kernel), no_std ladder, wasm — **met** (`xtask validate` incl. `gnaf-scan` clean; wasm build green).

## Tests / verification (all green)

`cargo test --workspace` (crate suites + root package incl. allocation census, 0 failures); `bdd` (109 scenarios / 361 steps); `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo run -p xtask -- validate` (R1/R4/R5/gnaf incl. the forbidden-construct P-4 scan); `python3 scripts/check_claim_wording.py`; `cargo build --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`.

## Evidence artifacts

Deployed causal evidence: `docs/skipmix_endtoend_causal_908.md` (result_cid `blake3:e32e4e33…`, #908). Plan doc reconciled S1 REVISE → PROMOTE (`docs/r4_intelligence_completion_plan.md`, append-only).

## Compatibility / migration

Backward-compatible: a bundle without the sections serves byte-identically to before. A newly-compiled bundle carries the lane and serves the promoted behavior. No recompile of an already-released bundle is forced by this PR; a bundle goes live on its next compile through the wired `score` path.

## Conformance effects / claim-boundary

New built capability **RF-31** (deployed-serving). The Guarantee-class structural properties (typed reroute, absent-section identity, allocation-free, P-4-legal) are asserted; the deployed-serving quality claim is the #908 Empirical Criterion.

## Negative / unavailable results

None — this is the promotion follow-through; the causal evidence is positive (#908).

## Intentionally excluded

`serving_eval` (Gate C readiness probe) and `cover_sweep` stay on the base decision; re-emitting/republishing the deployed `smollm2-360m-broad-clean` bundle is a release/ops step (the wired `score` path produces the sections on the next compile).

Closes #910


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

