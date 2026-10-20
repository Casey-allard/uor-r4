# #1320 — Correct relational-reader attribution and select competitive query-dependent access

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T00:36:01Z
- **Updated:** 2026-10-09T13:17:53Z
- **Closed:** 2026-09-21T00:37:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1320
- **Merged:** 2026-09-21T00:37:40Z
- **Merge commit:** 37bf2bd196b5ed4582a8ed389ed6398fd9d81a9f
- **Base / Head:** main / UOR-Foundation:codex/relational-result-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

PR #1319's categorical reader is trained with one relation index and evaluated with another, so its reported loss cannot establish a benefit specific to H4. Its pair-task construction admits a source-class shortcut, while generation reuses an earlier observation and the cost baseline still performs ring work. The learned relational-versus-exact synthetic improvement remains useful evidence.

This review corrects those claims across the README, current state, roadmap and continuation documents, separates covered decision success from conditional precision and soft expected cost from hard serving loss, and preserves the original reports and failed decision. The single next prompt implements a shared causal current-prefix inference boundary and learns query-dependent selection among competing plausible sources, with specific repairs incorporated. Dependent-read scheduling follows that usable first-read foundation; the broader geometric roadmap remains active.

Validation: two independent source/evidence reviews; saved JSON/population aggregation and SHA256 receipts; verified PR #1319 head/merge tree equality; all 781 checked local Markdown targets resolve; `git diff --check` and claim-wording check pass. Documentation only: no Rust/model execution, ledger debit, artifact mutation or deletion. Current report inventory's unlisted helper is disclosed without changing the preserved root.

References #973, #820, #1139, #962, #963, #964. No broad capability issue is closed.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5753935402 — 2026-09-21T00:38:56Z — Casey-allard

Delivered via merge 37bf2bd196b5ed4582a8ed389ed6398fd9d81a9f. Reviewed head 59e6df2037c47507752b0e0b2147bf269dcbaeec and merge share Git tree 715d90e0141bb6a3c2e7054004ddd4ea9b27590b. Six owning issues (#820/#973/#1139/#962/#963/#964) are synchronized and remain open; none has a project-board item. PR #1319 now carries a principal-correction banner above its preserved report.

Local validation: source and saved-result review by two independent agents, Markdown local-target check (781 resolved), claim-wording check and git diff --check. No Rust build or model run in this review. Ledger remains 182138565/191300000 ms. Final available storage 48,853,479,424 bytes (45.50 GiB); no files deleted. Owner checkout and unique artifacts preserved. DeepSeek checkout retains untracked d4.py (an obsolete documentation-edit helper), which was inspected and left intact rather than published as current code.

The single next task is docs/integration/deepseek-competitive-reader-step-2026-09-20.md. Runtime defects are assigned to that constructive experiment, not represented as fixed by this documentation PR.


---
_Comment/review ids in this file: 5753935402_

