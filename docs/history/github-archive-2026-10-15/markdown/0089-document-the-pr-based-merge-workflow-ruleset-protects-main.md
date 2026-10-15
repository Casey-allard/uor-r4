# #89 — Document the PR-based merge workflow (ruleset protects main)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T04:05:49Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-07-23T04:33:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/89
- **Merged:** 2026-07-23T04:33:38Z
- **Merge commit:** c6d896648196e0ff258a69fe2dd740c618a9e5bd
- **Base / Head:** main / UOR-Foundation:process/merge-queue-workflow
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Process change companion to the ruleset: main is now protected (ruleset id 19597522) — no direct pushes, five CI checks required with strict up-to-date policy. AGENTS.md gains the per-issue flow: assign -> branch -> PR -> checks -> merge -> close with DoD evidence. The merge-queue rule type was rejected by the API twice (plan/instance limitation); if the Settings UI toggle gets enabled, PRs route through the merge queue with the same workflow. This PR is itself the first change through the new protected path.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

