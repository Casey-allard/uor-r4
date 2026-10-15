# #1022 — CPU-native local generation with Apple Accelerate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-31T17:54:30Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-08-31T17:55:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1022
- **Merged:** 2026-08-31T17:55:48Z
- **Merge commit:** 7f5d45251d20610bd09cf82a490a766f9df292b9
- **Base / Head:** main / UOR-Foundation:issue-1019-cpu-native-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

- exposes the learned #1017 R4/Spin causal-softmax checkpoint through the short r4 generate alias and honors UOR_MODEL_STORE
- adds an explicit opt-in local-inference-accelerate feature for Apple Accelerate CPU BLAS while preserving exact uor-matmul as the portable default
- makes canonical and exact overrides bypass BLAS and report the arithmetic that actually executes
- removes the out-of-scope CUDA fallback from the optional #1019 trainer and keeps MPS as its only accelerator
- synchronizes the README, research record, roadmap, programme documents, glossary, ADR, lifecycle, and issue records around #1017 as the active usable model and #1019 as optional/paused

## Actual M1 measurement

Prompt: Once upon a time; 4 generated tokens.

- exact: token IDs [14, 403, 285, 261], text , there was a, generation 3.060506042 s, wall 3.41 s
- Apple Accelerate: identical token IDs and text, generation 0.116236875 s, wall 0.52 s
- speedup: 26.33x generation, 6.56x wall
- identical output CID: blake3:ad043d419e9a3f30cc9be75d6a84f519d988e370a7288f6455763afe6257818e
- identical attention-audit CID: blake3:1552cef6effdb28b3a4b5e1a29313a90beef7df0494ff7230040389c01d4fd78

Full report CIDs intentionally differ because backend provenance, counters, timing, decision identity, and persistent-state identity remain truthful.

## Focused verification

- cargo check --release --offline --features local-inference-accelerate --bin r4
- cargo check --release --offline --bin r4
- cargo fmt --all -- --check
- python3 scripts/check_claim_wording.py
- git diff --check
- paired real-model exact/Accelerate prompt run above

The broad workspace suite was intentionally not duplicated locally. The protected merge queue runs it once as the binding verdict.

References #973
References #1019

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

