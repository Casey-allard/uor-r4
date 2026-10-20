# #2000 — iCloud archive audit: blind spots, a defective backing store, and the disk arithmetic (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T08:26:33Z
- **Updated:** 2026-10-09T13:20:34Z
- **Closed:** 2026-10-09T08:28:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2000
- **Merged:** 2026-10-09T08:28:12Z
- **Merge commit:** d1d41d8c8b3696c719eecd428a4586141c7a3b2a
- **Base / Head:** main / UOR-Foundation:deepseek/icloud-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner instruction: make sure all of cloudstore is pushed to icloud. Index 397 to 572 objects, nothing deleted or overwritten, 1.8 GB of verified staging reclaimed. TWO FINDINGS: the chat corpus chat-v0-p2 - which every experiment in this session ran on - had NO archive at all while the store looked complete by object count, as did inputs/ and step0/; and workspace's 42 existing objects resolve to icloud:X10, the SSD with the defective flash region, so the largest archive was on the failing device. Its contents now exist on iCloud proper as 12 ws-* objects. THREE AUDIT BLIND SPOTS, each invisible to the previous check: name matching misses containers; timestamps catch stale archives (and because cloud-store refuses to overwrite, catching up needs a DATED NAME - which is why workspace-2026-10-01 exists); lab labels hide directories archived as many labelled objects. Both error directions occur. THE DISK ARITHMETIC: cloud-store stages a tar equal to the directory size, so a push needs about twice its size free; pushing workspace at once drove free disk from 18 GiB to 5.0 GiB and the job was killed, restoring 18 GiB - correct, because the disk floor is an owner cap outside the never-cancel protection. THE FIX IS STRUCTURAL: recurse to sub-directory granularity and check 2*size + 10 GiB <= free before each object, after which workspace pushed as 12 objects with one logged skip and uor-r4-lab as 41 with zero skips and zero failures. RESIDUE: one 16 KB staging shell capturing a snapshot-preserved directory owned with restrictive permissions; no unique data, left alone rather than forcing deletion of another lab preserved material. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

