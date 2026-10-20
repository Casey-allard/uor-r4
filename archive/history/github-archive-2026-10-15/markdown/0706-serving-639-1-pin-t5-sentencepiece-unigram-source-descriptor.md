# #706 — serving/#639-1: pin T5 SentencePiece Unigram source descriptor + κ-verify

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T14:12:20Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-14T14:32:17Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/706
- **Merged:** 2026-08-14T14:32:17Z
- **Merge commit:** d59bc6fca10932738f340a2149b979d8f5858a8b
- **Base / Head:** main / UOR-Foundation:i639-1-t5-source-pin
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First slice of #639 (SentencePiece/Unigram tokenizer adapter). Closes #705. **No CID change.**

Records the pinned Unigram source so the later slices build against a fixed, κ-verified input:
639-2 registry generalization → 639-3 the real Unigram adapter → 639-4 differential fixtures.

## Changes
- **`models/t5-base-tokenizer.json`** — source descriptor mirroring `models/gpt2-124m.json` (#669): repo `google-t5/t5-base`, revision `a9723ea7f1b39c1eae772870f3b547bf6ef7e6c1`, `spiece.model` κ `blake3:18af7072…` / 791656 bytes / scope `spiece.model`, `tokenizer_family: sentencepiece-unigram`, `license: Apache-2.0`.
- **`crates/uor-r4-core/tests/t5_tokenizer_pin.rs`** — CI well-formedness check + a #599 presence-gated κ-verify that reproduces the pin byte-for-byte from the real `spiece.model` and is UNAVAILABLE/vacuous when the snapshot is absent.
- **`.gitignore`** — keep the local T5 source dir out of the tree like other sources.
- **`docs/MODEL_LIFECYCLE.md`** — document the source pin next to the GPT-2 pin; the `sentencepiece-unigram` family stays refused-by-name until 639-3.

## Verification
Pin verified in the cloud against the live HF revision: 791656 bytes, blake3 `18af7072…`, sha256 `d60acb12…` — all match. Presence-gated test **ran for real** against the downloaded `spiece.model` (passed); on the Mac (no snapshot) it reports UNAVAILABLE and vacuous-passes. `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -D warnings` green.

## Non-goals
No change to `hf-byte-bpe/1`, the legacy llama2.c tokenizer, or any κ-pinned artifact. Registry generalization and the real Unigram algorithm follow in 639-2/3.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

