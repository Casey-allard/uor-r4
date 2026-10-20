# #1570 — recall=route: the whole log-sieve route in m-world evaluate (E4)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T08:34:53Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-01T08:46:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1570
- **Merged:** 2026-10-01T08:46:06Z
- **Merge commit:** 98d5546a7cbbae851dc4fef83ada839236f6be50
- **Base / Head:** main / UOR-Foundation:codex/log-sieve-route
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The four commits that were pushed to #1561's branch after it merged at `d2b680f1`. They are cherry-picked onto current main. **The patch is identical** to `d2b680f1..014fcbcf` (`git patch-id --stable` `0e2853c7…` for both).

**What it adds:**
- **`recall=route` in `m-world evaluate`:** the identity channel (R-sieve), then `RelationRoute`.
  - `RelationRoute` is a sparse word table at standardized scale, naming relation and act.
  - The value comes from the latest earlier user turn named as the asked relation.
  - Options: `route_paraphrases=` (a file list), `route_acts=fact|any` and `route_trunk=` (standardized trunk features appended).
- **`m-world compiler`:** `combined` and `table` heads, and paraphrase file lists.

**Results (#1552, cards v12–v17; development cell; diagnostic, not a gate claim):**
- The best route (v16) reaches MQAR **106/109** and open relation **36/52**, against the best trained read at 27/52.
- v17 (trunk features in the table) is a recorded negative.
- E3's declared gate is not met.

**Checks:**
- `cargo fmt --check`.
- On current main: `cargo test -p uor-r4-training --lib -- relation_compiler milestone_world_v2`, 42 passed.
- Release build.
- The exact-head GitHub run is on `codex/ci/v3/log-sieve-route`.

References #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5927932788 — 2026-10-01T08:45:14Z — Casey-allard

**Claude lab, exact-head self-review at `47f51480`: APPROVE** (owner rule, #820).
- The patch is identical to `d2b680f1..014fcbcf` (patch-id `0e2853c7…`). I reviewed those four commits as I wrote them, and each was exercised by the #1552 runs v12–v17.
- `recall=route` and its options are validated before the root is claimed. `recall=off` reports are unchanged.
- The route refuses trunk features unless it was fitted with them (tested).

Exact-head run [36837199079](https://github.com/UOR-Foundation/uor-r4/actions/runs/36837199079), workflow v3:
- fmt, all-targets check: success;
- `uor-r4-training` lib **423/0**;
- `m-world` example 5/0.

### comment 5927952942 — 2026-10-01T08:46:30Z — Casey-allard

Merged as 98d5546a7cbbae851dc4fef83ada839236f6be50. Verified: `git merge-tree --write-tree 02a81e71 47f51480` equals the squash tree (412cf657). The whole log-sieve route is now on main.


---
_Comment/review ids in this file: 5927932788, 5927952942_

