# #895 — docs(#822, #823): record the S1 and S2 REVISE stage verdicts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T10:44:35Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-08-21T11:05:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/895
- **Merged:** 2026-08-21T11:05:48Z
- **Merge commit:** 26badb2df35faf689f2c28543c8b987b62308872
- **Base / Head:** main / UOR-Foundation:stage-verdicts-s1-s2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

This morning the maintainer recorded the pending stage decisions; this docs-only PR mirrors them into the canonical plan and reconciles the live summary clauses, so the plan document and RESEARCH.md agree with the tracker records.

## Scope

- `docs/r4_intelligence_completion_plan.md` — appends the **S1 stage verdict — REVISE (2026-08-21)** and **S2 stage verdict — REVISE (2026-08-21)** sections after the S0 PROMOTE record, following its exact format (GitHub trackers #822/#823 remain the source of record). Each mirrors the tracker decision comment: S1 returns to representation/compiler redesign behind an RFC gate with the 20‰ causal floor unmoved; S2 sanctions one redesigned calibrator re-entry under the same frozen gates, with #839 re-scoped to legacy-coverage phase 1 (calibrated phase 2 still gated).
- `docs/RESEARCH.md` — reconciles the three "the stage verdict is the maintainer's call" clauses (S1 §6.3 summary, S2 item-B summary, S3 item-A summary) with parenthetical records of the decisions, per the append-and-reconcile discipline. No history rewritten.

## Non-goals

No code, no conformance changes, no claim upgrades — the verdicts are REVISE records (narrowing/an honest process state), not promotions. Tracker comments (posted this morning on #822, #823, #839, #840, #859) remain the decision records; this PR is the readable mirror.

## Verification

Docs-only diff (fast path applies PR-side; the queue re-verifies in full). `python3 scripts/check_claim_wording.py` and `cargo fmt --check` green locally.

References #822 #823 #839 #840 #859

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01J2R2qs9kfcYpsjUua1SYmi


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

