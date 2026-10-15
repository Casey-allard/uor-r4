# #705 — serving/#639-1: pin T5 SentencePiece source descriptor + presence-gated κ-verify

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T14:05:30Z
- **Updated:** 2026-08-14T14:32:19Z
- **Closed:** 2026-08-14T14:32:19Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/705
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Sub-issue 1 of #639 (SentencePiece/Unigram tokenizer adapter). Small, self-contained, **no CID change**.

## Scope
Record the pinned Unigram source so later slices (639-2 registry generalization, 639-3 Unigram adapter, 639-4 differential fixtures) build against a fixed, κ-verified input.

## Deliverables
- `models/t5-base-tokenizer.json` — source descriptor mirroring `models/gpt2-124m.json` (#669): repository `google-t5/t5-base`, revision `a9723ea7f1b39c1eae772870f3b547bf6ef7e6c1`, `spiece.model` κ `blake3:18af7072534a34d626ca166c92f5e37c7ccfcd6ce6877c3c25d325bc8201090d` / 791656 bytes / scope `spiece.model`, `tokenizer_family: sentencepiece-unigram`, `license: Apache-2.0`, `source_directory: .uor-models/sources/t5-base-tokenizer`.
- `.gitignore` — explicit entry for the gitignored local source dir.
- `crates/uor-r4-core/tests/t5_tokenizer_pin.rs` — CI test (descriptor well-formed) + presence-gated (#599 three-state) κ-verify test that reproduces the pin byte-for-byte when the snapshot is present, UNAVAILABLE/vacuous pass when absent.

## Verification
Pin verified in cloud against the live HF revision: 791656 bytes, blake3 `18af7072…`, sha256 `d60acb12…` — all match. Presence-gated test run for real against the downloaded `spiece.model`.

## Non-goals
No change to `hf-byte-bpe/1`, the legacy llama2.c tokenizer, or any κ-pinned artifact. Registry generalization and the real Unigram algorithm follow in 639-2/3.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

