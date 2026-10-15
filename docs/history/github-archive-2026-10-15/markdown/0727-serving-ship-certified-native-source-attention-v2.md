# #727 — serving: ship certified-native source attention v2

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-15T15:25:29Z
- **Updated:** 2026-10-09T14:17:17Z
- **Closed:** 2026-08-15T16:19:55Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/727
- **Merged:** 2026-08-15T16:19:55Z
- **Merge commit:** 6fbf718b4115859df6544545a5c43d7638a6ad0a
- **Base / Head:** main / UOR-Foundation:issue-704-certified-native-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- advance immutable source-attention aliases to v2 while preserving every v1 canonical record
- route Llama and GPT-2 QK/value reductions through certified-native f64 cell proofs with pinned exact `uor-matmul` fallback
- bind observation, compilation, and serving provenance to exact operator/source identities while preserving historical-era bundles
- make source download, compile, refresh, and Stage-A checkpoints crash-atomic and cross-process serialized
- keep GPT-2 Conv1D, MLP, and tied `lm_head` conventional

## Decision evidence

The committed-SHA real GPT-2 canary passed on the pinned 124M snapshot:

- candidate median: 1,273,138,500 ns
- conventional median: 1,259,395,084 ns
- paired median ratio: **1.010964087** (acceptance limit: 1.10)
- QK certification rate: 0.999973695
- value certification rate: 1.0
- exact recurrent output parity and zero-allocation guards passed
- pinned `uor-matmul` revision: `b13c98449948174f590e337c4dc25dfc394a07d0`

## Validation

- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all -- --check`
- graph-format `no_std` and `alloc` checks
- wasm32 root-library check
- claim-wording and diff checks
- release ignored real-GPT2 canary on the committed SHA
- independent model-source and whole-diff P0-P2 audits: CLEAN

References #704

#704 remains open because GPT-2 Conv1D, MLP, and tied `lm_head` migration is not part of this PR.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5303003840 — 2026-08-15T15:50:59Z — Casey-allard

R5 follow-up is pushed in `d92fc78d`.

The first PR run correctly caught 22 register-conformance findings. The repair removes the unsanctioned canary/checkpoint `Result` surfaces and adds only exact serde `Visitor` trait-signature carve-outs; there is no generic `A::Error` exemption.

Fresh exact-head evidence:

- `cargo test --workspace --offline`: PASS, including 100 BDD scenarios / 333 steps and doc tests
- workspace all-target/all-feature Clippy with `-D warnings`: PASS
- graph-format no_std + alloc checks: PASS
- wasm32 root lib check: PASS, with pre-existing target warnings only
- R1-R6 register conformance + `cargo deny check`: PASS
- fmt, claim wording, and diff-check: PASS
- real GPT-2 32-token / 5-pair canary: PASS
  - candidate median: 1,261,547,083 ns
  - conventional median: 1,284,445,125 ns
  - paired median ratio: **1.023257773x <= 1.10x**
  - QK certification: 99.9973695%
  - value certification: 100%
  - exact parity, zero hot allocations, and snapshot/toolchain/build identity: PASS

#704 remains open for the deliberately unshipped dense GPT-2 Conv1D/MLP/tied-lm-head scope.

### comment 5303046323 — 2026-08-15T16:02:01Z — Casey-allard

Final CI portability follow-up is pushed in `5d228ea6`.

Ubuntu Clippy exposed one cross-Unix ABI normalization: `libc::dev_t` is already `u64` on Linux but narrower on Darwin. The fix keeps the required Darwin conversion and scopes `allow(clippy::unnecessary_cast)` to that single documented statement. It passed native and `x86_64-unknown-linux-gnu` Clippy with `-D warnings`, both handle-bound traversal race tests, the full workspace suite, no_std, wasm32, fmt, claim wording, and diff-check. Independent review is CLEAN.

The real GPT-2 gate was rerun on the final head:

- candidate median: 1,287,708,750 ns
- conventional median: 1,278,866,709 ns
- paired median ratio: **1.007121087x <= 1.10x**
- QK certification: 99.9973695%
- value certification: 100%
- exact parity/allocation/identity guards: PASS


---
_Comment/review ids in this file: 5303003840, 5303046323_

