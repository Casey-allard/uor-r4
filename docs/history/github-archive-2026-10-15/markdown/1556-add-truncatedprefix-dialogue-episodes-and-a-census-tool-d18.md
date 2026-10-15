# #1556 — Add TruncatedPrefix dialogue episodes and a census tool (D18 §4)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T21:55:54Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-09-30T22:13:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1556
- **Merged:** 2026-09-30T22:13:41Z
- **Merge commit:** 4ec11d7de371f79f444c8aea5c8ee27e7ec1706e
- **Base / Head:** main / UOR-Foundation:codex/d18-truncated-prefix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this adds (D18 §4, source only)

**`PrefixPolicy::TruncatedPrefix{keep_last}`**
- An index built with `EpisodeIndex::with_policy` admits every response that fits whole after BOS and the assistant marker.
- Each episode keeps BOS and at most the last `keep_last` prefix IDs that fit beside the whole response.
- No response is cut, so there are no mid-response windows.

**FullPrefix and RoleOnly are unchanged.** Their eligibility expression, episodes and records stay the same, and `EpisodeIndex::new` is `with_policy(FullPrefix)`. The new fields `truncated_prefixes` and `truncated_prefix_visits` are omitted from serialized records when they are zero. An index materializes only its own eligibility.

**`dialogue-train policy=truncated_prefix[:KEEP]`** applies to training episodes only; the development panel stays FullPrefix. A bad policy is refused before the report root is claimed (checked with `:0`, `:300` and an unknown name).

**`dialogue-census`** prints each policy's eligible population and an episode digest (the SHA-256 of the eligible spans). It loads no model and writes nothing.

## Census receipt: chat-v0 train split

Manifest `01d7758a…`, tokens `4a554b0e…`, #1017 tokenizer; 2 s and 261 MB RSS.

| Policy | Eligible responses | Eligible response tokens | Prefix cut |
|---|---:|---:|---:|
| `full_prefix` | **14,826** of 129,486 | **1,048,098** of 56,650,286 | — |
| `truncated_prefix` (254) | 50,592 | 6,257,012 | 35,766 |
| `truncated_prefix:128` | 50,592 | 6,257,012 | 39,208 |

- FullPrefix recounts exactly as D18 requires. Its digest is `3f152ef21cdb1d6b7d7affdae86415c6f75faa5fdc873d387df0871efd7af9a4`.
- **Risk for the adoption gate (measured here, not yet trained).** Most added episodes come from `smol-rewrite` (85 → 10,413, of which 10,328 are cut) and `smol-summarize` (401 → 8,981, of which 8,580 are cut). In those, the cut prefix often drops the text being rewritten or summarized, so the target is not derivable from what the model sees. D18's non-regression rule on the relation and abstention cells is the guard, and the paired smoke must report per-source NLL.

## Checks run

- `cargo fmt --check -p uor-r4-training`: passed.
- `cargo test -p uor-r4-training --lib -- dialogue_episodes stack_dialogue`: 12 passed, 4 of them new. The new tests cover:
  - admission of long-history responses and exclusion of a response too long even alone;
  - reduction to FullPrefix (`keep_last` 254 on short documents) and to RoleOnly (`keep_last` = marker length);
  - refusal across eligibilities and of out-of-range `keep_last`;
  - the serde form and the command-line parser.
- A release `dialogue-train policy=truncated_prefix` smoke (344,576-parameter stack, 2 steps, 2 threads, 2 s) trained on the 50,592 responses and recorded the policy and population.
- The exact-head GitHub run is on `codex/ci/v2/truncated-prefix`: the `uor-r4-training` lib and the `geometric-stack` example.

**No training result.** Adoption waits on D18's paired smoke: 512 updates from S4 arm A, 2 seeds per arm, run when DeepSeek's A1 lane is free.

References #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5920632961 — 2026-09-30T22:12:46Z — Casey-allard

**Claude lab, exact-head self-review at `bbc0f2a1`: APPROVE** (owner rule, #820). I read the whole diff. FullPrefix and RoleOnly keep the original eligibility expression (`end - document <= context`) through `EpisodeIndex::new` = `with_policy(FullPrefix)`, and the new count fields are skipped when zero, so existing records serialize unchanged. The census confirms the chat-v0 recount (14,826 / 1,048,098).
- `kept_prefix` cannot underflow, because it is only called after the fit check and the marker lies inside the document.
- An index refuses any other eligibility.
- A bad `policy=` is refused before the report root is claimed.

Exact-head run [36782804274](https://github.com/UOR-Foundation/uor-r4/actions/runs/36782804274), evidence workflow v3, with `UOR_BUILD_SOURCE_COMMIT` set to this head:
- fmt: success.
- all-targets check: success.
- `uor-r4-training` lib: **402 passed, 0 failed**. The 3 `joint_campaign` tests pass once the commit is bound.
- `geometric-stack` example: 8 passed.

This is a full pass. No training result is claimed.

### comment 5920651708 — 2026-09-30T22:14:18Z — Casey-allard

Merged as 4ec11d7de371f79f444c8aea5c8ee27e7ec1706e. Verified: `git merge-tree --write-tree 8c1778ca bbc0f2a1` equals the squash tree (c40c0bbd). `policy=truncated_prefix[:KEEP]` and `dialogue-census` are on main. Adoption still waits on D18's paired smoke.


---
_Comment/review ids in this file: 5920632961, 5920651708_

