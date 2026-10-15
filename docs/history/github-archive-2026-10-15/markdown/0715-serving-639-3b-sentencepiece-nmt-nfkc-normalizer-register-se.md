# #715 — serving/#639-3b: SentencePiece nmt_nfkc normalizer + register (sentencepiece-unigram, 1)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T18:00:47Z
- **Updated:** 2026-08-14T18:23:07Z
- **Closed:** 2026-08-14T18:23:07Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/715
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second half of #639-3 (the #639 SentencePiece/Unigram adapter). Builds on the #639-3a Viterbi core. **No CID change; no dependency change.**

## Scope
Make `sentencepiece-unigram/1` a real registered tokenizer family, so a pinned `spiece.model` encodes raw text → ids faithfully.

- **`Normalizer`** (pure Rust, in `transformerless::sentencepiece`): parses the `NormalizerSpec` (`ModelProto` field 3) and applies the `precompiled_charsmap` — a Darts double-array trie of longest-match byte replacements (NFKC folding) — then the dummy-prefix / `▁`-escape / whitespace-collapse rules. Reproduces `sentencepiece`'s `Normalize` byte-for-byte.
- **`SentencePieceUnigramTokenizer`** = `Normalizer` + `UnigramModel` Viterbi, implementing `TokenizerModel`; `encode` normalizes then segments. Binds `blake3(spiece.model)` as its `tokenizer_cid` and derives a `TokenizerAdapter` identity.
- **Register `(sentencepiece-unigram, 1)`** in `hf_bpe::adapter_constructor`, replacing refuse-by-name.
- Refuse-by-name preserved via `SourceUnavailable`: non-Unigram `model_type`, `byte_fallback`, `normalization_rule_tsv`, or an absent `precompiled_charsmap` each fail closed.

## Validation
Both the normalizer and the full pipeline were validated **against the reference `sentencepiece` library** before porting. A presence-gated test reproduces the reference token ids on the real pinned T5 `spiece.model` across 22 raw-text fixtures spanning the charsmap cases — fullwidth (Ａ→A, ①②③→123, ＵＯＲ), ligatures (ﬁ→fi, ﬀﬃ), fractions (½), Roman numerals (Ⅻ), squared forms (㍿), tab/newline, whitespace collapse, and CJK/emoji → `<unk>` — plus determinism and the registry-resolution path.

## Deferred (follow-up)
Wiring this adapter into the `TokenizerKind` selection the observe/serve drivers use (so T5 sources actually select it) is a focused follow-up — kept out of this PR so the adapter lands fully tested first. Comprehensive differential fixtures are #639-4.

## Non-goals
No change to `hf-byte-bpe/1`, the legacy tokenizer, or any pinned artifact. Host/compile-side only (`#[cfg(not(target_arch = "wasm32"))]`).

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

