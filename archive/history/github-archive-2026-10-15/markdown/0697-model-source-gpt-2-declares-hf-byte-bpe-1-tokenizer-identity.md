# #697 — model-source: GPT-2 declares hf-byte-bpe/1 tokenizer identity (#696)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T05:06:48Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-14T05:26:12Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/697
- **Merged:** 2026-08-14T05:26:12Z
- **Merge commit:** bfc2a970c86993fbed2a89eed02b8f690bd6e5b4
- **Base / Head:** main / UOR-Foundation:i696-gpt2-tokenizer-identity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #696 — the last open item (item 4) of the #657 GPT-2 enablement tracker.

## Problem
`HuggingFaceGpt2Oracle::tokenizer_address` returned the generic `"huggingface-tokenizer"` — the same string the Llama oracle reports — so a compiled GPT-2 bundle's #601 tokenizer identity (recorded in `FixtureIdentity.tokenizer`) was indistinguishable from any other Hugging Face source.

## Fix
GPT-2 tokenizes with Hugging Face byte-level BPE (vocab.json + merges.txt / tokenizer.json) — exactly the `hf-byte-bpe/1` adapter family the core `hf_bpe` registry implements and keys `(family, version)` on. The oracle now declares that identity. Spelled as a literal because `uor-r4-core` depends on this crate, so `TokenizerAdapter::HF_BYTE_BPE_FAMILY`/`_VERSION` can't be imported without a cycle; a comment records the provenance. The pinned tokenizer **content** CID remains bound by core's `TokenizerAdapter.tokenizer_cid` (blake3 over the tokenizer bytes) where the pipeline builds it — the oracle surface is the family identity, not a content address, consistent with how every other oracle declares `tokenizer_address`.

Also refreshes the struct doc that still listed the #602 operator record, #603 trace-capture, tokenizer identity, and compile dispatch as deliberately absent — all four have since landed (#672 / trace-capture / this PR / #671). This is the last.

## Scope note
The tokenizer identity lives on the conformance-fixture surface (`FixtureIdentity.tokenizer`), not the cover report (which carries the #597 κ / #600 geometry / #602 operator rows). So the assertion is added as a focused oracle unit test rather than extending the #670 cover canary — the right layer for this surface.

## Verification
- New unit test `gpt2_oracle_reports_byte_bpe_tokenizer_identity` (builds the tiny-fixture oracle, asserts `hf-byte-bpe/1` and that it differs from the generic identity).
- `cargo test -p uor-r4-model-source`: all pass, including the unchanged `huggingface-llama` fixture identity assertion (`declared_baseline_config_records_and_passes_fixture` still expects `"huggingface-tokenizer"`) — SmolLM2/Llama identity and bytes unchanged.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets`: clean.
- `architecture_boundary` leak scan: pass (`hf-byte-bpe/1` is not a GPT-2 family token and stays inside the adapter crate).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

