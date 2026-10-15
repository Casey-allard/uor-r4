# #1301 — Recover frozen prior evidence before further training

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T04:06:39Z
- **Updated:** 2026-10-09T13:17:16Z
- **Closed:** 2026-09-20T04:07:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1301
- **Merged:** 2026-09-20T04:07:53Z
- **Merge commit:** 8765aa723b1ec5a057506508e7de0f917eff8fde
- **Base / Head:** main / UOR-Foundation:codex/frozen-prior-evaluation-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

PR #1300's retained real-text prior records a useful loss improvement, but its complete gate is unsupported: the permutation overwrites repeated occurrences and changes target frequencies, and the purported document bootstrap resamples windows. Correct the current claims and publish a comprehensive evaluation-only DeepSeek task for the saved step-512 artifact before another training decision.

The review preserves the positive loss observation, verifies final artifact/archive identities, distinguishes 32 actually evaluated documents from the 36-document pool, and separates full-fit marginal exposure from the optimizer's 4,096 consumed windows. The next run repairs the evaluator, replays the exact population, adds count references and diagnoses actual greedy cycles. The roadmap retains a subsequent learned ordered geometric state/read experiment and exact-memory integration; new state-tracking and distance-decoder research is scoped to that decision.

README, current state, canonical plan, continuation guide, project map, direction assessment, evidence corrections and resource interpretation now agree. Raw receipts, sealed reports and model weights are unchanged.

Validation: claim-wording check and git diff whitespace check passed; all 691 local Markdown links in changed documents resolve. Source/artifact review and arithmetic on saved losses only; no Rust build, model forward or training ran. No resource charge or allowance change. The proposed 30-minute evaluation tranche must be refreshed and recorded before use; the live recorded balance remains 152,238,565 / 154,400,000 ms.

References #973, #820, #963, #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

