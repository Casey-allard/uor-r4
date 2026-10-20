# #943 — runtime(#942): carry the generated-prefix signature in production

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T20:40:11Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-25T20:53:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/943
- **Merged:** 2026-08-25T20:53:48Z
- **Merge commit:** b8f78786eb8f7697b4419742008375a58fe8767b
- **Base / Head:** main / UOR-Foundation:issue-942-rolling-trajectory-signature
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Strict schema-2 production decoding now carries a fixed-size incremental signature over the complete admitted prompt plus every committed generated token while retaining the existing bounded scoring window. The incremental state is byte-identical to the prior batch token-history signature at every prefix and performs no per-step allocation.

EOS, abstention, decline, and speculative output do not enter the trajectory state. Existing admission and old-artifact behavior are unchanged.

## Focused verification

- `cargo fmt --check`
- `python3 scripts/check_claim_wording.py`
- `cargo test -p uor-r4-router session_signature --offline` — 5 passed
- `cargo test --lib production_trajectory_keeps_prefix_memory_beyond_runtime_window --offline` — passed
- `cargo test -p uor-r4-graph-runtime --test r4g1_runtime_test session_signature_enters_rout_fallback_as_secondary_probe --offline` — passed
- `cargo test --lib complete_production_envelope_installs_and_corpus_tamper_is_atomic --offline` — passed
- `cargo check -p uor-r4-wasm-router --all-targets --offline`

## Claim boundary

This establishes deployed full-prefix signature reachability. It does not establish coherent generation or revise the historical S3 `LIMIT`; the frozen #841 generation bar remains unchanged.

Closes #942

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

