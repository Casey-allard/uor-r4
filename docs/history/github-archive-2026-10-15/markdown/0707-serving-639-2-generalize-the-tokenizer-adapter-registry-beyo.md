# #707 — serving/#639-2: generalize the tokenizer-adapter registry beyond HfBpeTokenizer

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T14:45:12Z
- **Updated:** 2026-08-14T15:12:30Z
- **Closed:** 2026-08-14T15:12:30Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/707
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second slice of #639. Introduces the abstraction so a registered tokenizer family can encode/decode **without being an `HfBpeTokenizer`**, setting up the real SentencePiece Unigram adapter (639-3). **`hf-byte-bpe/1` behavior byte-unchanged** (regression gate).

## Scope
- Introduce `pub trait TokenizerModel` in `uor_r4_core::transformerless::hf_bpe` — the object-safe operation surface a resolved family provides (`encode` / `encode_lossy` / `decode` / `vocab_size` / `adapter`).
- `impl TokenizerModel for HfBpeTokenizer` by delegating to its inherent methods (zero behavior change).
- Widen `AdapterConstructor` to `fn(&[u8]) -> Option<Box<dyn TokenizerModel>>` and box in `adapter_constructor`'s `hf-byte-bpe/1` arm; refuse-by-name for every other `(family, version)` unchanged.

## Regression gate
The existing `registry_resolves_hf_byte_bpe_1` test still asserts the registry-built tokenizer's `adapter()` and `encode()` are **identical** to a directly-constructed `HfBpeTokenizer` — now through the widened `Box<dyn TokenizerModel>` seam. Full `tokenizer_adapter.rs` differential fixtures + the consumer-agreement (observation/eval/serving/export) seams stay green. No CID change.

## Deferred to 639-3 (deliberately)
The `TokenizerKind` runtime variant for the new family is added in 639-3 alongside the actual Unigram constructor + graph-cli selection, so the variant is genuinely constructed on the real path (a never-constructed variant would trip the wasm32 lib build's `-D warnings`). This slice keeps the generalization to the registry seam.

## Non-goals
No Unigram algorithm yet; `sentencepiece-unigram` stays refused-by-name until 639-3. No change to the legacy llama2.c tokenizer or any κ-pinned artifact.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

