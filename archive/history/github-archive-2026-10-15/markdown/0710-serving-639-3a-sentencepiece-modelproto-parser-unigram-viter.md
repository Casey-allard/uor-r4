# #710 — serving/#639-3a: SentencePiece ModelProto parser + Unigram Viterbi core

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T15:30:16Z
- **Updated:** 2026-08-14T15:49:43Z
- **Closed:** 2026-08-14T15:49:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/710
- **Merged:** 2026-08-14T15:49:43Z
- **Merge commit:** dff62bfe95923950d35044ce2fadfe38ff32f65a
- **Base / Head:** main / UOR-Foundation:i639-3a-sentencepiece-unigram-core
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First half of the #639-3 SentencePiece/Unigram adapter (split because faithful T5 parity needs the `nmt_nfkc` `precompiled_charsmap`, a substantial pure-Rust build — see #639-3b). Closes #709. **No CID change.**

Adds the algorithmic core as an **internal, not-yet-registered** module; the normalizer, the `(sentencepiece-unigram, 1)` registry entry, and the `TokenizerModel`/`TokenizerKind` wiring land in **#639-3b**, so no not-yet-faithful path is reachable from a driver here.

## Changes
- New `crates/uor-r4-core/src/transformerless/sentencepiece.rs`:
  - Dependency-free protobuf reader (varint / fixed32 / length-delimited) parsing `ModelProto` `pieces` (surface / `score` / `type`) + `trainer_spec` (`model_type` #3, `byte_fallback` #35, `unk_id` #40).
  - `UnigramModel::encode_normalized` — Unigram best-path (Viterbi) over already-normalized text. Every character is covered by a per-char unknown node scored `min_score − kUnkPenalty`; **consecutive unknowns collapse to a single `<unk>`** in the output, matching SentencePiece exactly. `decode` inverts it up to lossy spans.
  - **Refuse-by-name**: non-Unigram `model_type`, `byte_fallback` sources, missing/ambiguous `<unk>`, and malformed proto bytes each fail closed with a named `SentencePieceError`.

## Tests
6 synthetic CI-path unit tests (parse / Viterbi tie-break / unk-collapse / refusals) run everywhere. A #599 **presence-gated** integration test reproduces the reference `sentencepiece` token ids on the **real pinned T5 `spiece.model`** across 11 fixtures — ASCII, accents, digits, collapsed whitespace, single + adjacent-collapsed (CJK run) + non-adjacent unknowns — plus a determinism double-run; UNAVAILABLE/vacuous without the snapshot. Verified in the cloud against the real snapshot; `fmt` + `clippy --workspace --all-targets --all-features -D warnings` + wasm32 lib build all green; Mac transfer verified.

## Method note
The fixtures feed the **normalizer-independent** surface string (the joined reference pieces), isolating the Viterbi core from the normalizer that arrives in #639-3b. The algorithm was validated against the reference `sentencepiece` library on a broader corpus before porting to Rust.

## Non-goals
Normalization, registration, and wiring are #639-3b; full arbitrary-input differential parity is #639-4. No change to `hf-byte-bpe/1`, the legacy tokenizer, or any pinned artifact.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

