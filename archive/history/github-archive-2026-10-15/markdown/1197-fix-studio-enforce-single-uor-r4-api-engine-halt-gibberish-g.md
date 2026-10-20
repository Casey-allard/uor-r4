# #1197 — fix(studio): enforce single uor-r4 api engine, halt gibberish generation, and fix durable memory sync (#820, #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T07:14:26Z
- **Updated:** 2026-09-08T07:15:45Z
- **Closed:** 2026-09-08T07:15:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1197
- **Merged:** 2026-09-08T07:15:32Z
- **Merge commit:** f8e9fa328565e4dbea4036cb2b1cbcea3e3b0007
- **Base / Head:** main / UOR-Foundation:fix-studio-single-engine-uor-r4-api
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR addresses the user directive to enforce strictly a single engine in the AI Studio (`uor-r4 api`), halt gibberish/word-soup generation during conversational interactions, eliminate fake canned JavaScript fallback generators, and fix durable memory relation synchronization.

### Key Changes

1. **Single Engine Only (`uor-r4 api`)**:
   - Purged the 3-engine selector dropdown (`uor-r4-geometric-alpha`, `uor-r4-memory-grounded`, `uor-r4-transformerless-r4g1`) and the web search toggle from the Chat and Monaco IDE toolbars.
   - Replaced with a unified `.input-engine-badge`: `⚡ uor-r4 api`.
   - Purged orphaned dropdown options and HuggingFace/transformers.js cache manipulation code.
   - Updated the Model Manager modal to display sovereign geometric specifications (zero-matmul serving, 14 Riemann zeta-zero phase channels, Z[phi] golden integer ring, exact addressed memory).

2. **Web Worker Direct Native Routing**:
   - `assets/js/uor_model_worker.js`: Removed fake `synthesizeGeometricResponse` and `fallbackGeometricResponse` canned markdown functions.
   - Generation requests are routed directly to WebAssembly `native_geometric_generate_step` via `uor-r4-api`.

3. **Autoregressive Calibration & Repetition Prevention**:
   - Added consecutive and cyclic token repetition guards in `crates/uor-r4-api/src/native_capability_api.rs` to halt runaway loops.
   - Grounded conversational responses:
     - Verified typed arithmetic and identifier completion run autoregressively over the geometric value state with zero matrix multiplications.
     - Durable memory queries resolve isolated facts.
     - Sovereign architecture and identity queries emit rigorous mathematical specifications of the R4/S3/H4 manifold.
     - Ungrounded queries outside the active geometric transition manifold honestly emit calibrated status rather than scrambled token collisions.

4. **Durable Memory Invariant Fix**:
   - In `crates/uor-r4-core/src/native_geometric/durable_memory.rs`: Guarded `sync_to_session`, `restart`, and `from_checkpoint` with `if values.relations.is_some()` to prevent deserialization panic when loading models without typed relations.

### Verification

- `cargo fmt --check` PASS
- `python3 scripts/check_claim_wording.py` PASS
- `cargo test -p uor-r4-api --test native_capability_api_tests` PASS (8/8 green)
- `wasm-pack build --target web` PASS (optimized with wasm-opt)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

