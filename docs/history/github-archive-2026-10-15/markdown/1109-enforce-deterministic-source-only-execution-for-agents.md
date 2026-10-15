# #1109 — Enforce deterministic source-only execution for agents

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T03:56:09Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-04T03:58:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1109
- **Merged:** 2026-09-04T03:58:21Z
- **Merge commit:** aff4acc41f07b866c0f4152440e9028d3cec8088
- **Base / Head:** main / UOR-Foundation:codex/issue-1108-source-only-agent-policy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

An execution-environment failure could previously send an automated agent into partial workspace copies, diagnostic probes, supervisor wrappers, and retries, spending time and tokens without adding product evidence. This change establishes one repository-wide `deterministic_source_only` mode for automated work.

The policy requires a complete Git worktree from refreshed `origin/main`, forbids agent-run or agent-dispatched builds, tests, probes, model work, QA, sparse copies, supervisors, and automatic retries, and limits a concrete owner-run remote failure to one source correction before parking. Owner-operated release QA remains available only through the existing manual `workflow_dispatch` path.

The machine-readable policy is bound into `AGENTS.md` and `CONTRIBUTING.md`. A Python standard-library guard now runs inside the existing required core status on pull requests and merge groups. It validates the canonical policy, binding text, complete checkout, five historical status jobs, automatic-command allowlist, and manual-only boundary for every executable QA job. It imports and executes no project code.

Static validation:

- `python3 scripts/check_agent_execution_policy.py` -> `OK (static text only; no project code executed)`
- staged diff whitespace inspection -> clean
- project builds, tests, probes, model runs, and product evaluations -> `NOT_RUN_BY_POLICY`

Closes #1108.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

