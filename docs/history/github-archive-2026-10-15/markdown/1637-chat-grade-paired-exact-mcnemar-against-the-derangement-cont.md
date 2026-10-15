# #1637 — chat-grade: paired exact McNemar against the derangement control

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T00:25:39Z
- **Updated:** 2026-10-09T13:08:43Z
- **Closed:** 2026-10-03T01:46:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1637
- **Merged:** 2026-10-03T01:46:33Z
- **Merge commit:** 4fc2fa1ba9a2cba0e3bde165fc7f071606289aed
- **Base / Head:** main / UOR-Foundation:codex/chat-grade-paired
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

The owner-adopted audit (#820) found that the chat judge could not discriminate at the floor: emit-6r scored 3/64 acceptable against 2/64 for the derangement control. This adds a **paired exact McNemar test**. For every request the actual reply and its derangement control are paired, giving discordant counts and a two-sided exact p, on both the acceptable and the relevant verdicts. The report gains `paired_against_control`, and a chat reading counts only with `discriminates: true` (actual > control and p < 0.05).

**Checks:**
- Unit tests of the exact p-value: 8:0 gives 2/256; balanced gives 1; 10:2 < 0.05; 6:2 > 0.05.
- GitHub evidence: `codex/ci/v3/chat-grade-paired`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5398341902 — 2026-10-03T01:28:25Z — Casey-allard — COMMENTED

Exact-head review at 0bd7f77a39220d4fbe38d8517ca2a905e1954451 (Claude lab; self-review per owner rule on #820). GitHub evidence https://github.com/UOR-Foundation/uor-r4/actions/runs/37082014484 (parent = this head): fmt, check of all targets, lib 586 passed / 0 failed.

### comment 5964250723 — 2026-10-03T01:47:30Z — Casey-allard

Merged as 4fc2fa1b; every changed file on main equals the reviewed head (blob comparison).


---
_Comment/review ids in this file: 5964250723, 5398341902_

