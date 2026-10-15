# #1163 — Stop repeating native test builds on every PR and merge queue run

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-07T03:12:03Z
- **Updated:** 2026-10-09T13:17:39Z
- **Closed:** 2026-09-07T03:13:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1163
- **Merged:** 2026-09-07T03:13:43Z
- **Merge commit:** 591b45a7c26193a2dbe70c80ec35ca0f3034bd26
- **Base / Head:** main / UOR-Foundation:codex/manual-native-ci
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Routine PRs and merge groups were each compiling and running the native Rust suite under the historical `fmt / clippy / tests / no_std / κ` status, causing two long waits for each small change. At the owner's explicit request, make that status a transparent no-verification acknowledgement, matching the other four required compatibility statuses.

Preserve the exact native formatting, architecture-policy, model, context, allocation and CLI steps under a manually dispatched `qa_scope=native_checks` job. The broader `product_path` and `release_qa` options remain unchanged; actual workspace Clippy remains in manual release QA. Protected PR delivery and the merge queue remain enabled. AGENTS.md documents the behavior and manual command; changed Rust paths still require appropriate focused local verification.

Validation: parsed workflow YAML; checked all five required names and PR/merge-group conditions are unchanged and execute only echo acknowledgements; verified the manual native steps exactly equal the former automatic steps and every other job is unchanged. Claim-wording and diff-whitespace checks pass. No local Rust build or model work. Required acknowledgements are explicitly not test or model-quality evidence.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

