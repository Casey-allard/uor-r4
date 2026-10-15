# #2104 — train-native-prose: --samples N; skipping the demo completions makes runs ~25x faster

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:38:22Z
- **Updated:** 2026-10-09T22:40:17Z
- **Closed:** 2026-10-09T22:39:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2104
- **Merged:** 2026-10-09T22:39:30Z
- **Merge commit:** 85f58a60ffbd8578ea9f190fe0982000e6a99737
- **Base / Head:** main / UOR-Foundation:claude-native-samples-flag
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Profiling** (the owner's laptop, 2M-token window):
- **Stage-2 training is already parallel:** 1.9M tokens in 16.6 s at about 115k tok/s (rayon).
- **The demo completions were the bottleneck:** the run then spent about 10 minutes single-threaded generating 5 demo completions.
- That explains the "one core" behaviour seen on the pod.

**Change:** `--samples N` (default 5, unchanged; 0 skips them).

**Measured:** with `--samples 0` the same run takes **26.7 s** end to end (wall), down from about 11 minutes, with an **identical held-out BPB (1.5964)**. Experiments should pass `--samples 0`.

**Side finding, not fixed here:** the native learner's prose generation is slow per token, minutes for 5 short completions. That is a serving-speed issue to look at separately.

References #2029

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6090439919 — 2026-10-09T22:38:24Z — Casey-allard

Exact-head self-review at `58201a1ea7c2da1b6edbc7c628e1fffac75adafd`: built release locally; ran the 2M-token window with --samples 0 (26.7 s, BPB 1.5964 = default run).


---
_Comment/review ids in this file: 6090439919_

