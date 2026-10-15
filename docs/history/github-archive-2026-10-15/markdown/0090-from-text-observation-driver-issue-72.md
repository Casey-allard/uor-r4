# #90 — From-text observation driver (issue #72)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T04:49:52Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-07-23T04:57:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/90
- **Merged:** 2026-07-23T04:57:00Z
- **Merge commit:** 084da91b3a7e5791838bc31c91114e7dd474ce9a
- **Base / Head:** main / UOR-Foundation:issue-72-observe-text
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Teacher-forced encoding of natural text into v3 observation records, completing the D3 Simple English Wikipedia partition as a real observation corpus.

- observe_text.rs: per-article tokenize + teacher step, records via the shared encode_v3_record/softmax_top3_sample/byte_anchors helpers (no second encoder), sample-id content-addressed sharding identical to the generation path, atomic per-article checkpoints converging to the exact single-pass merged κ on resume
- Partition semantics: story = article ordinal; stories.jsonl maps story → {id,url,title,partition}; per-shard {construction, held_out} counts written at write time per the sealed corpus's D3 rule (blake3(id)[0] % 5 == 0)
- observe.rs: optional partition fields (skip_serializing_if=None — generation-path manifest bytes unchanged, pinned by a compat test)
- CLI: r4 transformerless observe-text [--input] [--out] [--shards] [--seconds] [--checkpoint|--source]

Smoke (10 sealed-corpus articles, legacy fixture checkpoint): 2331 records, 16 shards, 1819 construction / 512 held-out records (8/2 articles ≈ 20% held-out, matching the rule; independent recount matched), merged κ blake3:7ba60de6..., rerun writes 0 bytes (idempotent).

Verified in a clean detached worktree (no in-flight edits from other sessions): 47 test suites green, clippy -D warnings clean, cargo fmt --check clean. Unblocks #75 (HF-path full Gate C on the D3 distribution) and #73 (larger-teacher ceiling).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

