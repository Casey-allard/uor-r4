# #1542 — [Parked infrastructure] Bound volume-sentinel reads

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-09-30T08:54:27Z
- **Updated:** 2026-10-09T14:43:11Z
- **Labels:** bug, roadmap:parked
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1542
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — PARKED.** PR #1564 / D19; adoption task #1563.

Parent #1510. Keep the original concrete sentinel-hang repair. Activate for a real current blocker or justified maintenance task; no model experiment, global admission campaign or automatic cleanup follows. Existing outcome/recovery evidence remains.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

Follow-up from the 2026-09-30 hold-reconciliation (receipt: `codex/lab-state:runner1536/hold-reconciliation/receipt.json`; #820 comments 5907501908, 5907630945). References #1510, #1536, #1537.

**Observed.** The deployed lab-runner daemon wedged silently on the first post-hold admission: its main thread blocked in `host::check_storage -> host::verify_volume -> fs::read_to_string(&volume.sentinel) -> open()` on an external volume (/Volumes/UOR-Workspace quarantine-mounted APFS, /Volumes/X10 Pro exFAT-fskit), waiting on macOS removable-volume consent (`kTCCServiceSystemPolicyRemovableVolumes`) for the new unsigned binary's cdhash under launchd. Two `sample` stacks at different PIDs were identical at that frame; the same reads succeeded instantly from shell context. The daemon produced **no error, no admission-blocked.json, no log line** — it appeared alive while admitting nothing, until manually diagnosed with `sample`.

**Defect.** `verify_volume` (tools/lab-runner/src/host.rs:111-135) performs the sentinel `read_to_string` and the `diskutil info` call with **no timeout**. Any hung/slow volume read wedges the entire daemon's admission path silently.

**Ask.** Bound the sentinel read and the `diskutil info` call (a short explicit timeout, e.g. via a bounded spawn or a non-blocking/stat-first check), and on timeout/volume-unavailable return an explicit error that routes through the existing `blocked()` path so the condition surfaces as `admission-blocked.json` with a named reason instead of a silent hang. Keep the sentinel/UUID checks themselves unchanged; this is a failure-surfacing change, not a policy change.

**Scope/priority.** Robustness follow-up. Implement after the current deployment stabilizes (owner's OS consent for the new binary) and through the normal review/delivery cycle. Not a blocker for the reopened admission path once consent lands.

</details>


## Comments (1)

### comment 6083191028 — 2026-10-09T14:43:11Z — Casey-allard

Kept open as a bug in the 9 October tracker rebuild (#2028); related milestone: #2037.


---
_Comment/review ids in this file: 6083191028_

