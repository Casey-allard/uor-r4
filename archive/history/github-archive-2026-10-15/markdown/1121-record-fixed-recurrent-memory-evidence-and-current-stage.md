# #1121 — Record fixed recurrent memory evidence and current stage

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T17:42:42Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-04T17:44:03Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1121
- **Merged:** 2026-09-04T17:44:03Z
- **Merge commit:** 2a53110ee35e272b79f11df2b4963fa4e2b0e36c
- **Base / Head:** main / UOR-Foundation:codex/issue-973-fixed-recurrent-ledger
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Problem and result

PR #1120 delivered the fixed recurrent mechanism and canonical project track, but the machine-readable claim ledger still ended at the older source-only workbench candidate and pointed active sequencing at an optional #1084 manual qualification.

This docs-only correction adds one scoped empirical claim for `R4FixedRecurrentCausalKVBindingV1`, pins the merged source blob IDs and preserved raw-comparison digest, records the fixed-state/two-prompt measurements and their limits, and makes sparse geometric attention the current ledger stage. The former #1084 pointer remains in `handoff_history`; existing claim statements, scopes, assumptions, observations, evidence, outcomes, limits, and negative or unavailable findings are unchanged.

Independent review reconciled the commit, source blobs, raw hashes, metrics, claim classification, historical preservation, and project-stage pointer without finding a substantive defect.

Focused checks:

- `python3 -m json.tool docs/integration/claim-ledger.json`
- ledger source/evidence-reference/metric/stage-pointer identity check
- merged Git blob and preserved raw-summary digest check
- `python3 scripts/check_claim_wording.py`
- `python3 scripts/check_agent_execution_policy.py`
- `git diff --check`

References #973 and #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

