# #757 — fix(#755): reconstruct corpus context by (story, span_start), not disk order

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T04:12:14Z
- **Updated:** 2026-08-17T08:42:12Z
- **Closed:** 2026-08-17T04:33:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/757
- **Merged:** 2026-08-17T04:33:28Z
- **Merge commit:** 05f4067c2380993459f82fb5b3f2f9972744b58b
- **Base / Head:** main / UOR-Foundation:issue-755-corpus-ordering
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`compiler::load_corpus_bytes` derived per-story context (`input[i]`) from physical record adjacency in `corpus.records`, assuming a story's records sit contiguously on disk. The content-addressed sharded observation pipeline routes each record to a shard by a hash of local context, not by story or position, so that assumption doesn't hold — #752 measured **99.93%** of consecutive records crossing a story boundary on a real compiled bundle (`smollm2-135m-instruct`). Every crossing collapsed `input[i]` to a bogus BOS reset, turning clean observed text into word salad at compile time.

**Fix**: when the corpus format carries independent span anchors (v3/v4, `has_anchors`), sort a permutation of record indices by `(story, span_start)` before deriving `input[]`/context, and apply the identical permutation to every parallel array (`story`, `next`, `t_argmax`, `top_tokens`, `top_weights`, `span_start`, `span_end`, `byte_start`, `byte_end`) and to the teacher hidden-state rows (read separately, aligned to on-disk order).

## Decisive before/after test

Recompiled `smollm2-135m-instruct` from its existing, unmodified corpus bytes via `r4 transformerless compile-recorded` (no re-running the teacher model) — same corpus, same teacher, only the context-reconstruction order changed:

- **Pre-fix**: `r4 ask` hangs ~199s, then emits repetitive single-token-cycling garbage (`ounds Callounds Callounds Call...`).
- **Post-fix**: `r4 ask` answers in ~0.2s with real, grammatical English ("The city was founded in [...], was the first to reach [...] many different types of the Earth's atmosphere, which is a population [...]").

Not good text yet — topically wandering, and two different prompts returned the same output, so prompt-conditioning is still weak or absent at this scale — but categorically different from either garbage or a hang. This is strong evidence the interleaved-ordering bug was a major contributor to #745's degenerate output, though it doesn't by itself establish the architecture produces consistently *good*, prompt-conditioned text. The residual prompt-insensitivity is a real, separate open question, noted in `docs/RESEARCH.md` and left open on #745 for a possible follow-up issue.

## Tests

- New: `crates/uor-r4-core/tests/corpus_record_order_755.rs` — builds a small v4 corpus in natural per-story order and in deliberately interleaved order, asserts `load_corpus_bytes` reconstructs identical `(story, span_start, input, next)` content from both, plus direct spot-checks of chaining and BOS-reset semantics.
- Fixed: `observation_text::merged_records_load_as_v3_corpus` asserted the old buggy on-disk-order-equals-story-order invariant as correct behavior. Rewritten to compare parsed corpus content against expected records by `(story, span_start)` identity instead of raw array index — this is also a direct confirmation from first-party test infra that `merge_shards`' output is genuinely shard order (content-hash order), not story order.

## Verification

- `cargo fmt --check` — pass
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — pass
- `cargo run -p xtask -- audit-limits` (R5) — pass, no unsanctioned error types
- `cargo test --workspace --offline` — all green, no failures
- `python3 scripts/check_claim_wording.py` — pass

## Docs

`docs/RESEARCH.md` updated: the narrative paragraph and the #745/#755 open-work-table rows now reflect the decisive fix-and-retest result instead of describing #755 as merely scoped.

Closes #755
References #745, #752

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

