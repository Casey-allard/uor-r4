# #1799 — uor-pod up: fall back to one GPU when no host has two free

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T14:44:21Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-10-06T14:50:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1799
- **Merged:** 2026-10-06T14:50:35Z
- **Merge commit:** 27e87ed26bfc604ae044a057b2279c1602d5997c
- **Base / Head:** main / UOR-Foundation:claude/pod-count1-fallback
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1750.

- `scripts/pod/uor-pod`: in `up`'s retry loop, when `--count` was not given and the 2-GPU attempt finds no host, retry at once with 1 GPU (warning names it). `--count 2` keeps the old behaviour.
- `docs/labs/compute.md`: "Low" stock usually means one free card; run a second arm as its own pod.
- `scripts/pod/tests/uor-pod-dryrun.sh`: fake `runpodctl` gains `$FAKE/max-count`; new cases for the fallback and for explicit `--count 2`. The `--wait` case now expects the one-minute poll from #1797 (it was still checking 5 min).

Evidence (10-06): 2-GPU creates failed 15/15; 1-GPU creates worked first try twice (Step 8 arms P and Q).

Checks at head: `bash scripts/pod/tests/uor-pod-dryrun.sh` → passed 192, failed 0 (twice). Negative gate: with the `uor-pod` change reverted, the three fallback checks fail (189/3).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6018768383 — 2026-10-06T14:44:29Z — Casey-allard

Exact-head self-review at `42ea7973a212776f99abe1a975ba194eee36b50c` (owner rule: self-review allowed).

- Diff read in full: 6 lines in `uor-pod` (fallback only when `--count` is empty and count > 1, before the stock print and the --wait branch, so a dry run and --wait both see count=1 on the next pass), docs paragraph, test additions.
- `bash scripts/pod/tests/uor-pod-dryrun.sh` at this head: passed 192, failed 0 (run twice).
- Negative gate: `uor-pod` change reverted → 3 fallback checks FAIL; restored → pass.
- Running pods unaffected (tool-only change). APPROVE.


---
_Comment/review ids in this file: 6018768383_

