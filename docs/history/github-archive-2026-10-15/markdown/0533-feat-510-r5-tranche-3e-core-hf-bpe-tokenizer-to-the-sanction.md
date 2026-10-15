# #533 — feat(#510): R5 tranche 3e — core hf_bpe tokenizer to the sanctioned surface

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T07:50:05Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-09T08:08:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/533
- **Merged:** 2026-08-09T08:08:59Z
- **Merge commit:** b949ef25f77d483ad37e88df9ab152e71883e572
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t3e-hfbpe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Continues **core** (issue #510): the byte-level BPE tokenizer loader (`uor-r4-core::transformerless::hf_bpe`). Follows region_store/transitions/runtime/reasoning (#526/#527/#528/#532).

- `from_tokenizer_json_bytes` → `Option<Self>`; `parse_pre_tokenizer` → `Option<PreTokenizerConfig>`. Parsing a `tokenizer.json` either yields a valid byte-level BPE tokenizer or it does not — malformed JSON, a non-BPE model, an unsupported pre-tokenizer, or a bad id/merge entry all collapse to `None`.
- `from_dir` → `Result<Self, uor_r4_model_source::SourceUnavailable>`. The **host-ingestion boundary**: a missing `tokenizer.json` (io error, via `From<io::Error>`) or one that is not a valid byte-level BPE tokenizer both report the sanctioned `SourceUnavailable` carrying the path/diagnostic.

Callers unchanged in spirit: graph-cli maps `SourceUnavailable` through `Display` into its own `String` context; the serving path matches `Result`; `.ok()` sites are unaffected. Tokenizer-parse tests assert `.is_none()` on malformed inputs; success paths keep `.expect`.

`audit-limits`: `hf_bpe.rs` reports **zero** unsanctioned errors. Build / clippy (`-D warnings`) / fmt clean; the hf_bpe_tokenizer suite is green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

