# #724 — serving: bind source attention provenance (#704 A1)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-15T05:11:30Z
- **Updated:** 2026-10-09T14:17:17Z
- **Closed:** 2026-08-15T05:33:12Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/724
- **Merged:** 2026-08-15T05:33:12Z
- **Merge commit:** 1cd9c1dc5b122d157ca75d3e578da19e45b49155
- **Base / Head:** main / UOR-Foundation:issue-704a-attention-provenance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- bind every shipped source teacher and observation path to its exact registered attention-operator record
- publish `attention_operator.json` atomically for source compiles and propagate the recorded identity through API, server, cover, evaluation, and `compile-recorded`
- serialize observation sessions across identity validation, tokenizer export, reconciliation, row writes, and finalization; reject mismatched, malformed, unknown, symlinked, or stale provenance before mutation
- validate authoritative raw/text/compiler checkpoints, completed shard bytes and kappas, corpus/hidden tails, and canonical corpus pairing before resume
- replace the scale sweep's hand-written manifest parsing with the registry-validated `copy-recorded-attention` command
- document that source/snapshot κ remains a content identity; the versioned operator record carries the arithmetic era

## Scope and compatibility

This is the provenance-only A1 slice. It intentionally changes no teacher arithmetic and therefore does not move teacher outputs or require a κ re-pin.

Historical caller-supplied corpora with genuinely absent attention provenance retain the documented implicit `standard-source-attention/1` interpretation. Source-driven resume is stricter: existing payload without the explicit binding is refused rather than relabelled or mixed with a current teacher era.

The raw observation checkpoint still lacks per-shard committed lengths. This PR documents that aligned mid-story tails are not exactly-once recoverable; the checkpoint-format redesign remains a separate follow-up. Text observation already carries per-shard committed lengths.

The dense GPT-2 Conv1D/MLP/lm-head migration remains open. The broad exact-GEMM prototype measured about 220x slower, so the later arithmetic slice must retain the issue's acceptable-speed gate.

## Verification

- `cargo test --workspace --offline`
- `cargo test -p uor-r4-graph-compiler --offline` (131 unit; 26 induction + 1 ignored; 27 observe; 4 route-fit)
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all --check`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- `cargo deny check` (advisories, bans, licenses, and sources pass; only allowed duplicate warnings)
- `cargo run -q -p xtask -- check-model`
- `cargo run -q -p xtask -- audit-deferral`
- `cargo run -q -p xtask -- audit-limits`
- `python3 scripts/check_claim_wording.py`
- `bash -n scripts/scale_sweep.sh`
- non-vacuous Gate E: `/tmp/ref/out/model.bin` = 60,816,028 bytes; all 3 ignored release tests pass under `TLESS_CANONICAL_DETERMINISTIC=1`
- independent final review: CLEAN, no remaining P0-P2 findings

References #704


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

