# #1560 — Log-sieve E1 and E2: an untrained R-sieve route and recall lines in m-world evaluate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T02:20:54Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-01T03:17:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1560
- **Merged:** 2026-10-01T03:17:33Z
- **Merge commit:** e2bbe268667160a1bbf7fafa01b21a32effec796
- **Base / Head:** main / UOR-Foundation:codex/log-sieve-e1-e2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Source for the two evaluation-only experiments of the log-sieve design memo (#1559), with their results.

**What it adds:**
- **`Rule::Sieve` (R-sieve)**, the untrained identity channel: query content words, exact clause admission, latest-version ranking.
  - It is a route, not a freeze rule. `Rule::ALL` (the freeze set, read by `freeze_report` and the probe) is unchanged, and the pinned `STREAM_DIGEST` test passes.
  - `m-world route` scores it beside the reference over the draws `baselines` uses.
- **`m-world evaluate world=v2 recall=off|oracle|sieve recall_at=reply|query`**: one system "Memory: {value}." or "Memory: none." line per MQAR and relation query. `off` is the default and leaves reports unchanged. Values are checked before the report root is claimed.

**Also:** `m-world corpus world=v2 recall=oracle|sieve recall_at=… context=N` adds recall lines to training episodes for the emission curriculum (owner-approved, #1552 card v7). The meter still checks the plain episode.

**Results** (roots `uor-r4-lab/claude-log-sieve/`; detail on #1552):
- **E1 (instrument test):** development MQAR **747/747** at every N and distance; open relation 0/343, as designed.
- **E2 (existing A1 weights):** gate not met in either placement. Before the reply, the line is acknowledged as a new statement. Before the question, it is ignored as a copy source. Emission needs a recall-line curriculum (training), which goes to the owner.

**Checks run:**
- `cargo fmt --check`.
- `cargo test -p uor-r4-training --lib milestone_world_v2`: 37 passed, including the new `the_sieve_route_binds_by_exact_identity_and_the_latest_version` and the stream-digest pin.
- Release build.
- `recall=off` reproduces `eval-L1b-1` exactly.
- Bad `recall=` and `recall_at=` values claim no root.
- The exact-head GitHub run is on `codex/ci/v3/log-sieve-e1-e2`.

References #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5924016997 — 2026-10-01T03:16:41Z — Casey-allard

**Claude lab, exact-head self-review at `bebe7d9c`: APPROVE** (owner rule, #820). I read all three commits.
- **R-sieve stays outside `Rule::ALL`.** `freeze_report` and the probe still iterate only the two freeze rules, and the `STREAM_DIGEST` pin passes.
- **`recall`, `recall_at` and `context`** are validated before the root is claimed (checked with bad values: nothing claimed). `recall=off` reports and corpora are unchanged: `recall=off` reproduced `eval-L1b-1` exactly.
- **In the corpus**, the meter still checks the plain episode, and the lines are added after.

Exact-head run [36805847219](https://github.com/UOR-Foundation/uor-r4/actions/runs/36805847219), workflow v3:
- fmt, all-targets check: success;
- `uor-r4-training` lib **410/0**;
- `m-world` example 5/0.

**Results produced with this source** (#1552):
- E1: 747/747.
- E2: not met without training.
- Emission curriculum: end-to-end MQAR 106/109 with the sieve, and open relation 48/52 with the oracle line. Abstention 8/14 misses its gate.

### comment 5924060576 — 2026-10-01T03:21:09Z — Casey-allard

Merged as e2bbe268667160a1bbf7fafa01b21a32effec796. Verified: `git merge-tree --write-tree 6f21e584 bebe7d9c` equals the squash tree (e54eb0e8).


---
_Comment/review ids in this file: 5924016997, 5924060576_

