# #910 — serving(S1): activate the promoted skip-mix lane — RF-31 registration + serving/compile-path wiring

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T23:02:45Z
- **Updated:** 2026-08-25T06:11:48Z
- **Closed:** 2026-08-22T00:01:08Z
- **Milestone:** R4 Intelligence / S1 — Prompt-conditioned state
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/910
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

The S1-redesign skip-mix lane is **promoted**: #908 (PR #909) measured its deployed end-to-end causal effect at **+28.45‰ [25.57, 31.32]** (paired 95% lower bound clears the frozen 20‰ S1 floor), null collapses −236.95‰. Maintainer S1 verdict: **PROMOTE + activate** (2026-08-21).

This issue turns the lane on. It is a serving-semantics change; it gets its own PR (AGENTS.md). Child of #822 (S1).

## Execution scope

Deployed serving path + the offline compile path + the RF capability register. No teacher, no corpus change, no wire-format change (the SKMX/PSIB sections already exist since #897/#903).

## What "on" requires (three parts)

1. **Serving reroute (deployed-serving).** `R4Engine::predict_next_into` and `generate_into` (and the `serving_eval` single-step path) currently call `predict_decision(window)`. Route them through `predict_decision_candidates_with_skipmix(window, &mut StepCandidates)` using a reused fixed-capacity `StepCandidates` buffer. **Provably safe:** `predict_decision` and `predict_decision_candidates` share one implementation (`predict_signature_status_with_recent_candidates`, sink `None` vs `Some`) → identical served token/status/widening; and `predict_decision_candidates_with_skipmix` returns that base decision unchanged when neither SKMX nor PSIB section is present (absent-section identity). So the reroute is byte-identical on any bundle without the sections, and activates the lane only on bundles that carry them.

2. **Compile-path wiring (offline-compiler).** The production score/emit sites pass `skipmix_rows: &[]` / `psi_bag_rows: &[]` today (`crates/uor-r4-graph-cli/src/lib.rs` score stage; `crates/uor-r4-graph-cli/src/cover_sweep.rs`). Wire `skipmix_fit::fit_skipmix_tables(&corpus, DEFAULT_TOP_K)` in so freshly compiled bundles carry the SKMX/PSIB sections. Bundles live under `.uor-models/` (gitignored) — re-emitting the local `smollm2-360m-broad-clean` graph via the wired path makes the lane live locally (teacher-free, cheap).

3. **RF-31 registration (conformance).** Built-capability order: `model/ids.toml` RF-31 row (scope `normative-runtime`, reachability `deployed-serving`) → tagged Gherkin `features/suites/*.feature` → failing marker in `crates/repo-conformance/tests/registered.rs` + executable steps in `tests/bdd.rs` → implementation (parts 1–2) → regenerate `CONFORMANCE.md` via `cargo run -p xtask -- check-model --write` (never edit `CONFORMANCE.md` directly).

## Acceptance criteria

- Serving path routes through the skip-mix lane; byte-identical on a sections-absent bundle (asserted), lane-active on a sections-present bundle.
- Compiled bundles carry SKMX/PSIB; a re-emitted `smollm2-360m-broad-clean` shows `skipmix_tables_present() == (true, true)` and the deployed lane live.
- RF-31 registered; `CONFORMANCE.md` regenerated (not hand-edited); `repo-conformance` marker green.
- Invariants preserved: allocation-free hot path (`allocation_census`), P-4/gnaf (no multiply/divide/float in the deployed kernel — `xtask validate`), `no_std` ladder, wasm build.
- The #906/#908 deployed harnesses still green.

## Non-goals

No new measurement (the causal evidence is #908). No change to the 20‰ floor or the #838 selective gates. No teacher rebuild. D3 stays deferred. The segment lane (#836) is untouched.

## Verification

Full gate ladder (fmt, clippy `-D warnings`, `cargo test --workspace --no-fail-fast --offline`, no_std ladder, wasm, `xtask validate`, `cargo deny check bans`, claim-wording) + `allocation_census` + a re-emit-and-serve check on the local bundle.

## Claim status

Guarantee-class structural properties (typed reroute, absent-section identity, allocation-free, P-4-legal) are asserted; the deployed-serving quality claim is the #908 Empirical Criterion. Conformance mapping: **RF-31** (new built capability).


## Comments (1)

### comment 5406302837 — 2026-08-25T06:11:48Z — Casey-allard

## Append-only serving-scope correction — resolved by #933 (2026-08-25)

#910's RF-31 registration and compile/serving wiring remain part of the structural history, but the completed foundation audit established that #910 did **not** make SKMX/PSIB contributions part of the ADR-designated `R4G1Runtime` candidate/token selector. Its closure therefore did not by itself establish RF-31 at normative deployed-serving scope.

#933 closes that gap. PR #936, merge commit `e346816cb40b089583cbd3fff2cdd84924362c65`, routes the shared production adapters through one `R4G1Runtime` candidate owner, binds the full deployed-quality evidence and exact TLA comparator store into schema-2 admission, rejects malformed or substituted comparator evidence, and admits the exact bundle from an empty model store.

The bound census is 21,293 / 72,130 = 29.5203% versus same-position TLA 20,284 / 72,130 = 28.1214%, plus +34.479 permille [31.681, 37.277] versus the same-generation sections-absent control. Cross-surface mismatches are 0 / 72,138, absent-identity mismatches are 0 / 72,130, and witness replay is 64 / 64.

This is a scoped RATIFY for the exact #933 envelope—not a universal quality floor, live-teacher parity, instruction-following, reasoning, or free-running-generation result.

Record: https://github.com/UOR-Foundation/uor-r4/blob/main/docs/normative_r4g1_quality_933.md
PR: https://github.com/UOR-Foundation/uor-r4/pull/936
Binding queue run: https://github.com/UOR-Foundation/uor-r4/actions/runs/32814445068


---
_Comment/review ids in this file: 5406302837_

