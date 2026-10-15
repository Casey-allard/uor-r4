# #2450 — Rung-0 completion pre-registration sheet — TEST FITNESS requested (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T06:54:22Z
- **Updated:** 2026-10-14T06:55:59Z
- **Closed:** 2026-10-14T06:55:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2450
- **Merged:** 2026-10-14T06:55:37Z
- **Merge commit:** a83232d4e8ce5f10abb668319661f4694fc5e0c1
- **Base / Head:** main / UOR-Foundation:kimi/rung0-completion-sheet-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Successor sheet to the FIT(design) rung-0 sheet and the completed A2 pilot (#2449).

Registers the completion of the already-registered decisive experiment: seed 1 resumes from the pilot's checkpoint 1,640 (receipt-audited, 236 accepted updates preserved) through 16,404; seed 2 runs the full pass. Bars (i)/(ii), the guard, data pins, evaluator and outcome decisions are carried over unchanged; decisive at 100% on both seeds only.

What the pilot evidence changed and what it did not:
- Guard constant for seed 1 now measured: u = 105/14,287 ≈ 0.735% → threshold ≈ 3.67% (≈ 525 strict wins).
- Cost re-based from UNVERIFIED projection to pilot-measured rate: seed-1 remainder ≈ 15.36h ≈ $13.70; seed 2 ≈ 17.07h ≈ $15.23; combined ≈ $29, inside caps.
- Registered as diagnostics, not verdicts: the rank-vs-loss divergence (token hinge improved, address hinge worsened) and the 236/410 zero-count schedule bias.
- Explicitly NOT changed: initialization (collision-free init stays a named successor), schedule order (frequency reorder stays a named successor), seeds, data, evaluator — comparability with the pilot prefix is preserved.

Dependencies: D5 completion admission diff (Codex: dose-cap extension past the pilot's 1,640, resume admission, seed-2 path, new release scope) → Claude review → Kimi OwnerReleasePlan round → owner release decision. The sheet also surfaces the owner's dose-sequencing choice (two-seed continuous vs seed-1-then-decide) at release time.

TEST FITNESS: PENDING — requested from Claude lab, with Codex source check on D5 when posted.

References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

