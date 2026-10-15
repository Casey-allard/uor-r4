# #1510 — [Supporting epic] Resources, recovery and minimal lab reliability

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-09-29T20:06:37Z
- **Updated:** 2026-10-09T14:43:08Z
- **Closed:** 2026-10-09T14:43:08Z
- **Labels:** roadmap, epic, roadmap:parked
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1510
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — PARKED.** PR #1564 / D19; adoption task #1563.

Own concrete current execution needs, cumulative ledger/storage integrity and recoverability. Residuals: #1536 original UNKNOWN-attempt reconciliation, #1542 bounded sentinel reads, #1525 optional adapter/independent continuation. These are not blanket prerequisites to compiler work. Owner-directed FIFO aggregate admission and small direct bounded checks remain; use live physical/process evidence, not old detached-volume holds. New coordination-framework expansion is parked.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

**Current coordination update (30 September 2026):** Current durable-lab implementation: #1520, with recovery #1523, runner/accounting delivery #1524 and dependent operational activation #1525. Source checkpoint and reconciled governance are in PR #1521. Those issue states and evidence receipts determine readiness; a published plan does not establish deployment. Labs are peers with transferable ownership; necessary local extensions are prospectively recorded, and unique material remains preserved.

---

Plan of record: docs/plans/2026-09-29-path-to-chat.md (#1507).
- **`tools/lab-runner` (Rust, LaunchAgent):**
  - job specs carry a wall-time bound and a kill criterion;
  - admission control (threads ≤ 8, RSS ≤ 11 GiB, one GPU job, an exclusive mode for timing);
  - automatic ledger charges.

  It unblocks long runs that the OpenCode harness kills after about 7 minutes.
- **A locked, derived ledger:** charge and extension records are the truth; `model-time.json` is rebuilt from them.
- **Docs reorganization and a professional README;** `STATUS.md`.
- **Space cleanup:** a manifest, a 2-hour notice, and owner menus for unique data.
- **S1.2:** bundle and reload equality. **#1476:** session store serialization.

</details>


## Comments (1)

### comment 6083190035 — 2026-10-09T14:43:07Z — Casey-allard

Superseded by the simplified runner (#1543) and the `uor-pod` rules (compute board #2037).


---
_Comment/review ids in this file: 6083190035_

