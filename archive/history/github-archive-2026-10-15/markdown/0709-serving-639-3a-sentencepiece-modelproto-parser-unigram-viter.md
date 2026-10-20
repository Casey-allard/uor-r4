# #709 — serving/#639-3a: SentencePiece ModelProto parser + Unigram Viterbi core

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T15:19:26Z
- **Updated:** 2026-08-14T15:49:44Z
- **Closed:** 2026-08-14T15:49:44Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/709
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First half of the #639-3 Unigram adapter (split from #639-3 because faithful T5 parity requires the `nmt_nfkc` `precompiled_charsmap`, which is a substantial pure-Rust build — see #639-3b). Closes part of #639.

## Scope (internal core only — not registered, not wired)
- New `crates/uor-r4-core/src/transformerless/sentencepiece.rs`:
  - A minimal, dependency-free protobuf reader (varint / fixed32 / length-delimited) — only the fields we need.
  - Parse `ModelProto`: `pieces` (field 1: `piece`/`score`/`type`) + `trainer_spec` (field 2: `model_type` #3, `byte_fallback` #35, `unk_id` #40).
  - `UnigramModel` with the vocab, per-piece log-prob scores, piece types, `unk_id`, and a derived `byte_fallback`.
  - **Unigram Viterbi** best-path encode + decode over **already-normalized** text (the `▁`-escaped form); `<unk>` for unmatched spans (T5 has `byte_fallback=false`).
  - **Fail closed by name** on a non-Unigram `model_type` (`SSentencePiece` BPE etc.) — never approximate.

## Not in this slice
Normalization (`nmt_nfkc` precompiled_charsmap + dummy-prefix/whitespace), the `(sentencepiece-unigram, 1)` registry entry, the `TokenizerModel`/`TokenizerKind` wiring, and full arbitrary-input parity all land in **#639-3b**. Nothing here is registered or reachable from a driver, so no not-yet-faithful path is exposed.

## Tests
Presence-gated (#599) on the pinned `spiece.model`: the Viterbi core reproduces the reference token ids on NFKC-identity inputs (ASCII / already-composed text, where the charsmap is identity); determinism double-run. UNAVAILABLE/vacuous when the snapshot is absent.

## Non-goals
No CID change; no change to `hf-byte-bpe/1`, the legacy tokenizer, or any pinned artifact.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

