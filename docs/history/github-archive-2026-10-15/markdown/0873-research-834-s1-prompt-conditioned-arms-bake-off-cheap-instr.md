# #873 — research(#834): S1 prompt-conditioned arms bake-off cheap instrument

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T17:49:47Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-20T18:12:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/873
- **Merged:** 2026-08-20T18:12:14Z
- **Merge commit:** 678cf34297e806a87d9fc2a6cec999ffe421c3f9
- **Base / Head:** main / UOR-Foundation:issue-834-causal-arms-instrument
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S1 item B (#834, research/#822-B) asks whether prompt meaning has *causal* relevance beyond suffix/exact-context memory, by fitting several artifact-only prompt-conditioned arms against strong causal and equal-budget controls and selecting at most one. Its run contract is explicit: **no long run may launch until exact run values are posted and a binding cheap instrument passes on a committed small fixture.**

This PR lands that **binding cheap instrument** — the gate the issue requires *first* — as reference-only / off-serving code, so the real multi-arm study on the #833 canonical bundle can be launched (a maintainer decision) against a validated harness rather than ad hoc. It does **not** assert the S1 causal verdict.

## Scope

- `crates/uor-r4-api/tests/prompt_arms_bakeoff_834.rs` — an offline, integer, deterministic bake-off over the five arms #834 enumerates (`current-scoring`, `longer-local-context`, `persistent-state`, `conditional-residuals`, `candidate-support-expansion`) against the frozen `prompt-swap` / `suffix-only` / `shuffled-state` / `trivial-prior` and equal-budget controls, built on the #835 reference model idiom and binding the frozen #832 `capability_suite` vocabulary (`ControlKind`, `MetricStatus`, `ResolutionPath`, `AttributionHistogram`, `is_degenerate_control`, `compute_cid`, `detect_document_leakage`, `NORMATIVE_SCORER_ID`).
- `docs/prompt_arms_bakeoff_834.md` — per-issue evidence record: the arms/controls, the instrument's Verification mapping, reachability/power, and the run contract with a per-input **entry-gate assessment** (AVAILABLE vs UNAVAILABLE).
- `docs/RESEARCH.md` — reconciled with an "S1 item B" note.

## Non-goals (honored)

- No S1 causal verdict is asserted (that is the #833-scoped long run).
- No fixture tuning (a #834 non-goal); the fixture is a controlled instrument self-test, not the causal suite.
- No deployed/serving change; no arm is lowered (that is #836).

## Acceptance-criteria status

Acceptance criteria whose subject is the **real study on #833** are `UNAVAILABLE` here by design; the criteria the *instrument* must satisfy are met and machine-checked:

- [x] Candidate-support vs score-conditioning effects **separately identified** — `candidate_and_score_conditioning_separately_identified`.
- [x] A control **cannot pass through zero variance or identical outputs** — `every_control_is_non_degenerate`.
- [x] The decision vocabulary is complete (`SELECT` / `REVISE` / `NO PROMPT-CONDITIONING ARM ESTABLISHED`) and reachable — `verdict_vocabulary_is_complete`, `instrument_selects_causal_arm_and_rejects_planted_negatives`.
- [x] Raw per-pair attribution is replayable from **CID-bound** records — `independent_report_recomputation_from_cid`.
- [ ] All arms/controls run on **#833 identities and evaluation slices** — `UNAVAILABLE` (maintainer-gated long run).

## Tests and verification (all green locally)

- `cargo test -p uor-r4-api --offline --test prompt_arms_bakeoff_834` — 12 tests, implementing #834's five Verification items: intervention/attribution/shuffle operation, power fixes n+MDE before fitting, double-run/reordered-shard determinism, **planted prompt-insensitive AND diversity-only models fail the primary gate**, and independent report recomputation.
- `cargo test -p uor-r4-api --offline --no-fail-fast` — full api suite green.
- `cargo clippy -p uor-r4-api --all-targets --all-features --offline -- -D warnings` — clean.
- `cargo fmt --check` — clean.
- `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`) — clean.
- `python3 scripts/check_claim_wording.py` — passed.
- `cargo run -q -p xtask -- validate` — every gate passed (`CONFORMANCE.md equals the model, 29 ids`; R4/R5/gnaf clean).

## Evidence artifacts

`docs/prompt_arms_bakeoff_834.md` (run contract + entry-gate assessment) and the machine-checked harness. Report records are CID-bound (`compute_cid`) and replayable.

## Compatibility and migration

None. Research artifacts are non-serving; no format, ABI, or serving-path change. No `model/ids.toml` row and no `CONFORMANCE.md` regeneration — this follows the #835/#832 spec-leaf precedent for a leaf that binds existing capabilities (RF-01, RF-21, RF-22, RF-27, RF-28) rather than creating a new built capability.

## Conformance effects

No capability added or changed; `CONFORMANCE.md` unchanged and current (validator confirms 29 ids). RF mapping documented in the evidence record.

## Claim-boundary changes

None asserted. The S1 causal claim remains open; this PR only lands the instrument that gates its measurement and records the verdict as `UNAVAILABLE` pending the long run.

## Negative / unavailable results

The S1 causal verdict is `UNAVAILABLE` (not a vacuous pass): the #833 canonical teacher/tokenizer/corpus/partitions/bundle identities required for the real evaluation are not present in this offline instrument. The instrument's planted-negative tests are the falsifier teeth: a prompt-insensitive model and a diversity-only ("distinct but irrelevant") model both fail the primary gate.

## Intentionally excluded

- The hours-scale multi-arm run on #833 and its `SELECT`/`REVISE`/`NO-ARM` verdict — maintainer-gated (post exact run values → go/no-go). Only a predeclared positive verdict may trigger #836.
- Untracked working dirs (`obs-text/`, `tf1-pkg/`) were left untouched and unstaged.

**References #834** (genuinely partial — the instrument lands; the issue stays open for the maintainer-gated long run).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

