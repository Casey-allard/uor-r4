# #1193 — feat(studio): run native geometric model in GitHub Pages AI Studio (#1173)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T05:46:16Z
- **Updated:** 2026-09-08T05:48:32Z
- **Closed:** 2026-09-08T05:47:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1193
- **Merged:** 2026-09-08T05:47:18Z
- **Merge commit:** 63be41736ba873b950ebbf75154917d0d0b8b293
- **Base / Head:** main / UOR-Foundation:issue-1173-wasm-studio
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
This PR integrates the native geometric language model (`WasmModelRuntime`) into GitHub Pages AI Studio (`index.html`) and Web Worker (`r4_worker.js`), fulfilling Roadmap Position 11 ([#1173](https://github.com/UOR-Foundation/uor-r4/issues/1173)) under Programme Tracker [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).

### Key Deliverables
1. **Client-Side WASM Execution**:
   - Integrated `WasmModelRuntime` into `uor-r4-wasm-router` root crate facade (`src/native_wasm.rs`, re-exported in `src/lib.rs`), compiling via `wasm-pack build --target web` and executing entirely within the browser sandbox with zero external API calls or server weights.
2. **Web Worker Event-Loop Isolation**:
   - Extended `r4_worker.js` with native geometric runtime initialization, handle-based session generation, non-blocking UI responsiveness, and fallback to main-thread execution when workers are unavailable.
3. **Interactive Synthesis Engine Selection**:
   - Added `Native Geometric Language Model (WASM)` as an explicit selection in `#engineSelect`, preventing masquerading as reference backends while preserving legacy R4G1 / TLA envelope integrity without regression.
4. **Truthful Capability Presentation**:
   - Rendered dedicated status card displaying canonical UOR address (`uor:native-geometric/r4/1`), hot-path opcodes (zero matmul, zero float), geometric manifold (`Z[phi]`, $S^3$ Hopf), and Capability Truth Matrix explicitly disavowing unproven open-domain reasoning and frontier capability.
5. **Cooperative Cancellation**:
   - Implemented cooperative cancellation via `CANCEL_GENERATION` worker messaging and `wasm_cancel` handle invocation, interrupting multi-token generation gracefully without browser UI freezing.
6. **Durable Session State Portability**:
   - Added browser-level session export and import buttons wired to `wasm_export_session` / `wasm_import_session` with JSON file download/upload controls.
7. **Regression and Integrity Gates**:
   - Added unit test `static_wasm_native_geometric_model_is_connected_and_honest` verifying WASM exports and frontend honesty.
   - Preserved all 15 R4G1 production components and strict fail-closed installation contracts in `static_wasm_r4g1_is_complete_envelope_only_and_fail_closed`.
   - Verified that static WASM cannot masquerade as native R4/Spin softmax reference or local #1017 backends.
8. **Resource Accounting & Quality Gates**:
   - Recorded milestone in `docs/integration/current-state.md` with 4.500s model charge (cumulative 5,355.418 / 5,550s; 194.582s remaining).
   - Preserved 128 MiB storage stop margin.
   - Clean `cargo fmt --check` and `scripts/check_claim_wording.py` (0 violations).

Closes #1173. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

