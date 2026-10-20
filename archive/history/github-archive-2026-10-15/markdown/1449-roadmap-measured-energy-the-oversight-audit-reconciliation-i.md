# #1449 — Roadmap: measured energy, the oversight-audit reconciliation, integration interfaces and the next product milestone (References #820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T03:52:46Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-09-28T04:49:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1449
- **Merged:** 2026-09-28T04:49:20Z
- **Merge commit:** 15d2ce3fac9f66ddcc945a8097e5c6216352c453
- **Base / Head:** main / UOR-Foundation:lab/claude/roadmap-energy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Claude, director. This is a roadmap-only change: `ROADMAP.md` in 2 commits.

**Commit 1 (`ef3bc00a`), first measured serving energy.**
- The integer path measured 0.00413 J/token against 0.00096 for its float parent: whole-system marginal, about 4.3×, and about 2.6× slower.
- Per-product table emulation is recorded as FAILED on energy.
- `powermetrics` CPU power is UNAVAILABLE on Darwin 27. The raw numbers are in the [#820 note](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5862960224).

**Commit 2 (`8b65c611`), the oversight-audit reconciliation.** It carries out the owner's direction and the [director plan](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5863390710):
- **Base decision.** T1(a)'s full-exposure comparison is unpublished in the cloud sandbox, and it is the only dependency. The existing consolidation rule decides it. Until then no new learner starts, and cycle 5 is held.
- **T3.** The auditor misses six float forms, verified on main, #1446 and #1444, so "audit FULL PASS" is withdrawn until it is repaired. `uor-chat` sessions serve only width 256.
- **Other tracks.**
  - T4's #1433 frozen resume is queued.
  - T2 reports admission, ranking and imitation separately.
  - Stage C is parked, and the B1 closure is running.
- **Integration interfaces** I1–I5.
- **§8: the next product milestone, defined prospectively.**
  - One saved-and-reloaded D11 artifact, with responsive multi-turn dialogue, an updated relation and new simple instructions within its declared budget.
  - It applies only after the frozen studies, never retroactively.
- **§9:** a director log entry.

**Checks.** `python3 scripts/check_claim_wording.py` passes. The change touches only `ROADMAP.md`.

References #820, #973, #963

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

