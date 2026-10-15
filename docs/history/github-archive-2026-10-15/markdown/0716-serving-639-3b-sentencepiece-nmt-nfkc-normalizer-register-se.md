# #716 — serving/#639-3b: SentencePiece nmt_nfkc normalizer + register (sentencepiece-unigram, 1)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T18:01:53Z
- **Updated:** 2026-08-17T19:55:08Z
- **Closed:** 2026-08-14T18:23:06Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/716
- **Merged:** 2026-08-14T18:23:06Z
- **Merge commit:** 765d3a72028d9d617b1e63860ad10a9def85a8fc
- **Base / Head:** main / UOR-Foundation:i639-3b-sentencepiece-normalizer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second half of #639-3. Closes #715. Builds on the #639-3a Viterbi core. **No CID change; no dependency/Cargo.lock change.**

Makes `sentencepiece-unigram/1` a real registered tokenizer family, so a pinned `spiece.model` encodes raw text → ids faithfully.

## Changes
- **`Normalizer`** (pure Rust, `transformerless::sentencepiece`): parses the `NormalizerSpec` (`ModelProto` field 3) and applies the `precompiled_charsmap` — a Darts double-array trie of longest-match byte replacements (NFKC folding) — then the dummy-prefix / `▁`-escape / whitespace-collapse rules. Reproduces `sentencepiece`'s `Normalize` byte-for-byte.
- **`SentencePieceUnigramTokenizer`** = `Normalizer` + `UnigramModel` Viterbi, implementing `TokenizerModel`; `encode` normalizes then segments. Binds `blake3(spiece.model)` as `tokenizer_cid`; derives a `TokenizerAdapter` identity.
- **Registers `(sentencepiece-unigram, 1)`** in `hf_bpe::adapter_constructor`, replacing refuse-by-name. Refuse-by-name preserved via `SourceUnavailable`: non-Unigram `model_type`, `byte_fallback`, `normalization_rule_tsv`, or absent `precompiled_charsmap` each fail closed.
- Docs: `MODEL_LIFECYCLE.md` tokenizer section — the family is now live.

## Validation
Both the normalizer and the full pipeline were validated **against the reference `sentencepiece` library** before porting. The presence-gated test reproduces the reference token ids on the real pinned T5 `spiece.model` across **22 raw-text fixtures** spanning the charsmap cases — fullwidth (Ａ→A, ①②③→123, ＵＯＲ), ligatures (ﬁ→fi, ﬀﬃ), fractions (½), Roman numerals (Ⅻ), squared forms (㍿), tab/newline, whitespace collapse, CJK/emoji → `<unk>` — plus determinism and the registry-resolution path. The `hf-byte-bpe/1` regression + consumer-agreement seams stay green. Verified in the cloud with the real snapshot; Mac transfer verified. Full local gates: `fmt` + `clippy --workspace --all-targets --all-features -D warnings` + wasm32 lib build + R1–R5 register conformance + tokenizer suites.

## Deferred / non-goals
Wiring this adapter into the `TokenizerKind` selection the observe/serve drivers use is a focused follow-up (kept out so the adapter lands fully tested first). Comprehensive differential fixtures are #639-4. No change to `hf-byte-bpe/1`, the legacy tokenizer, or any pinned artifact. Host/compile-side only.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

