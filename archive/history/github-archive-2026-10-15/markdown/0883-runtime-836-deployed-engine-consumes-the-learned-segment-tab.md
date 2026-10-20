# #883 — runtime(#836): deployed engine consumes the learned segment table (4c-i)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T23:52:49Z
- **Updated:** 2026-08-21T01:56:23Z
- **Closed:** 2026-08-21T01:00:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/883
- **Merged:** 2026-08-21T01:00:15Z
- **Merge commit:** 345e30fd876d561cf4bb2d184c113b862dc3e448
- **Base / Head:** main / UOR-Foundation:issue-836-table
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 4c-i of #836 — closes the **mechanism gap** the run contract surfaced. The +17.5‰ came from a *learned content-token→candidate table*, but the deployed lane (#880) only did the recurrence rule. This makes the deployed lane consume the learned table packed in PSTATE, faithfully lowering the #834 §6.2 segment arm.

## What changed

- **`SegmentSession::content_keys()`** (graph-runtime) — the prompt content tokens still live in the ring.
- **`R4Engine::load`** extracts the PSTATE residual table into an owned, key-sorted `SegmentTable` when the section carries rows. A **config-only** descriptor (no rows) → `None` → the recurrence scorer; **no PSTATE** → `None` → byte-identical.
- **`table_score` / `table_contribution`** — each candidate gains the sum, over the live prompt content tokens, of that content token's learned `ScoreQ` contribution (two binary searches per token; saturating add; **P-4**, no mul/div/float).
- **`segment_adjusted_token_with_table`** + **`segment_lane_attribution_with_table`** over a shared `segment_argmax`; `predict_decision_candidates_with_segment[_witness]` branch to the table path when a table is present, else the recurrence path.

## Verification (all green)

- `table_score` is an exact two-level lookup; the table re-rank **promotes the learned candidate and attributes it**, and a content token mapping elsewhere changes nothing.
- `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -D warnings`; `cargo run -p xtask -- validate` (every gate incl. the **P-4 scan** + register/conformance; `CONFORMANCE.md` unchanged). `forbid(unsafe_code)` preserved.

## Stacking / non-goals

Based on #882 (witness); rebased onto clean `main` before enqueue once #882 lands. Still **dormant / no quality claim**. Next (**4c-ii**): teach `convert_with_segment_lane` (landed in #881) to emit a *fitted* table — the compiler→load→promote end-to-end test lands there — then **4c-iii** re-measures on the packed path (bounded by the ceiling arithmetic in the run contract).

## Closure

**References #836.**

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

