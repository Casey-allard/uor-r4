# #545 — feat(#510): R5 — audit-limits matches the Result type, not identifier suffixes

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T14:22:08Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-08-09T14:40:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/545
- **Merged:** 2026-08-09T14:40:55Z
- **Merge commit:** 3c1d67bd5e58efc4914a23dc3b55b2a7317ec638
- **Base / Head:** main / UOR-Foundation:r5/audit-word-boundary
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The R5 gate scanned for the substring `Result<`, which also matched a struct whose name merely ends in `Result` — `ShardReductionResult<R>` in graph-compiler's `observation_shards` carries no error surface but was flagged three times.

`find_result_type` only treats `Result<` as `std::result::Result` when it is not preceded by an identifier character. `io::Result<`, `-> Result<`, `<Result<..>>` still match; `FooResult<..>` does not.

**Effect:** graph-compiler 59 → 56 (three false positives cleared; observation_shards reads 0); no other crate's count changes.

Verified: `cargo build -p xtask`; `clippy -p xtask -D warnings` clean; `cargo fmt --check` clean; `audit-deferral` (R4) passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

