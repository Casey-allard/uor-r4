# #301 — Proposal (on hold, pending review): preflight parity gate, two-mode trend check, trend path-gating

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T06:18:42Z
- **Updated:** 2026-10-09T13:08:42Z
- **Closed:** 2026-07-31T03:51:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/301
- **Merged:** 2026-07-31T03:51:02Z
- **Merge commit:** 783c6f972764f1cc3d7894655da0e9de889806b4
- **Base / Head:** main / UOR-Foundation:ci-diet
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Proposal for the team's review — on hold, not for merge until reviewed. Contents, in one diff: a local preflight script mirroring the CI gate, a two-mode Gate C trend check (regression when the pin is untouched; re-pin self-consistency when a PR re-pins with an era note), AGENTS.md process-convention additions (per-worktree build isolation, PR watch discipline), and a path filter so the trend harness runs only when a scored-graph surface changed. Motivating measurements and incident history are recorded on #234/#243 and in the #293–#298 PR bodies.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 4824789330 — 2026-07-31T01:54:40Z — auser — COMMENTED

Looks good to me


---
_Comment/review ids in this file: 4824789330_

