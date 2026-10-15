# #551 — feat(#510): R5 tranche — graph-compiler behavioral probes to total (43→41)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T18:03:12Z
- **Updated:** 2026-10-09T14:17:27Z
- **Closed:** 2026-08-09T18:21:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/551
- **Merged:** 2026-08-09T18:21:58Z
- **Merge commit:** bc9d13e04dfa7947e1683e360c941bc372b6e01b
- **Base / Head:** main / UOR-Foundation:r5/gc-probes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `behavioral_probes.rs` off its `Result<_, BehavioralProbeError>` surface to total forms. **The module is now fully total (0 sanctioned-violation sites).** The `BehavioralProbeError` enum, its `Display`/`Error` impls, and `use std::fmt` are removed.

## Surfaces
- `InterventionRecord::new(...) -> Option<Self>` — `None` when the record is not a product of these inputs: the affected span is out of bounds for the observation, or the baseline/intervention outputs are empty or of mismatched dimension. Both former variants (`SpanOutOfBounds`, `InvalidOutputDimensions`) collapse to `None`.
- `BehavioralProbeHarness::evaluate_suite(...) -> BehavioralProbeReport` (was `Result`) — the anti-memorization guard's verdict is now carried in the report's existing `memorization_check_passed` bool rather than raised as an error. A validator reports the held property; a tripped guard is `false`, not a failure to produce a report. The report is always produced with the computed invariance/sensitivity scores. The never-constructed `AssertionFailed` variant goes with the enum.

`evaluate_suite` / `InterventionRecord::new` have no production callers (graph-certify's `anti_memorization_passed` / `HolographicProbeReport` are unrelated types); only tests and BDD call them.

## Ripple (same PR)
`tests/bdd.rs` — the `probe_suite_error` World field is dropped (the memorization scenario now stores the report and asserts `!memorization_check_passed`) and `probe_record_error` becomes `probe_record_rejected: bool`. The import drops `BehavioralProbeError`.

## Verification
audit-limits graph-compiler **43 → 41** (behavioral_probes.rs 0 sites). 86 lib tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

