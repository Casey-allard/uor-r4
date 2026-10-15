# #1507 — Plan of record 2026-09-29: the path to chat and geometric transformer replacement

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T20:05:59Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-09-29T20:14:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1507
- **Merged:** 2026-09-29T20:14:03Z
- **Merge commit:** 0fb03372f56c4746209a26f8bb00da1b0cf4a49e
- **Base / Head:** main / UOR-Foundation:lab/claude/plan-of-record
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973, #820. **Class A (docs), Lab 1, lead: the owner-approved plan of record.** One new file, `docs/plans/2026-09-29-path-to-chat.md`.

It is the reference every lab's new goal points to until the ROADMAP rewrite and D13 absorb it. It covers:
- **Honest answers:** chat is not achieved; geometric attention has no demonstrated advantage.
- **The root causes:**
  - no in-context retrieval;
  - starvation at about 3.3 tokens per parameter;
  - the 256 window discarding 88% of responses;
  - process.
- **The owner's decisions:**
  - the parity rule;
  - the teacher;
  - local compute;
  - the size ladders.
- **The direction:**
  - **Track A:** the native chat model;
  - **Track B:** geometric conversion of SmolLM2;
  - **four pillars:** flock attention, spherical-harmonic attention, 2I transport, E8 codes.
- **The first 48 hours of parallel tracks,** each with a gate and a kill criterion.
- **Process:**
  - the merge train;
  - the council;
  - the docs reorganization;
  - space cleanup;
  - the runner and ledger;
  - anti-stall rules.
- **The Codex lab's return.**

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

