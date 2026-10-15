# #708 — serving/#639-2: generalize the tokenizer-adapter registry beyond HfBpeTokenizer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T14:52:54Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-14T15:12:29Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/708
- **Merged:** 2026-08-14T15:12:29Z
- **Merge commit:** b6588b39c03d569699af4f8a03ee1b35c8f6d8d9
- **Base / Head:** main / UOR-Foundation:i639-2-tokenizer-registry-generalization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second slice of #639. Closes #707. Introduces the abstraction so a registered tokenizer family can encode/decode **without being an `HfBpeTokenizer`**, setting up the real SentencePiece Unigram adapter (639-3). **`hf-byte-bpe/1` behavior byte-unchanged.** No CID change.

## Changes
- New `pub trait TokenizerModel` in `transformerless::hf_bpe` — the object-safe `encode`/`encode_lossy`/`decode`/`vocab_size`/`adapter` surface a resolved family provides. `impl TokenizerModel for HfBpeTokenizer` delegates to its inherent methods (zero behavior change).
- `AdapterConstructor` widened to `fn(&[u8]) -> Option<Box<dyn TokenizerModel>>`; `adapter_constructor`'s `hf-byte-bpe/1` arm boxes the tokenizer. Every other `(family, version)` stays refused-by-name (`UnknownTokenizerAdapter`).
- `docs/MODEL_LIFECYCLE.md`: registry paragraph updated.

## Regression gate
The existing `registry_resolves_hf_byte_bpe_1` test passes **unchanged** — it asserts the registry-built tokenizer's `adapter()` and `encode()` are identical to a directly-constructed `HfBpeTokenizer`, now through the boxed `TokenizerModel` seam. Full `tokenizer_adapter` differential fixtures + the consumer-agreement (observation/eval/serving/export) seams green. Verified in cloud: `fmt` + `clippy --workspace --all-targets --all-features -D warnings` + tokenizer suites + **wasm32 lib build** all green; Mac transfer byte-verified.

## Deferred to 639-3 (deliberately)
The `TokenizerKind` runtime variant for the new family is added in 639-3 alongside the actual Unigram constructor + graph-cli selection, so the variant is genuinely constructed on the real path (a never-constructed variant would trip the wasm32 lib build's `-D warnings`).

## Non-goals
No Unigram algorithm yet; `sentencepiece-unigram` stays refused-by-name until 639-3. No change to the legacy llama2.c tokenizer or any κ-pinned artifact.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

