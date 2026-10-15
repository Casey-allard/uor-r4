# #929 — research(#845): the equal-budget measurement and the NO GEOMETRIC ADVANTAGE verdict

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T20:44:27Z
- **Updated:** 2026-08-22T21:05:05Z
- **Closed:** 2026-08-22T21:03:50Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/929
- **Merged:** 2026-08-22T21:03:50Z
- **Merge commit:** ddf4a14236973fe3c8e7076b8e5c9e3f1c6cb2cb
- **Base / Head:** main / UOR-Foundation:issue-845-measurement-verdict
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 4 of 4 for issue 845 (S4 item C): the equal-budget measurement on both pre-registered A2 axes, its append-only record, and the verdict — **NO GEOMETRIC ADVANTAGE**, the pre-registered negative branch, a successful falsification.

## Result

- **A2(a), budget axis (12 separating cells, frozen budget):** the geometry arm holds correct-outcome 1.0000 everywhere (the mapping does no harm) but reduces work nowhere: conjunction FAIL, 5/12 (the five passes are the structurally degenerate H = 1 no-regression cells; in every reduction cell geometry does 12–37% MORE work than the goal-distance beam; Holm 0/12).
- **A2(b), correctness axis (9 primary + 9 secondary tightened-budget cells):** conjunction FAIL, 0/9 primary and 0/9 secondary. The bar is an informed non-geometric ordering in every primary cell — goal-distance or Hamming/popcount at 1.0000 (0.9961 once) against geometry at 0.0742–0.6426.
- **Attribution (pre-registered reading rules):** geometry sits at the level of its own randomized, relabelled, and phase-swapped controls (0.6230 vs 0.6445–0.7090 at frontier-16), so its gain over breadth-first attributes to informed ordering generally, not to quadrangle structure; the pinned phase convention is mildly anti-aligned (recorded, not optimized against). Mechanism: mod-9 digit quantization wrap-around severs slot distance from task-step distance.
- The programme falsifier is discharged: a geometry arm that cannot beat Hamming/binary/VSA/spectral controls under equal bytes, candidates, and operation budgets stays out of the production runtime.

## What this PR contains

- `crates/uor-r4-graph-certify/tests/w33_measurement_845.rs` — the measurement instrument: both axes over the frozen grids (n = 512/cell, joint split, one `PlanBudget` per cell, arm mode on the parity-proven skeleton), the §5/§5-A per-cell readings, IU + Holm; the full grid `#[ignore]`d, with default non-vacuity gates asserting the A2(a) classifier and both pass rules fire in both directions and the machinery runs live end-to-end.
- `docs/w33_geometry_qualification_845.md` (new, append-only) — the complete measurement record: verdict, both result tables, the attribution table, verification behind the numbers, claim boundaries, the re-entry condition (a new mapping Definition with a fresh contract and probe), and downstream state.
- `docs/r4_intelligence_completion_plan.md` — the readable-mirror verdict entry for item C.

## Acceptance-criteria status (issue-level)

- Mapping/readout total, testable, label-independent: DONE (increment 2, machine-checked).
- Equal storage/candidate/operation budgets, non-degenerate baselines: DONE (increment 3, audited; every control able to fire and to fail).
- Primary interventional metric first; trajectory stability never gated: DONE (no trajectory metric was needed).
- Gain survives relabeling/isomorphism/topology/composition controls: the antecedent is empty — no gain exists to survive; the controls are what established that.
- Verdict recorded from the pre-registered space: **NO GEOMETRIC ADVANTAGE**.
- W(3,3) remains exploratory/reference; no production lowering is proposed; the f64 router is unmodified; no production graph crate gained any dependency.

## Verification

fmt, clippy `-D warnings` (workspace, all targets/features), wasm lib build, `cargo test --workspace --no-fail-fast --offline` (188 suites, 0 failures), `xtask validate` (R1/R4/R5/gnaf), `cargo deny check bans`, claim-wording gate — all green locally on exactly this tree; `audit-limits` re-run immediately before push. The measurement reproduces deterministically with `cargo test -p uor-r4-graph-certify --release --test w33_measurement_845 -- --ignored --nocapture`.

## Conformance, compatibility, claim boundary

Research/reference scope; certifier-instrument code and docs only. CONFORMANCE.md unchanged (33 ids); no new RF id; model/ledger.toml untouched; no production format, runtime, or dependency changes. The negative verdict narrows the public claim: no geometry statement stronger than "no equal-budget advantage measured for the pinned mapping" is licensed.

## Negative results

The verdict itself is the negative result, recorded as first-class completion evidence per the programme rules. No result in this issue is UNAVAILABLE: every declared instrument ran and returned a definite reading.

Closes #845


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

