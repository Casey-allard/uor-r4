# #907 — serving(#906): inject skip-mix/psi-bag-known candidates into the decided list

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T21:37:36Z
- **Updated:** 2026-08-21T22:05:58Z
- **Closed:** 2026-08-21T21:58:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/907
- **Merged:** 2026-08-21T21:58:42Z
- **Merge commit:** e478110df2078965340b0e2534aa8e007b149f4e
- **Base / Head:** main / UOR-Foundation:feat/906-skipmix-candidate-injection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Closes #906 (follow-up to #897/#904).

#904 diagnosed the #897 skip-mix lane's LOWERING-FIDELITY GAP (deployed
follow 41/87, below the predeclared 53/87 = 60% bar) as **breadth-bound**:
the deployed lane (`predict_decision_candidates_with_skipmix`) only
re-ranked the base engine's already-decided candidate list
(`StepCandidates::ranked()`, capped at `STEP_TOP_CANDIDATES = 8`) and
never consulted its own SKMX/PSIB tables as a candidate *source* — even
though 45/45 (100%) of the missing pair-sides already have their teacher
target recorded there as a co-occurrence partner.

Before choosing this fix, every width/selection lever on the *base*
engine's own candidate generation was tested empirically and ruled out
(full detail in `docs/skipmix_candidate_injection_906.md`):

- `STEP_TOP_CANDIDATES` alone (8 → 16/32/64): plateaus at 52-53/87.
- All four compile-time candidate-breadth caps together (64x widening,
  confirmed real via the compiler's own diagnostic): unchanged, 52-53/87.
- `EmissionSelection::Probability` (an existing, unused alternative
  ranking rule, verified as a large real quality change on its own terms):
  unchanged, 52/87 ceiling, 41/87 deployed follow.

Candidate injection was then prototyped off-serving (58/87 projected,
clearing the 60% bar) and implemented for real in this PR.

## Changes

- `crates/uor-r4-api/src/engine.rs`: `skipmix_adjusted_token`/
  `skipmix_lane_attribution` (re-rank-only) replaced by
  `skipmix_injected_argmax`/`skipmix_injected_lane_attribution`, which
  discover every token in a relevant SKMX row `(t, last_token)` or PSIB row
  `t` for the window's own unique tokens, in addition to the base `ranked`
  list — ranked by the non-additive "unit-safe" combine rule #904 arm 3
  already validated. Allocation-free (single-pass running-best
  accumulator, no set/Vec materialized); absent-section byte-identity
  preserved (neither table present ⇒ behavior unchanged).
- `predict_decision_candidates_with_skipmix` and
  `predict_decision_candidates_with_skipmix_witness` rewired onto the new
  functions.
- Unit tests updated/added (7 tests), including a new
  `skipmix_injects_candidate_absent_from_ranked` demonstrating the core
  new capability: a candidate not present in `ranked` at all is discovered
  and wins via table support alone.
- New confirmation test `crates/uor-r4-api/tests/skipmix_candidate_injection_906.rs`
  (`--ignored`, heavy): replays the same 87 favorable pairs used
  throughout #897/#904/#906 through the **real, patched**
  `predict_decision_candidates_with_skipmix` (not a simulation). Result
  written to `docs/skipmix_candidate_injection_906_result.json`.
- `docs/skipmix_candidate_injection_906.md`: full empirical record (ruled-out
  hypotheses, prototype, fix, confirmed result).

## Result

| quantity | value |
|---|---|
| deployed follow, **NEW** (candidate-injection combine) | **58 / 87 (66.7%)** |
| deployed follow, OLD (re-rank-only, #897/#904 record) | 41 / 87 (47.1%) |
| predeclared 60% fidelity bar | 53 / 87 |
| clears fidelity bar | **yes** |

The real production measurement matched the off-serving prototype
projection exactly on the first clean run.

## Scope / conformance

Production code changed: `engine.rs` only. Base engine candidate
generation (`STEP_TOP_CANDIDATES`, region clustering, context rows) and
the #836 segment lane are untouched. No wire-format/compile-time change;
no recompile of any released bundle required. The skip-mix lane remains
**dormant** (no `model/ids.toml` serving row) — this changes what the lane
*would* do if turned on, not live production behavior today.
`CONFORMANCE.md` is unaffected.

## Test plan

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo run -p xtask -- validate`
- [x] wasm32 build (`uor-r4-core`/`uor-r4-graph-format`/workspace-excl-xtask)
- [x] `cargo deny check`
- [x] `python3 scripts/check_claim_wording.py`
- [x] `cargo test -p uor-r4-api --release --test skipmix_candidate_injection_906 -- --ignored --nocapture` → 58/87, bar cleared

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
https://claude.ai/code/session_01R6gqsnxp6JUa5jDtN43dCF


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

