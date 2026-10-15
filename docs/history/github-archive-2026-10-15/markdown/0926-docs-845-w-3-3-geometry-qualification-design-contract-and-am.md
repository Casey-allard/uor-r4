# #926 — docs(#845): W(3,3) geometry-qualification design contract and Amendment A2

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T19:23:47Z
- **Updated:** 2026-08-22T21:05:05Z
- **Closed:** 2026-08-22T19:45:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/926
- **Merged:** 2026-08-22T19:45:42Z
- **Merge commit:** 6f6c3f1f536e864fc4448a44555f3014c60c84a8
- **Base / Head:** main / UOR-Foundation:issue-845-design-freeze
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

References #845 (S4 item C, increment 1 of 4 — design freeze; docs only).

Item C was released RESTRICTED after the #843 LIMITED verdict, but as framed it could not launch: the lowered RF-33 baseline sits at correct-outcome rate 1.0000 in all 20 frozen cells, so the reachable effect of any geometry arm on the frozen primary statistic was exactly 0 < δ_min. The 2026-08-22 maintainer decision on #845 (source-of-record comment there) converted the #859 edge under scoped terms, sanctioned benchmark Amendment A2, and authorized a binding failure-surface probe. This PR freezes all of that as the design contract.

## What this PR contains

- `docs/w33_geometry_qualification_spec_845.md` (new) — the frozen #845 design contract: the decision record; the normative W(3,3) Definitions (symplectic quadrangle over GF(3), 40 points / 40 lines, canonical representatives, collinearity distance, phase; the dashboard's 96-vertex canvas reconciled by disambiguation as a non-normative rendering motif); the total content-derived state mapping μ with quantization/readout semantics and a projected division-free integer realization; the arm/control roster with equal-budget and equal-byte obligations; pre-registered statistics for both A2 axes; the probe-verified run contract; the four-increment plan.
- `docs/compositional_planning_spec_844.md` §12 (appended) — Amendment A2: the budget axis A2(a) (paired expansion reduction at exactly preserved correctness on the 12 separating cells, ρ_min = 0.10) and the probe-admitted correctness axis A2(b) (nine primary tightened-frontier cells at H = 8 in the three separating families, bars 0.7422–0.8340; nine secondary cells non-gating). No frozen value moves.
- `docs/r4_intelligence_completion_plan.md` (appended) — the readable mirror entry for the item C activation.

## Probe evidence (binding cheap instrument — ran first, PASSED)

70 cells (five families × H ∈ {1, 2, 4, 8, 12, 16} frozen budget + frontier/expansion ladder at H = 8), n = 512/cell, joint split, 33.5 s single-threaded, teacher-free. 18 admissible cells; the two greedy-solvable families admitted zero cells at every setting, matching the #843 diagnosis. Frozen-grid overlap reproduces the #843 record (baseline 1.0000; identical strongest-null rates). The probe instrument (`geometry_probe_845.rs`) is deliberately not in this PR — it ships with increment 2 per the docs-only fast-path discipline; its two non-vacuity gates run green locally.

## Acceptance-criteria status (issue-level, at this increment)

- Mapping/readout totality and label-independence: specified (§3); tests land in increment 2.
- Equal storage/candidate/operation budgets, non-degenerate baselines: obligations frozen (§4); byte-parity audit lands in increment 3.
- Primary interventional metrics first, trajectory stability diagnostic-only: pre-registered (§5).
- Verdict space PROMOTE FOR LOWERING / REVISE / NO GEOMETRIC ADVANTAGE / NOT TRIGGERED: pre-registered (§5), measurement in increment 4.

## Verification

Docs-only diff (fast path): claim-wording gate passed locally; no Rust files changed, so the four local gates are unaffected on this PR; the queue re-verifies in full.

## Compatibility, conformance, claim boundary

Research/reference scope only. No production format, runtime, or dependency changes; CONFORMANCE.md and model/ledger.toml untouched; no new RF id. No geometry claim is licensed by this PR — it defines where one could be earned, and a negative outcome is a successful falsification. F0 (#859) remains held; formal backing gates any future lowering, never measurement.

## Intentionally excluded

`crates/uor-r4-graph-certify/tests/geometry_probe_845.rs` (increment 2), all arm/control implementations (increment 3), all measurement results beyond the probe record (increment 4).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

