# #638 — feat(#601): versioned tokenizer adapters + differential source fixtures

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-12T11:09:14Z
- **Updated:** 2026-10-09T14:17:07Z
- **Closed:** 2026-08-12T11:28:36Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/638
- **Merged:** 2026-08-12T11:28:36Z
- **Merge commit:** 0e073952a1b1aa897f3f995da0d65687f9c41720
- **Base / Head:** main / UOR-Foundation:i601-tokenizer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

feat(#601): versioned tokenizer adapters + differential source fixtures

The existing TokenizerKind/HfBpeTokenizer surface becomes a versioned
adapter boundary - no parallel API. TokenizerAdapter record (family
hf-byte-bpe/1, tokenizer_cid = the existing blake3-of-bytes rule,
policy incl. pre-tokenizers, byte fallback, added-tokens digest,
BOS/EOS, chat-template not-interpreted) with a declared-identity digest
over pinned canonical bytes (the #600 rationale). Registry refuses
unknown (family,version) by name through the sanctioned surface
(UnknownTokenizerAdapter kind); sentencepiece-unigram is a recognized-
but-rejected family with a follow-up issue. Differential fixtures: a
full-256-byte-alphabet tokenizer with pinned expected ids/bytes across
ASCII merges, Unicode (CJK, emoji ZWJ), byte fallback, added tokens,
BOS/EOS, digits-boundary differentials - pinning the post-#242/#253
verified behavior as the baseline so any drift fails. Consumer
agreement asserted across observation, evaluation, serving-selection,
and exported-runtime-tokenizer seams (ids + raw decode bytes + one
adapter identity). ObservationManifest.tokenizer_adapter serde(default)
auto-populated on observe-text paths; legacy bytes unchanged when
unset; legacy llama2.c tokenizer and historical kappas untouched.
11+1 new tests; no new deps; wasm-clean. Docs section added.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

