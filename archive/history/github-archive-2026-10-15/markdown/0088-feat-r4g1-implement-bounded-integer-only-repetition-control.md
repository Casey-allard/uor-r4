# #88 — feat(r4g1): implement bounded integer-only repetition control (#79)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-23T04:04:32Z
- **Updated:** 2026-07-23T04:09:44Z
- **Closed:** 2026-07-23T04:09:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/88
- **Merged:** not merged
- **Merge commit:** 9a04bdbf060b86f62ab572e61e45c3891af00831
- **Base / Head:** main / UOR-Foundation:fix-issue-79-repetition-control
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes #79

This PR introduces bounded, allocation-free repetition control to `GraphScorer::score_candidates`.

### Implementation Details:
1. **Recency Window**: `score_candidates` and `score_candidates_legacy` now accept a bounded `recent_tokens` slice (max 32).
2. **Penalty Application**: Saturated `-2,000,000` logic is dynamically applied via `ScoreQ::from_raw` during cloud prediction for any token appearing in the recent context.
3. **Telemetry & Gate C**: 
   - A deterministic greedy simulation on the first 5 viable test seeds is performed inside `evaluate_gate_c`.
   - `repetition_rate_rule12` and `repetition_rate_baseline` are now properly serialized inside the `score_report.json` for downstream monitoring.

### Validation
Passes all unit and integration tests (zero regressions on Gate C accuracy). Verified that the runtime penalty applies seamlessly without triggering memory allocations.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5054207317 — 2026-07-23T04:04:36Z — chatgpt-codex-connector[bot]

You have reached your Codex usage limits for code reviews. You can see your limits in the [Codex usage dashboard](https://chatgpt.com/codex/cloud/settings/usage).

### comment 5054235058 — 2026-07-23T04:09:43Z — Casey-allard

Closing as auser is already handling this.


---
_Comment/review ids in this file: 5054207317, 5054235058_

