# #341 — docs: record SmolLM2 135M P2 baseline (#320)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T23:51:48Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-08-02T00:06:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/341
- **Merged:** 2026-08-02T00:06:29Z
- **Merge commit:** e2547675697f2450b8b7321e431627e463f3d614
- **Base / Head:** main / UOR-Foundation:issue-320-smollm2-baseline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- record the complete 20k-token SmolLM2-135M rehearsal on the pinned 7e27bd9 revision
- record the HF teacher floor, TLA5/TLS1 row, R4G1 graph row, status split, and witness replay
- preserve the stories15M fixture as a non-comparable historical reference
- document the measured no-migration decision and leave P3/360M as a maintainer decision

## Evidence

- 20,000 records / 199 stories; 19,000 new records in 148 seconds (128.4 tokens/s)
- source κ: `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`
- held-out positions: 4,419
- TLA5/TLS1: 7.6% top-1, 13.0% teacher-argmax agreement, 20.6962 WB bits/token
- teacher floor: 4.9482 bits/token
- R4G1 Rule 1+2: 5.6% top-1, 48.1532 bits/token; normalized: 9.3% / 13.5412; margin: 9.1% / 13.5709
- same-corpus TLA baseline: 13.0% / 20.6962 bits/token
- status split: 1,190 exact context / 3,170 graph / 59 novel
- witness replay: 64/64

## Verification

- `cargo fmt --all -- --check`
- `cargo test --workspace --offline` (100 BDD scenarios / 334 steps pass after temporarily moving the ignored experimental source so the pinned parity suite vacuously skips it; with the experimental 20k bundle present, the existing S2 floor correctly rejects its 0.125 top-8 recall against the 0.14 floor)
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`

Related: #320

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

