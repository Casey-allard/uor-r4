# #673 — model-source: pin the GPT-2 tokenizer identity (#669)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T19:30:14Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-08-13T19:50:19Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/673
- **Merged:** 2026-08-13T19:50:19Z
- **Merge commit:** 8f31f4d1467853a3f1b041159ea9ef712bf39a7b
- **Base / Head:** main / UOR-Foundation:i669-gpt2-tokenizer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #657 (item 4). Closes #669.

## Problem
The GPT-2 source reported only the generic `tokenizer_address = "huggingface-tokenizer"` (same as Llama). The #601 versioned surface should bind GPT-2's **exact** tokenizer, not merely the id family, so a compiled GPT-2 bundle is verifiable against a pinned identity.

## Finding
GPT-2's tokenizer is a plain byte-level BPE — one `<|endoftext|>` added token, `ByteLevel(add_prefix_space=false)`, **no** digits splitter (the pre-tokenizer field that distinguishes it from SmolLM2). So the existing `hf-byte-bpe/1` machinery (`uor-r4-core::transformerless::hf_bpe`) ingests its `tokenizer.json` unchanged, and the observe manifest already records the adapter with that file's CID. **No new family or code path is needed** — the gap was a *pinned, verifiable* GPT-2 tokenizer identity.

## Change
- **`models/gpt2-124m.json`**: add `tokenizer_kappa` / `tokenizer_bytes` / `tokenizer_kappa_scope = "tokenizer.json"`, parallel to the `source_kappa` fields. The κ is the blake3 of the revision-`607a30d7` `openai-community/gpt2` `tokenizer.json` (1,355,256 bytes), fetched at the pinned revision.
- **`crates/uor-r4-core/tests/gpt2_tokenizer_pin.rs`**:
  - `gpt2_tokenizer_pin_is_well_formed` — CI-run: the pin is a well-formed `blake3:<64 hex>` scoped to `tokenizer.json` with the pinned byte length.
  - `real_gpt2_tokenizer_matches_the_pin` — presence-gated (#599 three-state): when the real 548 MB snapshot is on disk, its `tokenizer.json` reproduces the pin byte-for-byte and the `hf-byte-bpe/1` adapter built from it carries the same CID; reports UNAVAILABLE and passes vacuously when absent (never a CI download, never a silent skip).
- **`docs/MODEL_LIFECYCLE.md`**: GPT-2 tokenizer-family note under the #601 section.

## Regression gate
SmolLM2 and the shared `hf-byte-bpe` machinery are untouched; the descriptor change is additive (no strict-schema/`deny_unknown_fields` consumer of `models/*.json`).

## Verification (all local, pre-push)
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clean)
- `cargo build --target wasm32-unknown-unknown --lib` (clean)
- `cargo test -p uor-r4-core --test gpt2_tokenizer_pin` (2 pass; real-snapshot test took the UNAVAILABLE path, as expected — no gpt2 snapshot present)
- Register conformance R1–R5 (check-model, repo-conformance, audit-deferral, audit-limits). No new dependencies.

Claim discipline unchanged: this pins and verifies GPT-2's tokenizer identity; it does not compile or certify GPT-2 (items 2/5 remain: #667, #670).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

