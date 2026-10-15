# #756 — research(#752): decode observed corpus text, find #745's likely root cause

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T03:02:23Z
- **Updated:** 2026-08-17T08:42:11Z
- **Closed:** 2026-08-17T03:23:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/756
- **Merged:** 2026-08-17T03:23:01Z
- **Merge commit:** 72de67022016cf7f549ced1339962d2532ed5431
- **Base / Head:** main / UOR-Foundation:issue-752-corpus-inspection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- Adds `crates/uor-r4-core/examples/decode_corpus.rs`, a reusable diagnostic that decodes an observed teacher corpus (`corpus.meta`/`corpus.records`) back into text through a bundle's own tokenizer, using `compiler::load_corpus_bytes`'s own parsing rather than reimplementing the record layout (an earlier draft of this tool guessed a fixed record width and produced misleading output — this version goes through the real function to avoid repeating that).
- Comparing three local bundles with this tool found #745's likely root cause: `compiler::load_corpus_bytes`/`runtime::history_token` reconstruct per-story context from **physical record adjacency**, which the content-addressed sharded observation pipeline does not guarantee. `smollm2-135m-instruct`'s corpus vocabulary is indistinguishable from the known-clean `smollm2-360m-broad` control (#509's positive-result bundle), but 99.93% of its consecutive records cross a story boundary — reconstructing stories by grouping-and-file-order (what the compiler does) turns the same clean words into word salad, and explains #745's depth-witness finding.
- Updates `docs/RESEARCH.md`'s open-work table (#745, #752, new #755) and the "which track could work" narrative with this finding.
- Findings posted in full to #745; filed as **#755** (the fix issue, with the fix approach and decisive recompile-and-retest scoped).

Full evidence chain (per-bundle contiguity measurements, decoded story text side-by-side comparisons, code traces through `load_corpus_bytes`/`history_token`/`build_store`) is on #745's and #755's issue threads rather than duplicated here.

Closes #752
References #745, #751, #755

## Test plan

- [x] `cargo fmt --check`
- [x] `cargo clippy -p uor-r4-core --example decode_corpus --offline -- -D warnings`
- [x] `cargo run -p xtask -- audit-limits`
- [x] `cargo test --workspace --offline` (1411 passed, 0 failed)
- [x] `python3 scripts/check_claim_wording.py` (passed clean)
- [x] Manually ran `decode_corpus` against `smollm2-360m-broad`, `smollm2-135m-instruct`, and `smollm2-1-7b-instruct` locally and cross-checked the output against direct `xxd`/Python byte-level inspection of the underlying files


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

