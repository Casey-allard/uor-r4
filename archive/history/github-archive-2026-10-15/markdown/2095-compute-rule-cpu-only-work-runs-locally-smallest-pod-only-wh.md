# #2095 — Compute rule: CPU-only work runs locally; smallest pod only when it exceeds the laptop

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T21:51:59Z
- **Updated:** 2026-10-09T22:24:02Z
- **Closed:** 2026-10-09T21:53:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2095
- **Merged:** 2026-10-09T21:53:04Z
- **Merge commit:** fa75074c83de88ea59d8d4a715e8590db5951bf5
- **Base / Head:** main / UOR-Foundation:claude-cpu-local-rule
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction (9 October 2026), correcting text added earlier the same day (#2086):

- **CPU-only work runs on the owner's laptop.** That covers the native prose learner (`train-native-prose`), data preparation, grading and other CPU-bound jobs.
- **Rent a pod for CPU work only when it truly exceeds the local machine** (RAM, disk or wall time), and then choose the smallest pod.
- **GPU work** (geometric-stack training, CUDA evaluation) still goes through `uor-pod` and never falls back to the laptop CPU.

Why: a native-VSA test ran on a 2× RTX PRO 6000 pod (~$5/h), but the learner uses about one core per process and no GPU at all.

Files: `AGENTS.md` (pods bullet), `CONTRIBUTING.md` (data/compute bullets), `docs/labs/compute.md` (step 4). Claim wording passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6089867647 — 2026-10-09T21:52:09Z — Casey-allard

Exact-head self-review at `54214f22507eeeed53497627332b41c7088f3709`: three rule sentences, owner-directed; claim wording passes.


---
_Comment/review ids in this file: 6089867647_

