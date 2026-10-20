# #227 — fix(server): dynamically reload teacher oracle model matching active model selection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-26T18:35:40Z
- **Updated:** 2026-07-26T19:28:28Z
- **Closed:** 2026-07-26T19:28:28Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/227
- **Merged:** 2026-07-26T19:28:28Z
- **Merge commit:** 6a4773f9229fa93e29f449224bc3100f323c4c0d
- **Base / Head:** main / UOR-Foundation:fix/oracle-model-mismatch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Fixes the 12-second latency bottleneck during teacher oracle fallback:
1. **Teacher Oracle Model Mismatch**: At server startup and during `/v1/reload`, `server.rs` now reads `.uor-models/last_model_name.txt` and reloads `oracle` to match the active selected model (`smollm2-135m-instruct` vs `smollm2-1-7b-instruct`).
2. **Speed Improvement**: Prevents running 3.4GB 1.7B teacher forward passes when `smollm2-135m-instruct` is active, dropping prompt processing time from 12.6s down to **< 0.3s** (~40x speedup).

## Verification
- `cargo fmt --check`: **PASS**
- `cargo check --workspace`: **PASS**
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: **PASS (0 warnings)**

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

