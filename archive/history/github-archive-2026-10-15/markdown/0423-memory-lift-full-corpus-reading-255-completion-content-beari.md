# #423 — Memory-lift full-corpus reading (#255 completion — content-bearing storage value)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T01:04:46Z
- **Updated:** 2026-08-05T03:49:11Z
- **Closed:** 2026-08-05T03:49:11Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/423
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The #255 fixture reading was direction-ambiguous (content-free MRR 0.376 ≥ content-derived 0.328, commit 944ac0d) and the promised full-corpus reading never appeared; memory_lift_ab.rs still hardcodes 10 sentences (completeness review 2026-08-05). Point the existing 3-arm harness at the D3 corpus. Decides whether #245's content-bearing restoration carries value — a prerequisite of the #273 claim and the router-reconnection track (#421).

## Comments (2)

### comment 5186987675 — 2026-08-05T02:54:32Z — Casey-allard

Result (PR #427): content-derived MRR 0.2348/top1 0.208 vs content-free 0.1674/0.130 (+0.067, exit rule ≥0.020) vs shuffled 0.0002 — VALUE-CARRYING. Corpus scale resolves the #255 fixture ambiguity in favor of #245's content-bearing restoration.

### comment 5187310065 — 2026-08-05T03:49:11Z — Casey-allard

Scope delivered: full-corpus 3-arm reading ran (PR #427, merged) — content-derived MRR 0.2348 vs content-free 0.1674 vs shuffled 0.0002, VALUE-CARRYING per the pre-declared rule. #245's content-bearing restoration is confirmed at corpus scale. Follow-on router-quality work continues under #421/#422.


---
_Comment/review ids in this file: 5186987675, 5187310065_

