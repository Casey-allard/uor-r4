# #1517 — README and STATUS: a professional overview and one row per lab

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T20:24:31Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-09-29T20:37:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1517
- **Merged:** 2026-09-29T20:37:38Z
- **Merge commit:** 3bc296cafa89971d3aa0b98c0eca187d8a76ef06
- **Base / Head:** main / UOR-Foundation:lab/claude/readme-status
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820, #1510. **Class A (Lab 1): documentation.** Step 3 of the plan of record (#1507).

**README.md (rewritten, 132 lines)**
- What the project is, with an honest status line.
- The two tracks, and the four mechanisms **with their status**.
- The parity rule.
- The architecture, including its current limits (every weight read per token; 4.3× J/token).
- A repository map and a quick start.
- How the labs run, and the evidence rules: self-reported labels, and D12.
- The dated results log is not deleted. It stays at its pinned commit [`0fb03372`](https://github.com/UOR-Foundation/uor-r4/blob/0fb03372/README.md), so all its relative links keep working.

**STATUS.md (new)**
- The position, with every claim scoped.
- One row per lab: board, current item, latest result, next gate.
- Machine and ledger facts. Each lab edits only its own row.

**AGENTS.md (minimal)**
- The owner extended the lab charter to five labs on 29 September: the Codex lab rejoined for Track B.
- The read-first chain now starts README → STATUS → plan of record.

**Review.** A non-author Sonnet review returned CHANGES-REQUIRED on the first head. Every item is applied in `757296b6`:
- the capability claims are scoped (the Lorentz read won at width 128 in cycle 3; none in the 7M main line);
- QAT is marked self-reported;
- the KVAR and D2 wording is corrected;
- `uor-chat` is stated as having no stack profile yet;
- Track B is stated under D11;
- the time and disk figures are corrected.

**Checks**
- Every relative link in the three files resolves.
- `python3 scripts/check_claim_wording.py` passes.

**Not in this PR:** D13 and the ROADMAP rewrite (they follow the council), and the `docs/` reorganization (the Kimi lab).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

