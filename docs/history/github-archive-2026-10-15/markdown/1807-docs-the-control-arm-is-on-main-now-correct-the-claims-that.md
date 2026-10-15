# #1807 — docs: the control arm is on main now — correct the claims that say otherwise

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T22:30:50Z
- **Updated:** 2026-10-09T13:19:37Z
- **Closed:** 2026-10-06T22:32:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1807
- **Merged:** 2026-10-06T22:32:21Z
- **Merge commit:** bcc98c10594e8d755c773b4c6d794ea3cbb195ee
- **Base / Head:** main / UOR-Foundation:deepseek/control-landed-docs
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Documentation only — no code, no behaviour change.**

PR **#1806** landed the parameter-matched control on `main`, which made three statements in these two documents false **within the hour they were written**:

- `current-state.md` — a `**Next:**` paragraph saying the control *is not on `main`* and that landing it *needs a decision on the bench's `reachability` subsystem*
- the session record §15 — the same, plus *"three cherry-pick attempts were aborted rather than resolved blind"*
- the session record §10 — a reproduction note resting on the same premise

All three now record:

- **the control is on trunk** (PR #1806, `cherry-pick -x` of `5a37b981`, authorship preserved), so **D20 §2 condition 1 is runnable from `main`**
- **post-relocation verification**: the arm seals at **0.2529** against the original's 0.2529 (3-seed mean 0.2534)
- **why the aborts were wrong**: the hunk-2 conflict is not semantic. `main` defines `stage0_tally_dump`/`stage0_sieve` where the branch defines `read_firing`/`context_reachability` — **different functions added at the same line.** One `grep -nE '^fn '` on both files would have shown it, and this is the **third time** this session a symbol mismatch was read as an architectural question.

Leaving these uncorrected would have reproduced exactly the failure this session kept finding in other artifacts: a document that states the opposite of the truth, in a place readers trust.

References #820.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

