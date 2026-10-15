# #2116 — Pod dry-run suite: reap and build-lock checks no longer depend on machine speed (References #2037)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T00:50:59Z
- **Updated:** 2026-10-10T00:54:55Z
- **Closed:** 2026-10-10T00:54:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2116
- **Merged:** 2026-10-10T00:54:40Z
- **Merge commit:** 9bc7939c19b3b3b3071df01b7e96855aa25eb311
- **Base / Head:** main / UOR-Foundation:claude-pod-deflake
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Test-only change; `uor-pod` and the bootstrap are unchanged.

- **reap:** the check expected `idle 30 min`, but pod B's idle clock started when the suite did. Any run that took over a minute to reach the check reported 31 min or more. The test now restarts the clock just before the check.
- **build lock:** the no-progress exit raced the 2 s stale-heartbeat exit, because the holder file was never refreshed. The file is now dated an hour ahead (explicit UTC), so only the exit under test can fire.

**Evidence** (owner's laptop, all 8 cores pinned by `yes`):
- fixed file: **10/10 runs at 246/246**;
- main's file: **6/6 runs failing the reap check**.

The lock race didn't appear in those runs; that fix follows from `wait_for_other_build`.

Record: `docs/labs/pod-test-deflake-2026-10-09/README.md` and a current-state entry.

References #2037

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6091851468 — 2026-10-10T00:53:34Z — Casey-allard

Exact-head self-review at `7cde2a6f346db1587310a0d5c4ed6494d4059cf8` (origin/main merged; current-state.md conflict resolved by keeping both top entries): `bash scripts/pod/tests/uor-pod-dryrun.sh` → passed 246, failed 0. Load evidence in the PR body was at the pre-merge head with an identical test file.


---
_Comment/review ids in this file: 6091851468_

