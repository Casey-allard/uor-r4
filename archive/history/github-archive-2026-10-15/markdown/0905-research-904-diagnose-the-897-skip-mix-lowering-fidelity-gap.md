# #905 — research(#904): diagnose the #897 skip-mix lowering-fidelity gap

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T19:26:58Z
- **Updated:** 2026-10-09T14:17:02Z
- **Closed:** 2026-08-21T19:51:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/905
- **Merged:** 2026-08-21T19:51:25Z
- **Merge commit:** 123e89836e03701ec544c779f409efc381bbcd60
- **Base / Head:** main / UOR-Foundation:diag/897-skipmix-gap
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #904.

## What

Off-serving diagnostic decomposing the #897 LOWERING-FIDELITY GAP
(`docs/skipmix_lane_897_result.json`: deployed-lane follow 41/87) into three
measurement arms, per Casey's "diagnose the gap first" sign-off. **No
production code changes** — a new `--ignored` test plus two docs files.

## Result: BREADTH-BOUND

* `StepCandidates::STEP_TOP_CANDIDATES = 8` structurally excludes at least
  one pair's teacher target from the served candidate list on **35/87
  pairs (40%)**, before any skip-mix scoring runs.
* The coverage ceiling itself — **52/87 = 59.8%** — sits *below* the
  predeclared 53/87 (60%) faithfulness bar. Under the current candidate-list
  width, even a perfect scorer could never clear that bar.
* Among the 52 reachable pairs, the idealized (unquantized) reference
  formula follows all 52; the deployed (quantized, additive) lane only
  follows 41 of those 52 — an 11-pair secondary residual.
* A proposed unit-safe (non-additive) combine rule, meant to test the
  scale-mismatch hypothesis from #897's record, does **not** close that
  residual (40/87, slightly worse than the deployed 41/87) — the
  scale-mismatch hypothesis is not supported as a dominant cause.

Also corrects #897's record: the per-key top-64 cap is identical between the
reference and deployed arms and was never a differentiator.

## Files

* `crates/uor-r4-api/tests/skipmix_gap_diagnostic_904.rs` — the harness
  (`--ignored`; reuses #897's fit/emit/consume machinery against the real
  bundle; includes an instrument-fidelity control verified against the
  engine's actual output).
* `docs/skipmix_gap_diagnostic_904.md` — the empirical record.
* `docs/skipmix_gap_diagnostic_904_result.json` — machine-readable result.

## Gate ladder (local, all green)

fmt, clippy (workspace, `-D warnings`), full workspace test suite, `xtask
validate`, wasm32 build, `cargo deny check`, claim-wording check, and the
diagnostic test itself.

## Scope

Lane remains dormant; no `model/ids.toml` row; `CONFORMANCE.md` unchanged.
This is measurement only — what to do about the finding (widen the
candidate list architecturally, accept a relaxed bar, or leave the lane
dormant) is a follow-up decision, not made here.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

