# #1524 — Harden and deliver the existing Rust runner and accounting

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-30T01:35:01Z
- **Updated:** 2026-09-30T03:41:32Z
- **Closed:** 2026-09-30T03:41:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1524
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Child of #1520. Preserve original runner 4d0defa0, now included in #1521; harden existing code rather than replace it. Required: internal control state, mandatory verified host/storage identities, actual RSS/pressure/disk observations, immutable claim/attempt/source/executable/input binding, singleton and one heavy lane, gated launch and PID-reuse safety, conservative interrupted-state recovery, durable receipt then exactly-once charge then finalization, explicit reviewed ledger baseline and late-record reconciliation, GitHub-outage outbox and protected delivery checks. No smoke ledger may become production.

Acceptance: executed focused tests for race, stale-session, crash boundaries, process identity, resource stops, ledger replay/corruption and outage/replay; exact-head independent review and council for the shared policy; protected delivery. Production pilot is tracked separately so code delivery does not imply deployment.


## Comments (1)

### comment 5903597293 — 2026-09-30T03:41:31Z — Casey-allard

**Acceptance complete at this child issue's written scope.** The original runner was preserved and extended, then delivered through #1521's protected queue. The 86-test pass at the prior control-plane head is explicitly reused at its unchanged scope, with final PATH-delta compilation/three focused tests and 23 workflow fixtures. Two exact-head nonauthor approvals and three council seats are retained. Reviewed head a345e766 and merge7895501d have identical Git trees. Production activation, clients and administrative enforcement remain separate #1525 acceptance.

Independent final acceptance audit and exact evidence links/hashes: https://github.com/UOR-Foundation/uor-r4/blob/codex/lab-state/integration/20260930/closeout/child-acceptance-1523-1524.md (SHA-256 a20bd9d6f33a09b4358db93b836da425bf71ed1ec32309c06a01e8a69964e887).

Closing this completed child. #1520/#1525 remain open for their remaining operational requirements; no model capability or automatic client startup is implied.


---
_Comment/review ids in this file: 5903597293_

