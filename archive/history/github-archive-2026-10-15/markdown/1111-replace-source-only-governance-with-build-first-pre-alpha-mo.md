# #1111 — Replace source-only governance with build-first pre-alpha mode

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T04:52:41Z
- **Updated:** 2026-09-04T04:54:51Z
- **Closed:** 2026-09-04T04:53:58Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1111
- **Merged:** 2026-09-04T04:53:58Z
- **Merge commit:** f7093e4c19058da8ae4f25bb886b0f50cfe7db84
- **Base / Head:** main / UOR-Foundation:codex/lean-development-policy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The source-only policy blocked compilation and execution while requiring extensive research bookkeeping, so routine work could not reach a working product.

This change establishes `build_first_pre_alpha` until a source-free artifact loads and produces prompt-dependent text through the actual CLI or local service. It allows builds, focused tests, model work, services, browser flows, and direct behavior checks; defers proofs, claim/evidence ledgers, knowledge-index maintenance, independent review, experiment freezes, replay packages, publication, NEMESIS/W33 mapping, duplicate status updates, and broad release QA; sets automatic retries to zero; and requires explicit authorization plus limits for runs over 15 minutes, 10 GiB, or any external cost.

The tracked `$uor-project-workflow` skill, contributor guide, current map, roadmap, README, productization plan, and CI transport policy now follow that rule. Duplicated experiment narration was removed from operating entry points; the original research records and negative results remain intact.

Current implementation priority returns to #973. #1107 remains a historical unbuilt checkpoint, #1084 is parked, and #954 remains blocked by #973.

Validation:
- `python3 scripts/check_agent_execution_policy.py`
- `python3 -m json.tool docs/integration/agent-execution-policy.json`
- `git diff --check`

No Cargo suite, model run, proof pass, independent review, replay, ledger update, or knowledge-index refresh was run for this policy-only change.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

