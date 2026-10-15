# #2167 — Mechanism briefs: one page per novel mechanism, and the test-fitness gate (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T16:11:26Z
- **Updated:** 2026-10-10T16:27:06Z
- **Closed:** 2026-10-10T16:27:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2167
- **Merged:** 2026-10-10T16:27:00Z
- **Merge commit:** 09563cbb32ae677042602258a22bc345566f44b3
- **Base / Head:** main / UOR-Foundation:claude/mechanism-briefs
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: lab governance (owner request) · count n/a · headline unchanged

## What and why

The labs kept testing what they understood (data dose, loss tweaks) rather than the project's novel mechanisms, because nothing told them **why** a mechanism should work or **what a fair test needs**.

`docs/mechanisms/` adds one brief per mechanism:
- flock/rank-table reads;
- exact addressed memory;
- the identity-keyed pointer;
- read binding;
- protected legal construction.

Each brief has:
- the first-principles idea and what it replaces;
- where it lives in code;
- what was tried, honestly (PRs, numbers, and whether each test could judge it);
- what success and failure look like;
- a fair test;
- open design questions.

The Claude lab drafted them from the source and the records, at the owner's request. Numbers were spot-checked against the PRs, for example #2155: bound mass 0.73 → 0.90, wrong values 17 → 12.

Pointers:
- **README.md:** a one-line pointer in How it works.
- **AGENTS.md:** "work from the briefs; `TEST FITNESS: FIT` before mechanism compute".
- **current-state:** an entry.

The gate is enforced by scheduled reviews: an hourly fitness reviewer, plus the 3-hourly audit checking D22, the FIT gate and the README rule.

## Checklist
- [x] README updated (mechanism docs pointer); current-state entry with **Next:**
- [x] `check_claim_wording.py` passes
- [x] Docs only

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6099536653 — 2026-10-10T16:11:29Z — Casey-allard

Exact-head self-review at `1a01abdc596e7a1a7974f53c0db418e974010b12`: docs only, based on fresh origin/main; the claim-wording gate passes.


---
_Comment/review ids in this file: 6099536653_

