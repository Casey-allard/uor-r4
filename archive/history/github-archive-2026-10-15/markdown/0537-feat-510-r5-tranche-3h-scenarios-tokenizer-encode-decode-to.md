# #537 — feat(#510): R5 tranche 3h — scenarios tokenizer encode/decode to total ops

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T09:03:56Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-09T09:22:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/537
- **Merged:** 2026-08-09T09:22:27Z
- **Merge commit:** c787b89e3469b69db55e5537e44308a7d05f63f4
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t3h-encode
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Converts the legacy tokenizer's buffer-writing encode/decode off `io::Result`.

- `Tokenizer::encode_into` → `Option<usize>` and `decode_into` → `Option<usize>`. These returned `io::Result<usize>` only to signal "caller buffer full" (or a missing byte fallback) — a property of the caller's chosen `out` slice, never an I/O condition. Total now: `Some(count)` written, `None` when the buffer cannot hold the result. The κ-pinned encoding/decoding behavior is unchanged.

Callers updated to the `Option` surface: api `encode_text_into`/`decode_tokens_into` drop the redundant `.ok()`; root `tless_uor` and `r4g1` wrappers drop `.ok()`; `chat` maps `None` to `ChatError::EmptyGeneration` (doc broadened).

The host-ingestion legs of `scenarios.rs` (`try_load` / `from_bytes` / the HF export helpers) remain for a follow-up tranche using the `SourceUnavailable` boundary.

Build / clippy (`-D warnings`) / fmt clean; all workspace test targets compile; api suite green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

