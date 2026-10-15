# #1843 — prepare-text-parquet: stream a parquet text column into a pretrain corpus (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T14:57:51Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-10-07T15:02:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1843
- **Merged:** 2026-10-07T15:02:36Z
- **Merge commit:** 6d3f275a7b3b0db9db92453eafab1141b323dc68
- **Base / Head:** main / UOR-Foundation:claude/prepare-text-parquet
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adds `prepare-text-parquet` (feature `parquet-input`): streams a string column from parquet files into `tokens.u16` (ids + EOS per row, as prepare-text-corpus), with a prepare-text-corpus-schema `manifest.json` binding each input by SHA-256. Supports max_docs, max_tokens, min_chars; batched parallel encoding with deterministic order. Test pins token output, max_docs, min_chars and determinism.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6040678675 — 2026-10-07T14:58:07Z — Casey-allard

Exact-head self-review at `f4f4d6ab394d8fccb91722539734a34f608f7aa3` (owner rule: self-review allowed). New opt-in bin behind `parquet-input`; diff read. At this head: `cargo test -p uor-r4-training --features parquet-input --bin prepare-text-parquet` → 1 passed, 0 failed (tokens = tokenizer(row)+EOS, max_docs, min_chars, determinism); rustfmt clean. `geometric-stack train` consumes only tokens.u16, so the nested manifest layout does not affect training. APPROVE.


---
_Comment/review ids in this file: 6040678675_

