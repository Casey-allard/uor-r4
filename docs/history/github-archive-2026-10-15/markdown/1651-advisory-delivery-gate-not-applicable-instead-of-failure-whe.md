# #1651 — Advisory delivery gate: not-applicable instead of failure when no receipt exists

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T16:47:03Z
- **Updated:** 2026-10-09T13:08:42Z
- **Closed:** 2026-10-03T19:32:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1651
- **Merged:** 2026-10-03T19:32:42Z
- **Merge commit:** 4281e74a5d4492b99cf9e217c804dba49734111e
- **Base / Head:** main / UOR-Foundation:codex/advisory-gate-not-applicable
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

**Owner direction (3 October).** The advisory `Validate delivery evidence (status publisher)` check failed on nearly every PR and on every draft, because labs stopped publishing `codex/lab-state` receipts under the 30 September owner merge rule (exact-head review plus tests). The check is not one of main's required checks, but its red X made merge-ready PRs look broken.

**Change** (`lab-delivery.yml` only). The check now records a notice and publishes **no status** in two cases:
- a draft, closed or non-main PR;
- a PR head with no published receipt (`delivery/pr-N/<head>.json` absent on `codex/lab-state`, or the branch absent).

Published receipts are validated exactly as before, and an invalid receipt still fails. Nothing reports success without verification.

**Checks.**
- The YAML parses (Ruby `YAML.load_file`).
- The embedded github-script body passes `node --check` inside an async wrapper.
- `pull_request_target` runs main's copy of the workflow, so the change takes effect only after merge. The first PR events afterwards are the live check.

**Governance.** This touches `.github/workflows/`. The old D14 class-C council rule is superseded for this change by the owner's direct request in this session. Approval recorded on #820.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5401720956 — 2026-10-03T16:47:13Z — Casey-allard — COMMENTED

Exact-head review at e83e705e874bba9819ef9892fb6c762b8924780c (Claude lab; owner-requested change, self-review per owner rule on #820). Checked: the diff is limited to lab-delivery.yml; drafts, non-main PRs and heads without a receipt return early with a notice and no status; the receipt-present path is unchanged. Checks run at this head: the YAML parses (Ruby YAML.load_file) and the embedded script passes node --check inside an async wrapper. No Rust is changed.

### comment 5972866628 — 2026-10-03T19:48:46Z — Casey-allard

Merged as `4281e74a` through the merge queue at 19:32 UTC, from head `e83e705e`. Verified: the squash patch equals the PR diff (1 file, +26/−1). From now on, PRs without a delivery receipt get a not-applicable notice from the advisory gate instead of a failure.


---
_Comment/review ids in this file: 5972866628, 5401720956_

