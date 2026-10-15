# #241 — ci: split jobs by trigger context — heavy jobs in the merge queue, audit on the PR (#240)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:32:49Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-07-28T03:47:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/241
- **Merged:** 2026-07-28T03:47:52Z
- **Merge commit:** 02781ebc0c4ee849755a3f749cf10f12f60ad1f9
- **Base / Head:** main / UOR-Foundation:issue-240-ci-context-split
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #240.

Implements the context split proposed in the issue (see the table there): wasm-pack / Gate C trend / fuzz smoke defer to the merge queue's speculative merge (stub success in `pull_request` context); `cargo audit` inverts (real on the PR, stub in the queue — advisory-DB lookup is merge-content-independent); the core `fmt / clippy / tests / no_std / κ` job stays real in both contexts by design.

Every required-check name still reports in every gating context, so queue entry/merge mechanics are unchanged, and every check still runs for real at least once per merge at the point where its verdict is binding. `push: main` and `workflow_dispatch` behavior unchanged. Kani untouched (not in the required set; already single-run).

Note for reviewers: the interesting review question is the stub pattern — paired jobs sharing a `name:` with complementary `if:` conditions. The ruleset half of #240 (dropping the strict up-to-date requirement, admin-only) is separate and not needed for this PR to function.

Gates: all four green locally (macOS arm64, pinned 1.97.1, offline); YAML validated.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

