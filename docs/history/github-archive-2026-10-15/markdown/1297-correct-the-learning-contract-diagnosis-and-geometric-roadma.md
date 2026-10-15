# #1297 — Correct the learning contract diagnosis and geometric roadmap

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T02:21:10Z
- **Updated:** 2026-10-09T13:17:37Z
- **Closed:** 2026-09-20T02:23:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1297
- **Merged:** 2026-09-20T02:23:13Z
- **Merge commit:** 6a8778b48ff6fcca235d0e97856270cbd6b14936
- **Base / Head:** main / UOR-Foundation:codex/learning-contract-repair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

PR #1296 reports a correct training/serving contract and a failed contextual fitting gate, but source review finds a missing contextual logit scale, conflicting addition/subtraction targets, and an Adam cutoff that drops valid small gradients. Its nine passing tests do not establish the stated parity, exact resume or loader bounds.

This documentation change records the corrections and an explicit ternary representability witness, then gives DeepSeek one executable sequence: repair the common forward/gradient/optimizer and task; retry the valid small learned gate; run a floating comparator only if needed; continue into bounded real-text fitting when prerequisites pass. The roadmap now separates the local-context baseline from older-prefix-sensitive geometric state and exact-memory integration, then integrated capability and complete M1 cost. README, current state, evidence, model direction, project map and continuation pointers agree; original receipts/artifacts remain preserved.

Validation: independent numerical/source and experimental-design reviews, claim-wording gate, git diff --check, and 681 local Markdown links with no missing targets (one pre-existing historical file URI preserved). Documentation only; no Rust build/model execution or model-time charge. Recorded shared balance remains 149838565 / 154400000 ms. Proposed 3600000 ms tranche is not an executed charge; queue acknowledgements are not tests.

References #973, #820, #964, #963.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

