# #199 — feat(ui/wasm): Web Worker offloading and WASM event-loop isolation (#193)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-26T01:43:01Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-07-26T01:54:48Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/199
- **Merged:** 2026-07-26T01:54:48Z
- **Merge commit:** 987afd30cff0c5faab6a01a9e15aee369aaa1373
- **Base / Head:** main / UOR-Foundation:issue-193-web-worker-isolation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent Epic: #198
Closes #193

## Summary of Changes

- **Web Worker Event-Loop Isolation (`r4_worker.js`)**: Created `r4_worker.js` Web Worker to instantiate the local WebAssembly router module (`uor_r4_wasm_router.js` / `.wasm`) off the main browser DOM thread.
- **Asynchronous Message Protocol**: Implemented message handlers for `INIT_ENGINE`, `GENERATE_RESPONSE`, and `INDEX_SENTENCE` with promise-based callback resolution (`runWorkerGeneration`).
- **Non-blocking UI Integration (`index.html`)**: Updated DOM initialization to delegate WASM computation to `r4_worker.js` via `postMessage`, keeping user interaction and DOM execution completely unblocked.
- **CLI Progress Output (`crates/uor-r4-graph-cli/src/lib.rs`)**: Added periodic percentage progress output (`progress: evaluated X/Y positions (Z%, Ts)`) to `evaluate-report` for real-time CLI feedback.

## Acceptance & Gate Verification

- [x] **Local Workspace Tests**: `cargo test --workspace --offline` passed (all 94 BDD scenarios green).
- [x] **Clippy Check**: `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` passed with 0 warnings.
- [x] **Format Check**: `cargo fmt --check` passed cleanly.
- [x] **no_std Ladder**: `cargo check -p uor-r4-graph-format --no-default-features` & `cargo check -p uor-r4-graph-format --no-default-features --features alloc` passed cleanly.
- [x] **Main Thread Isolation**: WASM candidate scoring and generation run inside Web Worker `r4_worker.js`, preventing browser UI freezes.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

