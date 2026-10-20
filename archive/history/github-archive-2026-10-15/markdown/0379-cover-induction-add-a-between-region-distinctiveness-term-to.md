# #379 — Cover induction: add a between-region distinctiveness term to the split objective (#364 follow-up a)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T16:27:34Z
- **Updated:** 2026-08-03T19:34:12Z
- **Closed:** 2026-08-03T18:19:34Z
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/379
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up (a) from the #364 campaign synthesis (results: #374; sweep evidence: under probability/contrast the granularity knob converts monotonically — 36.98%→37.70% top-1, 7.60→7.41 bits at 38→102 regions — while under ratio/none it is inert).

The split objective in `induce_cover` (`crates/uor-r4-graph-compiler/src/induction.rs`, keep/split comparison + `entropy_gain_bits` floor) optimizes **within-region** next-token entropy; the scorer's measured payoff depends on **between-region contrast** against the global prior. Those objectives are not the same, and the sweep shows the mismatch is binding.

Scope: add a distinctiveness component to the induction objective — the induction-time analogue of the serving-side contrast (1 − overlap of region top-E vs prior top-E over `next_token_counts` of candidate children). Wire through `ObjectiveWeights` (the versioned slot; default weight 0 = byte-exact default cover), bump `OBJECTIVE_CONFIG_SCHEMA`, surface in `RegionDecisionAudit` → `cover_report.json`, and measure via the revived sweep (#373) at a non-zero weight vs the sweep's current frontier.


## Comments (2)

### comment 5170096997 — 2026-08-03T18:14:13Z — Casey-allard

Coordination: deferring to @auser's #386 — it covers this issue's scope (schema-2 default-off term, sweep flag, w=1.0 measurement) and the #364 entropy-collapse analysis establishes the sharper conclusion that the binding constraint is the proposal mechanism, not the acceptance filter. Unassigning per the assignment=active convention.

One supplementary datum still in flight on our side, posted here when it lands with clear labeling: a **w=4.0** sweep arm from an independent implementation whose estimator references the **parent's own distribution** (children-vs-parent) rather than the global prior — it answers 'does a much stronger veto-side weight move the frontier at all,' which the w=1.0 result leaves open. If it also leaves the frontier unmoved, that closes the veto-side question from both directions and the proposal-mechanism conclusion stands unqualified.

### comment 5170865413 — 2026-08-03T19:34:12Z — Casey-allard

Supplementary datum, closing the veto-side question from the strong end (promised above): an independent implementation of this issue's term — estimator referenced to the **parent's own distribution** (children-vs-parent) rather than the global prior — swept the same 9-point grid under probability/contrast at **w = 1.0 and w = 4.0**. Both arms: **all nine rows identical to the unweighted sweep, to the digit** (top-1, bits, regions, per point). Combined with #386's w=1.0 global-reference measurement: two estimator references × two weight scales, zero split decisions changed. The acceptance-side lever is exhausted; the #364 proposal-mechanism conclusion stands unqualified — distinctiveness has to come from what the clustering is given to propose from, not from how proposals are scored.


---
_Comment/review ids in this file: 5170096997, 5170865413_

