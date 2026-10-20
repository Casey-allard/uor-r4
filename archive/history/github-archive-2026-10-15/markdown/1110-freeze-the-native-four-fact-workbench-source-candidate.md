# #1110 — Freeze the native Four-fact workbench source candidate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T04:28:11Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-04T04:29:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1110
- **Merged:** 2026-09-04T04:29:21Z
- **Merge commit:** b50d5f901fc717fee9838d361cf7cc29159889f5
- **Base / Head:** main / UOR-Foundation:codex/issue-1107-workbench-source-only
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#1107 had a complete workbench source candidate, but its original delivery contract still expected automated build/probe work that the repository-wide deterministic source-only policy now forbids. This PR rebases that candidate onto current `origin/main`, preserves the safety guard, fixes five independently reviewed source/contract mismatches, and records the bounded terminal `WORKBENCH_CANDIDATE_SOURCE_FROZEN_UNBUILT`.

The source adds the opt-in `uor-r4-workbench` crate, private same-executable worker and comparison entries, and first Four-fact shell defined by ADR-0006. Static review corrected request error precedence, optional host-acceptance discovery behavior, private-comparison completion, inherited descriptor handling, and POST media-type precedence. The roadmap, continuation handoff, claim/evidence ledger, source records, and knowledge-index boundary now point to the source-only result. #1084 remains open and unassigned for an owner decision on separate manual qualification.

Validation:
- Independent static source/contract review: `ACCEPT_SOURCE_ONLY` after five corrections.
- Independent ledger/current-pointer review: `ACCEPT_LEDGER_SOURCE_ONLY`.
- `python3 scripts/check_agent_execution_policy.py` passed; it reads static policy text only.
- `git diff --check` was clean.

Compilation, tests, model operations, numerical comparison, qualification, HTTP/service execution, browser acceptance, portability, performance, security behavior, local knowledge ingestion, and owner-only QA dispatch are `NOT_RUN_BY_POLICY`. No runtime or capability claim is made.

Closes #1107


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

