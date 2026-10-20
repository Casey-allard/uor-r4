# #1523 — Recover and verify the restored external workspace

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-30T01:34:58Z
- **Updated:** 2026-09-30T03:41:29Z
- **Closed:** 2026-09-30T03:41:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1523
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Child of #1520, under #1510. Preserve the owner checkout and restored sparsebundle; quiesce the retrying supervisor; verify an exclusive recovery copy; inspect a read-only audit mount and filesystem; reconcile all 28 model links and 10 external worktrees; restore the canonical mount only after evidence; repair demonstrated path/cache aliases; independently load a recovered accepted model and verify representative sealed reports. Publish exact recovered/missing/unverified dispositions and the same-SSD backup limitation. No pruning or deletion follows from absent mounts. Current incident receipts live under the internal recovery root recorded in #1520. No scientific quality claim follows from recovery.

Acceptance: source/copy identities and full hash receipt; nonrepair filesystem verification; worktree HEAD/backlink/dirty-state inventory; link dispositions; independent reload/report verification; canonical mount plus precise unresolved material. No issue closure while these are only planned.


## Comments (2)

### comment 5902728716 — 2026-09-30T02:14:06Z — Casey-allard

Workspace restoration checkpoint: the verified original image is now mounted writable at /Volumes/UOR-Workspace, inner UUID DAB4FDD9-1A7A-4ECB-A6D7-41D6D853F7CF. The physical folder remains /Volumes/X10 Pro/UOR-R4. All28 original model symlinks now resolve; all10 registered external worktrees have matching HEADs/backlinks. Six are clean; four dirty worktrees are preserved. No worktree pruning or blanket cache deletion occurred. The separate owner target alias was repaired to its verified archived KVAR cache path.

Preservation evidence: full recovery copy195761554552bytes/5848files matches; fsck_apfs -n -x exit0; five representative S4 sealed roots/18files pass BLAKE3 and complete-file-set checks. Accepted S4 arm-A model SHA256 matches current-state. Same-SSD copy is not an independent device backup. Model reload, legacy ledger migration and per-client compiler qualification are still pending.

Refresh path handles on your next tool call. OpenCode B0 source is internal and remains intact. Do not recreate /Volumes paths, delete or prune missing registrations, or restart historical jobs. New model runs/heavy builds remain held: internal free space is still below40GiB and the real runner/accounting migration is not yet complete. Source work and independent reviews continue.

Runner checkpoint b659331995b887b32d4097c066620739a0f2e83f in draft#1521 passed81 focused Rust tests and22 data-only workflow fixtures. Final adversarial review found a narrow pre-spawn-failure reservation gap and an estimated-time labeling gap; corrections are underway before installation. These passes are infrastructure evidence, not model qualification.


### comment 5903596936 — 2026-09-30T03:41:28Z — Casey-allard

**Acceptance complete at this child issue's written scope.** The complete recovery copy/hash check, nonrepair filesystem verification, canonical remount, all 28 model links, all ten worktree HEADs/backlinks and dirty-state preservation, five sealed report roots, demonstrated target-alias repair, and independent recovered-model CPU load are verified. The later reload receipt supersedes the earlier 'pending' note. The recovered model loaded 78 parameter tensors; this establishes loadability only, not saved-transport execution, integer serving, generation or quality. Same-SSD rollback is still not an independent-device backup.

Independent final acceptance audit and exact evidence links/hashes: https://github.com/UOR-Foundation/uor-r4/blob/codex/lab-state/integration/20260930/closeout/child-acceptance-1523-1524.md (SHA-256 a20bd9d6f33a09b4358db93b836da425bf71ed1ec32309c06a01e8a69964e887).

Closing this completed child. #1520/#1525 remain open for their remaining operational requirements; no model capability or automatic client startup is implied.


---
_Comment/review ids in this file: 5902728716, 5903596936_

