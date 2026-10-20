# #2048 — Retire codex/lab-state: archive its records and history on main; no standing branches

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:49:29Z
- **Updated:** 2026-10-09T16:00:59Z
- **Closed:** 2026-10-09T15:51:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2048
- **Merged:** 2026-10-09T15:51:21Z
- **Merge commit:** 7f9c50eaebd45c07b87ef6fdf57eb84aa9fabe70
- **Base / Head:** main / UOR-Foundation:claude/retire-lab-state
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner decision (9 October 2026): retire the last standing branch.

**What `codex/lab-state` held:** an orphan branch written by `lab-runner coord`, with 1,006 commits and 143 of them in the last 24 hours, almost all 10-minute heartbeats from DeepSeek's and Codex's sessions. It also held 405 coordination records (research reviews, validation, integration, delivery, recovery, handoffs). Their last writes were on 2 October, and they existed only on that branch.

**This PR**
- `docs/history/lab-state-20261009/lab-state.bundle`: the complete branch history (`git bundle verify` passes), with restore instructions in the README.
- `archive/lab-state-20261009/records/`: verbatim plain copies of every directory except `events/` (the heartbeats are only in the bundle). They sit outside `docs/` so the claim-wording gate doesn't re-judge archived third-party wording.
- `AGENTS.md` and `docs/labs/protocol.md`: the standing-branch exception is gone. Claims go on milestone issues (#2028) and leases on compute board #2037. Don't run `lab-runner coord` or recreate the branch.
- `docs/labs/integration-queue.md`: its link into the branch now points at the archive.
- `.github/workflows/lab-delivery.yml` is retired. It read receipts from the branch and was advisory, not a required check, so it would fail on every PR once the branch is deleted.

The branch itself is deleted after this merges **and** after Codex and DeepSeek have stopped their heartbeat loops, so a running loop doesn't recreate it.

Checks: claim wording passes; the bundle verifies; the archive link target exists.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6084323308 — 2026-10-09T15:49:37Z — Casey-allard

Exact-head self-review at `42d96af2eb766b9a110bfc6cdcf432f82e75af0b`: AGENTS/protocol/integration-queue diffs are rule-text removals of the exception plus pointer fixes; bundle verifies; 405 records copied verbatim; workflow removal is advisory-only (not in ruleset 19597522's required checks); claim wording passes.


---
_Comment/review ids in this file: 6084323308_

