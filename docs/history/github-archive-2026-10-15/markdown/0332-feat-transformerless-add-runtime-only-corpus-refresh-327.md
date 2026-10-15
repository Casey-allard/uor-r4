# #332 — feat(transformerless): add runtime-only corpus refresh (#327)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T20:56:51Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-01T21:07:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/332
- **Merged:** 2026-08-01T21:07:45Z
- **Merge commit:** 43dceda3a1071223ac203c2941a842d858df6594
- **Base / Head:** main / UOR-Foundation:issue-327-corpus-scaling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the transformer-free corpus refresh path requested by #327.

- Adds `transformerless runtime-corpus`.
- Loads only a frozen table-native artifact and integer TLS1 store.
- Relabels seed records and extends the stream with greedy runtime predictions.
- Rebuilds the store with bounded coarse-grained workers and ordered insertion.
- Emits corpus files, copied artifacts, rebuilt store, and a BLAKE3 provenance manifest.
- Documents the explicit `runtime-self-distilled-v1` provenance boundary.

This path does not load a Hugging Face checkpoint, tokenizer, teacher oracle, hidden states, floating-point model, or matmul routine. It is not a teacher-fidelity/parity claim.

## Verification

- `cargo check -p uor-r4-graph-cli --offline`
- `cargo clippy -p uor-r4-graph-cli --all-targets --offline -- -D warnings`
- `cargo test -p uor-r4-graph-cli --offline` (15 passed)
- `cargo fmt --all -- --check`
- graph-format no-std checks (default-off and `alloc`)
- End-to-end runtime smoke: 200,000 seed + 100 generated records in ~14s.
- 1-thread and 4-thread outputs were byte-identical for corpus, metadata, artifacts, and store.

The full workspace test suite reaches the existing unrelated `allocation_census` failure: R4G1 reports 2 allocations / 594 bytes where the pre-existing test expects zero. The transformerless runtime census in that same test remains zero-allocation.

Closes #327.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5153449376 — 2026-08-01T21:09:08Z — auser

500k runtime-only benchmark completed from the frozen 200k artifact/store seed.

- 500,000 records total: 200,000 seed + 300,000 generated.
- Direct warm binary: 11.9s with 8 threads vs 13.8s with 1 thread (~14% faster).
- Both runs reported: no teacher or matmul loaded.
- Corpus and rebuilt TLS1 store are byte-identical across thread counts.
- Corpus SHA-256: 6064a79f4c5124ceb943328234eb009ac30a28080f1284a184e1d21e62680a1e
- Store SHA-256: c0d3ce68748e3e45d830a25bb8a8a8a56d99f62db9a73d38dbcc53d9227bf013


---
_Comment/review ids in this file: 5153449376_

