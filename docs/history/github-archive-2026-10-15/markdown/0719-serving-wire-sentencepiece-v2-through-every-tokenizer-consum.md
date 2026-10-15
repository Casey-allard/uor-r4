# #719 — serving: wire SentencePiece v2 through every tokenizer consumer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T23:36:29Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-15T00:00:05Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/719
- **Merged:** 2026-08-15T00:00:05Z
- **Merge commit:** a3def9dbb63f3ea54f248a02a61606ee6fd44905
- **Base / Head:** main / UOR-Foundation:issue-718-sentencepiece-consumers
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #718.
References #639.

## Outcome

Completes the final tokenizer-consumer increment for #639 using the maintainer-selected immutable version path:

- freezes `sentencepiece-unigram/1` as explicit-only historical behavior;
- publishes reference-correct `sentencepiece-unigram/2` as current;
- auto-selects `/2` for raw `spiece.model`;
- keeps the raw 32,000-piece definition distinct from the 32,100-slot Hugging Face T5 wrapper.

## Implementation

- Adds one registered, fail-closed source tokenizer resolver with atomic family/version selection.
- Threads the selected tokenizer through raw/serial/batched observation, compile/export, evaluation, typed API, HTTP serving, direct chat, and public R4G1 bundle paths.
- Adds full adapter provenance and byte-preserving incompatible-resume refusal.
- Adds a tagged decode-only runtime tokenizer carrying family/version/source CID/adapter digest.
- Binds exact `tokenizer.bin` bytes into R4G1 HEAD; rejects missing/swapped/zero-CID tagged artifacts and cross-id-space fallback.
- Preserves historical untagged llama2.c and byte-BPE runtime bytes.
- Reports SentencePiece loss honestly and emits no fabricated source-byte anchors.
- Hardens SentencePiece/BPE ingestion against unsupported semantics, malformed tables, invalid scores, sparse/conflicting ids, corrupt tagged headers, and oversized allocations.
- Carries exact token usage from the selected serving tier.
- Adds a deterministic external reference generator and checked-in current-`/2` fixture.

Pinned T5 evidence:

- revision: `a9723ea7f1b39c1eae772870f3b547bf6ef7e6c1`
- bytes: `791,656`
- SHA-256: `d60acb128cf7b7f2536e8f38a5b18a05535c9e14c7a355904270e15b0945ea86`
- source CID: `blake3:18af7072534a34d626ca166c92f5e37c7ccfcd6ce6877c3c25d325bc8201090d`
- frozen `/1` digest: `blake3:cc9b3d1ab746c3ba0cd1f4b94a6b9306a3bd8a41fe9e539800c538a2cee7d8ab`
- current `/2` digest: `blake3:e56dc74051fdaf4a90a626328ad01c81b7cd26044b2b071b8d50302c07bbacd1`

Reference environment: `sentencepiece==0.2.2`, `blake3==1.0.9`, `protobuf==6.33.6`.

## Verification

All required local gates are green on the branch:

- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all --check`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- `python3 scripts/check_claim_wording.py`
- pinned generator export/check and Python bytecode compilation
- `git diff --check`

Non-vacuous focused evidence:

- T5 pin: 2/2
- SentencePiece adapter: 4/4
- adapter registry/consumer agreement: 18/18
- external reference fixture: 3/3, including every one of 32,000 decode ids
- graph compiler library: 104/104
- graph CLI library: 44/44
- API: 29 passed, 1 intentional ignored E2E
- root serving library: 72/72
- graph-certify binding regression: 11/11
- real llama2.c observation/resume and hidden-state checks both executed and passed

The T5-focused run printed no `UNAVAILABLE`.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

