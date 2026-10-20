# #754 — fix(tokenizer): detect actual byte convention before applying GPT2 Ġ-remap

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T02:10:21Z
- **Updated:** 2026-08-17T08:41:59Z
- **Closed:** 2026-08-17T02:31:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/754
- **Merged:** 2026-08-17T02:31:07Z
- **Merge commit:** 6d69adf8ce25ab6dadb4acfe301f3251de21de50
- **Base / Head:** main / UOR-Foundation:issue-751-tokenizer-byte-convention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`Tokenizer::encode_into`'s byte-BPE path unconditionally rewrote spaces/newlines/etc. to GPT2's byte-level remap characters before matching against the vocab, assuming the vocab's own bytes use that same convention. Some real `tokenizer.bin` exports don't — `smollm2-1-7b-instruct` (49152 entries) stores literal raw bytes, with zero vocab entries containing the `'Ġ'` byte pair anywhere. Applying the remap there means no merged word token can ever match; every space gets split into two spurious single-byte fallback tokens instead.

Found while investigating #745. Confirmed via direct empirical test that this is a **real but separate** bug: re-running #745's bundle with this fix applied produced byte-identical word-salad output, so this PR does not close #745 (see #745's own thread for the actual root-cause narrowing). Filed and fixed as its own issue (#751) since it measurably improves prompt-encoding fidelity regardless — `"Hello, how are you?"` drops from 12 tokens (with junk byte-pairs at every word boundary) to 6 tokens using real merged tokens.

## Fix

Detect which convention a vocab actually uses once at parse time (does any entry start with the 2-byte `'Ġ'` sequence?) and cache it on `Tokenizer`, consulted by `encode_into` before applying the remap.

A note on why a naive round-trip test doesn't catch this: `decode()` happens to reconstitute the same bytes by coincidence — concatenating the two spurious fallback tokens' raw bytes reproduces the same `'Ġ'` character that `decode()`'s `.replace('Ġ', " ")` then turns back into a space. It only surfaces when inspecting actual token ids/counts, which is documented in the new code comments.

## Verification

- Two new regression tests: a synthetic literal-byte-convention vocab must encode a space-prefixed word to its single merged token (not split into byte-remap fallback pieces); a synthetic GPT2-remap-convention vocab must still have the remap applied (proving the fix is convention-detection, not convention-removal).
- `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`, `cargo run -p xtask -- audit-limits`, `cargo test --workspace --offline` all clean.

References #751

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

