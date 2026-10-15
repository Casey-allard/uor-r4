# #928 — test(#845): reference ordering skeleton, arm/control roster, and parity proofs

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T20:11:21Z
- **Updated:** 2026-08-22T21:05:05Z
- **Closed:** 2026-08-22T20:43:16Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/928
- **Merged:** 2026-08-22T20:43:16Z
- **Merge commit:** 287a61568a7f9836a247e8e4175fc1e85988e116
- **Base / Head:** main / UOR-Foundation:issue-845-ordering-harness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

References #845 (S4 item C, increment 3 of 4 — arms and harness).

The design contract requires every geometry claim to be earned against non-geometric ordering controls under equal budgets and equal bytes, on a search harness whose accounting provably matches the deployed planner. This PR delivers that harness and roster, with the parity claim machine-checked rather than assumed.

## What this PR contains

- `tests/support/ordering.rs` — the deployed layered search ported with exactly one seam (the ordering score) in two modes: parity mode (retention only; counter-exact against the deployed planner) and arm mode (retention plus stable descending-score expansion of the retained layer, the spec §3 ordering functional). Counter accounting mirrors the deployed `Episode` call-for-call, including the post-search unwind and considered-record reads.
- `tests/support/arms.rs` — the ordering-arm roster with pinned constants (spec §4-A): W(3,3) geometry, Hamming/popcount, learned-table-codes (same 40-class quantization as the geometry arm, deliberately), VSA-binding, spectral-embedding (fixed-sweep cyclic Jacobi, deterministic by construction), random-embedding (byte/shape/range-matched), isomorphic-relabel (tables conjugated by a pinned scramble asserted NOT to be a collinearity automorphism), phase-permuted, and the goal-distance incumbent.
- `tests/support/episode.rs` — shared episode machinery (generate/induce/pack/predicates/verify, deployed and reference runners, the four #843 nulls, fitting observations).
- `tests/support/stats.rs` — paired lower bounds, degenerate-case p-values, standard-direction Holm.
- `tests/w33_ordering_harness_845.rs` — the gates: counter-exact parity (outcome, plan, every `PlanCounters` field) for FIFO ≡ deployed breadth-first and goal-distance ≡ deployed beam over a 3-family x 3-horizon x 4-budget x 32-seed grid (>1,000 episodes per rule, non-vacuity asserted); per-arm determinism and separation; scramble non-automorphism; equal-bytes audit (every control at or under the geometry arm's 3,200-byte budget); seam liveness.
- `docs/w33_geometry_qualification_spec_845.md` — §4-A (pinned constants and the two seam modes) and §5-A (the A2(a) per-cell reading: reduction cells vs no-regression cells, from the structural fact that an L-step plan needs at least L expansions and every H=1 episode spends exactly one — recorded before any measurement, the #844 §11.6 shape on the budget axis).

## Verification

fmt, clippy `-D warnings` (workspace, all targets/features), wasm lib build, `cargo test --workspace --no-fail-fast --offline` (188 suites, 0 failures), `xtask validate` (R1/R4/R5/gnaf), `cargo deny check bans`, claim-wording gate — all green locally on this content; `audit-limits` re-run immediately before push. Certifier-instrument test code only; no shipped-crate source changes.

## Conformance, compatibility, claim boundary

No production format, runtime, or dependency changes; CONFORMANCE.md untouched; no new RF id. Nothing here licenses any geometry claim — this increment builds the instrument that can refute one, and its controls are asserted able to fire and to fail.

## Intentionally excluded

The measurement itself, its record, and the verdict (increment 4, next PR).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

