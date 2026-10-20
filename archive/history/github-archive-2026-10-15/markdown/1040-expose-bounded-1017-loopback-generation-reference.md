# #1040 — Expose bounded #1017 loopback generation reference

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T02:08:07Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-09-02T02:09:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1040
- **Merged:** 2026-09-02T02:09:30Z
- **Merge commit:** c6e3d95ee39fbb4797b0c8d51507eb5fef978a4a
- **Base / Head:** main / UOR-Foundation:issue-1039-bounded-reference-surface
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Productizes the existing frozen #1017 generator as a bounded local reference surface without changing the model or inference implementation.

- Adds disabled-by-default POST /uor/v1/r4-softmax-local/generate.
- Requires loopback startup and same-origin browser requests.
- Admits only prompt plus an 8-default/32-maximum token cap.
- Keeps attention enabled, greedy decoding, model selection, and exact-executor workers under operator policy.
- Reuses run_r4_softmax_local_generation; no second inference engine.
- Exposes a native-dashboard option only after exact checkpoint readiness.
- Keeps static/WASM, the default cascade, and /v1/chat/completions separate.
- Documents this strictly as a source-backed bounded local generation prototype.

## Real behavior evidence

The predeclared fresh-prompt CLI gate produced a non-looping 24-token continuation and replayed exactly after timing removal at 0.22s/0.17s on Apple Accelerate.

The implemented HTTP endpoint then returned HTTP 200 with an eight-token continuation in 0.16s. A post-review hardened rerun returned the same text and decision/output/audit/state CIDs in 0.20s.

- decision: blake3:87d8e9689132584a1437a244bab4e97e03af9d2609905ae6c74db0fa792c2522
- output: blake3:aef50423770f79714247e2e496d7f49381423893cbb8c636a31276ff9a5aa885
- audit: blake3:6ba5f8d8073748f57692e92a10291d507ef6963251e42069fa42256f56072ffc
- persistent state: blake3:0a837266df1c81d2cc2d54e32c41d6b4e6f0476dfe2bcf04103576278e995390
- loader tree: blake3:66ee347b23e818f1816682f0b942737c88f1eca831cd6d4f00b3d14fc00aaa37
- backend: Apple Accelerate CPU
- attention/decode/source-read audits: exact
- future/provider/Ollama reads: zero
- replay: exact after timing removal
- compact response: prompt/path/file fields absent

## Review hardening

Independent pre-commit review found and resolved:

1. A loader override through an extra Safetensors shard index. Preflight now admits exactly the frozen five-file export and the completed report must reproduce the frozen loader-tree/config/tokenizer/weights identities before HTTP 200.
2. A private checkpoint-path disclosure through the shared reservation subject. The subject is now non-sensitive and cross-operation scrubbing is asserted.
3. Worker wording now distinguishes the exact-executor bound from Accelerate's internal CPU scheduling.

Independent re-review found no remaining blocking issues.

## Focused verification

- 5 endpoint/server tests passed.
- CLI ownership/default test passed.
- Static/WASM masquerade guard passed.
- Real exact-export startup and release Accelerate HTTP generation passed.
- Targeted clippy with warnings denied passed.
- Rust formatting, claim wording, JavaScript syntax, and diff whitespace passed.

The protected merge queue remains the authoritative full integration ladder.

## Claim boundary

This does not establish geometry advantage, transformerlessness, source-free/table-native/multiply-free execution, broad coherence, correctness, reasoning, instruction following, chat quality, browser/WASM inference, release quality, or frontier capability. It does not change #973's V5 terminal, unblock #954, or implement #962.

Closes #1039


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

