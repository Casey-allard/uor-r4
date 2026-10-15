# #538 — feat(#510): R5 tranche 3i — scenarios host-ingestion; core now zero

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T09:39:37Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-09T10:59:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/538
- **Merged:** 2026-08-09T10:59:24Z
- **Merge commit:** caf104099a845917fa57255f9d39b0e742809b22
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t3i-scenarios
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Finishes **core's** R5 conversion: the legacy tokenizer's file/parse boundary. After this, `uor-r4-core` reports **zero** unsanctioned errors under `audit-limits`.

- `Tokenizer::from_bytes` → `Option<Self>` (parsing the runtime `tokenizer.bin` format; a truncated/negative length or truncated token → `None`).
- `Tokenizer::try_load` → `Result<Self, uor_r4_model_source::SourceUnavailable>` — the host-ingestion boundary: an `fs::read` failure (via `From<io::Error>`) or a `tokenizer.bin` that `from_bytes` rejects reports `SourceUnavailable` with the path. The `vocab.json` probe path is unchanged.
- `export_hf_bytelevel_tokenizer` / `_with_lengths` / `hf_bytelevel_tokens` → `Result<_, SourceUnavailable>` — ingesting an external HF `vocab.json` and writing the runtime table; read/parse failure, missing vocab, or an over-long token reports `SourceUnavailable`. Written-file behavior unchanged.

Ripple: api `LoadError::InvalidTokenizer` drops its `io::Error` payload (uses `ok_or`, Display → "invalid tokenizer bytes", `Error::source` arm removed; test asserts `None`); chat maps `try_load`'s `SourceUnavailable` back through `ChatError::Io`; r4g1's `.ok()`/`load` are unaffected.

Build / clippy (`-D warnings`) / fmt clean; all workspace test targets compile; api suite green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

